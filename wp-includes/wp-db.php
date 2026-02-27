<?php
/**
 * WordPress database access abstraction class.
 *
 * This file is deprecated, use 'wp-includes/class-wpdb.php' instead.
 *
 * @deprecated 6.1.0
 * @package WordPress
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/wp-db.php' ) === '/wp-includes/wp-db.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/wp-db.php' ) ) {
	exit;
}

if ( function_exists( '_deprecated_file' ) ) {
	// Note: WPINC may not be defined yet, so 'wp-includes' is used here.
	_deprecated_file( basename( __FILE__ ), '6.1.0', 'wp-includes/class-wpdb.php' );
}

/** wpdb class */
require_once __DIR__ . '/class-wpdb.php';
