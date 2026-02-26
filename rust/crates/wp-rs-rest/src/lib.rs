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
        let route = match self.routes.iter().find(|route| {
            route.path == request.path
                && route
                    .methods
                    .iter()
                    .any(|method| method.eq_ignore_ascii_case(&request.method))
        }) {
            Some(route) => route,
            None => {
                if self.routes.iter().any(|route| route.path == request.path) {
                    return RestDispatchResult::error(
                        405,
                        "rest_no_route",
                        "No route was found matching the URL and request method.",
                    );
                }
                return RestDispatchResult::error(
                    404,
                    "rest_no_route",
                    "No route was found matching the URL and request method.",
                );
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
                "route": route.path,
                "method": request.method.to_ascii_uppercase(),
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
    fn returns_method_not_allowed_for_known_path() {
        let registry = core_seed_routes();
        let request = RestRequest::new("DELETE", "/wp-json/wp/v2/posts");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 405);
    }

    #[test]
    fn contract_document_contains_paths() {
        let registry = core_seed_routes();
        let contract = registry.contract_document();
        assert!(contract["paths"].is_object());
        assert!(contract["paths"]["/wp-json/wp/v2/posts"]["get"].is_object());
    }
}
