use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::extract::{Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use wp_rs_auth::{resolve_current_user, sign_auth_cookie, AuthScheme, AuthSecrets, NonceService};
use wp_rs_config::RustGatewaySettings;
use wp_rs_db::OptionStore;
use wp_rs_http::detect_endpoint_kind;
use wp_rs_rest::core_seed_routes;

#[derive(Debug, Clone)]
struct AppState {
    options: Arc<Mutex<OptionStore>>,
    auth_secrets: AuthSecrets,
    nonce_service: NonceService,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let address: SocketAddr = "127.0.0.1:8088"
        .parse()
        .expect("hardcoded listen address must parse");

    let state = build_app_state();

    let app = Router::new()
        .route("/__wp_rust/health", get(health))
        .route("/__wp_rust/echo", get(echo))
        .route("/__wp_rust/proxy-decision", get(proxy_decision))
        .route("/__wp_rust/internal/options", get(internal_options))
        .route("/__wp_rust/internal/auth-cookie", get(internal_auth_cookie))
        .route(
            "/__wp_rust/internal/auth-session",
            get(internal_auth_session),
        )
        .route("/__wp_rust/internal/nonce", get(internal_nonce))
        .fallback(not_found)
        .with_state(state);

    info!("wp-rs-server listening on {address}");

    if let Err(error) = axum::serve(
        tokio::net::TcpListener::bind(address)
            .await
            .expect("failed to bind listener"),
        app,
    )
    .await
    {
        error!("server failed: {error}");
    }
}

async fn health() -> impl IntoResponse {
    let routes = core_seed_routes();
    rust_handled_json(json!({
        "status": "ok",
        "component": "wp-rs-server",
        "seed_rest_routes": routes.all().len(),
    }))
}

async fn echo(request: Request) -> impl IntoResponse {
    let endpoint_kind = detect_endpoint_kind(request.uri().path()).as_str();
    rust_handled_json(json!({
        "method": request.method().to_string(),
        "path": request.uri().path(),
        "query": request.uri().query().unwrap_or_default(),
        "endpoint_kind": endpoint_kind,
    }))
}

#[derive(Debug, Deserialize)]
struct ProxyDecisionQuery {
    endpoint: String,
}

#[derive(Debug, Serialize)]
struct ProxyDecisionResponse {
    enabled: bool,
    endpoint: String,
    should_route: bool,
    backend_url: String,
    timeout_ms: u64,
}

async fn proxy_decision(Query(query): Query<ProxyDecisionQuery>) -> impl IntoResponse {
    let settings = RustGatewaySettings::from_env();
    let response = ProxyDecisionResponse {
        enabled: settings.enabled,
        should_route: settings.should_route(&query.endpoint),
        endpoint: query.endpoint,
        backend_url: settings.backend_url,
        timeout_ms: settings.timeout_ms,
    };
    rust_handled_json(response)
}

async fn not_found(request: Request) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "not_found",
            "path": request.uri().path(),
            "message": "Route is not implemented in wp-rs-server yet."
        })),
    )
}

fn rust_handled_json<T: Serialize>(value: T) -> (HeaderMap, Json<T>) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    (headers, Json(value))
}

fn build_app_state() -> AppState {
    let mut options = OptionStore::default();
    options.set_option("blogname", "WordPress", true);
    options.set_option("blogdescription", "Just another WordPress site", true);
    options.set_option("home", "http://localhost", true);
    options.set_option("siteurl", "http://localhost", true);
    options.set_option("recently_edited", "[]", false);

    AppState {
        options: Arc::new(Mutex::new(options)),
        auth_secrets: AuthSecrets::default(),
        nonce_service: NonceService::default(),
    }
}

#[derive(Debug, Deserialize)]
struct InternalOptionsQuery {
    option: Option<String>,
    autoload_only: Option<bool>,
}

async fn internal_options(
    State(state): State<AppState>,
    Query(query): Query<InternalOptionsQuery>,
) -> impl IntoResponse {
    let mut options = state.options.lock().expect("options mutex poisoned");

    if let Some(option_name) = query.option {
        let value = options.get_option(&option_name);
        return rust_handled_json(json!({
            "scope": "single",
            "option": option_name,
            "found": value.is_some(),
            "value": value,
        }));
    }

    let autoload_only = query.autoload_only.unwrap_or(true);
    let values = if autoload_only {
        options.load_alloptions()
    } else {
        options.snapshot()
    };

    rust_handled_json(json!({
        "scope": "bulk",
        "autoload_only": autoload_only,
        "count": values.len(),
        "values": values,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalAuthCookieQuery {
    user_id: Option<u64>,
    username: Option<String>,
    token: Option<String>,
    expiration: Option<u64>,
    scheme: Option<String>,
}

async fn internal_auth_cookie(
    State(state): State<AppState>,
    Query(query): Query<InternalAuthCookieQuery>,
) -> impl IntoResponse {
    let now = unix_now();
    let user_id = query.user_id.unwrap_or(1);
    let username = query.username.unwrap_or_else(|| "admin".to_string());
    let token = query.token.unwrap_or_else(|| "session-token".to_string());
    let expiration = query.expiration.unwrap_or(now + 3_600);
    let scheme = parse_auth_scheme(query.scheme.as_deref()).unwrap_or(AuthScheme::LoggedIn);
    let cookie_value = sign_auth_cookie(
        user_id,
        &username,
        expiration,
        &token,
        scheme,
        &state.auth_secrets,
    );
    let cookie_name = format!("{}fixture", scheme.cookie_prefix());

    rust_handled_json(json!({
        "cookie_name": cookie_name,
        "cookie_value": cookie_value,
        "scheme": scheme.as_str(),
        "user_id": user_id,
        "username": username,
        "expiration": expiration,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalAuthSessionQuery {
    now: Option<u64>,
}

async fn internal_auth_session(
    State(state): State<AppState>,
    Query(query): Query<InternalAuthSessionQuery>,
    request: Request,
) -> impl IntoResponse {
    let now = query.now.unwrap_or_else(unix_now);
    let cookie_header = request
        .headers()
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let resolved = resolve_current_user(cookie_header, now, &state.auth_secrets);

    rust_handled_json(json!({
        "cookie_header_present": !cookie_header.is_empty(),
        "resolved": resolved,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalNonceQuery {
    action: Option<String>,
    user_id: Option<u64>,
    token: Option<String>,
    now: Option<u64>,
}

async fn internal_nonce(
    State(state): State<AppState>,
    Query(query): Query<InternalNonceQuery>,
) -> impl IntoResponse {
    let now = query.now.unwrap_or_else(unix_now);
    let action = query.action.unwrap_or_else(|| "sample-action".to_string());
    let user_id = query.user_id.unwrap_or(1);
    let token = query.token.unwrap_or_else(|| "session-token".to_string());

    let nonce = state
        .nonce_service
        .create_nonce(&action, user_id, &token, now);
    let is_valid = state
        .nonce_service
        .verify_nonce(&nonce, &action, user_id, &token, now);

    rust_handled_json(json!({
        "action": action,
        "user_id": user_id,
        "token": token,
        "nonce": nonce,
        "valid": is_valid,
    }))
}

fn parse_auth_scheme(value: Option<&str>) -> Option<AuthScheme> {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "auth" => Some(AuthScheme::Auth),
        "secure_auth" | "secure" => Some(AuthScheme::SecureAuth),
        "logged_in" | "loggedin" => Some(AuthScheme::LoggedIn),
        _ => None,
    }
}

fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs()
}
