<?php
/**
 * Dependencies API: WP_Styles class
 *
 * This file is deprecated, use 'wp-includes/class-wp-styles.php' instead.
 *
 * @deprecated 6.1.0
 * @package WordPress
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/class.wp-styles.php' ) === '/wp-includes/class.wp-styles.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/class.wp-styles.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '6.1.0', WPINC . '/class-wp-styles.php' );

/** WP_Styles class */
require_once ABSPATH . WPINC . '/class-wp-styles.php';
