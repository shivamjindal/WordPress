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
			'/wp-login.php',
			'/wp-signup.php',
			'/wp-activate.php',
			'/wp-comments-post.php',
			'/wp-mail.php',
			'/wp-trackback.php',
			'/wp-links-opml.php',
			'/wp-admin',
			'/wp-admin/',
			'/wp-admin/index.php',
			'/wp-admin/admin.php',
			'/wp-admin/profile.php',
			'/wp-admin/user-edit.php',
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
			'/wp-admin/plugins.php',
			'/wp-admin/themes.php',
			'/wp-admin/users.php',
			'/wp-admin/tools.php',
			'/wp-admin/site-health.php',
			'/wp-admin/export.php',
			'/wp-admin/import.php',
			'/wp-admin/export-personal-data.php',
			'/wp-admin/erase-personal-data.php',
			'/wp-admin/network.php',
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
