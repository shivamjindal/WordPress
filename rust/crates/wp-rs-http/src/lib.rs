use serde::Serialize;

/// High-level WordPress endpoint categories used by the migration gateway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointKind {
    Frontend,
    Login,
    Signup,
    Activate,
    CommentsPost,
    Cron,
    XmlRpc,
    Mail,
    Trackback,
    LinksOpml,
    AdminAjax,
    AdminPost,
    AsyncUpload,
    Unknown,
}

impl EndpointKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EndpointKind::Frontend => "frontend",
            EndpointKind::Login => "wp-login",
            EndpointKind::Signup => "wp-signup",
            EndpointKind::Activate => "wp-activate",
            EndpointKind::CommentsPost => "wp-comments-post",
            EndpointKind::Cron => "wp-cron",
            EndpointKind::XmlRpc => "xmlrpc",
            EndpointKind::Mail => "wp-mail",
            EndpointKind::Trackback => "wp-trackback",
            EndpointKind::LinksOpml => "wp-links-opml",
            EndpointKind::AdminAjax => "wp-admin/admin-ajax",
            EndpointKind::AdminPost => "wp-admin/admin-post",
            EndpointKind::AsyncUpload => "wp-admin/async-upload",
            EndpointKind::Unknown => "unknown",
        }
    }
}

pub fn detect_endpoint_kind(path: &str) -> EndpointKind {
    let normalized = normalize_path(path);
    match normalized.as_str() {
        "/index.php" | "/" => EndpointKind::Frontend,
        "/wp-login.php" => EndpointKind::Login,
        "/wp-signup.php" => EndpointKind::Signup,
        "/wp-activate.php" => EndpointKind::Activate,
        "/wp-comments-post.php" => EndpointKind::CommentsPost,
        "/wp-cron.php" => EndpointKind::Cron,
        "/xmlrpc.php" => EndpointKind::XmlRpc,
        "/wp-mail.php" => EndpointKind::Mail,
        "/wp-trackback.php" => EndpointKind::Trackback,
        "/wp-links-opml.php" => EndpointKind::LinksOpml,
        "/wp-admin/admin-ajax.php" => EndpointKind::AdminAjax,
        "/wp-admin/admin-post.php" => EndpointKind::AdminPost,
        "/wp-admin/async-upload.php" => EndpointKind::AsyncUpload,
        _ => EndpointKind::Unknown,
    }
}

