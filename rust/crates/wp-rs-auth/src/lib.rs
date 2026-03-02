use std::collections::HashMap;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Parses an HTTP Cookie header into a key/value map.
pub fn parse_cookie_header(cookie_header: &str) -> HashMap<String, String> {
    cookie_header
        .split(';')
        .filter_map(|pair| {
            let mut pieces = pair.trim().splitn(2, '=');
            let key = pieces.next()?.trim();
            let value = pieces.next().unwrap_or_default().trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AuthScheme {
    Auth,
    SecureAuth,
    LoggedIn,
}

impl AuthScheme {
    pub fn cookie_prefix(&self) -> &'static str {
        match self {
            AuthScheme::Auth => "wordpress_",
            AuthScheme::SecureAuth => "wordpress_sec_",
            AuthScheme::LoggedIn => "wordpress_logged_in_",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthScheme::Auth => "auth",
            AuthScheme::SecureAuth => "secure_auth",
            AuthScheme::LoggedIn => "logged_in",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthSecrets {
    pub auth_key: String,
    pub secure_auth_key: String,
    pub logged_in_key: String,
    pub nonce_key: String,
    pub nonce_salt: String,
}

impl Default for AuthSecrets {
    fn default() -> Self {
        Self {
            auth_key: "dev-auth-key".to_string(),
            secure_auth_key: "dev-secure-auth-key".to_string(),
            logged_in_key: "dev-logged-in-key".to_string(),
            nonce_key: "dev-nonce-key".to_string(),
            nonce_salt: "dev-nonce-salt".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthenticatedUser {
    pub user_id: u64,
    pub username: String,
    pub token: String,
    pub expiration: u64,
    pub scheme: AuthScheme,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid cookie format")]
    InvalidCookieFormat,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("cookie expired")]
    ExpiredCookie,
    #[error("invalid number: {0}")]
    InvalidNumber(String),
}

pub fn sign_auth_cookie(
    user_id: u64,
    username: &str,
    expiration: u64,
    token: &str,
    scheme: AuthScheme,
    secrets: &AuthSecrets,
) -> String {
    let payload = format!(
        "{user_id}|{username}|{expiration}|{token}|{}",
        scheme.as_str()
    );
    let signature = sign_payload(&payload, scheme, secrets);
    format!("{user_id}|{username}|{expiration}|{token}|{signature}")
}

pub fn verify_auth_cookie(
    cookie_value: &str,
    scheme: AuthScheme,
    now_timestamp: u64,
    secrets: &AuthSecrets,
) -> Result<AuthenticatedUser, AuthError> {
    let parts = cookie_value.split('|').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(AuthError::InvalidCookieFormat);
    }

    let user_id = parts[0]
        .parse::<u64>()
        .map_err(|_| AuthError::InvalidNumber(parts[0].to_string()))?;
    let username = parts[1].to_string();
    let expiration = parts[2]
        .parse::<u64>()
        .map_err(|_| AuthError::InvalidNumber(parts[2].to_string()))?;
    let token = parts[3].to_string();
    let received_signature = parts[4];

    if now_timestamp > expiration {
        return Err(AuthError::ExpiredCookie);
    }

    let payload = format!(
        "{user_id}|{username}|{expiration}|{token}|{}",
        scheme.as_str()
    );
    let expected_signature = sign_payload(&payload, scheme, secrets);

    if expected_signature != received_signature {
        return Err(AuthError::InvalidSignature);
    }

    Ok(AuthenticatedUser {
        user_id,
        username,
        token,
        expiration,
        scheme,
    })
}

pub fn resolve_current_user(
    cookie_header: &str,
    now_timestamp: u64,
    secrets: &AuthSecrets,
) -> Option<AuthenticatedUser> {
    let cookies = parse_cookie_header(cookie_header);
    let candidates = [
        (AuthScheme::LoggedIn, "wordpress_logged_in_"),
        (AuthScheme::SecureAuth, "wordpress_sec_"),
        (AuthScheme::Auth, "wordpress_"),
    ];

    for (scheme, prefix) in candidates {
        let matching_cookie = cookies
            .iter()
            .find(|(name, _)| name.starts_with(prefix))
            .map(|(_, value)| value);

        if let Some(cookie_value) = matching_cookie {
            if let Ok(user) = verify_auth_cookie(cookie_value, scheme, now_timestamp, secrets) {
                return Some(user);
            }
        }
    }

    None
}

#[derive(Debug, Clone)]
pub struct NonceService {
    pub nonce_life: Duration,
    pub secrets: AuthSecrets,
}

impl Default for NonceService {
    fn default() -> Self {
        Self {
            nonce_life: Duration::from_secs(24 * 60 * 60),
            secrets: AuthSecrets::default(),
        }
    }
}

impl NonceService {
    pub fn create_nonce(
        &self,
        action: &str,
        user_id: u64,
        session_token: &str,
        now_timestamp: u64,
    ) -> String {
        let tick = self.tick(now_timestamp);
        self.nonce_for_tick(tick, action, user_id, session_token)
    }

    pub fn verify_nonce(
        &self,
        nonce: &str,
        action: &str,
        user_id: u64,
        session_token: &str,
        now_timestamp: u64,
    ) -> bool {
        let current_tick = self.tick(now_timestamp);
        let current = self.nonce_for_tick(current_tick, action, user_id, session_token);
        let previous = self.nonce_for_tick(
            current_tick.saturating_sub(1),
            action,
            user_id,
            session_token,
        );
        nonce == current || nonce == previous
    }

    fn tick(&self, now_timestamp: u64) -> u64 {
        let half_life = (self.nonce_life.as_secs() / 2).max(1);
        now_timestamp / half_life
    }

    fn nonce_for_tick(&self, tick: u64, action: &str, user_id: u64, session_token: &str) -> String {
        let payload = format!("{tick}|{action}|{user_id}|{session_token}");
        let key = format!("{}{}", self.secrets.nonce_key, self.secrets.nonce_salt);
        let signature = sign_raw(&payload, &key);
        signature.chars().take(12).collect()
    }
}

fn sign_payload(payload: &str, scheme: AuthScheme, secrets: &AuthSecrets) -> String {
    let key = match scheme {
        AuthScheme::Auth => &secrets.auth_key,
        AuthScheme::SecureAuth => &secrets.secure_auth_key,
        AuthScheme::LoggedIn => &secrets.logged_in_key,
    };
    sign_raw(payload, key)
}

fn sign_raw(payload: &str, key: &str) -> String {
    let mut hmac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC key should be valid");
    hmac.update(payload.as_bytes());
    URL_SAFE_NO_PAD.encode(hmac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cookie_header_pairs() {
        let cookies = parse_cookie_header("foo=bar; test=value");
        assert_eq!(cookies.get("foo"), Some(&"bar".to_string()));
        assert_eq!(cookies.get("test"), Some(&"value".to_string()));
    }

    #[test]
    fn signs_and_verifies_logged_in_cookie() {
        let secrets = AuthSecrets::default();
        let cookie = sign_auth_cookie(
            15,
            "admin",
            2_000_000_000,
            "session-token",
            AuthScheme::LoggedIn,
            &secrets,
        );
        let user = verify_auth_cookie(&cookie, AuthScheme::LoggedIn, 1_999_999_999, &secrets)
            .expect("cookie should verify");

        assert_eq!(user.user_id, 15);
        assert_eq!(user.username, "admin");
        assert_eq!(user.scheme, AuthScheme::LoggedIn);
    }

    #[test]
    fn signs_and_verifies_auth_cookie() {
        let secrets = AuthSecrets::default();
        let cookie = sign_auth_cookie(
            9,
            "editor",
            2_000_000_000,
            "auth-token",
            AuthScheme::Auth,
            &secrets,
        );
        let user = verify_auth_cookie(&cookie, AuthScheme::Auth, 1_999_999_999, &secrets)
            .expect("auth cookie should verify");

        assert_eq!(user.user_id, 9);
        assert_eq!(user.username, "editor");
        assert_eq!(user.scheme, AuthScheme::Auth);
    }

    #[test]
    fn signs_and_verifies_secure_auth_cookie() {
        let secrets = AuthSecrets::default();
        let cookie = sign_auth_cookie(
            11,
            "admin",
            2_000_000_000,
            "secure-token",
            AuthScheme::SecureAuth,
            &secrets,
        );
        let user = verify_auth_cookie(&cookie, AuthScheme::SecureAuth, 1_999_999_999, &secrets)
            .expect("secure auth cookie should verify");

        assert_eq!(user.user_id, 11);
        assert_eq!(user.username, "admin");
        assert_eq!(user.scheme, AuthScheme::SecureAuth);
    }

    #[test]
    fn rejects_expired_cookie() {
        let secrets = AuthSecrets::default();
        let cookie = sign_auth_cookie(
            15,
            "admin",
            1_000,
            "session-token",
            AuthScheme::LoggedIn,
            &secrets,
        );
        let result = verify_auth_cookie(&cookie, AuthScheme::LoggedIn, 2_000, &secrets);
        assert!(matches!(result, Err(AuthError::ExpiredCookie)));
    }

    #[test]
    fn resolves_user_from_cookie_header() {
        let secrets = AuthSecrets::default();
        let cookie = sign_auth_cookie(
            33,
            "author",
            5_000,
            "token-1",
            AuthScheme::LoggedIn,
            &secrets,
        );
        let header = format!("wordpress_logged_in_fixture={cookie}; other=1");
        let resolved = resolve_current_user(&header, 4_999, &secrets).expect("user should resolve");
        assert_eq!(resolved.user_id, 33);
        assert_eq!(resolved.username, "author");
    }

    #[test]
    fn nonce_service_validates_current_and_previous_tick() {
        let service = NonceService::default();
        let now = 100_000;
        let nonce = service.create_nonce("save-post", 7, "token", now);
        assert!(service.verify_nonce(&nonce, "save-post", 7, "token", now));
        assert!(!service.verify_nonce(&nonce, "delete-post", 7, "token", now));
    }

    #[test]
    fn nonce_service_rejects_after_two_tick_rollovers() {
        let service = NonceService::default();
        let now = 100_000;
        let nonce = service.create_nonce("save-post", 7, "token", now);
        let half_life = service.nonce_life.as_secs() / 2;
        let verify_now = now + (half_life * 2);
        assert!(!service.verify_nonce(&nonce, "save-post", 7, "token", verify_now));
    }
}
