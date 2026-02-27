use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

use axum::body::{to_bytes, Bytes};
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use wp_rs_admin::{core_admin_actions, AdminRequest, AdminSurface};
use wp_rs_auth::{resolve_current_user, sign_auth_cookie, AuthScheme, AuthSecrets, NonceService};
use wp_rs_config::{php_runtime_core_endpoints, RustGatewaySettings};
use wp_rs_content::{extract_block_names, parse_front_route, FrontRouteKind};
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

    let listen_addr =
        std::env::var("WP_RUST_SERVER_LISTEN").unwrap_or_else(|_| "127.0.0.1:8088".to_string());
    let address: SocketAddr = listen_addr.parse().expect("listen address must parse");

    let state = build_app_state();

    let app = Router::new()
        .route("/__wp_rust/health", get(health))
        .route("/__wp_rust/echo", get(echo))
        .route("/__wp_rust/proxy-decision", get(proxy_decision))
        .route("/__wp_rust/maintenance", any(maintenance_live_dispatch))
        .route("/wp-login.php", any(login_live_dispatch))
        .route("/wp-signup.php", any(signup_live_dispatch))
        .route("/wp-activate.php", any(activate_live_dispatch))
        .route("/wp-comments-post.php", any(comments_post_live_dispatch))
        .route("/wp-mail.php", any(mail_live_dispatch))
        .route("/wp-trackback.php", any(trackback_live_dispatch))
        .route("/wp-links-opml.php", any(links_opml_live_dispatch))
        .route("/wp-admin", get(admin_dashboard_live))
        .route("/wp-admin/", get(admin_dashboard_live))
        .route("/wp-admin/admin.php", any(admin_bootstrap_live_dispatch))
        .route("/wp-admin/install.php", any(install_live_dispatch))
        .route(
            "/wp-admin/setup-config.php",
            any(setup_config_live_dispatch),
        )
        .route(
            "/wp-admin/install-helper.php",
            any(install_helper_live_dispatch),
        )
        .route("/wp-admin/options.php", any(options_live_dispatch))
        .route(
            "/wp-admin/options-general.php",
            any(options_general_live_dispatch),
        )
        .route(
            "/wp-admin/options-writing.php",
            any(options_writing_live_dispatch),
        )
        .route(
            "/wp-admin/options-reading.php",
            any(options_reading_live_dispatch),
        )
        .route(
            "/wp-admin/options-discussion.php",
            any(options_discussion_live_dispatch),
        )
        .route(
            "/wp-admin/options-media.php",
            any(options_media_live_dispatch),
        )
        .route(
            "/wp-admin/options-permalink.php",
            any(options_permalink_live_dispatch),
        )
        .route(
            "/wp-admin/options-privacy.php",
            any(options_privacy_live_dispatch),
        )
        .route(
            "/wp-admin/privacy-policy-guide.php",
            any(privacy_policy_guide_live_dispatch),
        )
        .route("/wp-admin/tools.php", any(tools_live_dispatch))
        .route("/wp-admin/site-health.php", any(site_health_live_dispatch))
        .route("/wp-admin/export.php", any(export_live_dispatch))
        .route("/wp-admin/import.php", any(import_live_dispatch))
        .route(
            "/wp-admin/export-personal-data.php",
            any(export_personal_data_live_dispatch),
        )
        .route(
            "/wp-admin/erase-personal-data.php",
            any(erase_personal_data_live_dispatch),
        )
        .route("/wp-admin/network.php", any(network_live_dispatch))
        .route(
            "/wp-admin/network/setup.php",
            any(network_setup_live_dispatch),
        )
        .route("/wp-admin/network", any(network_index_live_dispatch))
        .route("/wp-admin/network/", any(network_index_live_dispatch))
        .route(
            "/wp-admin/network/index.php",
            any(network_index_live_dispatch),
        )
        .route(
            "/wp-admin/network/sites.php",
            any(network_sites_live_dispatch),
        )
        .route(
            "/wp-admin/network/users.php",
            any(network_users_live_dispatch),
        )
        .route(
            "/wp-admin/network/themes.php",
            any(network_themes_live_dispatch),
        )
        .route(
            "/wp-admin/network/plugins.php",
            any(network_plugins_live_dispatch),
        )
        .route(
            "/wp-admin/network/settings.php",
            any(network_settings_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-new.php",
            any(network_site_new_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-info.php",
            any(network_site_info_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-settings.php",
            any(network_site_settings_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-users.php",
            any(network_site_users_live_dispatch),
        )
        .route(
            "/wp-admin/network/site-themes.php",
            any(network_site_themes_live_dispatch),
        )
        .route(
            "/wp-admin/network/user-new.php",
            any(network_user_new_live_dispatch),
        )
        .route(
            "/wp-admin/network/edit.php",
            any(network_edit_live_dispatch),
        )
        .route(
            "/wp-admin/network/update.php",
            any(network_update_live_dispatch),
        )
        .route(
            "/wp-admin/network/update-core.php",
            any(network_update_core_live_dispatch),
        )
        .route(
            "/wp-admin/network/plugin-install.php",
            any(network_plugin_install_live_dispatch),
        )
        .route(
            "/wp-admin/network/plugin-editor.php",
            any(network_plugin_editor_live_dispatch),
        )
        .route(
            "/wp-admin/network/theme-editor.php",
            any(network_theme_editor_live_dispatch),
        )
        .route(
            "/wp-admin/network/privacy.php",
            any(network_privacy_live_dispatch),
        )
        .route(
            "/wp-admin/network/about.php",
            any(network_about_live_dispatch),
        )
        .route(
            "/wp-admin/network/credits.php",
            any(network_credits_live_dispatch),
        )
        .route(
            "/wp-admin/network/contribute.php",
            any(network_contribute_live_dispatch),
        )
        .route(
            "/wp-admin/network/freedoms.php",
            any(network_freedoms_live_dispatch),
        )
        .route(
            "/wp-admin/network/profile.php",
            any(network_profile_live_dispatch),
        )
        .route(
            "/wp-admin/network/user-edit.php",
            any(network_user_edit_live_dispatch),
        )
        .route(
            "/wp-admin/network/theme-install.php",
            any(network_theme_install_live_dispatch),
        )
        .route(
            "/wp-admin/ms-delete-site.php",
            any(ms_delete_site_live_dispatch),
        )
        .route("/wp-admin/upgrade.php", any(upgrade_live_dispatch))
        .route("/wp-admin/maint/repair.php", any(repair_live_dispatch))
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
        .route(
            "/__wp_rust/internal/maintenance-status",
            get(internal_maintenance_status),
        )
        .route(
            "/__wp_rust/internal/plugin-compat-matrix",
            get(internal_plugin_compat_matrix),
        )
        .route("/", get(front_live_dispatch_root))
        .route("/*front_path", get(front_live_dispatch))
        .fallback(not_found)
        .with_state(state)
        .layer(middleware::from_fn(latency_middleware));

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

async fn latency_middleware(request: Request, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let start = Instant::now();
    let mut response = next.run(request).await;
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    let elapsed_us = elapsed.as_micros();
    if let Ok(value) = HeaderValue::from_str(&elapsed_ms.to_string()) {
        response.headers_mut().insert("X-WP-Rust-Latency-Ms", value);
    }
    if let Ok(value) = HeaderValue::from_str(&elapsed_us.to_string()) {
        response
            .headers_mut()
            .insert("X-WP-Rust-Latency-Micros", value);
    }

    info!(
        method = %method,
        path = %path,
        status = %response.status().as_u16(),
        latency_ms = elapsed_ms,
        latency_us = elapsed_us,
        "request handled"
    );
    response
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
    deployment_profile: String,
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
    let should_route = settings.should_route(&query.endpoint);
    let mut methods = settings
        .method_allowlist
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    methods.sort();
    let response = ProxyDecisionResponse {
        enabled: settings.enabled,
        deployment_profile: settings.deployment_profile.clone(),
        fallback_enabled: settings.fallback_enabled,
        should_route,
        endpoint: query.endpoint,
        method_allowlist: methods,
        backend_url: settings.backend_url,
        timeout_ms: settings.timeout_ms,
        plugin_compat_mode: settings.plugin_compat_mode,
    };
    rust_handled_json(response)
}

async fn maintenance_live_dispatch() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/html; charset=UTF-8"),
    );
    headers.insert("Retry-After", HeaderValue::from_static("600"));

    (
        StatusCode::SERVICE_UNAVAILABLE,
        headers,
        "<!doctype html><html><body><h1>Maintenance</h1><p>Briefly unavailable for scheduled maintenance. Check back in a minute.</p></body></html>".to_string(),
    )
        .into_response()
}

async fn login_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let now = unix_now();
    let query_params = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let action = query_params
        .get("action")
        .cloned()
        .unwrap_or_else(|| "login".to_string());

    if action == "lostpassword" || action == "retrievepassword" {
        if parts.method == axum::http::Method::POST {
            let body_bytes = read_request_body(body).await;
            let content_type = parts
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
            let has_identity = params
                .get("user_login")
                .or_else(|| params.get("user_email"))
                .is_some_and(|value| !value.trim().is_empty());

            if !has_identity {
                return rust_handled_json_with_status(
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "missing_identity",
                        "message": "lostpassword requires user_login or user_email.",
                    }),
                )
                .into_response();
            }

            return rust_handled_redirect("/wp-login.php?checkemail=confirm").into_response();
        }

        return rust_handled_html(
            StatusCode::OK,
            "<!doctype html><html><body><h1>Lost Password (Rust)</h1></body></html>".to_string(),
        )
        .into_response();
    }

    if action == "rp" || action == "resetpass" {
        if parts.method == axum::http::Method::POST {
            let body_bytes = read_request_body(body).await;
            let content_type = parts
                .headers
                .get("content-type")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
            let has_reset_token = params
                .get("key")
                .or_else(|| query_params.get("key"))
                .is_some_and(|value| !value.trim().is_empty());
            let has_login = params
                .get("login")
                .or_else(|| query_params.get("login"))
                .is_some_and(|value| !value.trim().is_empty());
            let has_password = params
                .get("pass1")
                .or_else(|| params.get("password"))
                .is_some_and(|value| !value.trim().is_empty());

            if !has_reset_token || !has_login || !has_password {
                return rust_handled_json_with_status(
                    StatusCode::BAD_REQUEST,
                    json!({
                        "error": "invalid_reset_payload",
                        "message": "resetpass requires key, login, and pass1/password.",
                    }),
                )
                .into_response();
            }

            return rust_handled_redirect("/wp-login.php?password=changed").into_response();
        }

        let key = query_params.get("key").cloned().unwrap_or_default();
        let login = query_params.get("login").cloned().unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body><h1>Reset Password (Rust)</h1><p>login={login}</p><p>key={key}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if action == "logout" {
        let mut headers = HeaderMap::new();
        headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
        headers.insert(
            "Location",
            HeaderValue::from_static("/wp-login.php?loggedout=true"),
        );
        headers.append(
            "Set-Cookie",
            HeaderValue::from_static(
                "wordpress_logged_in_rust=deleted; Path=/; HttpOnly; Max-Age=0",
            ),
        );
        return (StatusCode::FOUND, headers, String::new()).into_response();
    }

    if parts.method == axum::http::Method::GET {
        let mut html = "<!doctype html><html><body><h1>WordPress Login (Rust)</h1>".to_string();
        if query_params.contains_key("loggedout") {
            html.push_str("<p>You are now logged out.</p>");
        }
        html.push_str("</body></html>");
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-login.php currently supports GET, POST, and logout action.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);

    let username = params
        .get("log")
        .or_else(|| params.get("username"))
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let password_present = params
        .get("pwd")
        .or_else(|| params.get("password"))
        .is_some_and(|value| !value.trim().is_empty());

    if username.is_empty() || !password_present {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_credentials",
                "message": "Missing username or password in login request.",
            }),
        )
        .into_response();
    }

    let user_id = params
        .get("user_id")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1);
    let expiration = now + 3_600;
    let session_token = format!("rust-session-{user_id}");
    let cookie_value = sign_auth_cookie(
        user_id,
        &username,
        expiration,
        &session_token,
        AuthScheme::LoggedIn,
        &state.auth_secrets,
    );

    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert("Location", HeaderValue::from_static("/wp-admin/"));
    if let Ok(cookie_header) = HeaderValue::from_str(&format!(
        "wordpress_logged_in_rust={cookie_value}; Path=/; HttpOnly"
    )) {
        headers.append("Set-Cookie", cookie_header);
    }

    let body = format!(
        "<!doctype html><html><body><h1>Login Success</h1><p>user={username}</p></body></html>"
    );
    (StatusCode::FOUND, headers, body).into_response()
}

