<?php
/**
 * Rust gateway helpers for incremental PHP -> Rust migration.
 *
 * This file is intentionally dependency-light so it can be safely required
 * by early entrypoints before full WordPress bootstrap.
 *
 * @package WordPress
 */

if ( ! function_exists( 'wp_rust_gateway_get_settings' ) ) {
	/**
	 * Returns normalized Rust gateway settings from constants/environment.
	 *
	 * @return array{
	 *     enabled: bool,
	 *     deployment_profile: string,
	 *     fallback_enabled: bool,
	 *     backend_url: string,
	 *     timeout_ms: int,
	 *     endpoint_allowlist: string[],
	 *     method_allowlist: string[],
	 *     plugin_compat_mode: string
	 * }
	 */
	function wp_rust_gateway_get_settings() {
		$enabled = false;
		if ( defined( 'WP_RUST_GATEWAY_ENABLED' ) ) {
			$enabled = (bool) WP_RUST_GATEWAY_ENABLED;
		} else {
			$enabled_env = getenv( 'WP_RUST_GATEWAY_ENABLED' );
			if ( false !== $enabled_env ) {
				$enabled = wp_rust_gateway_parse_truthy( $enabled_env );
			}
		}

		$fallback_enabled = true;
		if ( defined( 'WP_RUST_GATEWAY_FALLBACK_ENABLED' ) ) {
			$fallback_enabled = (bool) WP_RUST_GATEWAY_FALLBACK_ENABLED;
		} else {
			$fallback_env = getenv( 'WP_RUST_GATEWAY_FALLBACK_ENABLED' );
			if ( false !== $fallback_env ) {
				$fallback_enabled = wp_rust_gateway_parse_truthy( $fallback_env );
			}
		}

		$deployment_profile = defined( 'WP_RUST_DEPLOYMENT_PROFILE' ) ? WP_RUST_DEPLOYMENT_PROFILE : '';
		if ( ! $deployment_profile ) {
			$profile_env = getenv( 'WP_RUST_DEPLOYMENT_PROFILE' );
			$deployment_profile = false !== $profile_env ? $profile_env : 'legacy-safe';
		}

		$backend_url = defined( 'WP_RUST_GATEWAY_BACKEND_URL' ) ? WP_RUST_GATEWAY_BACKEND_URL : '';
		if ( ! $backend_url ) {
			$backend_env = getenv( 'WP_RUST_GATEWAY_BACKEND_URL' );
			$backend_url = false !== $backend_env ? $backend_env : 'http://127.0.0.1:8088';
		}
		$backend_url = rtrim( trim( $backend_url ), '/' );

		$timeout_ms = defined( 'WP_RUST_GATEWAY_TIMEOUT_MS' ) ? (int) WP_RUST_GATEWAY_TIMEOUT_MS : 0;
		if ( $timeout_ms <= 0 ) {
			$timeout_env = getenv( 'WP_RUST_GATEWAY_TIMEOUT_MS' );
			$timeout_ms = (int) ( false !== $timeout_env ? $timeout_env : 1500 );
		}
		if ( $timeout_ms <= 0 ) {
			$timeout_ms = 1500;
		}

		$allowlist_raw = defined( 'WP_RUST_ENDPOINT_ALLOWLIST' ) ? WP_RUST_ENDPOINT_ALLOWLIST : '';
		if ( ! $allowlist_raw ) {
			$allowlist_env = getenv( 'WP_RUST_ENDPOINT_ALLOWLIST' );
			$allowlist_raw = false !== $allowlist_env ? $allowlist_env : '/__wp_rust/health';
		}

		$endpoint_allowlist = array_values(
			array_filter(
				array_map(
					'trim',
					explode( ',', $allowlist_raw )
				)
			)
		);

		if ( empty( $endpoint_allowlist ) ) {
			$endpoint_allowlist = array( '/__wp_rust/health' );
		}

		$method_allowlist_raw = defined( 'WP_RUST_METHOD_ALLOWLIST' ) ? WP_RUST_METHOD_ALLOWLIST : '';
		if ( ! $method_allowlist_raw ) {
			$method_allowlist_env = getenv( 'WP_RUST_METHOD_ALLOWLIST' );
			$method_allowlist_raw = false !== $method_allowlist_env ? $method_allowlist_env : 'GET,HEAD';
		}
		$method_allowlist = array_values(
			array_filter(
				array_map(
					'strtoupper',
					array_map(
						'trim',
						explode( ',', (string) $method_allowlist_raw )
					)
				)
			)
		);
		if ( empty( $method_allowlist ) ) {
			$method_allowlist = array( 'GET', 'HEAD' );
		}

		$plugin_compat_mode = defined( 'WP_RUST_PLUGIN_COMPAT_MODE' ) ? WP_RUST_PLUGIN_COMPAT_MODE : '';
		if ( ! $plugin_compat_mode ) {
			$compat_mode_env = getenv( 'WP_RUST_PLUGIN_COMPAT_MODE' );
			$plugin_compat_mode = false !== $compat_mode_env ? $compat_mode_env : 'php-runtime';
		}

		$settings = array(
			'enabled'           => $enabled,
			'deployment_profile' => trim( (string) $deployment_profile ),
			'fallback_enabled'  => $fallback_enabled,
			'backend_url'       => $backend_url,
			'timeout_ms'        => $timeout_ms,
			'endpoint_allowlist' => $endpoint_allowlist,
			'method_allowlist'  => $method_allowlist,
			'plugin_compat_mode' => trim( (string) $plugin_compat_mode ),
		);
		return wp_rust_gateway_apply_profile_overrides( $settings );
	}
}

if ( ! function_exists( 'wp_rust_gateway_parse_truthy' ) ) {
	/**
	 * Parses a truthy string value.
	 *
	 * @param string $value Raw value.
	 * @return bool
	 */
	function wp_rust_gateway_parse_truthy( $value ) {
		return in_array( strtolower( trim( (string) $value ) ), array( '1', 'true', 'yes', 'on' ), true );
	}
}

