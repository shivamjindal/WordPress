use std::collections::HashMap;
use std::time::Duration;

use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use bcrypt::verify as bcrypt_verify;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Digest;
use sha2::{Sha256, Sha384};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;

/// Parses an HTTP Cookie header into a key/value map.
pub fn parse_cookie_header(cookie_header: &str) -> HashMap<String, String> {
    parse_cookie_pairs(cookie_header).into_iter().collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CookieConstants {
    pub cookiehash: String,
    pub user_cookie: String,
    pub pass_cookie: String,
    pub auth_cookie: String,
    pub secure_auth_cookie: String,
    pub logged_in_cookie: String,
    pub test_cookie: String,
    pub recovery_mode_cookie: String,
}

pub fn cookie_constants(site_url: Option<&str>) -> CookieConstants {
    let cookiehash = site_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("{:x}", md5::compute(value.as_bytes())))
        .unwrap_or_default();

    CookieConstants {
        user_cookie: format!("wordpressuser_{cookiehash}"),
        pass_cookie: format!("wordpresspass_{cookiehash}"),
        auth_cookie: format!("wordpress_{cookiehash}"),
        secure_auth_cookie: format!("wordpress_sec_{cookiehash}"),
        logged_in_cookie: format!("wordpress_logged_in_{cookiehash}"),
        test_cookie: "wordpress_test_cookie".to_string(),
        recovery_mode_cookie: format!("wordpress_rec_{cookiehash}"),
        cookiehash,
    }
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
    let cookies = parse_cookie_pairs(cookie_header);
    let candidates = [
        (AuthScheme::LoggedIn, "wordpress_logged_in_"),
        (AuthScheme::SecureAuth, "wordpress_sec_"),
        (AuthScheme::Auth, "wordpress_"),
    ];

    for (scheme, prefix) in candidates {
        let matching_cookies = cookies
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(_, value)| value)
            .collect::<Vec<_>>();

        for cookie_value in matching_cookies {
            if let Ok(user) = verify_auth_cookie(cookie_value, scheme, now_timestamp, secrets) {
                return Some(user);
            }
        }
    }

    None
}

/// Verifies WordPress password hashes for modern bcrypt (`$2y$`, `$2b$`, `$2a$`)
/// and legacy phpass (`$P$`, `$H$`) formats.
pub fn verify_password(password: &str, hash: &str) -> bool {
    if hash.len() <= 32 && hash.chars().all(|character| character.is_ascii_hexdigit()) {
        let md5_hash = format!("{:x}", md5::compute(password.as_bytes()));
        return md5_hash.eq_ignore_ascii_case(hash);
    }

    if let Some(prefixed_hash) = hash.strip_prefix("$wp") {
        let mut hmac = HmacSha384::new_from_slice(b"wp-sha384").expect("HMAC key should be valid");
        hmac.update(password.as_bytes());
        let transformed = STANDARD.encode(hmac.finalize().into_bytes());
        let normalized_hash = normalize_bcrypt_hash_prefix(prefixed_hash);
        return bcrypt_verify(&transformed, &normalized_hash).unwrap_or(false);
    }

    if hash.starts_with("$P$") || hash.starts_with("$H$") {
        return verify_phpass_password(password, hash);
    }

    let normalized_hash = normalize_bcrypt_hash_prefix(hash);

    bcrypt_verify(password, &normalized_hash).unwrap_or(false)
}

/// Mirrors WordPress `wp_password_needs_rehash()` default behavior for bcrypt:
/// legacy md5/phpass and non-prefixed bcrypt hashes require rehashing.
pub fn password_needs_rehash(hash: &str) -> bool {
    if hash.len() <= 32 && hash.chars().all(|character| character.is_ascii_hexdigit()) {
        return true;
    }

    if hash.starts_with("$P$") || hash.starts_with("$H$") {
        return true;
    }

    if hash.starts_with("$wp$") {
        return false;
    }

    true
}

fn normalize_bcrypt_hash_prefix(hash: &str) -> String {
    if hash.starts_with("$2y$") {
        hash.replacen("$2y$", "$2b$", 1)
    } else {
        hash.to_string()
    }
}

fn verify_phpass_password(password: &str, hash: &str) -> bool {
    if hash.len() != 34 {
        return false;
    }

    let setting = &hash[..12];
    let expected = crypt_private(password, setting);
    expected == hash
}

