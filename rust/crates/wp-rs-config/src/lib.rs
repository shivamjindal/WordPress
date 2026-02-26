use std::collections::{HashMap, HashSet};
use std::env;

/// Environment-backed runtime settings for the PHP -> Rust gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustGatewaySettings {
    /// Whether Rust routing is enabled globally.
    pub enabled: bool,
    /// Whether legacy PHP fallback is allowed if Rust fails.
    pub fallback_enabled: bool,
    /// Base URL where the Rust server is reachable from PHP.
    pub backend_url: String,
    /// Timeout for proxy requests in milliseconds.
    pub timeout_ms: u64,
    /// Endpoints allowed to route through Rust.
    pub endpoint_allowlist: HashSet<String>,
    /// HTTP methods allowed for proxy routing.
    pub method_allowlist: HashSet<String>,
    /// Compatibility mode for plugin/theme execution.
    pub plugin_compat_mode: String,
}

impl Default for RustGatewaySettings {
    fn default() -> Self {
        let endpoint_allowlist = ["/__wp_rust/health".to_string()].into_iter().collect();
        let method_allowlist = ["GET".to_string(), "HEAD".to_string()]
            .into_iter()
            .collect();
        Self {
            enabled: false,
            fallback_enabled: true,
            backend_url: "http://127.0.0.1:8088".to_string(),
            timeout_ms: 1_500,
            endpoint_allowlist,
            method_allowlist,
            plugin_compat_mode: "php-runtime".to_string(),
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

        if let Ok(value) = env::var("WP_RUST_GATEWAY_FALLBACK_ENABLED") {
            settings.fallback_enabled = parse_truthy(&value);
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

        if let Ok(value) = env::var("WP_RUST_METHOD_ALLOWLIST") {
            let parsed: HashSet<String> = value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(|method| method.to_ascii_uppercase())
                .collect();
            if !parsed.is_empty() {
                settings.method_allowlist = parsed;
            }
        }

        if let Ok(value) = env::var("WP_RUST_PLUGIN_COMPAT_MODE") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                settings.plugin_compat_mode = trimmed.to_string();
            }
        }

        settings
    }

    /// Returns true when a specific endpoint path should route through Rust.
    pub fn should_route(&self, endpoint: &str) -> bool {
        self.enabled
            && (self.endpoint_allowlist.contains("*") || self.endpoint_allowlist.contains(endpoint))
    }

    pub fn allows_method(&self, method: &str) -> bool {
        self.method_allowlist
            .contains(&method.trim().to_ascii_uppercase())
    }
}

fn parse_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Subset of WordPress default constants used by migration foundation services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordPressConstants {
    pub wp_debug: bool,
    pub wp_content_dir: String,
    pub wp_plugins_dir: String,
    pub wp_lang_dir: String,
    pub wp_temp_dir: String,
    pub wp_memory_limit: String,
    pub wp_max_memory_limit: String,
}

impl Default for WordPressConstants {
    fn default() -> Self {
        Self {
            wp_debug: false,
            wp_content_dir: "wp-content".to_string(),
            wp_plugins_dir: "wp-content/plugins".to_string(),
            wp_lang_dir: "wp-content/languages".to_string(),
            wp_temp_dir: "wp-content/uploads".to_string(),
            wp_memory_limit: "40M".to_string(),
            wp_max_memory_limit: "256M".to_string(),
        }
    }
}

impl WordPressConstants {
    /// Builds constants from env variables.
    pub fn from_env() -> Self {
        let env_map = env::vars().collect::<HashMap<_, _>>();
        Self::from_map(&env_map)
    }