if ( ! function_exists( 'wp_rust_gateway_current_request_path' ) ) {
	/**
	 * Returns normalized current request path.
	 *
	 * @param string $fallback Fallback endpoint.
	 * @return string
	 */
	function wp_rust_gateway_current_request_path( $fallback = '/' ) {
		$request_uri = isset( $_SERVER['REQUEST_URI'] ) ? (string) $_SERVER['REQUEST_URI'] : '';
		if ( '' === $request_uri ) {
			return (string) $fallback;
		}

		$path = parse_url( $request_uri, PHP_URL_PATH );
		if ( ! is_string( $path ) || '' === $path ) {
			return (string) $fallback;
		}

		return '/' . ltrim( $path, '/' );
	}
}

if ( ! function_exists( 'wp_rust_gateway_apply_profile_overrides' ) ) {
	/**
	 * Applies deployment profile defaults.
	 *
	 * @param array $settings Gateway settings.
	 * @return array
	 */
	function wp_rust_gateway_apply_profile_overrides( $settings ) {
		$profile = isset( $settings['deployment_profile'] ) ? strtolower( trim( (string) $settings['deployment_profile'] ) ) : 'legacy-safe';
		if ( 'production-rust' !== $profile ) {
			return $settings;
		}

		$settings['enabled'] = true;
		$settings['fallback_enabled'] = false;
		$settings['endpoint_allowlist'] = array( '*' );
		$settings['method_allowlist'] = array( '*' );
		if ( empty( $settings['plugin_compat_mode'] ) || 'php-runtime' === $settings['plugin_compat_mode'] ) {
			$settings['plugin_compat_mode'] = 'rust-only';
		}

		return $settings;
	}
}

if ( ! function_exists( 'wp_rust_gateway_should_proxy' ) ) {
	/**
	 * Determines whether the current endpoint should route through Rust.
	 *
	 * @param string $endpoint Endpoint path (for example `/wp-login.php`).
	 * @param array|null $settings Optional settings override.
	 * @return bool
	 */
	function wp_rust_gateway_should_proxy( $endpoint, $settings = null ) {
		if ( null === $settings ) {
			$settings = wp_rust_gateway_get_settings();
		}

		if ( empty( $settings['enabled'] ) ) {
			return false;
		}

		if ( ! wp_rust_gateway_plugin_mode_allows_endpoint( $endpoint, $settings ) ) {
			return false;
		}

		return wp_rust_gateway_endpoint_allowed( $endpoint, $settings['endpoint_allowlist'] );
	}
}

if ( ! function_exists( 'wp_rust_gateway_endpoint_allowed' ) ) {
	/**
	 * Checks whether endpoint matches allowlist entries.
	 *
	 * Supports:
	 * - `*` (all)
	 * - exact path entries (for example `/wp-login.php`)
	 * - prefix wildcard entries (for example `/wp-json/*`)
	 *
	 * @param string   $endpoint Endpoint path.
	 * @param string[] $allowlist Allowlist entries.
	 * @return bool
	 */
	function wp_rust_gateway_endpoint_allowed( $endpoint, $allowlist ) {
		foreach ( (array) $allowlist as $entry ) {
			$entry = trim( (string) $entry );
			if ( '' === $entry ) {
				continue;
			}
			if ( '*' === $entry ) {
				return true;
			}
			if ( '*' === substr( $entry, -1 ) ) {
				$prefix = rtrim( substr( $entry, 0, -1 ), '/' );
				if ( '' !== $prefix && 0 === strpos( $endpoint, $prefix ) ) {
					return true;
				}
				continue;
			}
			if ( $endpoint === $entry ) {
				return true;
			}
		}
		return false;
	}
}