fn crypt_private(password: &str, setting: &str) -> String {
    const ITOA64: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    if setting.len() < 12 {
        return "*0".to_string();
    }

    let id = &setting[..3];
    if id != "$P$" && id != "$H$" {
        return "*0".to_string();
    }

    let Some(count_log2) = ITOA64
        .iter()
        .position(|ch| *ch == setting.as_bytes()[3])
        .map(|index| index as u8)
    else {
        return "*0".to_string();
    };

    if !(7..=30).contains(&count_log2) {
        return "*0".to_string();
    }

    let count = 1u32 << count_log2;
    let salt = &setting[4..12];
    if salt.len() != 8 {
        return "*0".to_string();
    }

    let password_bytes = password.as_bytes();
    let mut digest = md5::compute([salt.as_bytes(), password_bytes].concat())
        .0
        .to_vec();
    for _ in 0..count {
        digest = md5::compute([digest.as_slice(), password_bytes].concat())
            .0
            .to_vec();
    }

    let mut output = setting.to_string();
    output.push_str(&encode64(&digest, 16));
    output
}

fn encode64(input: &[u8], count: usize) -> String {
    const ITOA64: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    let mut output = String::new();
    let mut i = 0usize;
    while i < count {
        let mut value = u32::from(input[i]);
        i += 1;
        output.push(ITOA64[(value & 0x3f) as usize] as char);

        if i < count {
            value |= u32::from(input[i]) << 8;
        }
        output.push(ITOA64[((value >> 6) & 0x3f) as usize] as char);
        if i >= count {
            break;
        }

        i += 1;
        if i < count {
            value |= u32::from(input[i]) << 16;
        }
        output.push(ITOA64[((value >> 12) & 0x3f) as usize] as char);
        if i >= count {
            break;
        }

        i += 1;
        output.push(ITOA64[((value >> 18) & 0x3f) as usize] as char);
    }

    output
}

fn parse_cookie_pairs(cookie_header: &str) -> Vec<(String, String)> {
    cookie_header
        .split(';')
        .filter_map(|pair| {
            let mut pieces = pair.trim().splitn(2, '=');
            let key = pieces.next()?.trim();
            let value = pieces.next().unwrap_or_default().trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_string(), decode_cookie_value(value)))
        })
        .collect()
}