async fn admin_dashboard_live(State(state): State<AppState>, request: Request) -> Response {
    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);

    if !authenticated {
        return rust_handled_redirect("/wp-login.php?redirect_to=%2Fwp-admin%2F").into_response();
    }

    let mut html = "<!doctype html><html><body><h1>WordPress Admin (Rust)</h1>".to_string();
    if !capabilities.is_empty() {
        html.push_str("<ul>");
        for capability in capabilities {
            html.push_str(&format!("<li>{capability}</li>"));
        }
        html.push_str("</ul>");
    }
    html.push_str("</body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn admin_bootstrap_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !authenticated {
        return rust_handled_redirect("/wp-login.php?redirect_to=%2Fwp-admin%2Fadmin.php")
            .into_response();
    }

    rust_handled_json(json!({
        "component": "admin-bootstrap",
        "authenticated": true,
        "capabilities": capabilities,
        "message": "Rust admin bootstrap compatibility shim loaded.",
    }))
    .into_response()
}

async fn install_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let query_params = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let step = query_params
        .get("step")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    if parts.method == axum::http::Method::GET {
        let html = format!(
            "<!doctype html><html><body><h1>WordPress Install (Rust)</h1><p>step={step}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let has_title = params
        .get("weblog_title")
        .or_else(|| params.get("site_title"))
        .is_some_and(|value| !value.trim().is_empty());
    let has_user = params
        .get("user_login")
        .or_else(|| params.get("user_name"))
        .is_some_and(|value| !value.trim().is_empty());
    let has_email = params
        .get("admin_email")
        .or_else(|| params.get("user_email"))
        .is_some_and(|value| !value.trim().is_empty());

    if !has_title || !has_user || !has_email {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_install_payload",
                "message": "Installation requires site title, username, and admin email.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-admin/?installed=1").into_response()
}

async fn setup_config_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/setup-config.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let step = params
        .get("step")
        .cloned()
        .unwrap_or_else(|| "0".to_string());
    if method == axum::http::Method::POST {
        let has_db_name = params
            .get("dbname")
            .is_some_and(|value| !value.trim().is_empty());
        let has_db_user = params
            .get("uname")
            .or_else(|| params.get("dbuser"))
            .is_some_and(|value| !value.trim().is_empty());
        if !has_db_name || !has_db_user {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_db_config_payload",
                    "message": "setup-config requires dbname and uname/dbuser fields.",
                }),
            )
            .into_response();
        }

        return rust_handled_redirect("/wp-admin/install.php?step=2").into_response();
    }

    let html = format!(
        "<!doctype html><html><body><h1>WordPress Setup Config (Rust)</h1><p>step={step}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn install_helper_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/install-helper.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    rust_handled_json(json!({
        "component": "install-helper",
        "status": "available",
        "message": "Rust install-helper compatibility shim loaded.",
    }))
    .into_response()
}

