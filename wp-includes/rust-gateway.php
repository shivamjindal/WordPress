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
	 *     backend_url: string,
	 *     timeout_ms: int,
	 *     endpoint_allowlist: string[]
	 * }
	 */
	function wp_rust_gateway_get_settings() {
		$enabled = false;
		if ( defined( 'WP_RUST_GATEWAY_ENABLED' ) ) {
			$enabled = (bool) WP_RUST_GATEWAY_ENABLED;
		} elseif ( getenv( 'WP_RUST_GATEWAY_ENABLED' ) ) {
			$enabled = wp_rust_gateway_parse_truthy( getenv( 'WP_RUST_GATEWAY_ENABLED' ) );
		}

		$backend_url = defined( 'WP_RUST_GATEWAY_BACKEND_URL' ) ? WP_RUST_GATEWAY_BACKEND_URL : '';
		if ( ! $backend_url ) {
			$backend_url = getenv( 'WP_RUST_GATEWAY_BACKEND_URL' ) ? getenv( 'WP_RUST_GATEWAY_BACKEND_URL' ) : 'http://127.0.0.1:8088';
		}
		$backend_url = rtrim( trim( $backend_url ), '/' );

		$timeout_ms = defined( 'WP_RUST_GATEWAY_TIMEOUT_MS' ) ? (int) WP_RUST_GATEWAY_TIMEOUT_MS : 0;
		if ( $timeout_ms <= 0 ) {
			$timeout_ms = (int) ( getenv( 'WP_RUST_GATEWAY_TIMEOUT_MS' ) ? getenv( 'WP_RUST_GATEWAY_TIMEOUT_MS' ) : 1500 );
		}
		if ( $timeout_ms <= 0 ) {
			$timeout_ms = 1500;
		}

		$allowlist_raw = defined( 'WP_RUST_ENDPOINT_ALLOWLIST' ) ? WP_RUST_ENDPOINT_ALLOWLIST : '';
		if ( ! $allowlist_raw ) {
			$allowlist_raw = getenv( 'WP_RUST_ENDPOINT_ALLOWLIST' ) ? getenv( 'WP_RUST_ENDPOINT_ALLOWLIST' ) : '/__wp_rust/health';
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

		return array(
			'enabled'           => $enabled,
			'backend_url'       => $backend_url,
			'timeout_ms'        => $timeout_ms,
			'endpoint_allowlist' => $endpoint_allowlist,
		);
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

		if ( in_array( '*', $settings['endpoint_allowlist'], true ) ) {
			return true;
		}

		return in_array( $endpoint, $settings['endpoint_allowlist'], true );
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
		if ( ! in_array( $request_method, array( 'GET', 'HEAD' ), true ) ) {
			// Restrict to safe methods until request body replay is implemented.
			return false;
		}

		$request_uri = isset( $_SERVER['REQUEST_URI'] ) ? $_SERVER['REQUEST_URI'] : $endpoint;
		$target_url  = $settings['backend_url'] . $request_uri;

		$headers = array(
			'X-WP-Rust-Gateway: 1',
			'X-WP-Rust-Endpoint: ' . $endpoint,
		);

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

		$context = stream_context_create(
			array(
				'http' => array(
					'method'        => $request_method,
					'header'        => implode( "\r\n", $headers ),
					'ignore_errors' => true,
					'timeout'       => max( 1, (float) $settings['timeout_ms'] / 1000 ),
				),
			)
		);

		$response_body = @file_get_contents( $target_url, false, $context );
		$response_headers = isset( $http_response_header ) ? $http_response_header : array(); // phpcs:ignore WordPress.WP.GlobalVariablesOverride.Prohibited

		if ( false === $response_body && empty( $response_headers ) ) {
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

		if ( ! $handled_by_rust || $status_code >= 500 || 404 === $status_code || 501 === $status_code ) {
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