pub fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }

    let mut normalized = path.trim().to_string();
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    normalized
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum XmlRpcPermission {
    Public,
    Authenticated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlRpcMethod {
    pub name: String,
    pub permission: XmlRpcPermission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlRpcDispatchResult {
    pub success: bool,
    pub fault_code: Option<i32>,
    pub message: String,
    pub method_name: String,
}

#[derive(Debug, Default, Clone)]
pub struct XmlRpcRegistry {
    methods: Vec<XmlRpcMethod>,
}

impl XmlRpcRegistry {
    pub fn register(&mut self, name: impl Into<String>, permission: XmlRpcPermission) {
        self.methods.push(XmlRpcMethod {
            name: name.into(),
            permission,
        });
    }

    pub fn methods(&self) -> &[XmlRpcMethod] {
        &self.methods
    }

    pub fn dispatch(&self, method_name: &str, authenticated: bool) -> XmlRpcDispatchResult {
        let method = match self
            .methods
            .iter()
            .find(|method| method.name == method_name)
        {
            Some(method) => method,
            None => {
                return XmlRpcDispatchResult {
                    success: false,
                    fault_code: Some(-32601),
                    message: "Method not found".to_string(),
                    method_name: method_name.to_string(),
                };
            }
        };

        if method.permission == XmlRpcPermission::Authenticated && !authenticated {
            return XmlRpcDispatchResult {
                success: false,
                fault_code: Some(403),
                message: "Authentication required".to_string(),
                method_name: method_name.to_string(),
            };
        }

        XmlRpcDispatchResult {
            success: true,
            fault_code: None,
            message: "Method executed".to_string(),
            method_name: method_name.to_string(),
        }
    }
}

pub fn core_xmlrpc_registry() -> XmlRpcRegistry {
    let mut registry = XmlRpcRegistry::default();
    registry.register("system.listMethods", XmlRpcPermission::Public);
    registry.register("system.multicall", XmlRpcPermission::Public);
    registry.register("demo.sayHello", XmlRpcPermission::Public);
    registry.register("pingback.ping", XmlRpcPermission::Public);
    registry.register("wp.getUsersBlogs", XmlRpcPermission::Authenticated);
    registry.register("metaWeblog.getRecentPosts", XmlRpcPermission::Authenticated);
    registry.register("wp.getMediaLibrary", XmlRpcPermission::Authenticated);
    registry.register("wp.newPost", XmlRpcPermission::Authenticated);
    registry.register("wp.editPost", XmlRpcPermission::Authenticated);
    registry.register("wp.deletePost", XmlRpcPermission::Authenticated);
    registry
}

pub fn parse_xmlrpc_method_name(payload: &str) -> Option<String> {
    let start_tag = "<methodName>";
    let end_tag = "</methodName>";
    let start = payload.find(start_tag)? + start_tag.len();
    let end = payload[start..].find(end_tag)? + start;
    let method_name = payload[start..end].trim();
    if method_name.is_empty() {
        None
    } else {
        Some(method_name.to_string())
    }
}

pub fn xmlrpc_success_response(method_name: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?><methodResponse><params><param><value><string>{method_name}</string></value></param></params></methodResponse>"
    )
}

pub fn xmlrpc_fault_response(code: i32, message: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?><methodResponse><fault><value><struct><member><name>faultCode</name><value><int>{code}</int></value></member><member><name>faultString</name><value><string>{message}</string></value></member></struct></value></fault></methodResponse>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_login_endpoint() {
        assert_eq!(detect_endpoint_kind("/wp-login.php"), EndpointKind::Login);
    }

    #[test]
    fn normalizes_double_slashes() {
        assert_eq!(
            normalize_path("wp-admin//admin-ajax.php"),
            "/wp-admin/admin-ajax.php"
        );
    }

    #[test]
    fn parses_xmlrpc_method_name() {
        let payload =
            "<?xml version=\"1.0\"?><methodCall><methodName>wp.getUsersBlogs</methodName></methodCall>";
        assert_eq!(
            parse_xmlrpc_method_name(payload),
            Some("wp.getUsersBlogs".to_string())
        );
    }

    #[test]
    fn empty_xmlrpc_method_name_returns_none() {
        let payload =
            "<?xml version=\"1.0\"?><methodCall><methodName>   </methodName></methodCall>";
        assert_eq!(parse_xmlrpc_method_name(payload), None);
    }

    #[test]
    fn xmlrpc_dispatch_requires_auth_when_needed() {
        let registry = core_xmlrpc_registry();
        let anonymous = registry.dispatch("wp.getUsersBlogs", false);
        assert!(!anonymous.success);
        assert_eq!(anonymous.fault_code, Some(403));

        let authenticated = registry.dispatch("wp.getUsersBlogs", true);
        assert!(authenticated.success);
    }

    #[test]
    fn xmlrpc_public_method_dispatches_without_auth() {
        let registry = core_xmlrpc_registry();
        let result = registry.dispatch("demo.sayHello", false);
        assert!(result.success);
        assert_eq!(result.method_name, "demo.sayHello");
    }

    #[test]
    fn xmlrpc_authenticated_content_methods_require_auth() {
        let registry = core_xmlrpc_registry();
        let anonymous = registry.dispatch("wp.newPost", false);
        assert!(!anonymous.success);
        assert_eq!(anonymous.fault_code, Some(403));

        let authenticated = registry.dispatch("wp.newPost", true);
        assert!(authenticated.success);
    }

    #[test]
    fn xmlrpc_registry_contains_core_post_methods() {
        let registry = core_xmlrpc_registry();
        let method_names = registry
            .methods()
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>();
        assert!(method_names.contains(&"wp.newPost"));
        assert!(method_names.contains(&"wp.editPost"));
        assert!(method_names.contains(&"wp.deletePost"));
    }

    #[test]
    fn unknown_xmlrpc_method_returns_fault() {
        let registry = core_xmlrpc_registry();
        let result = registry.dispatch("unknown.method", false);
        assert_eq!(result.fault_code, Some(-32601));
    }
}
