<?php
/**
 * Deprecated. Use rss.php instead.
 *
 * @package WordPress
 * @deprecated 2.1.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/rss-functions.php' ) === '/wp-includes/rss-functions.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/rss-functions.php' ) ) {
	exit;
}

if ( ! defined( 'ABSPATH' ) ) {
	exit();
}

_deprecated_file( basename( __FILE__ ), '2.1.0', WPINC . '/rss.php' );
require_once ABSPATH . WPINC . '/rss.php';
