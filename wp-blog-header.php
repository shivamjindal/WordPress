<?php
/**
 * Loads the WordPress environment and template.
 *
 * @package WordPress
 */
require_once __DIR__ . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( wp_rust_gateway_current_request_path( '/wp-blog-header.php' ) ) ) {
	exit;
}

if ( ! isset( $wp_did_header ) ) {

	$wp_did_header = true;

	// Load the WordPress library.
	require_once __DIR__ . '/wp-load.php';

	// Set up the WordPress query.
	wp();

	// Load the theme template.
	require_once ABSPATH . WPINC . '/template-loader.php';

}
