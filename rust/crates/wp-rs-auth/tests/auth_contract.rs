use wp_rs_auth::{
    AuthError, AuthScheme, AuthSecrets, NonceService, resolve_current_user, sign_auth_cookie,
    verify_auth_cookie,
};

#[test]
fn logged_in_cookie_round_trip_resolves_current_user() {
    let secrets = AuthSecrets::default();
    let cookie = sign_auth_cookie(
        101,
        "subscriber",
        2_100_000_000,
        "token-101",
        AuthScheme::LoggedIn,
        &secrets,
    );
    let header = format!("wordpress_logged_in_fixture={cookie}");

    let user = resolve_current_user(&header, 2_000_000_000, &secrets).expect("user should resolve");
    assert_eq!(user.user_id, 101);
    assert_eq!(user.username, "subscriber");
    assert_eq!(user.scheme, AuthScheme::LoggedIn);
}

#[test]
fn tampered_cookie_is_rejected() {
    let secrets = AuthSecrets::default();
    let cookie = sign_auth_cookie(
        5,
        "editor",
        2_100_000_000,
        "token-5",
        AuthScheme::LoggedIn,
        &secrets,
    );
    let tampered = cookie.replacen("token-5", "token-6", 1);

    let result = verify_auth_cookie(&tampered, AuthScheme::LoggedIn, 2_000_000_000, &secrets);
    assert!(matches!(result, Err(AuthError::InvalidSignature)));
}

#[test]
fn nonce_from_previous_tick_remains_valid() {
    let service = NonceService::default();
    let action = "update-profile";
    let user_id = 22;
    let token = "session-22";
    let now = 100_000;
    let half_life = service.nonce_life.as_secs() / 2;

    let nonce = service.create_nonce(action, user_id, token, now);
    assert!(service.verify_nonce(&nonce, action, user_id, token, now + half_life));
}
