use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::{to_bytes, Bytes};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{any, get};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use wp_rs_admin::{core_admin_actions, AdminRequest, AdminSurface};
use wp_rs_auth::{resolve_current_user, sign_auth_cookie, AuthScheme, AuthSecrets, NonceService};
use wp_rs_config::RustGatewaySettings;
use wp_rs_content::{extract_block_names, parse_front_route};
use wp_rs_cron::{parse_doing_wp_cron, CronEvent, CronScheduler};
use wp_rs_db::{MultisiteResolver, NetworkSite, OptionStore};
use wp_rs_http::{
    core_xmlrpc_registry, detect_endpoint_kind, parse_xmlrpc_method_name, xmlrpc_fault_response,
    xmlrpc_success_response,
};
use wp_rs_rest::{core_seed_routes, RestRequest};

#[derive(Debug, Clone)]
struct AppState {
    options: Arc<Mutex<OptionStore>>,
    auth_secrets: AuthSecrets,
    nonce_service: NonceService,
    cron_scheduler: Arc<Mutex<CronScheduler>>,
    multisite_resolver: MultisiteResolver,
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
        .route("/wp-json", any(rest_dispatch_root))
        .route("/wp-json/", any(rest_dispatch_root))
        .route("/wp-json/*rest_path", any(rest_dispatch))
        .route("/wp-admin/admin-ajax.php", any(admin_ajax_dispatch))
        .route("/wp-admin/admin-post.php", any(admin_post_dispatch))
        .route("/wp-admin/async-upload.php", any(async_upload_dispatch))
        .route("/xmlrpc.php", any(xmlrpc_live_dispatch))
        .route("/wp-cron.php", any(cron_live_dispatch))
        .route("/__wp_rust/internal/options", get(internal_options))
        .route("/__wp_rust/internal/auth-cookie", get(internal_auth_cookie))
        .route(
            "/__wp_rust/internal/auth-session",
            get(internal_auth_session),
        )
        .route("/__wp_rust/internal/nonce", get(internal_nonce))
        .route(
            "/__wp_rust/internal/content-route",
            get(internal_content_route),
        )
        .route("/__wp_rust/internal/block-parse", get(internal_block_parse))
        .route(
            "/__wp_rust/internal/rest-contract",
            get(internal_rest_contract),
        )
        .route(
            "/__wp_rust/internal/rest-dispatch",
            get(internal_rest_dispatch),
        )
        .route(
            "/__wp_rust/internal/admin-contract",
            get(internal_admin_contract),
        )
        .route(
            "/__wp_rust/internal/admin-dispatch",
            get(internal_admin_dispatch),
        )
        .route(
            "/__wp_rust/internal/xmlrpc-contract",
            get(internal_xmlrpc_contract),
        )
        .route(
            "/__wp_rust/internal/xmlrpc-dispatch",
            get(internal_xmlrpc_dispatch),
        )
        .route(
            "/__wp_rust/internal/cron-schedule",
            get(internal_cron_schedule),
        )
        .route("/__wp_rust/internal/cron-due", get(internal_cron_due))
        .route(
            "/__wp_rust/internal/multisite-resolve",
            get(internal_multisite_resolve),
        )
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
    fallback_enabled: bool,
    endpoint: String,
    should_route: bool,
    method_allowlist: Vec<String>,
    backend_url: String,
    timeout_ms: u64,
    plugin_compat_mode: String,
}

async fn proxy_decision(Query(query): Query<ProxyDecisionQuery>) -> impl IntoResponse {
    let settings = RustGatewaySettings::from_env();
    let response = ProxyDecisionResponse {
        enabled: settings.enabled,
        fallback_enabled: settings.fallback_enabled,
        should_route: settings.should_route(&query.endpoint),
        endpoint: query.endpoint,
        method_allowlist: {
            let mut methods = settings
                .method_allowlist
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            methods.sort();
            methods
        },
        backend_url: settings.backend_url,
        timeout_ms: settings.timeout_ms,
        plugin_compat_mode: settings.plugin_compat_mode,
    };
    rust_handled_json(response)
}

