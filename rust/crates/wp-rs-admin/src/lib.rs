use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::json;

/// Parses the action from admin-ajax/admin-post style query payloads.
pub fn extract_action(query_pairs: &[(&str, &str)]) -> Option<String> {
    query_pairs
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("action"))
        .and_then(|(_, value)| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AdminSurface {
    Ajax,
    AdminPost,
    AsyncUpload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminAuth {
    Public,
    Authenticated,
    Capability(String),
}

#[derive(Debug, Clone)]
pub struct AdminAction {
    pub name: String,
    pub surface: AdminSurface,
    pub auth: AdminAuth,
    pub requires_nonce: bool,
    pub allow_nopriv: bool,
}

#[derive(Debug, Default, Clone)]
pub struct AdminActionRegistry {
    actions: Vec<AdminAction>,
}

impl AdminActionRegistry {
    pub fn register(
        &mut self,
        surface: AdminSurface,
        name: impl Into<String>,
        auth: AdminAuth,
        requires_nonce: bool,
        allow_nopriv: bool,
    ) {
        self.actions.push(AdminAction {
            name: name.into(),
            surface,
            auth,
            requires_nonce,
            allow_nopriv,
        });
    }

    pub fn dispatch(&self, request: &AdminRequest) -> AdminDispatchResult {
        let action = match request.action.trim() {
            "" => {
                return AdminDispatchResult::error(
                    400,
                    "invalid_action",
                    "Missing or invalid action parameter.",
                );
            }
            action => action,
        };

        let registered = match self
            .actions
            .iter()
            .find(|candidate| candidate.surface == request.surface && candidate.name == action)
        {
            Some(registered) => registered,
            None => {
                return AdminDispatchResult::error(
                    404,
                    "unknown_action",
                    "Action is not registered for this admin surface.",
                );
            }
        };

        if registered.requires_nonce && !request.nonce_valid {
            return AdminDispatchResult::error(403, "invalid_nonce", "Nonce validation failed.");
        }

        match &registered.auth {
            AdminAuth::Public => {}
            AdminAuth::Authenticated if !request.authenticated && !registered.allow_nopriv => {
                return AdminDispatchResult::error(
                    401,
                    "not_logged_in",
                    "Authentication required.",
                );
            }
            AdminAuth::Capability(_capability)
                if !request.authenticated && !registered.allow_nopriv =>
            {
                return AdminDispatchResult::error(
                    401,
                    "not_logged_in",
                    "Authentication required.",
                );
            }
            AdminAuth::Capability(capability)
                if !request.capabilities.contains(capability) && !registered.allow_nopriv =>
            {
                return AdminDispatchResult::error(
                    403,
                    "forbidden",
                    "You are not allowed to perform this action.",
                );
            }
            _ => {}
        }

        AdminDispatchResult {
            status_code: 200,
            body: json!({
                "surface": format!("{:?}", request.surface),
                "action": action,
                "ok": true,
            }),
            error_code: None,
        }
    }

    pub fn contract_map(&self) -> BTreeMap<String, Vec<String>> {
        let mut contract = BTreeMap::<String, Vec<String>>::new();
        for action in &self.actions {
            let key = format!("{:?}", action.surface).to_ascii_lowercase();
            contract.entry(key).or_default().push(action.name.clone());
        }
        contract
    }
}

#[derive(Debug, Clone)]
pub struct AdminRequest {
    pub surface: AdminSurface,
    pub action: String,
    pub authenticated: bool,
    pub nonce_present: bool,
    pub nonce_valid: bool,
    pub capabilities: BTreeSet<String>,
}

impl AdminRequest {
    pub fn new(surface: AdminSurface, action: impl Into<String>) -> Self {
        Self {
            surface,
            action: action.into(),
            authenticated: false,
            nonce_present: false,
            nonce_valid: false,
            capabilities: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminDispatchResult {
    pub status_code: u16,
    pub body: serde_json::Value,
    pub error_code: Option<String>,
}

impl AdminDispatchResult {
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

pub fn core_admin_actions() -> AdminActionRegistry {
    let mut registry = AdminActionRegistry::default();
    registry.register(
        AdminSurface::Ajax,
        "heartbeat",
        AdminAuth::Authenticated,
        true,
        false,
    );
    registry.register(
        AdminSurface::Ajax,
        "rest-nonce",
        AdminAuth::Authenticated,
        true,
        false,
    );
    registry.register(
        AdminSurface::Ajax,
        "logged-out",
        AdminAuth::Public,
        false,
        true,
    );
    registry.register(
        AdminSurface::Ajax,
        "delete-comment",
        AdminAuth::Capability("moderate_comments".to_string()),
        true,
        false,
    );
    registry.register(
        AdminSurface::Ajax,
        "save-widget",
        AdminAuth::Capability("edit_theme_options".to_string()),
        true,
        false,
    );
    registry.register(
        AdminSurface::AdminPost,
        "save_post",
        AdminAuth::Capability("edit_posts".to_string()),
        true,
        false,
    );
    registry.register(
        AdminSurface::AdminPost,
        "update-user",
        AdminAuth::Capability("edit_users".to_string()),
        true,
        false,
    );
    registry.register(
        AdminSurface::AsyncUpload,
        "upload-attachment",
        AdminAuth::Capability("upload_files".to_string()),
        true,
        false,
    );
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_action_if_present() {
        let action = extract_action(&[("foo", "bar"), ("action", "heartbeat")]);
        assert_eq!(action, Some("heartbeat".to_string()));
    }

    #[test]
    fn rejects_missing_action_parameter() {
        let registry = core_admin_actions();
        let request = AdminRequest::new(AdminSurface::Ajax, "");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 400);
        assert_eq!(result.error_code.as_deref(), Some("invalid_action"));
    }

    #[test]
    fn rejects_unknown_action_for_surface() {
        let registry = core_admin_actions();
        let request = AdminRequest::new(AdminSurface::Ajax, "unknown-action");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 404);
        assert_eq!(result.error_code.as_deref(), Some("unknown_action"));
    }

    #[test]
    fn rejects_missing_nonce_for_ajax_action() {
        let registry = core_admin_actions();
        let mut request = AdminRequest::new(AdminSurface::Ajax, "heartbeat");
        request.authenticated = true;
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 403);
    }

    #[test]
    fn allows_nopriv_action_for_logged_out_request() {
        let registry = core_admin_actions();
        let request = AdminRequest::new(AdminSurface::Ajax, "logged-out");
        let result = registry.dispatch(&request);
        assert_eq!(result.status_code, 200);
    }

    #[test]
    fn enforces_capability_for_admin_post() {
        let registry = core_admin_actions();
        let mut request = AdminRequest::new(AdminSurface::AdminPost, "save_post");
        request.authenticated = true;
        request.nonce_present = true;
        request.nonce_valid = true;
        let forbidden = registry.dispatch(&request);
        assert_eq!(forbidden.status_code, 403);

        request.capabilities.insert("edit_posts".to_string());
        let allowed = registry.dispatch(&request);
        assert_eq!(allowed.status_code, 200);
    }

    #[test]
    fn enforces_comment_moderation_capability_for_ajax_comment_delete() {
        let registry = core_admin_actions();
        let mut request = AdminRequest::new(AdminSurface::Ajax, "delete-comment");
        request.authenticated = true;
        request.nonce_present = true;
        request.nonce_valid = true;

        let forbidden = registry.dispatch(&request);
        assert_eq!(forbidden.status_code, 403);

        request.capabilities.insert("moderate_comments".to_string());
        let allowed = registry.dispatch(&request);
        assert_eq!(allowed.status_code, 200);
    }

    #[test]
    fn enforces_edit_users_capability_for_admin_post_update_user() {
        let registry = core_admin_actions();
        let mut request = AdminRequest::new(AdminSurface::AdminPost, "update-user");
        request.authenticated = true;
        request.nonce_present = true;
        request.nonce_valid = true;

        let forbidden = registry.dispatch(&request);
        assert_eq!(forbidden.status_code, 403);

        request.capabilities.insert("edit_users".to_string());
        let allowed = registry.dispatch(&request);
        assert_eq!(allowed.status_code, 200);
    }
}
