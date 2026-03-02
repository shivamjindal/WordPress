use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthRequirement {
    Public,
    Authenticated,
    Capability(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestRoute {
    pub path: String,
    pub methods: Vec<String>,
    pub summary: String,
    pub auth_requirement: AuthRequirement,
}

#[derive(Debug, Default, Clone)]
pub struct RestRouteRegistry {
    routes: Vec<RestRoute>,
}

impl RestRouteRegistry {
    pub fn register(
        &mut self,
        path: impl Into<String>,
        methods: &[&str],
        summary: impl Into<String>,
        auth_requirement: AuthRequirement,
    ) {
        self.routes.push(RestRoute {
            path: path.into(),
            methods: methods.iter().map(|method| (*method).to_string()).collect(),
            summary: summary.into(),
            auth_requirement,
        });
    }

    pub fn all(&self) -> &[RestRoute] {
        &self.routes
    }

    pub fn contract_document(&self) -> Value {
        let mut paths = BTreeMap::<String, BTreeMap<String, Value>>::new();
        for route in &self.routes {
            let method_map = paths.entry(route.path.clone()).or_default();
            for method in &route.methods {
                method_map.insert(
                    method.to_ascii_lowercase(),
                    json!({
                        "summary": route.summary,
                        "auth_requirement": format_auth_requirement(&route.auth_requirement),
                    }),
                );
            }
        }

        json!({
            "openapi": "3.0.0",
            "info": {
                "title": "wp-rs REST contract",
                "version": "0.1.0",
            },
            "paths": paths,
        })
    }

    pub fn dispatch(&self, request: &RestRequest) -> RestDispatchResult {
        let request_method = request.method.trim().to_ascii_uppercase();
        let request_path = normalize_path(&request.path);
        let matching_path_routes = self
            .routes
            .iter()
            .filter(|route| normalize_path(&route.path) == request_path)
            .collect::<Vec<_>>();
        let allowed_methods = collect_allowed_methods(&matching_path_routes);
        if matching_path_routes.is_empty() {
            return RestDispatchResult::error(
                404,
                "rest_no_route",
                "No route was found matching the URL and request method.",
            );
        }

        if request_method == "OPTIONS" {
            return RestDispatchResult {
                status_code: 200,
                body: json!({
                    "route": request_path,
                    "method": "OPTIONS",
                    "allow": allowed_methods,
                    "ok": true,
                }),
                error_code: None,
            };
        }

        let route = match matching_path_routes
            .into_iter()
            .find(|route| method_allowed(route, &request_method))
        {
            Some(route) => route,
            None => {
                return RestDispatchResult {
                    status_code: 405,
                    body: json!({
                        "code": "rest_no_route",
                        "message": "No route was found matching the URL and request method.",
                        "allow": allowed_methods,
                    }),
                    error_code: Some("rest_no_route".to_string()),
                };
            }
        };

        match &route.auth_requirement {
            AuthRequirement::Public => {}
            AuthRequirement::Authenticated if !request.authenticated => {
                return RestDispatchResult::error(
                    401,
                    "rest_not_logged_in",
                    "You are not currently logged in.",
                );
            }
            AuthRequirement::Capability(capability)
                if !request.capabilities.contains(capability.as_str()) =>
            {
                return RestDispatchResult::error(
                    403,
                    "rest_forbidden",
                    "Sorry, you are not allowed to do that.",
                );
            }
            _ => {}
        }

        RestDispatchResult {
            status_code: 200,
            body: json!({
                "route": request_path,
                "method": request_method,
                "ok": true,
            }),
            error_code: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RestRequest {
    pub method: String,
    pub path: String,
    pub authenticated: bool,
    pub capabilities: BTreeSet<String>,
}

impl RestRequest {
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            authenticated: false,
            capabilities: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestDispatchResult {
    pub status_code: u16,
    pub body: Value,
    pub error_code: Option<String>,
}

impl RestDispatchResult {
    fn error(status_code: u16, error_code: &str, message: &str) -> Self {
        Self {
            status_code,
            body: json!({
                "code": error_code,
                "message": message,
            }),
            error_code: Some(error_code.to_string()),
        }
    }
}

pub fn core_seed_routes() -> RestRouteRegistry {
    let mut registry = RestRouteRegistry::default();
    registry.register(
        "/wp-json",
        &["GET"],
        "REST root index",
        AuthRequirement::Public,
    );
    registry.register(
        "/wp-json/wp/v2/posts",
        &["GET"],
        "List posts",
        AuthRequirement::Public,
    );
    registry.register(
        "/wp-json/wp/v2/posts",
        &["POST"],
        "Create post",
        AuthRequirement::Capability("edit_posts".to_string()),
    );
    registry.register(
        "/wp-json/wp/v2/users/me",
        &["GET"],
        "Current user profile",
        AuthRequirement::Authenticated,
    );
    registry.register(
        "/wp-json/wp/v2/settings",
        &["GET"],
        "Read site settings",
        AuthRequirement::Capability("manage_options".to_string()),
    );
    registry
}

fn format_auth_requirement(requirement: &AuthRequirement) -> String {
    match requirement {
        AuthRequirement::Public => "public".to_string(),
        AuthRequirement::Authenticated => "authenticated".to_string(),
        AuthRequirement::Capability(capability) => format!("capability:{capability}"),
    }
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }

    let prefixed = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    if prefixed.len() > 1 {
        prefixed.trim_end_matches('/').to_string()
    } else {
        prefixed
    }
}

fn method_allowed(route: &RestRoute, request_method: &str) -> bool {
    route.methods.iter().any(|method| {
        let registered = method.trim().to_ascii_uppercase();
        registered == request_method || (request_method == "HEAD" && registered == "GET")
    })
}

fn collect_allowed_methods(routes: &[&RestRoute]) -> Vec<String> {
    let mut allowed_methods = BTreeSet::new();
    for route in routes {
        for method in &route.methods {
            allowed_methods.insert(method.trim().to_ascii_uppercase());
        }
    }
    if allowed_methods.contains("GET") {
        allowed_methods.insert("HEAD".to_string());
    }
    if !routes.is_empty() {
        allowed_methods.insert("OPTIONS".to_string());
    }
    allowed_methods.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_seed_routes() {
        let registry = core_seed_routes();
        assert!(!registry.all().is_empty());
    }

    #[test]
    fn public_route_dispatches_successfully() {
        let registry = core_seed_routes();
        let request = RestRequest::new("GET", "/wp-json/wp/v2/posts");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 200);
    }

    #[test]
    fn protected_route_requires_capability() {
        let registry = core_seed_routes();
        let mut request = RestRequest::new("POST", "/wp-json/wp/v2/posts");
        request.authenticated = true;
        let forbidden = registry.dispatch(&request);
        assert_eq!(forbidden.status_code, 403);

        request.capabilities.insert("edit_posts".to_string());
        let ok = registry.dispatch(&request);
        assert_eq!(ok.status_code, 200);
    }

    #[test]
    fn authenticated_route_requires_login() {
        let registry = core_seed_routes();
        let request = RestRequest::new("GET", "/wp-json/wp/v2/users/me");
        let unauthorized = registry.dispatch(&request);
        assert_eq!(unauthorized.status_code, 401);
    }

    #[test]
    fn authenticated_route_allows_logged_in_request() {
        let registry = core_seed_routes();
        let mut request = RestRequest::new("GET", "/wp-json/wp/v2/users/me");
        request.authenticated = true;
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 200);
    }

    #[test]
    fn returns_method_not_allowed_for_known_path() {
        let registry = core_seed_routes();
        let request = RestRequest::new("DELETE", "/wp-json/wp/v2/posts");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 405);
        let allow = result.body["allow"]
            .as_array()
            .expect("allow list should be present");
        assert!(allow.contains(&Value::String("GET".to_string())));
        assert!(allow.contains(&Value::String("POST".to_string())));
        assert!(allow.contains(&Value::String("HEAD".to_string())));
        assert!(allow.contains(&Value::String("OPTIONS".to_string())));
    }

    #[test]
    fn returns_not_found_for_unknown_path() {
        let registry = core_seed_routes();
        let request = RestRequest::new("GET", "/wp-json/wp/v2/unknown");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 404);
    }

    #[test]
    fn settings_route_requires_manage_options_capability() {
        let registry = core_seed_routes();
        let mut request = RestRequest::new("GET", "/wp-json/wp/v2/settings");
        request.authenticated = true;
        let forbidden = registry.dispatch(&request);
        assert_eq!(forbidden.status_code, 403);

        request.capabilities.insert("manage_options".to_string());
        let allowed = registry.dispatch(&request);
        assert_eq!(allowed.status_code, 200);
    }

    #[test]
    fn head_request_dispatches_against_get_routes() {
        let registry = core_seed_routes();
        let request = RestRequest::new("HEAD", "/wp-json/wp/v2/posts");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 200);
        assert_eq!(result.body["method"], Value::String("HEAD".to_string()));
    }

    #[test]
    fn dispatch_normalizes_trailing_slashes() {
        let registry = core_seed_routes();
        let request = RestRequest::new("GET", "/wp-json/wp/v2/posts/");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 200);
        assert_eq!(
            result.body["route"],
            Value::String("/wp-json/wp/v2/posts".to_string())
        );
    }

    #[test]
    fn options_request_returns_allowed_methods_for_matching_path() {
        let registry = core_seed_routes();
        let request = RestRequest::new("OPTIONS", "/wp-json/wp/v2/posts");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 200);
        assert_eq!(result.body["method"], Value::String("OPTIONS".to_string()));

        let allow = result.body["allow"]
            .as_array()
            .expect("allow list should be present");
        assert!(allow.contains(&Value::String("GET".to_string())));
        assert!(allow.contains(&Value::String("POST".to_string())));
        assert!(allow.contains(&Value::String("HEAD".to_string())));
        assert!(allow.contains(&Value::String("OPTIONS".to_string())));
    }

    #[test]
    fn contract_document_contains_paths() {
        let registry = core_seed_routes();
        let contract = registry.contract_document();
        assert!(contract["paths"].is_object());
        assert!(contract["paths"]["/wp-json/wp/v2/posts"]["get"].is_object());
    }
}