if ( ! function_exists( 'wp_rust_gateway_plugin_mode_allows_endpoint' ) ) {
	/**
	 * Returns endpoint list allowed in php-runtime compatibility mode.
	 *
	 * @return string[]
	 */
	function wp_rust_gateway_php_runtime_core_endpoints() {
		return array(
			'/',
			'/index.php',
			'/search',
			'/feed',
			'/Admin',
			'/wp-blog-header.php',
			'/wp-load.php',
			'/wp-settings.php',
			'/wp-login.php',
			'/wp-signup.php',
			'/wp-activate.php',
			'/wp-comments-post.php',
			'/wp-mail.php',
			'/wp-trackback.php',
			'/wp-links-opml.php',
			'/wp-includes/load.php',
			'/wp-includes/vars.php',
			'/wp-includes/update.php',
			'/wp-includes/wp-db.php',
			'/wp-includes/utf8.php',
			'/wp-includes/user.php',
			'/wp-includes/functions.php',
			'/wp-includes/formatting.php',
			'/wp-includes/plugin.php',
			'/wp-includes/pluggable.php',
			'/wp-includes/capabilities.php',
			'/wp-includes/option.php',
			'/wp-includes/post.php',
			'/wp-includes/class-wp-hook.php',
			'/wp-includes/class-wp.php',
			'/wp-includes/class-wp-query.php',
			'/wp-includes/class-wp-rewrite.php',
			'/wp-includes/class-wp-role.php',
			'/wp-includes/class-wp-roles.php',
			'/wp-includes/class-wp-user.php',
			'/wp-includes/class-wp-session-tokens.php',
			'/wp-includes/class-wp-user-meta-session-tokens.php',
			'/wp-includes/class-wp-user-query.php',
			'/wp-includes/class-wp-meta-query.php',
			'/wp-includes/class-wp-date-query.php',
			'/wp-includes/class-wp-tax-query.php',
			'/wp-includes/class-wp-term-query.php',
			'/wp-includes/class-wp-comment-query.php',
			'/wp-includes/class-wp-network-query.php',
			'/wp-includes/class-wp-site-query.php',
			'/wp-includes/class-wp-post-type.php',
			'/wp-includes/class-wp-post.php',
			'/wp-includes/class-wp-error.php',
			'/wp-includes/class-wp-http.php',
			'/wp-includes/class-wp-http-cookie.php',
			'/wp-includes/class-wp-http-encoding.php',
			'/wp-includes/class-wp-http-response.php',
			'/wp-includes/class-wp-http-curl.php',
			'/wp-includes/class-wp-http-streams.php',
			'/wp-includes/class-wp-http-proxy.php',
			'/wp-includes/class-wp-http-requests-hooks.php',
			'/wp-includes/class-wp-http-requests-response.php',
			'/wp-includes/class-wp-network.php',
			'/wp-includes/class-wp-site.php',
			'/wp-includes/class-wp-taxonomy.php',
			'/wp-includes/class-wp-theme.php',
			'/wp-includes/class-wp-widget.php',
			'/wp-includes/class-wp-scripts.php',
			'/wp-includes/class-wp-styles.php',
			'/wp-includes/class-wp-dependencies.php',
			'/wp-includes/class-wp-dependency.php',
			'/wp-includes/class-wp-script-modules.php',
			'/wp-includes/class-IXR.php',
			'/wp-includes/class-avif-info.php',
			'/wp-includes/class-feed.php',
			'/wp-includes/class-http.php',
			'/wp-includes/class-json.php',
			'/wp-includes/class-oembed.php',
			'/wp-includes/class-phpass.php',
			'/wp-includes/class-phpmailer.php',
			'/wp-includes/class-pop3.php',
			'/wp-includes/class-requests.php',
			'/wp-includes/class-simplepie.php',
			'/wp-includes/class-smtp.php',
			'/wp-includes/class-snoopy.php',
			'/wp-includes/class-walker-category-dropdown.php',
			'/wp-includes/class-walker-category.php',
			'/wp-includes/class-walker-comment.php',
			'/wp-includes/class-walker-nav-menu.php',
			'/wp-includes/class-walker-page-dropdown.php',
			'/wp-includes/class-walker-page.php',
			'/wp-includes/class-wpdb.php',
			'/wp-includes/class.wp-dependencies.php',
			'/wp-includes/class.wp-scripts.php',
			'/wp-includes/class.wp-styles.php',
			'/wp-includes/compat-utf8.php',
			'/wp-includes/cron.php',
			'/wp-includes/date.php',
			'/wp-includes/default-constants.php',
			'/wp-includes/default-widgets.php',
			'/wp-includes/deprecated.php',
			'/wp-includes/embed-template.php',
			'/wp-includes/embed.php',
			'/wp-includes/error-protection.php',
			'/wp-includes/feed-atom-comments.php',
			'/wp-includes/feed-atom.php',
			'/wp-includes/feed-rdf.php',
			'/wp-includes/feed-rss.php',
			'/wp-includes/feed-rss2-comments.php',
			'/wp-includes/feed-rss2.php',
			'/wp-includes/feed.php',
			'/wp-includes/fonts.php',
			'/wp-includes/functions.wp-scripts.php',
			'/wp-includes/functions.wp-styles.php',
			'/wp-includes/global-styles-and-settings.php',
			'/wp-includes/http.php',
			'/wp-includes/https-detection.php',
			'/wp-includes/https-migration.php',
			'/wp-includes/kses.php',
			'/wp-includes/l10n.php',
			'/wp-includes/locale.php',
			'/wp-includes/media-template.php',
			'/wp-includes/media.php',
			'/wp-includes/meta.php',
			'/wp-includes/ms-blogs.php',
			'/wp-includes/ms-default-constants.php',
			'/wp-includes/ms-default-filters.php',
			'/wp-includes/ms-deprecated.php',
			'/wp-includes/ms-files.php',
			'/wp-includes/ms-functions.php',
			'/wp-includes/ms-load.php',
			'/wp-includes/ms-network.php',
			'/wp-includes/ms-settings.php',
			'/wp-includes/ms-site.php',
			'/wp-includes/nav-menu-template.php',
			'/wp-includes/nav-menu.php',
			'/wp-includes/pluggable-deprecated.php',
			'/wp-includes/post-formats.php',
			'/wp-includes/post-template.php',
			'/wp-includes/post-thumbnail-template.php',
			'/wp-includes/query.php',
			'/wp-includes/registration-functions.php',
			'/wp-includes/registration.php',
			'/wp-includes/rest-api.php',
			'/wp-includes/revision.php',
			'/wp-includes/rewrite.php',
			'/wp-includes/robots-template.php',
			'/wp-includes/rss-functions.php',
			'/wp-includes/rss.php',
			'/wp-includes/script-loader.php',
			'/wp-includes/session.php',
			'/wp-includes/spl-autoload-compat.php',
			'/wp-includes/category-template.php',
			'/wp-includes/category.php',
			'/wp-includes/comment-template.php',
			'/wp-includes/comment.php',
			'/wp-includes/compat.php',
			'/wp-includes/bookmark-template.php',
			'/wp-includes/bookmark.php',
			'/wp-includes/cache-compat.php',
			'/wp-includes/cache.php',
			'/wp-includes/canonical.php',
			'/wp-includes/block-bindings.php',
			'/wp-includes/block-editor.php',
			'/wp-includes/block-patterns.php',
			'/wp-includes/block-template-utils.php',
			'/wp-includes/block-template.php',
			'/wp-includes/abilities-api.php',
			'/wp-includes/abilities.php',
			'/wp-includes/admin-bar.php',
			'/wp-includes/atomlib.php',
			'/wp-includes/author-template.php',
			'/wp-includes/class-wp-theme-json-data.php',
			'/wp-includes/class-wp-theme-json-resolver.php',
			'/wp-includes/class-wp-token-map.php',
			'/wp-includes/class-wp-url-pattern-prefixer.php',
			'/wp-includes/class-wp-walker.php',
			'/wp-includes/class-wp-simplepie-file.php',
			'/wp-includes/class-wp-simplepie-sanitize-kses.php',
			'/wp-includes/class-wp-speculation-rules.php',
			'/wp-includes/class-wp-text-diff-renderer-inline.php',
			'/wp-includes/class-wp-text-diff-renderer-table.php',
			'/wp-includes/class-wp-navigation-fallback.php',
			'/wp-includes/class-wp-object-cache.php',
			'/wp-includes/class-wp-oembed-controller.php',
			'/wp-includes/class-wp-paused-extensions-storage.php',
			'/wp-includes/class-wp-phpmailer.php',
			'/wp-includes/class-wp-customize-setting.php',
			'/wp-includes/class-wp-customize-widgets.php',
			'/wp-includes/class-wp-feed-cache-transient.php',
			'/wp-includes/class-wp-feed-cache.php',
			'/wp-includes/class-wp-http-ixr-client.php',
			'/wp-includes/class-wp-customize-control.php',
			'/wp-includes/class-wp-customize-manager.php',
			'/wp-includes/class-wp-customize-nav-menus.php',
			'/wp-includes/class-wp-customize-panel.php',
			'/wp-includes/class-wp-customize-section.php',
			'/wp-includes/class-wp-block-supports.php',
			'/wp-includes/class-wp-block-template.php',
			'/wp-includes/class-wp-block-type.php',
			'/wp-includes/class-wp-classic-to-block-menu-converter.php',
			'/wp-includes/class-wp-duotone.php',
			'/wp-includes/class-wp-block-pattern-categories-registry.php',
			'/wp-includes/class-wp-block-patterns-registry.php',
			'/wp-includes/class-wp-block-styles-registry.php',
			'/wp-includes/class-wp-block-templates-registry.php',
			'/wp-includes/class-wp-block-type-registry.php',
			'/wp-includes/class-wp-block-metadata-registry.php',
			'/wp-includes/class-wp-block-parser.php',
			'/wp-includes/class-wp-block-parser-block.php',
			'/wp-includes/class-wp-block-parser-frame.php',
			'/wp-includes/class-wp-block-processor.php',
			'/wp-includes/class-wp-block-bindings-registry.php',
			'/wp-includes/class-wp-block-bindings-source.php',
			'/wp-includes/class-wp-block-editor-context.php',
			'/wp-includes/class-wp-block-list.php',
			'/wp-includes/class-wp-block.php',
			'/wp-includes/class-wp-recovery-mode.php',
			'/wp-includes/class-wp-recovery-mode-cookie-service.php',
			'/wp-includes/class-wp-recovery-mode-link-service.php',
			'/wp-includes/class-wp-recovery-mode-key-service.php',
			'/wp-includes/class-wp-recovery-mode-email-service.php',
			'/wp-includes/class-wp-xmlrpc-server.php',
			'/wp-includes/class-wp-widget-factory.php',
			'/wp-includes/class-wp-theme-json.php',
			'/wp-includes/class-wp-theme-json-schema.php',
			'/wp-includes/class-wp-textdomain-registry.php',
			'/wp-includes/class-wp-image-editor.php',
			'/wp-includes/class-wp-image-editor-gd.php',
			'/wp-includes/class-wp-image-editor-imagick.php',
			'/wp-includes/class-wp-exception.php',
			'/wp-includes/class-wp-fatal-error-handler.php',
			'/wp-includes/class-wp-admin-bar.php',
			'/wp-includes/class-wp-ajax-response.php',
			'/wp-includes/class-wp-embed.php',
			'/wp-includes/class-wp-editor.php',
			'/wp-includes/class-wp-oembed.php',
			'/wp-includes/class-wp-comment.php',
			'/wp-includes/class-wp-term.php',
			'/wp-includes/class-wp-user-request.php',
			'/wp-includes/class-wp-application-passwords.php',
			'/wp-includes/class-wp-plugin-dependencies.php',
			'/wp-includes/class-wp-locale.php',
			'/wp-includes/class-wp-locale-switcher.php',
			'/wp-includes/class-wp-matchesmapregex.php',
			'/wp-includes/class-wp-list-util.php',
			'/wp-includes/class-wp-metadata-lazyloader.php',
			'/wp-includes/general-template.php',
			'/wp-includes/link-template.php',
			'/wp-includes/default-filters.php',
			'/wp-includes/blocks.php',
			'/wp-includes/theme.php',
			'/wp-includes/theme-templates.php',
			'/wp-includes/theme-previews.php',
			'/wp-includes/speculative-loading.php',
			'/wp-includes/template-loader.php',
			'/wp-includes/template-canvas.php',
			'/wp-includes/template.php',
			'/wp-includes/taxonomy.php',
			'/wp-includes/shortcodes.php',
			'/wp-includes/widgets.php',
			'/wp-includes/ID3/getid3.lib.php',
			'/wp-includes/ID3/getid3.php',
			'/wp-includes/ID3/module.audio-video.asf.php',
			'/wp-includes/ID3/module.audio-video.flv.php',
			'/wp-includes/ID3/module.audio-video.matroska.php',
			'/wp-includes/ID3/module.audio-video.quicktime.php',
			'/wp-includes/ID3/module.audio-video.riff.php',
			'/wp-includes/ID3/module.audio.ac3.php',
			'/wp-includes/ID3/module.audio.dts.php',
			'/wp-includes/ID3/module.audio.flac.php',
			'/wp-includes/ID3/module.audio.mp3.php',
			'/wp-includes/ID3/module.audio.ogg.php',
			'/wp-includes/ID3/module.tag.apetag.php',
			'/wp-includes/ID3/module.tag.id3v1.php',
			'/wp-includes/ID3/module.tag.id3v2.php',
			'/wp-includes/ID3/module.tag.lyrics3.php',
			'/wp-includes/IXR/class-IXR-base64.php',
			'/wp-includes/IXR/class-IXR-client.php',
			'/wp-includes/IXR/class-IXR-clientmulticall.php',
			'/wp-includes/IXR/class-IXR-date.php',
			'/wp-includes/IXR/class-IXR-error.php',
			'/wp-includes/IXR/class-IXR-introspectionserver.php',
			'/wp-includes/IXR/class-IXR-message.php',
			'/wp-includes/IXR/class-IXR-request.php',
			'/wp-includes/IXR/class-IXR-server.php',
			'/wp-includes/IXR/class-IXR-value.php',
			'/wp-includes/PHPMailer/DSNConfigurator.php',
			'/wp-includes/PHPMailer/Exception.php',
			'/wp-includes/PHPMailer/OAuth.php',
			'/wp-includes/PHPMailer/OAuthTokenProvider.php',
			'/wp-includes/PHPMailer/PHPMailer.php',
			'/wp-includes/PHPMailer/POP3.php',
			'/wp-includes/PHPMailer/SMTP.php',
			'/wp-includes/Requests/library/Requests.php',
			'/wp-includes/Requests/src/Auth.php',
			'/wp-includes/Requests/src/Auth/Basic.php',
			'/wp-includes/Requests/src/Autoload.php',
			'/wp-includes/Requests/src/Capability.php',
			'/wp-includes/Requests/src/Cookie.php',
			'/wp-includes/Requests/src/Cookie/Jar.php',
			'/wp-includes/Requests/src/Exception.php',
			'/wp-includes/Requests/src/Exception/ArgumentCount.php',
			'/wp-includes/Requests/src/Exception/Http.php',
			'/wp-includes/Requests/src/Exception/Http/Status304.php',
			'/wp-includes/Requests/src/Exception/Http/Status305.php',
			'/wp-includes/Requests/src/Exception/Http/Status306.php',
			'/wp-includes/Requests/src/Exception/Http/Status400.php',
			'/wp-includes/Requests/src/Exception/Http/Status401.php',
			'/wp-includes/Requests/src/Exception/Http/Status402.php',
			'/wp-includes/Requests/src/Exception/Http/Status403.php',
			'/wp-includes/Requests/src/Exception/Http/Status404.php',
			'/wp-includes/Requests/src/Exception/Http/Status405.php',
			'/wp-includes/Requests/src/Exception/Http/Status406.php',
			'/wp-includes/Requests/src/Exception/Http/Status407.php',
			'/wp-includes/Requests/src/Exception/Http/Status408.php',
			'/wp-includes/Requests/src/Exception/Http/Status409.php',
			'/wp-includes/Requests/src/Exception/Http/Status410.php',
			'/wp-includes/Requests/src/Exception/Http/Status411.php',
			'/wp-includes/Requests/src/Exception/Http/Status412.php',
			'/wp-includes/Requests/src/Exception/Http/Status413.php',
			'/wp-includes/Requests/src/Exception/Http/Status414.php',
			'/wp-includes/Requests/src/Exception/Http/Status415.php',
			'/wp-includes/Requests/src/Exception/Http/Status416.php',
			'/wp-includes/Requests/src/Exception/Http/Status417.php',
			'/wp-includes/Requests/src/Exception/Http/Status418.php',
			'/wp-includes/Requests/src/Exception/Http/Status428.php',
			'/wp-includes/Requests/src/Exception/Http/Status429.php',
			'/wp-includes/Requests/src/Exception/Http/Status431.php',
			'/wp-includes/Requests/src/Exception/Http/Status500.php',
			'/wp-includes/Requests/src/Exception/Http/Status501.php',
			'/wp-includes/Requests/src/Exception/Http/Status502.php',
			'/wp-includes/Requests/src/Exception/Http/Status503.php',
			'/wp-includes/Requests/src/Exception/Http/Status504.php',
			'/wp-includes/Requests/src/Exception/Http/Status505.php',
			'/wp-includes/Requests/src/Exception/Http/Status511.php',
			'/wp-includes/Requests/src/Exception/Http/StatusUnknown.php',
			'/wp-includes/Requests/src/Exception/InvalidArgument.php',
			'/wp-includes/Requests/src/Exception/Transport/Curl.php',
			'/wp-includes/Requests/src/Exception/Transport.php',
			'/wp-includes/Requests/src/HookManager.php',
			'/wp-includes/Requests/src/Hooks.php',
			'/wp-includes/Requests/src/IdnaEncoder.php',
			'/wp-includes/Requests/src/Ipv6.php',
			'/wp-includes/Requests/src/Iri.php',
			'/wp-includes/Requests/src/Port.php',
			'/wp-includes/Requests/src/Proxy/Http.php',
			'/wp-includes/Requests/src/Proxy.php',
			'/wp-includes/Requests/src/Requests.php',
			'/wp-includes/Requests/src/Response/Headers.php',
			'/wp-includes/Requests/src/Response.php',
			'/wp-includes/Requests/src/Session.php',
			'/wp-includes/Requests/src/Ssl.php',
			'/wp-includes/Requests/src/Transport/Curl.php',
			'/wp-includes/Requests/src/Transport/Fsockopen.php',
			'/wp-includes/Requests/src/Transport.php',
			'/wp-includes/Requests/src/Utility/CaseInsensitiveDictionary.php',
			'/wp-includes/Requests/src/Utility/FilteredIterator.php',
			'/wp-includes/Requests/src/Utility/InputValidator.php',
			'/wp-includes/SimplePie/autoloader.php',
			'/wp-includes/SimplePie/library/SimplePie/Author.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/Base.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/DB.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/File.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/Memcache.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/Memcached.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/MySQL.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache/Redis.php',
			'/wp-includes/SimplePie/library/SimplePie/Cache.php',
			'/wp-includes/SimplePie/library/SimplePie/Caption.php',
			'/wp-includes/SimplePie/library/SimplePie/Category.php',
			'/wp-includes/SimplePie/library/SimplePie/Content/Type/Sniffer.php',
			'/wp-includes/SimplePie/library/SimplePie/Copyright.php',
			'/wp-includes/SimplePie/library/SimplePie/Core.php',
			'/wp-includes/SimplePie/library/SimplePie/Credit.php',
			'/wp-includes/SimplePie/library/SimplePie/Decode/HTML/Entities.php',
			'/wp-includes/SimplePie/library/SimplePie/Enclosure.php',
			'/wp-includes/SimplePie/library/SimplePie/Exception.php',
			'/wp-includes/SimplePie/library/SimplePie/File.php',
			'/wp-includes/SimplePie/library/SimplePie/HTTP/Parser.php',
			'/wp-includes/SimplePie/library/SimplePie/IRI.php',
			'/wp-includes/SimplePie/library/SimplePie/Item.php',
			'/wp-includes/SimplePie/library/SimplePie/Locator.php',
			'/wp-includes/SimplePie/library/SimplePie/Misc.php',
			'/wp-includes/style-engine.php',
			'/wp-includes/sitemaps.php',
			'/wp-includes/script-modules.php',
			'/wp-includes/version.php',
			'/wp-includes/wp-diff.php',
			'/wp-includes/view-transitions.php',
			'/wp-includes/js/tinymce/wp-tinymce.php',
			'/wp-admin',
			'/wp-admin/',
			'/wp-admin/index.php',
			'/wp-admin/admin.php',
			'/wp-admin/load-scripts.php',
			'/wp-admin/load-styles.php',
			'/wp-admin/custom-background.php',
			'/wp-admin/custom-header.php',
			'/wp-admin/admin-functions.php',
			'/wp-admin/options-head.php',
			'/wp-admin/menu.php',
			'/wp-admin/includes/admin-filters.php',
			'/wp-admin/includes/admin.php',
			'/wp-admin/includes/ajax-actions.php',
			'/wp-admin/includes/bookmark.php',
			'/wp-admin/includes/class-automatic-upgrader-skin.php',
			'/wp-admin/includes/class-bulk-plugin-upgrader-skin.php',
			'/wp-admin/includes/class-bulk-theme-upgrader-skin.php',
			'/wp-admin/includes/class-bulk-upgrader-skin.php',
			'/wp-admin/includes/class-core-upgrader.php',
			'/wp-admin/includes/class-custom-background.php',
			'/wp-admin/includes/class-custom-image-header.php',
			'/wp-admin/includes/class-file-upload-upgrader.php',
			'/wp-admin/includes/class-ftp-pure.php',
			'/wp-admin/includes/class-ftp-sockets.php',
			'/wp-admin/includes/class-ftp.php',
			'/wp-admin/includes/class-language-pack-upgrader-skin.php',
			'/wp-admin/includes/class-language-pack-upgrader.php',
			'/wp-admin/includes/class-pclzip.php',
			'/wp-admin/includes/class-plugin-installer-skin.php',
			'/wp-admin/includes/class-plugin-upgrader-skin.php',
			'/wp-admin/includes/class-plugin-upgrader.php',
			'/wp-admin/includes/class-theme-installer-skin.php',
			'/wp-admin/includes/class-theme-upgrader-skin.php',
			'/wp-admin/includes/class-theme-upgrader.php',
			'/wp-admin/includes/class-walker-category-checklist.php',
			'/wp-admin/includes/class-walker-nav-menu-checklist.php',
			'/wp-admin/includes/class-walker-nav-menu-edit.php',
			'/wp-admin/includes/class-wp-ajax-upgrader-skin.php',
			'/wp-admin/includes/class-wp-application-passwords-list-table.php',
			'/wp-admin/includes/class-wp-automatic-updater.php',
			'/wp-admin/includes/class-wp-comments-list-table.php',
			'/wp-admin/includes/class-wp-community-events.php',
			'/wp-admin/includes/class-wp-debug-data.php',
			'/wp-admin/includes/class-wp-filesystem-base.php',
			'/wp-admin/includes/class-wp-filesystem-direct.php',
			'/wp-admin/includes/class-wp-filesystem-ftpext.php',
			'/wp-admin/includes/class-wp-filesystem-ftpsockets.php',
			'/wp-admin/includes/class-wp-filesystem-ssh2.php',
			'/wp-admin/includes/class-wp-importer.php',
			'/wp-admin/includes/class-wp-internal-pointers.php',
			'/wp-admin/includes/class-wp-links-list-table.php',
			'/wp-admin/includes/class-wp-list-table-compat.php',
			'/wp-admin/includes/class-wp-list-table.php',
			'/wp-admin/includes/class-wp-media-list-table.php',
			'/wp-admin/includes/class-wp-ms-sites-list-table.php',
			'/wp-admin/includes/class-wp-ms-themes-list-table.php',
			'/wp-admin/includes/class-wp-ms-users-list-table.php',
			'/wp-admin/includes/class-wp-plugin-install-list-table.php',
			'/wp-admin/includes/class-wp-plugins-list-table.php',
			'/wp-admin/includes/class-wp-post-comments-list-table.php',
			'/wp-admin/includes/class-wp-posts-list-table.php',
			'/wp-admin/includes/class-wp-privacy-data-export-requests-list-table.php',
			'/wp-admin/includes/class-wp-privacy-data-removal-requests-list-table.php',
			'/wp-admin/includes/class-wp-privacy-policy-content.php',
			'/wp-admin/includes/class-wp-privacy-requests-table.php',
			'/wp-admin/includes/class-wp-screen.php',
			'/wp-admin/includes/class-wp-site-health-auto-updates.php',
			'/wp-admin/includes/class-wp-site-health.php',
			'/wp-admin/includes/class-wp-site-icon.php',
			'/wp-admin/includes/class-wp-terms-list-table.php',
			'/wp-admin/includes/class-wp-theme-install-list-table.php',
			'/wp-admin/includes/class-wp-themes-list-table.php',
			'/wp-admin/includes/class-wp-upgrader.php',
			'/wp-admin/includes/class-wp-upgrader-skin.php',
			'/wp-admin/includes/class-wp-upgrader-skins.php',
			'/wp-admin/includes/class-wp-users-list-table.php',
			'/wp-admin/includes/comment.php',
			'/wp-admin/includes/continents-cities.php',
			'/wp-admin/includes/credits.php',
			'/wp-admin/includes/dashboard.php',
			'/wp-admin/includes/deprecated.php',
			'/wp-admin/includes/edit-tag-messages.php',
			'/wp-admin/includes/export.php',
			'/wp-admin/includes/file.php',
			'/wp-admin/includes/image-edit.php',
			'/wp-admin/includes/image.php',
			'/wp-admin/includes/import.php',
			'/wp-admin/includes/list-table.php',
			'/wp-admin/includes/media.php',
			'/wp-admin/includes/menu.php',
			'/wp-admin/includes/meta-boxes.php',
			'/wp-admin/includes/misc.php',
			'/wp-admin/includes/ms-admin-filters.php',
			'/wp-admin/includes/ms-deprecated.php',
			'/wp-admin/includes/ms.php',
			'/wp-admin/includes/nav-menu.php',
			'/wp-admin/includes/network.php',
			'/wp-admin/includes/noop.php',
			'/wp-admin/includes/options.php',
			'/wp-admin/includes/plugin-install.php',
			'/wp-admin/includes/plugin.php',
			'/wp-admin/includes/post.php',
			'/wp-admin/includes/privacy-tools.php',
			'/wp-admin/includes/revision.php',
			'/wp-admin/includes/schema.php',
			'/wp-admin/includes/screen.php',
			'/wp-admin/includes/taxonomy.php',
			'/wp-admin/includes/template.php',
			'/wp-admin/includes/theme-install.php',
			'/wp-admin/includes/theme.php',
			'/wp-admin/includes/translation-install.php',
			'/wp-admin/includes/update-core.php',
			'/wp-admin/includes/update.php',
			'/wp-admin/includes/upgrade.php',
			'/wp-admin/includes/user.php',
			'/wp-admin/includes/widgets.php',
			'/wp-admin/menu-header.php',
			'/wp-admin/admin-header.php',
			'/wp-admin/admin-footer.php',
			'/wp-admin/edit-form-blocks.php',
			'/wp-admin/edit-form-advanced.php',
			'/wp-admin/edit-form-comment.php',
			'/wp-admin/edit-link-form.php',
			'/wp-admin/edit-tag-form.php',
			'/wp-admin/link-parse-opml.php',
			'/wp-admin/upgrade-functions.php',
			'/wp-admin/network/menu.php',
			'/wp-admin/user/menu.php',
			'/wp-admin/user/admin.php',
			'/wp-admin/user/index.php',
			'/wp-admin/user/profile.php',
			'/wp-admin/user/user-edit.php',
			'/wp-admin/user/about.php',
			'/wp-admin/user/credits.php',
			'/wp-admin/user/contribute.php',
			'/wp-admin/user/freedoms.php',
			'/wp-admin/user/privacy.php',
			'/wp-admin/profile.php',
			'/wp-admin/user-edit.php',
			'/wp-admin/user-new.php',
			'/wp-admin/post-new.php',
			'/wp-admin/post.php',
			'/wp-admin/install.php',
			'/wp-admin/setup-config.php',
			'/wp-admin/install-helper.php',
			'/wp-admin/options.php',
			'/wp-admin/options-general.php',
			'/wp-admin/options-writing.php',
			'/wp-admin/options-reading.php',
			'/wp-admin/options-discussion.php',
			'/wp-admin/options-media.php',
			'/wp-admin/options-permalink.php',
			'/wp-admin/options-privacy.php',
			'/wp-admin/privacy-policy-guide.php',
			'/wp-admin/about.php',
			'/wp-admin/credits.php',
			'/wp-admin/contribute.php',
			'/wp-admin/freedoms.php',
			'/wp-admin/privacy.php',
			'/wp-admin/plugin-install.php',
			'/wp-admin/plugin-editor.php',
			'/wp-admin/theme-install.php',
			'/wp-admin/theme-editor.php',
			'/wp-admin/widgets.php',
			'/wp-admin/widgets-form.php',
			'/wp-admin/widgets-form-blocks.php',
			'/wp-admin/nav-menus.php',
			'/wp-admin/font-library.php',
			'/wp-admin/customize.php',
			'/wp-admin/authorize-application.php',
			'/wp-admin/site-editor.php',
			'/wp-admin/press-this.php',
			'/wp-admin/term.php',
			'/wp-admin/revision.php',
			'/wp-admin/moderation.php',
			'/wp-admin/my-sites.php',
			'/wp-admin/ms-sites.php',
			'/wp-admin/ms-users.php',
			'/wp-admin/ms-themes.php',
			'/wp-admin/ms-edit.php',
			'/wp-admin/ms-admin.php',
			'/wp-admin/ms-options.php',
			'/wp-admin/ms-upgrade-network.php',
			'/wp-admin/plugins.php',
			'/wp-admin/themes.php',
			'/wp-admin/users.php',
			'/wp-admin/edit.php',
			'/wp-admin/edit-tags.php',
			'/wp-admin/edit-comments.php',
			'/wp-admin/comment.php',
			'/wp-admin/link-manager.php',
			'/wp-admin/link-add.php',
			'/wp-admin/link.php',
			'/wp-admin/media.php',
			'/wp-admin/media-upload.php',
			'/wp-admin/upload.php',
			'/wp-admin/media-new.php',
			'/wp-admin/tools.php',
			'/wp-admin/site-health.php',
			'/wp-admin/site-health-info.php',
			'/wp-admin/export.php',
			'/wp-admin/import.php',
			'/wp-admin/export-personal-data.php',
			'/wp-admin/erase-personal-data.php',
			'/wp-admin/network.php',
			'/wp-admin/network/admin.php',
			'/wp-admin/network/setup.php',
			'/wp-admin/ms-delete-site.php',
			'/wp-admin/network/index.php',
			'/wp-admin/network/sites.php',
			'/wp-admin/network/users.php',
			'/wp-admin/network/themes.php',
			'/wp-admin/network/plugins.php',
			'/wp-admin/network/settings.php',
			'/wp-admin/network/site-new.php',
			'/wp-admin/network/site-info.php',
			'/wp-admin/network/site-settings.php',
			'/wp-admin/network/site-users.php',
			'/wp-admin/network/site-themes.php',
			'/wp-admin/network/user-new.php',
			'/wp-admin/network/edit.php',
			'/wp-admin/network/update.php',
			'/wp-admin/network/update-core.php',
			'/wp-admin/network/plugin-install.php',
			'/wp-admin/network/plugin-editor.php',
			'/wp-admin/network/theme-editor.php',
			'/wp-admin/network/privacy.php',
			'/wp-admin/network/about.php',
			'/wp-admin/network/credits.php',
			'/wp-admin/network/contribute.php',
			'/wp-admin/network/freedoms.php',
			'/wp-admin/network/profile.php',
			'/wp-admin/network/user-edit.php',
			'/wp-admin/network/upgrade.php',
			'/wp-admin/network/theme-install.php',
			'/wp-admin/upgrade.php',
			'/wp-admin/update.php',
			'/wp-admin/update-core.php',
			'/wp-admin/maint/repair.php',
			'/wp-admin/admin-ajax.php',
			'/wp-admin/admin-post.php',
			'/wp-admin/async-upload.php',
			'/xmlrpc.php',
			'/wp-cron.php',
		);
	}

	/**
	 * Enforces plugin compatibility mode endpoint restrictions.
	 *
	 * @param string     $endpoint Endpoint path.
	 * @param array|null $settings Optional settings override.
	 * @return bool
	 */
	function wp_rust_gateway_plugin_mode_allows_endpoint( $endpoint, $settings = null ) {
		if ( null === $settings ) {
			$settings = wp_rust_gateway_get_settings();
		}

		$mode = isset( $settings['plugin_compat_mode'] ) ? strtolower( trim( (string) $settings['plugin_compat_mode'] ) ) : 'php-runtime';
		if ( 'php-runtime' !== $mode ) {
			return true;
		}

		if ( 0 === strpos( $endpoint, '/__wp_rust/' ) || 0 === strpos( $endpoint, '/wp-json' ) ) {
			return true;
		}

		return in_array( $endpoint, wp_rust_gateway_php_runtime_core_endpoints(), true );
	}
}