async fn options_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage = authenticated && capabilities.contains("manage_options");

    if !can_manage {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::GET {
        let options = state.options.lock().expect("options mutex poisoned");
        let snapshot = options.snapshot();
        let html = format!(
            "<!doctype html><html><body><h1>Options (Rust)</h1><p>count={}</p></body></html>",
            snapshot.len()
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);

    let mut options = state.options.lock().expect("options mutex poisoned");
    let mut updated_count = 0usize;
    for (key, value) in params {
        if matches!(
            key.as_str(),
            "action" | "option_page" | "_wpnonce" | "_wp_http_referer" | "submit"
        ) || key.starts_with('_')
        {
            continue;
        }
        options.set_option(&key, &value, false);
        updated_count += 1;
    }

    let target = format!(
        "/wp-admin/options-general.php?settings-updated=true&updated_count={updated_count}"
    );
    rust_handled_redirect(&target).into_response()
}

async fn options_general_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-general.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-general.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let blogname = options
        .get_option("blogname")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "WordPress".to_string());
    let blogdescription = options
        .get_option("blogdescription")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Just another WordPress site".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>General Settings (Rust)</h1><p>blogname={blogname}</p><p>blogdescription={blogdescription}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_writing_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-writing.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-writing.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let default_category = options
        .get_option("default_category")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let default_post_format = options
        .get_option("default_post_format")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Writing Settings (Rust)</h1><p>default_category={default_category}</p><p>default_post_format={default_post_format}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_reading_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-reading.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-reading.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let show_on_front = options
        .get_option("show_on_front")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "posts".to_string());
    let posts_per_page = options
        .get_option("posts_per_page")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "10".to_string());
    let page_on_front = options
        .get_option("page_on_front")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let page_for_posts = options
        .get_option("page_for_posts")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Reading Settings (Rust)</h1><p>show_on_front={show_on_front}</p><p>posts_per_page={posts_per_page}</p><p>page_on_front={page_on_front}</p><p>page_for_posts={page_for_posts}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_discussion_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-discussion.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-discussion.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let default_pingback_flag = options
        .get_option("default_pingback_flag")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let default_ping_status = options
        .get_option("default_ping_status")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "open".to_string());
    let default_comment_status = options
        .get_option("default_comment_status")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "open".to_string());
    let comments_notify = options
        .get_option("comments_notify")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let moderation_notify = options
        .get_option("moderation_notify")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Discussion Settings (Rust)</h1><p>default_pingback_flag={default_pingback_flag}</p><p>default_ping_status={default_ping_status}</p><p>default_comment_status={default_comment_status}</p><p>comments_notify={comments_notify}</p><p>moderation_notify={moderation_notify}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_media_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-media.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-media.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let thumbnail_size_w = options
        .get_option("thumbnail_size_w")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "150".to_string());
    let thumbnail_size_h = options
        .get_option("thumbnail_size_h")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "150".to_string());
    let medium_size_w = options
        .get_option("medium_size_w")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "300".to_string());
    let medium_size_h = options
        .get_option("medium_size_h")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "300".to_string());
    let large_size_w = options
        .get_option("large_size_w")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1024".to_string());
    let large_size_h = options
        .get_option("large_size_h")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1024".to_string());
    let uploads_use_yearmonth_folders = options
        .get_option("uploads_use_yearmonth_folders")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Media Settings (Rust)</h1><p>thumbnail_size_w={thumbnail_size_w}</p><p>thumbnail_size_h={thumbnail_size_h}</p><p>medium_size_w={medium_size_w}</p><p>medium_size_h={medium_size_h}</p><p>large_size_w={large_size_w}</p><p>large_size_h={large_size_h}</p><p>uploads_use_yearmonth_folders={uploads_use_yearmonth_folders}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_permalink_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-permalink.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    if !(authenticated && capabilities.contains("manage_options")) {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_options capability is required for wp-admin/options-permalink.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let permalink_structure = options
        .get_option("permalink_structure")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let category_base = options
        .get_option("category_base")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "category".to_string());
    let tag_base = options
        .get_option("tag_base")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "tag".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Permalink Settings (Rust)</h1><p>permalink_structure={permalink_structure}</p><p>category_base={category_base}</p><p>tag_base={tag_base}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn options_privacy_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/options-privacy.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_privacy = authenticated
        && (capabilities.contains("manage_privacy_options")
            || capabilities.contains("manage_options"));
    if !can_manage_privacy {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_privacy_options capability is required for wp-admin/options-privacy.php.",
            }),
        )
        .into_response();
    }

    let query_params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let show_policy_guide = query_params
        .get("tab")
        .is_some_and(|value| value.eq_ignore_ascii_case("policyguide"));

    let options = state.options.lock().expect("options mutex poisoned");
    let privacy_page_id = options
        .get_option("wp_page_for_privacy_policy")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let blog_public = options
        .get_option("blog_public")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "1".to_string());
    let html = if show_policy_guide {
        format!(
            "<!doctype html><html><body><h1>Privacy Policy Guide (Rust)</h1><p>wp_page_for_privacy_policy={privacy_page_id}</p><p>guide_mode=tab</p></body></html>"
        )
    } else {
        format!(
            "<!doctype html><html><body><h1>Privacy Settings (Rust)</h1><p>wp_page_for_privacy_policy={privacy_page_id}</p><p>blog_public={blog_public}</p></body></html>"
        )
    };
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn privacy_policy_guide_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/privacy-policy-guide.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_privacy = authenticated
        && (capabilities.contains("manage_privacy_options")
            || capabilities.contains("manage_options"));
    if !can_manage_privacy {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_privacy_options capability is required for wp-admin/privacy-policy-guide.php.",
            }),
        )
        .into_response();
    }

    let options = state.options.lock().expect("options mutex poisoned");
    let privacy_page_id = options
        .get_option("wp_page_for_privacy_policy")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Privacy Policy Guide (Rust)</h1><p>wp_page_for_privacy_policy={privacy_page_id}</p><p>guide_mode=direct</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn tools_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/tools.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_access_tools = authenticated
        && (capabilities.contains("edit_posts")
            || capabilities.contains("manage_options")
            || capabilities.contains("import"));
    if !can_access_tools {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_posts capability is required for wp-admin/tools.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    if params.contains_key("wp-privacy-policy-guide") {
        return rust_handled_redirect("/wp-admin/options-privacy.php?tab=policyguide")
            .into_response();
    }

    if let Some(page) = params.get("page") {
        if page == "export_personal_data" {
            return rust_handled_redirect("/wp-admin/export-personal-data.php").into_response();
        }
        if page == "remove_personal_data" {
            return rust_handled_redirect("/wp-admin/erase-personal-data.php").into_response();
        }
    }

    let html = "<!doctype html><html><body><h1>Tools (Rust)</h1><p>status=available</p><p>converter=categories-tags</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn site_health_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/site-health.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_view_site_health = authenticated
        && (capabilities.contains("view_site_health_checks")
            || capabilities.contains("manage_options"));
    if !can_view_site_health {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "view_site_health_checks capability is required for wp-admin/site-health.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let tab = params
        .get("tab")
        .cloned()
        .unwrap_or_else(|| "status".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Site Health (Rust)</h1><p>tab={tab}</p><p>status=available</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn export_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/export.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_export = authenticated
        && (capabilities.contains("export") || capabilities.contains("manage_options"));
    if !can_export {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "export capability is required for wp-admin/export.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    if params.contains_key("download") {
        let requested_content = params
            .get("content")
            .cloned()
            .unwrap_or_else(|| "all".to_string());
        let export_xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"><channel><title>WordPress Export (Rust)</title><exported_content>{requested_content}</exported_content></channel></rss>"
        );
        let mut response =
            rust_handled_text(StatusCode::OK, "application/xml; charset=UTF-8", export_xml)
                .into_response();
        if let Ok(value) =
            HeaderValue::from_str("attachment; filename=\"wordpress-rust-export.xml\"")
        {
            response.headers_mut().insert("Content-Disposition", value);
        }
        return response;
    }

    let html = "<!doctype html><html><body><h1>Export (Rust)</h1><p>status=ready</p></body></html>"
        .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn import_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/import.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_import = authenticated
        && (capabilities.contains("import")
            || capabilities.contains("install_plugins")
            || capabilities.contains("manage_options"));
    if !can_import {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "import capability is required for wp-admin/import.php.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let invalid = params.get("invalid").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Import (Rust)</h1><p>status=ready</p><p>invalid={invalid}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn export_personal_data_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/export-personal-data.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_export_personal_data = authenticated
        && (capabilities.contains("export_others_personal_data")
            || capabilities.contains("manage_options"));
    if !can_export_personal_data {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "export_others_personal_data capability is required for wp-admin/export-personal-data.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
        let requested_identity = params
            .get("username_or_email_for_privacy_request")
            .cloned()
            .unwrap_or_default();
        return rust_handled_json_with_status(
            StatusCode::OK,
            json!({
                "status": "request_registered",
                "requester": requested_identity,
            }),
        )
        .into_response();
    }

    let html = "<!doctype html><html><body><h1>Export Personal Data (Rust)</h1><p>status=ready</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn erase_personal_data_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/erase-personal-data.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let has_erase_caps = capabilities.contains("erase_others_personal_data")
        && capabilities.contains("delete_users");
    let can_erase_personal_data =
        authenticated && (has_erase_caps || capabilities.contains("manage_options"));
    if !can_erase_personal_data {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "erase_others_personal_data and delete_users capabilities are required for wp-admin/erase-personal-data.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
        let requested_identity = params
            .get("username_or_email_for_privacy_request")
            .cloned()
            .unwrap_or_default();
        return rust_handled_json_with_status(
            StatusCode::OK,
            json!({
                "status": "erasure_request_registered",
                "requester": requested_identity,
            }),
        )
        .into_response();
    }

    let html = "<!doctype html><html><body><h1>Erase Personal Data (Rust)</h1><p>status=ready</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_setup_network = authenticated
        && (capabilities.contains("setup_network") || capabilities.contains("manage_options"));
    if !can_setup_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "setup_network capability is required for wp-admin/network.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
        let has_sitename = params
            .get("sitename")
            .is_some_and(|value| !value.trim().is_empty());
        let has_email = params
            .get("email")
            .is_some_and(|value| !value.trim().is_empty());
        if !has_sitename || !has_email {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_network_payload",
                    "message": "network.php requires sitename and email.",
                }),
            )
            .into_response();
        }
        return rust_handled_redirect("/wp-admin/network.php?installed=1").into_response();
    }

    let html =
        "<!doctype html><html><body><h1>Network Setup (Rust)</h1><p>status=ready</p></body></html>"
            .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_setup_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    network_live_dispatch(State(state), request).await
}

