<?php
/**
 * Session API
 *
 * @since 4.0.0
 * @deprecated 4.7.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/session.php' ) === '/wp-includes/session.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/session.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '4.7.0' );

require_once ABSPATH . WPINC . '/class-wp-session-tokens.php';
require_once ABSPATH . WPINC . '/class-wp-user-meta-session-tokens.php';
