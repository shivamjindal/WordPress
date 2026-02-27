use std::collections::{HashMap, HashSet};
use std::env;

/// Environment-backed runtime settings for the PHP -> Rust gateway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustGatewaySettings {
    /// Whether Rust routing is enabled globally.
    pub enabled: bool,
    /// Deployment profile controlling cutover defaults.
    pub deployment_profile: String,
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
            deployment_profile: "legacy-safe".to_string(),
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

        if let Ok(value) = env::var("WP_RUST_DEPLOYMENT_PROFILE") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                settings.deployment_profile = trimmed.to_string();
            }
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

        settings.apply_profile_overrides();
        settings
    }

    /// Returns true when a specific endpoint path should route through Rust.
    pub fn should_route(&self, endpoint: &str) -> bool {
        self.enabled
            && self.endpoint_allowed(endpoint)
            && self.plugin_mode_allows_endpoint(endpoint)
    }

    pub fn allows_method(&self, method: &str) -> bool {
        self.method_allowlist
            .contains(&method.trim().to_ascii_uppercase())
    }

    fn plugin_mode_allows_endpoint(&self, endpoint: &str) -> bool {
        if !self.plugin_compat_mode.eq_ignore_ascii_case("php-runtime") {
            return true;
        }

        endpoint.starts_with("/__wp_rust/")
            || endpoint.starts_with("/wp-json")
            || php_runtime_core_endpoints().contains(&endpoint)
    }

    fn endpoint_allowed(&self, endpoint: &str) -> bool {
        self.endpoint_allowlist.iter().any(|entry| {
            if entry == "*" {
                return true;
            }

            if let Some(prefix) = entry.strip_suffix('*') {
                let normalized_prefix = prefix.trim_end_matches('/');
                return !normalized_prefix.is_empty() && endpoint.starts_with(normalized_prefix);
            }

            entry == endpoint
        })
    }

    fn apply_profile_overrides(&mut self) {
        if !self
            .deployment_profile
            .eq_ignore_ascii_case("production-rust")
        {
            return;
        }

        self.enabled = true;
        self.fallback_enabled = false;
        self.endpoint_allowlist = ["*".to_string()].into_iter().collect();
        self.method_allowlist = ["*".to_string()].into_iter().collect();
        if self.plugin_compat_mode == "php-runtime" {
            self.plugin_compat_mode = "rust-only".to_string();
        }
    }
}

