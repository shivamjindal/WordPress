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
}
