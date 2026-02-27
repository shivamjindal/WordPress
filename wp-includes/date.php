<?php
/**
 * Class for generating SQL clauses that filter a primary query according to date.
 *
 * This file is deprecated, use 'wp-includes/class-wp-date-query.php' instead.
 *
 * @deprecated 5.3.0
 * @package WordPress
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/date.php' ) === '/wp-includes/date.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/date.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '5.3.0', WPINC . '/class-wp-date-query.php' );

/** WP_Date_Query class */
require_once ABSPATH . WPINC . '/class-wp-date-query.php';