    /// Builds constants from key/value map.
    pub fn from_map(values: &HashMap<String, String>) -> Self {
        let mut constants = Self::default();

        if let Some(value) = values.get("WP_DEBUG") {
            constants.wp_debug = parse_truthy(value);
        }

        if let Some(value) = values.get("WP_CONTENT_DIR") {
            if !value.trim().is_empty() {
                constants.wp_content_dir = value.trim().to_string();
            }
        }

        if let Some(value) = values.get("WP_PLUGIN_DIR") {
            if !value.trim().is_empty() {
                constants.wp_plugins_dir = value.trim().to_string();
            }
        }

        if let Some(value) = values.get("WP_LANG_DIR") {
            if !value.trim().is_empty() {
                constants.wp_lang_dir = value.trim().to_string();
            }
        }

        if let Some(value) = values.get("WP_TEMP_DIR") {
            if !value.trim().is_empty() {
                constants.wp_temp_dir = value.trim().to_string();
            }
        }

        if let Some(value) = values.get("WP_MEMORY_LIMIT") {
            if !value.trim().is_empty() {
                constants.wp_memory_limit = value.trim().to_string();
            }
        }

        if let Some(value) = values.get("WP_MAX_MEMORY_LIMIT") {
            if !value.trim().is_empty() {
                constants.wp_max_memory_limit = value.trim().to_string();
            }
        }

        constants
    }

    pub fn memory_limit_bytes(&self) -> Option<u64> {
        parse_php_size_to_bytes(&self.wp_memory_limit)
    }

    pub fn max_memory_limit_bytes(&self) -> Option<u64> {
        parse_php_size_to_bytes(&self.wp_max_memory_limit)
    }
}

pub fn parse_php_size_to_bytes(input: &str) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (digits, suffix) =
        trimmed.split_at(trimmed.find(|character: char| !character.is_ascii_digit())?);
    let value = digits.parse::<u64>().ok()?;
    let multiplier = match suffix.trim().to_ascii_lowercase().as_str() {
        "" => 1,
        "k" | "kb" => 1024,
        "m" | "mb" => 1024_u64.pow(2),
        "g" | "gb" => 1024_u64.pow(3),
        _ => return None,
    };
    Some(value.saturating_mul(multiplier))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeProfile {
    Production,
    Development,
}

impl RuntimeProfile {
    pub fn from_constants(constants: &WordPressConstants) -> Self {
        if constants.wp_debug {
            Self::Development
        } else {
            Self::Production
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe_and_conservative() {
        let settings = RustGatewaySettings::default();
        assert!(!settings.enabled);
        assert!(settings.fallback_enabled);
        assert!(settings.endpoint_allowlist.contains("/__wp_rust/health"));
        assert!(settings.method_allowlist.contains("GET"));
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

    #[test]
    fn method_allowlist_enforces_uppercase_lookup() {
        let settings = RustGatewaySettings::default();
        assert!(settings.allows_method("get"));
        assert!(!settings.allows_method("post"));
    }

    #[test]
    fn parses_php_size_units() {
        assert_eq!(parse_php_size_to_bytes("40M"), Some(40 * 1024 * 1024));
        assert_eq!(parse_php_size_to_bytes("2g"), Some(2 * 1024 * 1024 * 1024));
        assert_eq!(parse_php_size_to_bytes("bad"), None);
    }

    #[test]
    fn constants_map_overrides_defaults() {
        let values = HashMap::from([
            ("WP_DEBUG".to_string(), "1".to_string()),
            ("WP_MEMORY_LIMIT".to_string(), "128M".to_string()),
            ("WP_MAX_MEMORY_LIMIT".to_string(), "512M".to_string()),
            ("WP_CONTENT_DIR".to_string(), "/srv/wp-content".to_string()),
        ]);
        let constants = WordPressConstants::from_map(&values);
        assert!(constants.wp_debug);
        assert_eq!(constants.wp_content_dir, "/srv/wp-content");
        assert_eq!(constants.memory_limit_bytes(), Some(128 * 1024 * 1024));
        assert_eq!(
            RuntimeProfile::from_constants(&constants),
            RuntimeProfile::Development
        );
    }
}