async fn rest_dispatch_root(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    rest_dispatch_inner(state, request, "/wp-json".to_string())
}

async fn rest_dispatch(
    State(state): State<AppState>,
    Path(rest_path): Path<String>,
    request: Request,
) -> impl IntoResponse {
    let normalized = rest_path.trim_matches('/');
    let route_path = if normalized.is_empty() {
        "/wp-json".to_string()
    } else {
        format!("/wp-json/{normalized}")
    };
    rest_dispatch_inner(state, request, route_path)
}

fn rest_dispatch_inner(state: AppState, request: Request, route_path: String) -> impl IntoResponse {
    let registry = core_seed_routes();
    let headers = request.headers();
    let (authenticated, capabilities) = auth_context_from_headers(headers, &state.auth_secrets);

    let mut rest_request = RestRequest::new(request.method().to_string(), route_path);
    rest_request.authenticated = authenticated;
    rest_request.capabilities = capabilities;

    let result = registry.dispatch(&rest_request);
    rust_handled_json_with_status(
        StatusCode::from_u16(result.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        result,
    )
}

async fn admin_ajax_dispatch(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    admin_dispatch_inner(state, request, AdminSurface::Ajax).await
}

async fn admin_post_dispatch(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    admin_dispatch_inner(state, request, AdminSurface::AdminPost).await
}

async fn async_upload_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    admin_dispatch_inner(state, request, AdminSurface::AsyncUpload).await
}

async fn admin_dispatch_inner(
    state: AppState,
    request: Request,
    surface: AdminSurface,
) -> impl IntoResponse {
    let registry = core_admin_actions();
    let (parts, body) = request.into_parts();
    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);

    let action = params
        .get("action")
        .cloned()
        .unwrap_or_else(|| match surface {
            AdminSurface::AsyncUpload => "upload-attachment".to_string(),
            _ => String::new(),
        });

    let nonce_present = params
        .get("_ajax_nonce")
        .or_else(|| params.get("_wpnonce"))
        .is_some_and(|value| !value.trim().is_empty());

    let mut admin_request = AdminRequest::new(surface, action);
    admin_request.authenticated = authenticated;
    admin_request.capabilities = capabilities;
    admin_request.nonce_present = nonce_present;

    let result = registry.dispatch(&admin_request);
    rust_handled_json_with_status(
        StatusCode::from_u16(result.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        result,
    )
}

async fn xmlrpc_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    let registry = core_xmlrpc_registry();
    let (parts, body) = request.into_parts();
    let body_bytes = read_request_body(body).await;
    let payload = String::from_utf8_lossy(&body_bytes).to_string();
    let method_name = parse_xmlrpc_method_name(&payload);
    let (authenticated, _) = auth_context_from_headers(&parts.headers, &state.auth_secrets);

    let (result, status_code, xml_body) = match method_name {
        Some(method_name) => {
            let result = registry.dispatch(&method_name, authenticated);
            let xml_body = if result.success {
                xmlrpc_success_response(&method_name)
            } else {
                xmlrpc_fault_response(result.fault_code.unwrap_or(-32603), &result.message)
            };
            let status = StatusCode::from_u16(if result.success { 200 } else { 403 })
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (result, status, xml_body)
        }
        None => (
            registry.dispatch("", false),
            StatusCode::BAD_REQUEST,
            xmlrpc_fault_response(-32600, "Invalid XML-RPC request"),
        ),
    };

    rust_handled_xml(status_code, xml_body, result.success)
}

