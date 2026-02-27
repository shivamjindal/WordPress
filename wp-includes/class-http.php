<?php
/**
 * Core class used for managing HTTP transports and making HTTP requests.
 *
 * This file is deprecated, use 'wp-includes/class-wp-http.php' instead.
 *
 * @deprecated 5.9.0
 * @package WordPress
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/class-http.php' ) === '/wp-includes/class-http.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/class-http.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '5.9.0', WPINC . '/class-wp-http.php' );

/** WP_Http class */
require_once ABSPATH . WPINC . '/class-wp-http.php';
