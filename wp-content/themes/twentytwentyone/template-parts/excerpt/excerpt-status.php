<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyone/template-parts/excerpt/excerpt-status.php' );
if ( '/wp-content/themes/twentytwentyone/template-parts/excerpt/excerpt-status.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Shows the appropriate content for the Status post format.
 *
 * @link https://developer.wordpress.org/themes/basics/template-hierarchy/
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_One
 * @since Twenty Twenty-One 1.0
 */

// Print the full content.
the_content();
