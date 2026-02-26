use std::collections::HashSet;
use std::env;

/// Environment-backed runtime settings for the PHP -> Rust gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustGatewaySettings {
    /// Whether Rust routing is enabled globally.
    pub enabled: bool,
    /// Base URL where the Rust server is reachable from PHP.
    pub backend_url: String,
    /// Timeout for proxy requests in milliseconds.
    pub timeout_ms: u64,
    /// Endpoints allowed to route through Rust.
    pub endpoint_allowlist: HashSet<String>,
}

impl Default for RustGatewaySettings {
    fn default() -> Self {
        let endpoint_allowlist = ["/__wp_rust/health".to_string()].into_iter().collect();
        Self {
            enabled: false,
            backend_url: "http://127.0.0.1:8088".to_string(),
            timeout_ms: 1_500,
            endpoint_allowlist,
        }
    }
}

impl RustGatewaySettings {
    /// Builds settings from environment variables.
    ///
    /// Supported variables:
    /// - `WP_RUST_GATEWAY_ENABLED` (`1`, `true`, `on`, `yes`)
    /// - `WP_RUST_GATEWAY_BACKEND_URL` (base URL)
    /// - `WP_RUST_GATEWAY_TIMEOUT_MS` (u64 milliseconds)
    /// - `WP_RUST_ENDPOINT_ALLOWLIST` (comma-separated list, `*` wildcard allowed)
    pub fn from_env() -> Self {
        let mut settings = Self::default();

        if let Ok(value) = env::var("WP_RUST_GATEWAY_ENABLED") {
            settings.enabled = parse_truthy(&value);
        }

        if let Ok(value) = env::var("WP_RUST_GATEWAY_BACKEND_URL") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                settings.backend_url = trimmed.to_string();
            }
        }

        if let Ok(value) = env::var("WP_RUST_GATEWAY_TIMEOUT_MS") {
            if let Ok(parsed) = value.parse::<u64>() {
                settings.timeout_ms = parsed;
            }
        }

        if let Ok(value) = env::var("WP_RUST_ENDPOINT_ALLOWLIST") {
            let parsed: HashSet<String> = value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            if !parsed.is_empty() {
                settings.endpoint_allowlist = parsed;
            }
        }

        settings
    }

    /// Returns true when a specific endpoint path should route through Rust.
    pub fn should_route(&self, endpoint: &str) -> bool {
        self.enabled
            && (self.endpoint_allowlist.contains("*") || self.endpoint_allowlist.contains(endpoint))
    }
}

fn parse_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe_and_conservative() {
        let settings = RustGatewaySettings::default();
        assert!(!settings.enabled);
        assert!(settings.endpoint_allowlist.contains("/__wp_rust/health"));
        assert!(!settings.should_route("/wp-login.php"));
    }

    #[test]
    fn should_route_supports_wildcard() {
        let mut settings = RustGatewaySettings::default();
        settings.enabled = true;
        settings.endpoint_allowlist = ["*".to_string()].into_iter().collect();
        assert!(settings.should_route("/wp-login.php"));
        assert!(settings.should_route("/wp-admin/admin-ajax.php"));
    }
}