async fn cron_live_dispatch(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    let now = unix_now();
    let query_string = request.uri().query().unwrap_or_default();
    let doing_wp_cron = parse_doing_wp_cron(query_string);

    let mut scheduler = state
        .cron_scheduler
        .lock()
        .expect("cron scheduler mutex poisoned");

    let lock_acquired = scheduler.acquire_lock(Duration::from_secs(60));
    if !lock_acquired {
        return rust_handled_json(json!({
            "running": false,
            "lock_acquired": false,
            "due_count": 0,
            "doing_wp_cron": doing_wp_cron,
        }));
    }

    let due = scheduler.due_events(now);
    scheduler.release_lock();
    rust_handled_json(json!({
        "running": true,
        "lock_acquired": true,
        "due_count": due.len(),
        "events": due,
        "doing_wp_cron": doing_wp_cron,
        "next_event_timestamp": scheduler.next_event_timestamp(),
    }))
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

fn rust_handled_json_with_status<T: Serialize>(
    status: StatusCode,
    value: T,
) -> (StatusCode, HeaderMap, Json<T>) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    (status, headers, Json(value))
}

fn rust_handled_xml(
    status: StatusCode,
    body: String,
    _success: bool,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/xml; charset=UTF-8"),
    );
    (status, headers, body)
}

fn auth_context_from_headers(
    headers: &HeaderMap,
    auth_secrets: &AuthSecrets,
) -> (bool, BTreeSet<String>) {
    let cookie_header = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let authenticated_from_cookie =
        resolve_current_user(cookie_header, unix_now(), auth_secrets).is_some();
    let authenticated_from_header = headers
        .get("x-wp-rust-authenticated")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true") || value == "1");

    let mut capabilities = BTreeSet::new();
    if let Some(capability_header) = headers
        .get("x-wp-rust-capabilities")
        .and_then(|value| value.to_str().ok())
    {
        for capability in capability_header
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            capabilities.insert(capability.to_string());
        }
    }

    (
        authenticated_from_cookie || authenticated_from_header,
        capabilities,
    )
}

fn merged_params(
    uri_query: Option<&str>,
    body_bytes: &Bytes,
    content_type: &str,
) -> BTreeMap<String, String> {
    let mut params = parse_urlencoded(uri_query.unwrap_or_default());
    if content_type.contains("application/x-www-form-urlencoded") {
        let body = String::from_utf8_lossy(body_bytes);
        for (key, value) in parse_urlencoded(&body) {
            params.insert(key, value);
        }
    }
    params
}

