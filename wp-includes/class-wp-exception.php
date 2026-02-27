<?php
/**
 * WP_Exception class
 *
 * @package WordPress
 * @since 6.7.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/class-wp-exception.php' ) === '/wp-includes/class-wp-exception.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/class-wp-exception.php' ) ) {
	exit;
}

/**
 * Core base Exception class.
 *
 * Future, more specific, Exceptions should always extend this base class.
 *
 * @since 6.7.0
 */
class WP_Exception extends Exception {}
