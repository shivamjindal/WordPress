<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyone/template-parts/excerpt/excerpt-link.php' );
if ( '/wp-content/themes/twentytwentyone/template-parts/excerpt/excerpt-link.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Shows the appropriate content for the Link post format.
 *
 * @link https://developer.wordpress.org/themes/basics/template-hierarchy/
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_One
 * @since Twenty Twenty-One 1.0
 */

// Print the 1st instance of a paragraph block. If none is found, print the content.
if ( has_block( 'core/paragraph', get_the_content() ) ) {

	twenty_twenty_one_print_first_instance_of_block( 'core/paragraph', get_the_content() );
} else {

	the_content();
}