fn parse_urlencoded(raw: &str) -> BTreeMap<String, String> {
    raw.split('&')
        .filter_map(|entry| {
            let mut parts = entry.splitn(2, '=');
            let key = parts.next()?.trim();
            if key.is_empty() {
                return None;
            }
            let value = parts.next().unwrap_or_default().trim();
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

async fn read_request_body(body: axum::body::Body) -> Bytes {
    to_bytes(body, 1024 * 1024)
        .await
        .unwrap_or_else(|_| Bytes::new())
}

fn build_app_state() -> AppState {
    let mut options = OptionStore::default();
    options.set_option("blogname", "WordPress", true);
    options.set_option("blogdescription", "Just another WordPress site", true);
    options.set_option("home", "http://localhost", true);
    options.set_option("siteurl", "http://localhost", true);
    options.set_option("recently_edited", "[]", false);

    let mut scheduler = CronScheduler::default();
    scheduler.schedule_event(CronEvent {
        hook: "wp_version_check".to_string(),
        timestamp: unix_now() + 60,
        schedule: Some("twicedaily".to_string()),
        args: vec![],
    });

    let mut multisite_resolver = MultisiteResolver::default();
    multisite_resolver.register_site(NetworkSite {
        blog_id: 1,
        domain: "example.com".to_string(),
        path: "/".to_string(),
        is_public: true,
    });
    multisite_resolver.register_site(NetworkSite {
        blog_id: 2,
        domain: "example.com".to_string(),
        path: "/blog/".to_string(),
        is_public: true,
    });

    AppState {
        options: Arc::new(Mutex::new(options)),
        auth_secrets: AuthSecrets::default(),
        nonce_service: NonceService::default(),
        cron_scheduler: Arc::new(Mutex::new(scheduler)),
        multisite_resolver,
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

#[derive(Debug, Deserialize)]
struct InternalContentRouteQuery {
    path: Option<String>,
    query: Option<String>,
}

async fn internal_content_route(
    Query(query): Query<InternalContentRouteQuery>,
) -> impl IntoResponse {
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let query_string = query.query.unwrap_or_default();
    let route = parse_front_route(&path, &query_string);
    rust_handled_json(route)
}

#[derive(Debug, Deserialize)]
struct InternalBlockParseQuery {
    content: Option<String>,
}

async fn internal_block_parse(Query(query): Query<InternalBlockParseQuery>) -> impl IntoResponse {
    let content = query.content.unwrap_or_default();
    let block_names = extract_block_names(&content);
    rust_handled_json(json!({
        "count": block_names.len(),
        "blocks": block_names,
    }))
}

async fn internal_rest_contract() -> impl IntoResponse {
    let registry = core_seed_routes();
    rust_handled_json(registry.contract_document())
}

#[derive(Debug, Deserialize)]
struct InternalRestDispatchQuery {
    method: Option<String>,
    path: Option<String>,
    authenticated: Option<bool>,
    capabilities: Option<String>,
}

async fn internal_rest_dispatch(
    Query(query): Query<InternalRestDispatchQuery>,
) -> impl IntoResponse {
    let registry = core_seed_routes();
    let mut request = RestRequest::new(
        query.method.unwrap_or_else(|| "GET".to_string()),
        query
            .path
            .unwrap_or_else(|| "/wp-json/wp/v2/posts".to_string()),
    );
    request.authenticated = query.authenticated.unwrap_or(false);

    if let Some(capabilities) = query.capabilities {
        for capability in capabilities
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request.capabilities.insert(capability.to_string());
        }
    }

    let result = registry.dispatch(&request);
    rust_handled_json(result)
}

async fn internal_admin_contract() -> impl IntoResponse {
    let registry = core_admin_actions();
    rust_handled_json(registry.contract_map())
}

#[derive(Debug, Deserialize)]
struct InternalAdminDispatchQuery {
    surface: Option<String>,
    action: Option<String>,
    authenticated: Option<bool>,
    nonce_present: Option<bool>,
    capabilities: Option<String>,
}

async fn internal_admin_dispatch(
    Query(query): Query<InternalAdminDispatchQuery>,
) -> impl IntoResponse {
    let registry = core_admin_actions();
    let surface = parse_admin_surface(query.surface.as_deref()).unwrap_or(AdminSurface::Ajax);
    let mut request = AdminRequest::new(surface, query.action.unwrap_or_default());
    request.authenticated = query.authenticated.unwrap_or(false);
    request.nonce_present = query.nonce_present.unwrap_or(false);

    if let Some(capabilities) = query.capabilities {
        for capability in capabilities
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            request.capabilities.insert(capability.to_string());
        }
    }

    let result = registry.dispatch(&request);
    rust_handled_json(result)
}

async fn internal_xmlrpc_contract() -> impl IntoResponse {
    let registry = core_xmlrpc_registry();
    rust_handled_json(registry.methods().to_vec())
}

#[derive(Debug, Deserialize)]
struct InternalXmlRpcDispatchQuery {
    method: Option<String>,
    payload: Option<String>,
    authenticated: Option<bool>,
}

async fn internal_xmlrpc_dispatch(
    Query(query): Query<InternalXmlRpcDispatchQuery>,
) -> impl IntoResponse {
    let registry = core_xmlrpc_registry();
    let authenticated = query.authenticated.unwrap_or(false);
    let method_name = query.method.or_else(|| {
        query
            .payload
            .and_then(|payload| parse_xmlrpc_method_name(&payload))
    });

    let Some(method_name) = method_name else {
        return rust_handled_json(json!({
            "status_code": 400,
            "result": {
                "success": false,
                "fault_code": -32600,
                "message": "Invalid XML-RPC request",
                "method_name": "",
            },
            "xml": xmlrpc_fault_response(-32600, "Invalid XML-RPC request"),
        }));
    };

    let result = registry.dispatch(&method_name, authenticated);
    let xml = if result.success {
        xmlrpc_success_response(&method_name)
    } else {
        xmlrpc_fault_response(result.fault_code.unwrap_or(-32603), &result.message)
    };

    rust_handled_json(json!({
        "status_code": if result.success { 200 } else { 403 },
        "result": result,
        "xml": xml,
    }))
}

#[derive(Debug, Deserialize)]
struct InternalCronScheduleQuery {
    hook: Option<String>,
    timestamp: Option<u64>,
    schedule: Option<String>,
    args: Option<String>,
    query_string: Option<String>,
}

async fn internal_cron_schedule(
    State(state): State<AppState>,
    Query(query): Query<InternalCronScheduleQuery>,
) -> impl IntoResponse {
    let hook = query
        .hook
        .unwrap_or_else(|| "wp_scheduled_delete".to_string());
    let timestamp = query.timestamp.unwrap_or_else(|| unix_now() + 60);
    let schedule = query.schedule;
    let args = query
        .args
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let doing_wp_cron = parse_doing_wp_cron(&query.query_string.unwrap_or_default());

    let mut scheduler = state
        .cron_scheduler
        .lock()
        .expect("cron scheduler mutex poisoned");
    scheduler.schedule_event(CronEvent {
        hook,
        timestamp,
        schedule,
        args,
    });

    rust_handled_json(json!({
        "scheduled": true,
        "doing_wp_cron": doing_wp_cron,
        "next_event_timestamp": scheduler.next_event_timestamp(),
    }))
}

#[derive(Debug, Deserialize)]
struct InternalCronDueQuery {
    now: Option<u64>,
}

async fn internal_cron_due(
    State(state): State<AppState>,
    Query(query): Query<InternalCronDueQuery>,
) -> impl IntoResponse {
    let now = query.now.unwrap_or_else(unix_now);
    let mut scheduler = state
        .cron_scheduler
        .lock()
        .expect("cron scheduler mutex poisoned");

    let lock_acquired = scheduler.acquire_lock(Duration::from_secs(60));
    if !lock_acquired {
        return rust_handled_json(json!({
            "lock_acquired": false,
            "due_count": 0,
            "events": [],
        }));
    }

    let due = scheduler.due_events(now);
    scheduler.release_lock();
    rust_handled_json(json!({
        "lock_acquired": true,
        "due_count": due.len(),
        "events": due,
        "next_event_timestamp": scheduler.next_event_timestamp(),
    }))
}

#[derive(Debug, Deserialize)]
struct InternalMultisiteResolveQuery {
    domain: Option<String>,
    path: Option<String>,
}

async fn internal_multisite_resolve(
    State(state): State<AppState>,
    Query(query): Query<InternalMultisiteResolveQuery>,
) -> impl IntoResponse {
    let domain = query.domain.unwrap_or_else(|| "example.com".to_string());
    let path = query.path.unwrap_or_else(|| "/".to_string());
    let resolved = state.multisite_resolver.resolve(&domain, &path);

    rust_handled_json(json!({
        "domain": domain,
        "path": path,
        "resolved": resolved,
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

fn parse_admin_surface(value: Option<&str>) -> Option<AdminSurface> {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "ajax" | "admin-ajax" => Some(AdminSurface::Ajax),
        "admin-post" | "post" => Some(AdminSurface::AdminPost),
        "async-upload" | "upload" => Some(AdminSurface::AsyncUpload),
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