async fn network_index_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/index.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(request.headers(), &state.auth_secrets);
    let can_manage_network = authenticated
        && (capabilities.contains("manage_network") || capabilities.contains("manage_options"));
    if !can_manage_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for wp-admin/network/index.php.",
            }),
        )
        .into_response();
    }

    let html = "<!doctype html><html><body><h1>Network Dashboard (Rust)</h1><p>status=ready</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_sites_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/sites.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/sites.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if params.get("action").is_some_and(|value| value == "confirm") {
        let action2 = params.get("action2").cloned().unwrap_or_default();
        let site_id = params.get("id").cloned().unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body><h1>Network Sites Confirm (Rust)</h1><p>action2={action2}</p><p>id={site_id}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if method == axum::http::Method::POST || params.contains_key("action") {
        let action = params.get("action").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/sites.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let html =
        "<!doctype html><html><body><h1>Network Sites (Rust)</h1><p>status=ready</p></body></html>"
            .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_users_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/users.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_users = authenticated
        && (capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_network_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_users capability is required for wp-admin/network/users.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if params
        .get("action")
        .is_some_and(|value| value == "deleteuser")
    {
        let user_id = params.get("id").cloned().unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body><h1>Network Users Delete (Rust)</h1><p>id={user_id}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "allusers")
    {
        let bulk_action = params.get("bulk_action").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/users.php?updated=true&action={bulk_action}");
        return rust_handled_redirect(&target).into_response();
    }

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "dodelete")
    {
        return rust_handled_redirect("/wp-admin/network/users.php?updated=true&action=delete")
            .into_response();
    }

    let html =
        "<!doctype html><html><body><h1>Network Users (Rust)</h1><p>status=ready</p></body></html>"
            .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_themes_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/themes.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_themes = authenticated
        && (capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_network_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_themes capability is required for wp-admin/network/themes.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if action == "enable" {
        return rust_handled_redirect("/wp-admin/network/themes.php?enabled=1").into_response();
    }
    if action == "disable" {
        return rust_handled_redirect("/wp-admin/network/themes.php?disabled=1").into_response();
    }
    if action == "enable-selected" {
        return rust_handled_redirect("/wp-admin/network/themes.php?enabled=1").into_response();
    }
    if action == "disable-selected" {
        return rust_handled_redirect("/wp-admin/network/themes.php?disabled=1").into_response();
    }
    if action == "delete-selected" {
        return rust_handled_redirect("/wp-admin/network/themes.php?deleted=1").into_response();
    }
    if action == "update-selected" {
        let html =
            "<!doctype html><html><body><h1>Network Themes Update (Rust)</h1><p>status=in_progress</p></body></html>"
                .to_string();
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    let search = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Themes (Rust)</h1><p>status=ready</p><p>search={search}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_plugins_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/plugins.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_plugins = authenticated
        && (capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network")
            || capabilities.contains("activate_plugins")
            || capabilities.contains("manage_options"));
    if !can_manage_network_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_plugins capability is required for wp-admin/network/plugins.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if !action.is_empty() {
        let target = format!("/wp-admin/network/plugins.php?updated_action={action}");
        return rust_handled_redirect(&target).into_response();
    }

    let html = "<!doctype html><html><body><h1>Network Plugins (Rust)</h1><p>status=ready</p></body></html>"
        .to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_settings_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/settings.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network_options = authenticated
        && (capabilities.contains("manage_network_options")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_network_options {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_options capability is required for wp-admin/network/settings.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if let Some(hash) = params.get("network_admin_hash") {
        if hash == "confirm-rust-admin-email" {
            return rust_handled_redirect("/wp-admin/network/settings.php?updated=true")
                .into_response();
        }
        return rust_handled_redirect("/wp-admin/network/settings.php?updated=false")
            .into_response();
    }

    if params
        .get("dismiss")
        .is_some_and(|value| value == "new_network_admin_email")
    {
        return rust_handled_redirect("/wp-admin/network/settings.php?updated=true")
            .into_response();
    }

    if method == axum::http::Method::POST {
        let site_name = params
            .get("site_name")
            .cloned()
            .unwrap_or_else(|| "WordPress Network".to_string());
        let new_admin_email = params.get("new_admin_email").cloned().unwrap_or_default();
        let registration = params.get("registration").cloned().unwrap_or_default();
        {
            let mut options = state.options.lock().expect("options mutex poisoned");
            options.set_option("network_site_name", &site_name, false);
            if !new_admin_email.is_empty() {
                options.set_option("network_admin_email", &new_admin_email, false);
            }
            if !registration.is_empty() {
                options.set_option("network_registration", &registration, false);
            }
        }
        return rust_handled_redirect("/wp-admin/network/settings.php?updated=true")
            .into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let options = state.options.lock().expect("options mutex poisoned");
    let site_name = options
        .get_option("network_site_name")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "WordPress Network".to_string());
    let network_admin_email = options
        .get_option("network_admin_email")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "admin@example.com".to_string());
    let html = format!(
        "<!doctype html><html><body><h1>Network Settings (Rust)</h1><p>updated={updated}</p><p>site_name={site_name}</p><p>admin_email={network_admin_email}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_new_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_create_sites = authenticated
        && (capabilities.contains("create_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_create_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "create_sites capability is required for wp-admin/network/site-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "add-site" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_new_action",
                    "message": "site-new.php requires action=add-site for POST requests.",
                }),
            )
            .into_response();
        }

        let domain = params
            .get("blog[domain]")
            .or_else(|| params.get("domain"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let title = params
            .get("blog[title]")
            .or_else(|| params.get("title"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let email = params
            .get("blog[email]")
            .or_else(|| params.get("email"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();

        if domain.is_empty() || title.is_empty() || email.is_empty() || !email.contains('@') {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_new_payload",
                    "message": "site-new.php requires blog[domain], blog[title], and a valid blog[email].",
                }),
            )
            .into_response();
        }

        let site_id = params
            .get("site_id")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "2".to_string());
        let target = format!("/wp-admin/network/site-new.php?update=added&id={site_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let site_id = params.get("id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Add Site (Rust)</h1><p>update={update}</p><p>id={site_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_info_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-info.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-info.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-info.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "update-site" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_info_action",
                    "message": "site-info.php requires action=update-site for POST requests.",
                }),
            )
            .into_response();
        }
        let target = format!("/wp-admin/network/site-info.php?update=updated&id={site_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site (Rust)</h1><p>id={site_id}</p><p>update={update}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_settings_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-settings.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-settings.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-settings.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "update-site" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_site_settings_action",
                    "message": "site-settings.php requires action=update-site for POST requests.",
                }),
            )
            .into_response();
        }

        let mut updated_count = 0usize;
        {
            let mut options = state.options.lock().expect("options mutex poisoned");
            for (key, value) in &params {
                if let Some(option_name) = key
                    .strip_prefix("option[")
                    .and_then(|suffix| suffix.strip_suffix(']'))
                {
                    if option_name.is_empty() {
                        continue;
                    }
                    let namespaced_key = format!("site_{site_id}_{option_name}");
                    options.set_option(&namespaced_key, value, false);
                    updated_count += 1;
                }
            }
        }

        let target = format!(
            "/wp-admin/network/site-settings.php?update=updated&id={site_id}&updated_count={updated_count}"
        );
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let updated_count = params.get("updated_count").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site Settings (Rust)</h1><p>id={site_id}</p><p>update={update}</p><p>updated_count={updated_count}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_users_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-users.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-users.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-users.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let update = match action.as_str() {
            "adduser" => "adduser",
            "newuser" => "newuser",
            "remove" => "remove",
            "promote" => "promote",
            "update-site" => "updated",
            _ if !action.is_empty() => "updated",
            _ => "updated",
        };
        let target = format!("/wp-admin/network/site-users.php?id={site_id}&update={update}");
        return rust_handled_redirect(&target).into_response();
    }

    if action == "update-site" {
        let target = format!("/wp-admin/network/site-users.php?id={site_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site Users (Rust)</h1><p>id={site_id}</p><p>update={update}</p><p>action={action}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_site_themes_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/site-themes.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_sites = authenticated
        && (capabilities.contains("manage_sites")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_sites {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_sites capability is required for wp-admin/network/site-themes.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let site_id = params
        .get("id")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or_default();
    if site_id == 0 {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_site_id",
                "message": "site-themes.php requires a valid id query parameter.",
            }),
        )
        .into_response();
    }

    let action = params.get("action").cloned().unwrap_or_default();
    if method == axum::http::Method::POST || !action.is_empty() {
        let (status_key, count) = match action.as_str() {
            "enable" | "enable-selected" => ("enabled", "1"),
            "disable" | "disable-selected" => ("disabled", "1"),
            _ => ("error", "none"),
        };
        let target = format!("/wp-admin/network/site-themes.php?id={site_id}&{status_key}={count}");
        return rust_handled_redirect(&target).into_response();
    }

    let enabled = params.get("enabled").cloned().unwrap_or_default();
    let disabled = params.get("disabled").cloned().unwrap_or_default();
    let error = params.get("error").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Edit Site Themes (Rust)</h1><p>id={site_id}</p><p>enabled={enabled}</p><p>disabled={disabled}</p><p>error={error}</p><p>search={search}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_user_new_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/user-new.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_create_users = authenticated
        && (capabilities.contains("create_users")
            || capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_create_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "create_users capability is required for wp-admin/network/user-new.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        let action = params.get("action").cloned().unwrap_or_default();
        if action != "add-user" {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_user_new_action",
                    "message": "user-new.php requires action=add-user for POST requests.",
                }),
            )
            .into_response();
        }

        let username = params
            .get("user[username]")
            .or_else(|| params.get("username"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let email = params
            .get("user[email]")
            .or_else(|| params.get("email"))
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        if username.is_empty() || email.is_empty() || !email.contains('@') {
            return rust_handled_json_with_status(
                StatusCode::BAD_REQUEST,
                json!({
                    "error": "invalid_user_new_payload",
                    "message": "user-new.php requires user[username] and valid user[email].",
                }),
            )
            .into_response();
        }

        let user_id = params
            .get("user_id")
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "2".to_string());
        let target = format!("/wp-admin/network/user-new.php?update=added&user_id={user_id}");
        return rust_handled_redirect(&target).into_response();
    }

    let update = params.get("update").cloned().unwrap_or_default();
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Add User (Rust)</h1><p>update={update}</p><p>user_id={user_id}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_edit_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_network = authenticated
        && (capabilities.contains("manage_network") || capabilities.contains("manage_options"));
    if !can_manage_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for wp-admin/network/edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    if action.trim().is_empty() {
        return rust_handled_redirect("/wp-admin/network/").into_response();
    }

    let target = format!("/wp-admin/network/?updated_action={action}");
    rust_handled_redirect(&target).into_response()
}

