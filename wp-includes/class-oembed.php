<?php
/**
 * API for fetching the HTML to embed remote content based on a provided URL.
 *
 * This file is deprecated, use 'wp-includes/class-wp-oembed.php' instead.
 *
 * @deprecated 5.3.0
 * @package WordPress
 * @subpackage oEmbed
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/class-oembed.php' ) === '/wp-includes/class-oembed.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/class-oembed.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '5.3.0', WPINC . '/class-wp-oembed.php' );

/** WP_oEmbed class */
require_once ABSPATH . WPINC . '/class-wp-oembed.php';