if ( ! function_exists( 'wp_rust_gateway_try_proxy' ) ) {
	/**
	 * Attempts to proxy request execution to the Rust runtime.
	 *
	 * This returns false when proxying should be skipped or if Rust does not
	 * explicitly signal that it handled the request.
	 *
	 * @param string $endpoint Endpoint path (for example `/wp-login.php`).
	 * @return bool True when request has been proxied and output is complete.
	 */
	function wp_rust_gateway_try_proxy( $endpoint ) {
		$settings = wp_rust_gateway_get_settings();
		if ( ! wp_rust_gateway_should_proxy( $endpoint, $settings ) ) {
			return false;
		}

		$request_method = isset( $_SERVER['REQUEST_METHOD'] ) ? strtoupper( $_SERVER['REQUEST_METHOD'] ) : 'GET';
		if ( ! wp_rust_gateway_method_allowed( $request_method, $settings ) ) {
			if ( empty( $settings['fallback_enabled'] ) ) {
				wp_rust_gateway_hard_fail( 'unsupported_method', 502 );
				return true;
			}
			return false;
		}

		$request_uri = isset( $_SERVER['REQUEST_URI'] ) ? $_SERVER['REQUEST_URI'] : $endpoint;
		$proxy_uri   = $request_uri;
		if ( 0 === strpos( $endpoint, '/__wp_rust/' ) ) {
			$proxy_uri = $endpoint;
		}
		$target_url  = $settings['backend_url'] . $proxy_uri;
		$request_body = '';
		if ( in_array( $request_method, array( 'POST', 'PUT', 'PATCH', 'DELETE' ), true ) ) {
			$request_body = file_get_contents( 'php://input' );
			if ( false === $request_body ) {
				$request_body = '';
			}
		}

		$headers = array(
			'X-WP-Rust-Gateway: 1',
			'X-WP-Rust-Endpoint: ' . $endpoint,
		);
		if ( ! empty( $_SERVER['CONTENT_TYPE'] ) ) {
			$headers[] = 'Content-Type: ' . $_SERVER['CONTENT_TYPE'];
		}

		foreach ( $_SERVER as $key => $value ) {
			if ( 0 !== strpos( $key, 'HTTP_' ) ) {
				continue;
			}

			$header_name = str_replace( '_', '-', substr( $key, 5 ) );
			if ( in_array( strtoupper( $header_name ), array( 'HOST', 'CONTENT-LENGTH', 'CONNECTION' ), true ) ) {
				continue;
			}
			$headers[] = $header_name . ': ' . $value;
		}
		if ( ! empty( $_SERVER['HTTP_HOST'] ) ) {
			$headers[] = 'X-Forwarded-Host: ' . $_SERVER['HTTP_HOST'];
		}

		$context = stream_context_create(
			array(
				'http' => array(
					'method'        => $request_method,
					'header'        => implode( "\r\n", $headers ),
					'content'       => $request_body,
					'ignore_errors' => true,
					'follow_location' => 0,
					'max_redirects' => 0,
					'timeout'       => max( 1, (float) $settings['timeout_ms'] / 1000 ),
				),
			)
		);

		$response_body = @file_get_contents( $target_url, false, $context );
		$response_headers = isset( $http_response_header ) ? $http_response_header : array(); // phpcs:ignore WordPress.WP.GlobalVariablesOverride.Prohibited

		if ( false === $response_body && empty( $response_headers ) ) {
			if ( empty( $settings['fallback_enabled'] ) ) {
				wp_rust_gateway_hard_fail( 'rust_unavailable', 502 );
				return true;
			}
			return false;
		}

		$status_code = 0;
		if ( ! empty( $response_headers[0] ) && preg_match( '#\s(\d{3})\s#', $response_headers[0], $matches ) ) {
			$status_code = (int) $matches[1];
		}

		$handled_by_rust = false;
		foreach ( $response_headers as $header_line ) {
			if ( 0 === stripos( $header_line, 'X-WP-Rust-Handled:' ) && false !== stripos( $header_line, '1' ) ) {
				$handled_by_rust = true;
				break;
			}
		}

		$maintenance_response = '/__wp_rust/maintenance' === $endpoint && 503 === $status_code;
		if ( ! $handled_by_rust || ( $status_code >= 500 && ! $maintenance_response ) || 404 === $status_code || 501 === $status_code ) {
			if ( empty( $settings['fallback_enabled'] ) ) {
				wp_rust_gateway_hard_fail( 'rust_unhandled', 502 );
				return true;
			}
			return false;
		}

		if ( $status_code > 0 ) {
			http_response_code( $status_code );
		}

		foreach ( $response_headers as $header_line ) {
			if ( false === strpos( $header_line, ':' ) ) {
				continue;
			}
			list( $name ) = explode( ':', $header_line, 2 );
			$normalized_name = strtolower( trim( $name ) );
			if ( in_array( $normalized_name, array( 'connection', 'transfer-encoding', 'content-length' ), true ) ) {
				continue;
			}
			header( $header_line, false );
		}

		if ( 'HEAD' !== $request_method ) {
			echo $response_body; // phpcs:ignore WordPress.Security.EscapeOutput.OutputNotEscaped
		}

		return true;
	}
}