async fn network_update_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/update.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_update_network = authenticated
        && (capabilities.contains("update_plugins")
            || capabilities.contains("update_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_update_network {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "network update capability is required for wp-admin/network/update.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    let iframe_request = matches!(
        action.as_str(),
        "update-selected" | "activate-plugin" | "update-selected-themes"
    );
    let html = format!(
        "<!doctype html><html><body><h1>Network Update (Rust)</h1><p>action={action}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_update_core_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/update-core.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_update_core = authenticated
        && (capabilities.contains("update_core")
            || capabilities.contains("update_plugins")
            || capabilities.contains("update_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_update_core {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "update_core capability is required for wp-admin/network/update-core.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let action = params.get("action").cloned().unwrap_or_default();
    let operation = if method == axum::http::Method::POST {
        "apply"
    } else {
        "preview"
    };
    let html = format!(
        "<!doctype html><html><body><h1>Network Update Core (Rust)</h1><p>action={action}</p><p>operation={operation}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_plugin_install_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/plugin-install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_install_plugins = authenticated
        && (capabilities.contains("install_plugins")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_install_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "install_plugins capability is required for wp-admin/network/plugin-install.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tab = params.get("tab").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let iframe_request = tab == "plugin-information";
    let html = format!(
        "<!doctype html><html><body><h1>Network Plugin Install (Rust)</h1><p>tab={tab}</p><p>search={search}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_plugin_editor_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/plugin-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_plugins = authenticated
        && (capabilities.contains("edit_plugins")
            || capabilities.contains("manage_network_plugins")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_edit_plugins {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_plugins capability is required for wp-admin/network/plugin-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("update"))
    {
        let plugin = params.get("plugin").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/plugin-editor.php?plugin={plugin}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let plugin = params.get("plugin").cloned().unwrap_or_default();
    let file = params.get("file").cloned().unwrap_or_default();
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Plugin Editor (Rust)</h1><p>plugin={plugin}</p><p>file={file}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_theme_editor_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/theme-editor.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_themes = authenticated
        && (capabilities.contains("edit_themes")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_edit_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_themes capability is required for wp-admin/network/theme-editor.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value.eq_ignore_ascii_case("update"))
    {
        let theme = params.get("theme").cloned().unwrap_or_default();
        let target = format!("/wp-admin/network/theme-editor.php?theme={theme}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let theme = params.get("theme").cloned().unwrap_or_default();
    let file = params.get("file").cloned().unwrap_or_default();
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Theme Editor (Rust)</h1><p>theme={theme}</p><p>file={file}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_privacy_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/privacy.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_privacy = authenticated
        && (capabilities.contains("manage_network_options")
            || capabilities.contains("manage_privacy_options")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_privacy {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network_options capability is required for wp-admin/network/privacy.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/network/privacy.php?updated=true").into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Privacy (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_about_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    network_information_page_live_dispatch(state, request, "Network About (Rust)").await
}

async fn network_credits_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    network_information_page_live_dispatch(state, request, "Network Credits (Rust)").await
}

async fn network_contribute_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    network_information_page_live_dispatch(state, request, "Network Contribute (Rust)").await
}

async fn network_freedoms_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    network_information_page_live_dispatch(state, request, "Network Freedoms (Rust)").await
}

async fn network_information_page_live_dispatch(
    state: AppState,
    request: Request,
    title: &str,
) -> Response {
    let (parts, _body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "network informational pages currently support GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_view = authenticated
        && (capabilities.contains("manage_network")
            || capabilities.contains("manage_options")
            || capabilities.contains("read"));
    if !can_view {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "manage_network capability is required for this network page.",
            }),
        )
        .into_response();
    }

    let html = format!("<!doctype html><html><body><h1>{title}</h1></body></html>");
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_profile_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, _body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/profile.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_manage_profile = authenticated
        && (capabilities.contains("read")
            || capabilities.contains("edit_user")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_manage_profile {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "read capability is required for wp-admin/network/profile.php.",
            }),
        )
        .into_response();
    }

    if method == axum::http::Method::POST {
        return rust_handled_redirect("/wp-admin/network/profile.php?updated=true").into_response();
    }

    let params = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network Profile (Rust)</h1><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_user_edit_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/user-edit.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_edit_users = authenticated
        && (capabilities.contains("edit_users")
            || capabilities.contains("promote_users")
            || capabilities.contains("manage_network_users")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_edit_users {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "edit_users capability is required for wp-admin/network/user-edit.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let user_id = params.get("user_id").cloned().unwrap_or_default();
    if method == axum::http::Method::POST {
        let target = format!("/wp-admin/network/user-edit.php?user_id={user_id}&updated=true");
        return rust_handled_redirect(&target).into_response();
    }

    let updated = params.get("updated").cloned().unwrap_or_default();
    let html = format!(
        "<!doctype html><html><body><h1>Network User Edit (Rust)</h1><p>user_id={user_id}</p><p>updated={updated}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn network_theme_install_live_dispatch(
    State(state): State<AppState>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/network/theme-install.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_install_themes = authenticated
        && (capabilities.contains("install_themes")
            || capabilities.contains("manage_network_themes")
            || capabilities.contains("manage_network")
            || capabilities.contains("manage_options"));
    if !can_install_themes {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "install_themes capability is required for wp-admin/network/theme-install.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    let tab = params.get("tab").cloned().unwrap_or_default();
    let search = params.get("s").cloned().unwrap_or_default();
    let iframe_request = tab == "theme-information";
    let html = format!(
        "<!doctype html><html><body><h1>Network Theme Install (Rust)</h1><p>tab={tab}</p><p>search={search}</p><p>iframe_request={iframe_request}</p></body></html>"
    );
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn ms_delete_site_live_dispatch(State(state): State<AppState>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();
    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/ms-delete-site.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let (authenticated, capabilities) =
        auth_context_from_headers(&parts.headers, &state.auth_secrets);
    let can_delete_site = authenticated
        && (capabilities.contains("delete_site") || capabilities.contains("manage_options"));
    if !can_delete_site {
        return rust_handled_json_with_status(
            StatusCode::FORBIDDEN,
            json!({
                "error": "rest_forbidden",
                "message": "delete_site capability is required for wp-admin/ms-delete-site.php.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if let Some(hash) = params.get("h") {
        if hash == "confirm-rust-delete" {
            let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>site_deleted=true</p></body></html>".to_string();
            return rust_handled_html(StatusCode::OK, html).into_response();
        }
        let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>error=stale_link</p></body></html>".to_string();
        return rust_handled_html(StatusCode::BAD_REQUEST, html).into_response();
    }

    if method == axum::http::Method::POST
        && params
            .get("action")
            .is_some_and(|value| value == "deleteblog")
        && params
            .get("confirmdelete")
            .is_some_and(|value| value == "1")
    {
        let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>delete_request=queued</p></body></html>".to_string();
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    let html = "<!doctype html><html><body><h1>Delete Site (Rust)</h1><p>status=confirm_required</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn upgrade_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let method = parts.method.clone();

    if method != axum::http::Method::GET && method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/upgrade.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let params = if method == axum::http::Method::POST {
        let body_bytes = read_request_body(body).await;
        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        merged_params(parts.uri.query(), &body_bytes, &content_type)
    } else {
        parse_urlencoded(parts.uri.query().unwrap_or_default())
    };

    if params
        .get("step")
        .is_some_and(|value| value.eq_ignore_ascii_case("upgrade_db"))
    {
        return rust_handled_text(StatusCode::OK, "text/plain; charset=UTF-8", "0".to_string())
            .into_response();
    }

    let html = "<!doctype html><html><body><h1>WordPress Upgrade (Rust)</h1><p>No update required.</p></body></html>".to_string();
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn repair_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-admin/maint/repair.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let params = parse_urlencoded(request.uri().query().unwrap_or_default());
    let html = if let Some(mode) = params.get("repair") {
        let optimize = mode == "2";
        format!(
            "<!doctype html><html><body><h1>Database Repair (Rust)</h1><p>repair_run=true</p><p>optimize={optimize}</p></body></html>"
        )
    } else {
        "<!doctype html><html><body><h1>Database Repair (Rust)</h1><p>WP_ALLOW_REPAIR must be enabled.</p></body></html>".to_string()
    };
    rust_handled_html(StatusCode::OK, html).into_response()
}

async fn signup_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if parts.method == axum::http::Method::GET {
        let html = "<!doctype html><html><body><h1>Sign Up (Rust)</h1></body></html>".to_string();
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-signup.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let has_identity = params
        .get("user_login")
        .or_else(|| params.get("user_email"))
        .is_some_and(|value| !value.trim().is_empty());
    if !has_identity {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "missing_signup_identity",
                "message": "Provide user_login or user_email for signup.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-signup.php?checkemail=registered").into_response()
}

async fn activate_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let query = parse_urlencoded(parts.uri.query().unwrap_or_default());
    let key_from_query = query.get("key").cloned();

    if parts.method == axum::http::Method::GET {
        let activation_state = if key_from_query
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            "ready"
        } else {
            "missing_key"
        };
        let html = format!(
            "<!doctype html><html><body><h1>Activate Account (Rust)</h1><p>state={activation_state}</p></body></html>"
        );
        return rust_handled_html(StatusCode::OK, html).into_response();
    }

    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-activate.php currently supports GET and POST.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let key = params
        .get("key")
        .cloned()
        .or(key_from_query)
        .unwrap_or_default();
    if key.trim().is_empty() {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "missing_activation_key",
                "message": "Provide activation key to continue.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect("/wp-login.php?checkemail=activated").into_response()
}

async fn comments_post_live_dispatch(request: Request) -> Response {
    let (parts, body) = request.into_parts();
    if parts.method != axum::http::Method::POST {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-comments-post.php currently supports POST only.",
            }),
        )
        .into_response();
    }

    let body_bytes = read_request_body(body).await;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let params = merged_params(parts.uri.query(), &body_bytes, &content_type);
    let comment_post_id = params.get("comment_post_ID").cloned().unwrap_or_default();
    let has_comment_body = params
        .get("comment")
        .is_some_and(|value| !value.trim().is_empty());

    if comment_post_id.trim().is_empty() || !has_comment_body {
        return rust_handled_json_with_status(
            StatusCode::BAD_REQUEST,
            json!({
                "error": "invalid_comment_payload",
                "message": "comment_post_ID and comment are required.",
            }),
        )
        .into_response();
    }

    rust_handled_redirect(&format!("/?p={comment_post_id}#comment-rust")).into_response()
}

async fn mail_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-mail.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    rust_handled_json(json!({
        "processed": false,
        "message": "wp-mail processing scaffolded in Rust; mailbox polling not yet implemented.",
    }))
    .into_response()
}

async fn trackback_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::POST {
        return rust_handled_text(
            StatusCode::OK,
            "text/plain; charset=UTF-8",
            "0\nRust trackback endpoint ready.\n".to_string(),
        )
        .into_response();
    }

    rust_handled_text(
        StatusCode::OK,
        "text/plain; charset=UTF-8",
        "0\nRust trackback accepted.\n".to_string(),
    )
    .into_response()
}

async fn links_opml_live_dispatch(request: Request) -> Response {
    if request.method() != axum::http::Method::GET {
        return rust_handled_json_with_status(
            StatusCode::METHOD_NOT_ALLOWED,
            json!({
                "error": "method_not_allowed",
                "message": "wp-links-opml.php currently supports GET only.",
            }),
        )
        .into_response();
    }

    let opml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="1.0">
  <head><title>WordPress Links (Rust)</title></head>
  <body>
    <outline text="WordPress.org" title="WordPress.org" type="link" xmlUrl="https://wordpress.org/news/feed/" htmlUrl="https://wordpress.org/"/>
  </body>
</opml>"#
        .to_string();
    rust_handled_text(StatusCode::OK, "text/xml; charset=UTF-8", opml).into_response()
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

async fn front_live_dispatch_root(
    State(state): State<AppState>,
    request: Request,
) -> impl IntoResponse {
    front_live_dispatch_inner(state, request, "/".to_string()).await
}

async fn front_live_dispatch(
    State(state): State<AppState>,
    Path(front_path): Path<String>,
    request: Request,
) -> impl IntoResponse {
    let path = format!("/{}", front_path.trim_start_matches('/'));
    front_live_dispatch_inner(state, request, path).await
}

async fn front_live_dispatch_inner(state: AppState, request: Request, path: String) -> Response {
    if path.starts_with("/__wp_rust/")
        || path.starts_with("/wp-json")
        || path.starts_with("/wp-admin/")
        || path == "/xmlrpc.php"
        || path == "/wp-cron.php"
    {
        return not_found(request).await.into_response();
    }

    let query = request.uri().query().unwrap_or_default();
    let matched = parse_front_route(&path, query);
    let resolved_site = resolve_multisite_site(&state, request.headers(), &matched.request.path);

    if let Some(target) = &matched.canonical_redirect {
        if *target != path {
            let redirect_target = if query.is_empty() {
                target.clone()
            } else {
                format!("{target}?{query}")
            };
            return rust_handled_redirect(&redirect_target).into_response();
        }
    }

    match matched.kind {
        FrontRouteKind::Feed => {
            let feed_title = resolved_site
                .as_ref()
                .map(|site| format!("WordPress Feed (blog {})", site.blog_id))
                .unwrap_or_else(|| "WordPress Feed".to_string());
            let xml = format!(
                "<?xml version=\"1.0\"?><rss><channel><title>{}</title><description>Rust feed route</description><link>{}</link></channel></rss>",
                feed_title, matched.request.path
            );
            rust_handled_xml(StatusCode::OK, xml, true).into_response()
        }
        FrontRouteKind::NotFound => rust_handled_html(
            StatusCode::NOT_FOUND,
            "<!doctype html><html><body><h1>404 Not Found</h1></body></html>".to_string(),
        )
        .into_response(),
        _ => {
            let title = match matched.kind {
                FrontRouteKind::Home => "Home",
                FrontRouteKind::Single => "Single",
                FrontRouteKind::Page => "Page",
                FrontRouteKind::Archive => "Archive",
                FrontRouteKind::Search => "Search",
                FrontRouteKind::Feed | FrontRouteKind::NotFound => "Front",
            };
            let mut html = format!(
                "<!doctype html><html><body><h1>{title}</h1><p>path={}</p>",
                matched.request.path
            );
            if let Some(site) = &resolved_site {
                html.push_str(&format!(
                    "<p>blog_id={}</p><p>network_domain={}</p><p>network_path={}</p>",
                    site.blog_id, site.domain, site.path
                ));
            }
            if !matched.query_vars.is_empty() {
                html.push_str("<ul>");
                for (key, value) in matched.query_vars {
                    html.push_str(&format!("<li>{key}={value}</li>"));
                }
                html.push_str("</ul>");
            }
            html.push_str("</body></html>");
            rust_handled_html(StatusCode::OK, html).into_response()
        }
    }
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

fn rust_handled_html(status: StatusCode, body: String) -> (StatusCode, HeaderMap, String) {
    rust_handled_text(status, "text/html; charset=UTF-8", body)
}

fn rust_handled_text(
    status: StatusCode,
    content_type: &'static str,
    body: String,
) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    headers.insert("Content-Type", HeaderValue::from_static(content_type));
    (status, headers, body)
}

fn rust_handled_redirect(target: &str) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert("X-WP-Rust-Handled", HeaderValue::from_static("1"));
    if let Ok(location) = HeaderValue::from_str(target) {
        headers.insert("Location", location);
    }
    (StatusCode::MOVED_PERMANENTLY, headers, String::new())
}

fn resolve_multisite_site(
    state: &AppState,
    headers: &HeaderMap,
    request_path: &str,
) -> Option<NetworkSite> {
    let forwarded_host = headers
        .get("x-forwarded-host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let host = if forwarded_host.trim().is_empty() {
        headers
            .get("host")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("example.com")
    } else {
        forwarded_host
    };
    let domain = host.split(':').next().unwrap_or("example.com").trim();
    if domain.is_empty() {
        return None;
    }

    state.multisite_resolver.resolve(domain, request_path)
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

#[derive(Debug, Deserialize)]
struct InternalMaintenanceStatusQuery {
    upgrading: Option<u64>,
}

async fn internal_maintenance_status(
    Query(query): Query<InternalMaintenanceStatusQuery>,
) -> impl IntoResponse {
    let now = unix_now();
    let upgrading = query.upgrading.unwrap_or(0);
    let age_secs = if upgrading > 0 {
        now.saturating_sub(upgrading)
    } else {
        0
    };
    let stale = upgrading > 0 && age_secs >= 600;
    let active = upgrading > 0 && !stale;

    rust_handled_json(json!({
        "active": active,
        "upgrading": if upgrading > 0 { Some(upgrading) } else { None::<u64> },
        "age_secs": age_secs,
        "stale": stale,
        "retry_after_secs": 600,
        "now": now,
    }))
}

async fn internal_plugin_compat_matrix() -> impl IntoResponse {
    let settings = RustGatewaySettings::from_env();
    let core_endpoints = php_runtime_core_endpoints()
        .iter()
        .map(|endpoint| endpoint.to_string())
        .collect::<Vec<_>>();

    rust_handled_json(json!({
        "plugin_compat_mode": settings.plugin_compat_mode,
        "deployment_profile": settings.deployment_profile,
        "endpoint_allowlist": settings.endpoint_allowlist,
        "method_allowlist": settings.method_allowlist,
        "php_runtime_core_endpoints": core_endpoints,
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