pub fn php_runtime_core_endpoints() -> &'static [&'static str] {
    &[
        "/wp-login.php",
        "/wp-signup.php",
        "/wp-activate.php",
        "/wp-comments-post.php",
        "/wp-mail.php",
        "/wp-trackback.php",
        "/wp-links-opml.php",
        "/wp-includes/js/tinymce/wp-tinymce.php",
        "/wp-admin",
        "/wp-admin/",
        "/wp-admin/index.php",
        "/wp-admin/admin.php",
        "/wp-admin/load-scripts.php",
        "/wp-admin/load-styles.php",
        "/wp-admin/user/admin.php",
        "/wp-admin/user/index.php",
        "/wp-admin/user/profile.php",
        "/wp-admin/user/user-edit.php",
        "/wp-admin/user/about.php",
        "/wp-admin/user/credits.php",
        "/wp-admin/user/contribute.php",
        "/wp-admin/user/freedoms.php",
        "/wp-admin/user/privacy.php",
        "/wp-admin/profile.php",
        "/wp-admin/user-edit.php",
        "/wp-admin/user-new.php",
        "/wp-admin/post-new.php",
        "/wp-admin/post.php",
        "/wp-admin/install.php",
        "/wp-admin/setup-config.php",
        "/wp-admin/install-helper.php",
        "/wp-admin/options.php",
        "/wp-admin/options-general.php",
        "/wp-admin/options-writing.php",
        "/wp-admin/options-reading.php",
        "/wp-admin/options-discussion.php",
        "/wp-admin/options-media.php",
        "/wp-admin/options-permalink.php",
        "/wp-admin/options-privacy.php",
        "/wp-admin/privacy-policy-guide.php",
        "/wp-admin/about.php",
        "/wp-admin/credits.php",
        "/wp-admin/contribute.php",
        "/wp-admin/freedoms.php",
        "/wp-admin/privacy.php",
        "/wp-admin/plugin-install.php",
        "/wp-admin/plugin-editor.php",
        "/wp-admin/theme-install.php",
        "/wp-admin/theme-editor.php",
        "/wp-admin/widgets.php",
        "/wp-admin/widgets-form.php",
        "/wp-admin/widgets-form-blocks.php",
        "/wp-admin/nav-menus.php",
        "/wp-admin/font-library.php",
        "/wp-admin/customize.php",
        "/wp-admin/authorize-application.php",
        "/wp-admin/site-editor.php",
        "/wp-admin/press-this.php",
        "/wp-admin/term.php",
        "/wp-admin/revision.php",
        "/wp-admin/moderation.php",
        "/wp-admin/my-sites.php",
        "/wp-admin/ms-sites.php",
        "/wp-admin/ms-users.php",
        "/wp-admin/ms-themes.php",
        "/wp-admin/ms-edit.php",
        "/wp-admin/ms-admin.php",
        "/wp-admin/ms-options.php",
        "/wp-admin/ms-upgrade-network.php",
        "/wp-admin/plugins.php",
        "/wp-admin/themes.php",
        "/wp-admin/users.php",
        "/wp-admin/edit.php",
        "/wp-admin/edit-tags.php",
        "/wp-admin/edit-comments.php",
        "/wp-admin/comment.php",
        "/wp-admin/link-manager.php",
        "/wp-admin/link-add.php",
        "/wp-admin/link.php",
        "/wp-admin/media.php",
        "/wp-admin/media-upload.php",
        "/wp-admin/upload.php",
        "/wp-admin/media-new.php",
        "/wp-admin/tools.php",
        "/wp-admin/site-health.php",
        "/wp-admin/export.php",
        "/wp-admin/import.php",
        "/wp-admin/export-personal-data.php",
        "/wp-admin/erase-personal-data.php",
        "/wp-admin/network.php",
        "/wp-admin/network/admin.php",
        "/wp-admin/network/setup.php",
        "/wp-admin/ms-delete-site.php",
        "/wp-admin/network/index.php",
        "/wp-admin/network/sites.php",
        "/wp-admin/network/users.php",
        "/wp-admin/network/themes.php",
        "/wp-admin/network/plugins.php",
        "/wp-admin/network/settings.php",
        "/wp-admin/network/site-new.php",
        "/wp-admin/network/site-info.php",
        "/wp-admin/network/site-settings.php",
        "/wp-admin/network/site-users.php",
        "/wp-admin/network/site-themes.php",
        "/wp-admin/network/user-new.php",
        "/wp-admin/network/edit.php",
        "/wp-admin/network/update.php",
        "/wp-admin/network/update-core.php",
        "/wp-admin/network/plugin-install.php",
        "/wp-admin/network/plugin-editor.php",
        "/wp-admin/network/theme-editor.php",
        "/wp-admin/network/privacy.php",
        "/wp-admin/network/about.php",
        "/wp-admin/network/credits.php",
        "/wp-admin/network/contribute.php",
        "/wp-admin/network/freedoms.php",
        "/wp-admin/network/profile.php",
        "/wp-admin/network/user-edit.php",
        "/wp-admin/network/upgrade.php",
        "/wp-admin/network/theme-install.php",
        "/wp-admin/upgrade.php",
        "/wp-admin/update.php",
        "/wp-admin/update-core.php",
        "/wp-admin/maint/repair.php",
        "/wp-admin/admin-ajax.php",
        "/wp-admin/admin-post.php",
        "/wp-admin/async-upload.php",
        "/xmlrpc.php",
        "/wp-cron.php",
    ]
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
        assert_eq!(settings.deployment_profile, "legacy-safe");
        assert!(settings.fallback_enabled);
        assert!(settings.endpoint_allowlist.contains("/__wp_rust/health"));
        assert!(settings.method_allowlist.contains("GET"));
        assert!(!settings.should_route("/wp-login.php"));
    }

    #[test]
    fn should_route_supports_wildcard() {
        let mut settings = RustGatewaySettings::default();
        settings.enabled = true;
        settings.plugin_compat_mode = "rust-only".to_string();
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
    fn production_profile_forces_full_cutover() {
        let mut settings = RustGatewaySettings::default();
        settings.deployment_profile = "production-rust".to_string();
        settings.apply_profile_overrides();

        assert!(settings.enabled);
        assert!(!settings.fallback_enabled);
        assert!(settings.endpoint_allowlist.contains("*"));
        assert!(settings.method_allowlist.contains("*"));
        assert_eq!(settings.plugin_compat_mode, "rust-only");
    }

    #[test]
    fn php_runtime_profile_restricts_non_core_endpoints() {
        let mut settings = RustGatewaySettings::default();
        settings.enabled = true;
        settings.endpoint_allowlist = ["*".to_string()].into_iter().collect();
        settings.plugin_compat_mode = "php-runtime".to_string();

        assert!(settings.should_route("/wp-json/wp/v2/posts"));
        assert!(settings.should_route("/wp-admin/admin-ajax.php"));
        assert!(settings.should_route("/wp-login.php"));
        assert!(settings.should_route("/wp-includes/js/tinymce/wp-tinymce.php"));
        assert!(settings.should_route("/wp-admin/admin.php"));
        assert!(settings.should_route("/wp-admin/index.php"));
        assert!(settings.should_route("/wp-admin/load-scripts.php"));
        assert!(settings.should_route("/wp-admin/load-styles.php"));
        assert!(settings.should_route("/wp-admin/user/admin.php"));
        assert!(settings.should_route("/wp-admin/user/index.php"));
        assert!(settings.should_route("/wp-admin/user/profile.php"));
        assert!(settings.should_route("/wp-admin/user/user-edit.php"));
        assert!(settings.should_route("/wp-admin/user/about.php"));
        assert!(settings.should_route("/wp-admin/user/credits.php"));
        assert!(settings.should_route("/wp-admin/user/contribute.php"));
        assert!(settings.should_route("/wp-admin/user/freedoms.php"));
        assert!(settings.should_route("/wp-admin/user/privacy.php"));
        assert!(settings.should_route("/wp-admin/profile.php"));
        assert!(settings.should_route("/wp-admin/user-edit.php"));
        assert!(settings.should_route("/wp-admin/user-new.php"));
        assert!(settings.should_route("/wp-admin/post-new.php"));
        assert!(settings.should_route("/wp-admin/post.php"));
        assert!(settings.should_route("/wp-admin/setup-config.php"));
        assert!(settings.should_route("/wp-admin/options.php"));
        assert!(settings.should_route("/wp-admin/options-general.php"));
        assert!(settings.should_route("/wp-admin/options-writing.php"));
        assert!(settings.should_route("/wp-admin/options-reading.php"));
        assert!(settings.should_route("/wp-admin/options-discussion.php"));
        assert!(settings.should_route("/wp-admin/options-media.php"));
        assert!(settings.should_route("/wp-admin/options-permalink.php"));
        assert!(settings.should_route("/wp-admin/options-privacy.php"));
        assert!(settings.should_route("/wp-admin/privacy-policy-guide.php"));
        assert!(settings.should_route("/wp-admin/about.php"));
        assert!(settings.should_route("/wp-admin/credits.php"));
        assert!(settings.should_route("/wp-admin/contribute.php"));
        assert!(settings.should_route("/wp-admin/freedoms.php"));
        assert!(settings.should_route("/wp-admin/privacy.php"));
        assert!(settings.should_route("/wp-admin/plugin-install.php"));
        assert!(settings.should_route("/wp-admin/plugin-editor.php"));
        assert!(settings.should_route("/wp-admin/theme-install.php"));
        assert!(settings.should_route("/wp-admin/theme-editor.php"));
        assert!(settings.should_route("/wp-admin/widgets.php"));
        assert!(settings.should_route("/wp-admin/widgets-form.php"));
        assert!(settings.should_route("/wp-admin/widgets-form-blocks.php"));
        assert!(settings.should_route("/wp-admin/nav-menus.php"));
        assert!(settings.should_route("/wp-admin/font-library.php"));
        assert!(settings.should_route("/wp-admin/customize.php"));
        assert!(settings.should_route("/wp-admin/authorize-application.php"));
        assert!(settings.should_route("/wp-admin/site-editor.php"));
        assert!(settings.should_route("/wp-admin/press-this.php"));
        assert!(settings.should_route("/wp-admin/term.php"));
        assert!(settings.should_route("/wp-admin/revision.php"));
        assert!(settings.should_route("/wp-admin/moderation.php"));
        assert!(settings.should_route("/wp-admin/my-sites.php"));
        assert!(settings.should_route("/wp-admin/ms-sites.php"));
        assert!(settings.should_route("/wp-admin/ms-users.php"));
        assert!(settings.should_route("/wp-admin/ms-themes.php"));
        assert!(settings.should_route("/wp-admin/ms-edit.php"));
        assert!(settings.should_route("/wp-admin/ms-admin.php"));
        assert!(settings.should_route("/wp-admin/ms-options.php"));
        assert!(settings.should_route("/wp-admin/ms-upgrade-network.php"));
        assert!(settings.should_route("/wp-admin/plugins.php"));
        assert!(settings.should_route("/wp-admin/themes.php"));
        assert!(settings.should_route("/wp-admin/users.php"));
        assert!(settings.should_route("/wp-admin/edit.php"));
        assert!(settings.should_route("/wp-admin/edit-tags.php"));
        assert!(settings.should_route("/wp-admin/edit-comments.php"));
        assert!(settings.should_route("/wp-admin/comment.php"));
        assert!(settings.should_route("/wp-admin/link-manager.php"));
        assert!(settings.should_route("/wp-admin/link-add.php"));
        assert!(settings.should_route("/wp-admin/link.php"));
        assert!(settings.should_route("/wp-admin/media.php"));
        assert!(settings.should_route("/wp-admin/media-upload.php"));
        assert!(settings.should_route("/wp-admin/upload.php"));
        assert!(settings.should_route("/wp-admin/media-new.php"));
        assert!(settings.should_route("/wp-admin/tools.php"));
        assert!(settings.should_route("/wp-admin/site-health.php"));
        assert!(settings.should_route("/wp-admin/export.php"));
        assert!(settings.should_route("/wp-admin/import.php"));
        assert!(settings.should_route("/wp-admin/export-personal-data.php"));
        assert!(settings.should_route("/wp-admin/erase-personal-data.php"));
        assert!(settings.should_route("/wp-admin/network.php"));
        assert!(settings.should_route("/wp-admin/network/admin.php"));
        assert!(settings.should_route("/wp-admin/network/setup.php"));
        assert!(settings.should_route("/wp-admin/ms-delete-site.php"));
        assert!(settings.should_route("/wp-admin/network/index.php"));
        assert!(settings.should_route("/wp-admin/network/sites.php"));
        assert!(settings.should_route("/wp-admin/network/users.php"));
        assert!(settings.should_route("/wp-admin/network/themes.php"));
        assert!(settings.should_route("/wp-admin/network/plugins.php"));
        assert!(settings.should_route("/wp-admin/network/settings.php"));
        assert!(settings.should_route("/wp-admin/network/site-new.php"));
        assert!(settings.should_route("/wp-admin/network/site-info.php"));
        assert!(settings.should_route("/wp-admin/network/site-settings.php"));
        assert!(settings.should_route("/wp-admin/network/site-users.php"));
        assert!(settings.should_route("/wp-admin/network/site-themes.php"));
        assert!(settings.should_route("/wp-admin/network/user-new.php"));
        assert!(settings.should_route("/wp-admin/network/edit.php"));
        assert!(settings.should_route("/wp-admin/network/update.php"));
        assert!(settings.should_route("/wp-admin/network/update-core.php"));
        assert!(settings.should_route("/wp-admin/network/plugin-install.php"));
        assert!(settings.should_route("/wp-admin/network/plugin-editor.php"));
        assert!(settings.should_route("/wp-admin/network/theme-editor.php"));
        assert!(settings.should_route("/wp-admin/network/privacy.php"));
        assert!(settings.should_route("/wp-admin/network/about.php"));
        assert!(settings.should_route("/wp-admin/network/credits.php"));
        assert!(settings.should_route("/wp-admin/network/contribute.php"));
        assert!(settings.should_route("/wp-admin/network/freedoms.php"));
        assert!(settings.should_route("/wp-admin/network/profile.php"));
        assert!(settings.should_route("/wp-admin/network/user-edit.php"));
        assert!(settings.should_route("/wp-admin/network/upgrade.php"));
        assert!(settings.should_route("/wp-admin/network/theme-install.php"));
        assert!(settings.should_route("/wp-admin/update.php"));
        assert!(settings.should_route("/wp-admin/update-core.php"));
        assert!(!settings.should_route("/plugin-custom/endpoint"));
    }

    #[test]
    fn endpoint_allowlist_supports_prefix_wildcards() {
        let mut settings = RustGatewaySettings::default();
        settings.enabled = true;
        settings.plugin_compat_mode = "rust-only".to_string();
        settings.endpoint_allowlist = ["/wp-json/*".to_string()].into_iter().collect();

        assert!(settings.should_route("/wp-json/wp/v2/posts"));
        assert!(!settings.should_route("/xmlrpc.php"));
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