if ( ! function_exists( 'wp_rust_gateway_method_allowed' ) ) {
	/**
	 * Checks whether an HTTP method is allowed for Rust proxying.
	 *
	 * @param string $method HTTP request method.
	 * @param array|null $settings Optional settings override.
	 * @return bool
	 */
	function wp_rust_gateway_method_allowed( $method, $settings = null ) {
		if ( null === $settings ) {
			$settings = wp_rust_gateway_get_settings();
		}

		$method = strtoupper( trim( (string) $method ) );
		if ( '' === $method ) {
			return false;
		}

		return in_array( $method, $settings['method_allowlist'], true ) || in_array( '*', $settings['method_allowlist'], true );
	}
}

if ( ! function_exists( 'wp_rust_gateway_hard_fail' ) ) {
	/**
	 * Emits deterministic failure when Rust cutover disallows PHP fallback.
	 *
	 * @param string $error_code Error code.
	 * @param int    $status_code HTTP status code.
	 * @return void
	 */
	function wp_rust_gateway_hard_fail( $error_code, $status_code ) {
		http_response_code( (int) $status_code );
		header( 'Content-Type: application/json; charset=utf-8', true );
		echo json_encode(
			array(
				'error' => (string) $error_code,
			)
		); // phpcs:ignore WordPress.Security.EscapeOutput.OutputNotEscaped
	}
}