fn decode_cookie_value(value: &str) -> String {
    let wrapped = format!("value={value}");
    form_urlencoded::parse(wrapped.as_bytes())
        .find_map(|(key, decoded)| (key == "value").then(|| decoded.into_owned()))
        .unwrap_or_else(|| value.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionTokenEntry {
    pub expiration: u64,
    pub login: u64,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionTokenStore {
    sessions: HashMap<u64, HashMap<String, SessionTokenEntry>>,
}

impl SessionTokenStore {
    pub fn upsert_session(
        &mut self,
        user_id: u64,
        token: &str,
        expiration: u64,
        now_timestamp: u64,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> bool {
        self.prune_expired_for_user(user_id, now_timestamp);
        let verifier = session_token_verifier(token);
        let entry = SessionTokenEntry {
            expiration,
            login: now_timestamp,
            ip,
            user_agent,
        };

        let user_sessions = self.sessions.entry(user_id).or_default();
        let changed = user_sessions.get(&verifier) != Some(&entry);
        user_sessions.insert(verifier, entry);
        changed
    }

    pub fn replace_with_session(
        &mut self,
        user_id: u64,
        token: &str,
        expiration: u64,
        now_timestamp: u64,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> (bool, usize) {
        let changed =
            self.upsert_session(user_id, token, expiration, now_timestamp, ip, user_agent);
        let removed_count = self.destroy_other_sessions(user_id, token, now_timestamp);
        (changed, removed_count)
    }

    pub fn verify_session(
        &mut self,
        user_id: u64,
        token: &str,
        now_timestamp: u64,
    ) -> Option<SessionTokenEntry> {
        self.prune_expired_for_user(user_id, now_timestamp);
        self.sessions
            .get(&user_id)
            .and_then(|sessions| sessions.get(&session_token_verifier(token)).cloned())
    }

    pub fn destroy_session(&mut self, user_id: u64, token: &str) -> bool {
        let verifier = session_token_verifier(token);
        let removed = self
            .sessions
            .get_mut(&user_id)
            .and_then(|sessions| sessions.remove(&verifier))
            .is_some();
        if self.sessions.get(&user_id).is_some_and(HashMap::is_empty) {
            self.sessions.remove(&user_id);
        }
        removed
    }

    pub fn destroy_other_sessions(
        &mut self,
        user_id: u64,
        token: &str,
        now_timestamp: u64,
    ) -> usize {
        self.prune_expired_for_user(user_id, now_timestamp);
        let verifier = session_token_verifier(token);
        let Some(sessions) = self.sessions.get_mut(&user_id) else {
            return 0;
        };

        let before = sessions.len();
        sessions.retain(|stored_verifier, _| stored_verifier == &verifier);
        let removed = before.saturating_sub(sessions.len());
        if sessions.is_empty() {
            self.sessions.remove(&user_id);
        }
        removed
    }

    pub fn destroy_all_sessions(&mut self, user_id: u64) -> usize {
        self.sessions
            .remove(&user_id)
            .map_or(0, |sessions| sessions.len())
    }

    pub fn count_active_sessions(&mut self, user_id: u64, now_timestamp: u64) -> usize {
        self.prune_expired_for_user(user_id, now_timestamp);
        self.sessions.get(&user_id).map_or(0, HashMap::len)
    }

    fn prune_expired_for_user(&mut self, user_id: u64, now_timestamp: u64) -> usize {
        let Some(sessions) = self.sessions.get_mut(&user_id) else {
            return 0;
        };

        let before = sessions.len();
        sessions.retain(|_, session| session.expiration > now_timestamp);
        let removed = before.saturating_sub(sessions.len());
        if sessions.is_empty() {
            self.sessions.remove(&user_id);
        }
        removed
    }
}

fn session_token_verifier(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
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
        self.verify_nonce_code(nonce, action, user_id, session_token, now_timestamp)
            .is_some()
    }

    /// Mirrors `wp_verify_nonce()` semantics:
    /// - `Some(1)` current tick
    /// - `Some(2)` previous tick
    /// - `None` invalid
    pub fn verify_nonce_code(
        &self,
        nonce: &str,
        action: &str,
        user_id: u64,
        session_token: &str,
        now_timestamp: u64,
    ) -> Option<u8> {
        let current_tick = self.tick(now_timestamp);
        let current = self.nonce_for_tick(current_tick, action, user_id, session_token);
        let previous = self.nonce_for_tick(
            current_tick.saturating_sub(1),
            action,
            user_id,
            session_token,
        );
        if nonce == current {
            Some(1)
        } else if nonce == previous {
            Some(2)
        } else {
            None
        }
    }

    fn tick(&self, now_timestamp: u64) -> u64 {
        let half_life = (self.nonce_life.as_secs() / 2).max(1);
        now_timestamp.saturating_add(half_life.saturating_sub(1)) / half_life
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
    fn parses_cookie_header_decodes_urlencoded_values() {
        let cookies = parse_cookie_header("auth=user%7C123%7Ctoken");
        assert_eq!(cookies.get("auth"), Some(&"user|123|token".to_string()));
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
    fn cookie_constants_include_site_hash() {
        let constants = cookie_constants(Some("https://example.com"));
        assert_eq!(constants.cookiehash, "c984d06aafbecf6bc55569f964148ea3");
        assert_eq!(
            constants.logged_in_cookie,
            "wordpress_logged_in_c984d06aafbecf6bc55569f964148ea3"
        );
        assert_eq!(constants.test_cookie, "wordpress_test_cookie");
    }

    #[test]
    fn cookie_constants_allow_empty_hash_for_missing_site_url() {
        let constants = cookie_constants(None);
        assert_eq!(constants.cookiehash, "");
        assert_eq!(constants.auth_cookie, "wordpress_");
        assert_eq!(constants.secure_auth_cookie, "wordpress_sec_");
        assert_eq!(constants.recovery_mode_cookie, "wordpress_rec_");
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
    fn resolves_user_from_urlencoded_cookie_header() {
        let secrets = AuthSecrets::default();
        let cookie = sign_auth_cookie(
            33,
            "author",
            5_000,
            "token-1",
            AuthScheme::LoggedIn,
            &secrets,
        );
        let encoded_cookie = cookie.replace('|', "%7C");
        let header = format!("wordpress_logged_in_fixture={encoded_cookie}; other=1");
        let resolved = resolve_current_user(&header, 4_999, &secrets).expect("user should resolve");
        assert_eq!(resolved.user_id, 33);
        assert_eq!(resolved.username, "author");
    }

    #[test]
    fn resolves_user_when_invalid_prefixed_cookie_is_present() {
        let secrets = AuthSecrets::default();
        let valid_cookie =
            sign_auth_cookie(44, "manager", 5_000, "token-2", AuthScheme::Auth, &secrets);
        let header = format!(
            "wordpress_test_cookie=WP+Cookie+check; wordpress_fixture=invalid; wordpress_fixture_auth={valid_cookie}"
        );
        let resolved = resolve_current_user(&header, 4_999, &secrets)
            .expect("user should resolve despite malformed prefixed cookies");
        assert_eq!(resolved.user_id, 44);
        assert_eq!(resolved.username, "manager");
        assert_eq!(resolved.scheme, AuthScheme::Auth);
    }

    #[test]
    fn nonce_service_validates_current_and_previous_tick() {
        let service = NonceService::default();
        let now = 100_000;
        let nonce = service.create_nonce("save-post", 7, "token", now);
        assert!(service.verify_nonce(&nonce, "save-post", 7, "token", now));
        assert_eq!(
            service.verify_nonce_code(&nonce, "save-post", 7, "token", now),
            Some(1)
        );
        assert!(!service.verify_nonce(&nonce, "delete-post", 7, "token", now));
    }

    #[test]
    fn nonce_service_returns_previous_tick_code() {
        let service = NonceService::default();
        let issued_at = 100_000;
        let nonce = service.create_nonce("save-post", 7, "token", issued_at);
        let verify_now = issued_at + service.nonce_life.as_secs() / 2;
        assert_eq!(
            service.verify_nonce_code(&nonce, "save-post", 7, "token", verify_now),
            Some(2)
        );
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

    #[test]
    fn nonce_service_uses_wordpress_tick_boundary_semantics() {
        let service = NonceService::default();
        let half_life = service.nonce_life.as_secs() / 2;
        let issued_at = 1;
        let nonce = service.create_nonce("save-post", 7, "token", issued_at);

        assert_eq!(
            service.verify_nonce_code(&nonce, "save-post", 7, "token", half_life),
            Some(1)
        );
        assert_eq!(
            service.verify_nonce_code(&nonce, "save-post", 7, "token", half_life + 1),
            Some(2)
        );
    }

    #[test]
    fn session_token_store_round_trips_and_removes_sessions() {
        let mut sessions = SessionTokenStore::default();
        assert!(sessions.upsert_session(
            7,
            "session-a",
            2_000_003_600,
            2_000_000_000,
            Some("127.0.0.1".to_string()),
            Some("Firefox".to_string())
        ));

        let stored = sessions
            .verify_session(7, "session-a", 2_000_000_001)
            .expect("session should verify");
        assert_eq!(stored.expiration, 2_000_003_600);
        assert_eq!(stored.ip, Some("127.0.0.1".to_string()));
        assert_eq!(sessions.count_active_sessions(7, 2_000_000_001), 1);

        assert!(sessions.destroy_session(7, "session-a"));
        assert!(sessions
            .verify_session(7, "session-a", 2_000_000_001)
            .is_none());
    }

    #[test]
    fn session_token_store_prunes_expired_sessions() {
        let mut sessions = SessionTokenStore::default();
        sessions.upsert_session(
            7,
            "session-expired",
            2_000_000_010,
            2_000_000_000,
            None,
            None,
        );
        assert!(sessions
            .verify_session(7, "session-expired", 2_000_000_011)
            .is_none());
        assert_eq!(sessions.count_active_sessions(7, 2_000_000_011), 0);
    }

    #[test]
    fn session_token_store_can_destroy_other_sessions() {
        let mut sessions = SessionTokenStore::default();
        sessions.upsert_session(7, "session-a", 2_000_003_600, 2_000_000_000, None, None);
        sessions.upsert_session(7, "session-b", 2_000_003_600, 2_000_000_000, None, None);
        sessions.upsert_session(7, "session-c", 2_000_003_600, 2_000_000_000, None, None);

        let removed = sessions.destroy_other_sessions(7, "session-b", 2_000_000_100);
        assert_eq!(removed, 2);
        assert!(sessions
            .verify_session(7, "session-b", 2_000_000_100)
            .is_some());
        assert!(sessions
            .verify_session(7, "session-a", 2_000_000_100)
            .is_none());
        assert!(sessions
            .verify_session(7, "session-c", 2_000_000_100)
            .is_none());
    }

    #[test]
    fn session_token_store_replace_with_session_keeps_single_token() {
        let mut sessions = SessionTokenStore::default();
        sessions.upsert_session(7, "session-a", 2_000_003_600, 2_000_000_000, None, None);
        sessions.upsert_session(7, "session-b", 2_000_003_600, 2_000_000_000, None, None);

        let (changed, removed_count) = sessions.replace_with_session(
            7,
            "session-c",
            2_000_003_600,
            2_000_000_200,
            Some("198.51.100.15".to_string()),
            Some("Safari".to_string()),
        );
        assert!(changed);
        assert_eq!(removed_count, 2);
        assert_eq!(sessions.count_active_sessions(7, 2_000_000_201), 1);

        let replacement = sessions
            .verify_session(7, "session-c", 2_000_000_201)
            .expect("replacement session should remain");
        assert_eq!(replacement.ip, Some("198.51.100.15".to_string()));
        assert_eq!(replacement.user_agent, Some("Safari".to_string()));
        assert!(sessions
            .verify_session(7, "session-a", 2_000_000_201)
            .is_none());
        assert!(sessions
            .verify_session(7, "session-b", 2_000_000_201)
            .is_none());
    }

    #[test]
    fn verifies_modern_bcrypt_password_hashes() {
        let hash = bcrypt::hash("s3cret-pass", 4).expect("bcrypt hash should generate");
        assert!(verify_password("s3cret-pass", &hash));
        assert!(!verify_password("wrong-pass", &hash));
    }

    #[test]
    fn verifies_wordpress_2y_bcrypt_hash_prefix() {
        let hash = bcrypt::hash("s3cret-pass", 4).expect("bcrypt hash should generate");
        let wordpress_hash = hash.replacen("$2b$", "$2y$", 1);
        assert!(verify_password("s3cret-pass", &wordpress_hash));
    }

    #[test]
    fn verifies_legacy_phpass_password_hashes() {
        let hash = "$P$B/x5z53S8OFO34SWjip8BphQFAhFsJ1";
        assert!(verify_password("password123", hash));
        assert!(!verify_password("not-password123", hash));
    }

    #[test]
    fn verifies_legacy_md5_password_hashes() {
        let hash = "482c811da5d5b4bc6d497ffa98491e38";
        assert!(verify_password("password123", hash));
        assert!(!verify_password("incorrect-password", hash));
    }

    #[test]
    fn verifies_wp_prefixed_bcrypt_password_hashes() {
        let mut hmac = HmacSha384::new_from_slice(b"wp-sha384").expect("HMAC key should be valid");
        hmac.update("password123".as_bytes());
        let transformed = STANDARD.encode(hmac.finalize().into_bytes());
        let bcrypt_hash = bcrypt::hash(&transformed, 4).expect("bcrypt hash should generate");
        let wp_hash = format!("$wp{bcrypt_hash}");

        assert!(verify_password("password123", &wp_hash));
        assert!(!verify_password("wrong-password", &wp_hash));
    }

    #[test]
    fn password_needs_rehash_flags_legacy_and_unprefixed_bcrypt_hashes() {
        assert!(password_needs_rehash("482c811da5d5b4bc6d497ffa98491e38"));
        assert!(password_needs_rehash("$P$B/x5z53S8OFO34SWjip8BphQFAhFsJ1"));
        assert!(password_needs_rehash(
            "$2y$04$vYwbi8PAi/C6aSx5LVikLetY8UzH0Dfljt1jT7OqvzuA1mJjseLlG"
        ));
        assert!(password_needs_rehash(
            "$argon2id$v=19$m=65536,t=2,p=1$abc$def"
        ));
        assert!(!password_needs_rehash(
            "$wp$2y$04$vYwbi8PAi/C6aSx5LVikLetY8UzH0Dfljt1jT7OqvzuA1mJjseLlG"
        ));
    }
}
