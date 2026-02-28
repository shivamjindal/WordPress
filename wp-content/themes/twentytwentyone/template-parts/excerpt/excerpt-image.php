<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyone/template-parts/excerpt/excerpt-image.php' );
if ( '/wp-content/themes/twentytwentyone/template-parts/excerpt/excerpt-image.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Shows the appropriate content for the Image post format.
 *
 * @link https://developer.wordpress.org/themes/basics/template-hierarchy/
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_One
 * @since Twenty Twenty-One 1.0
 */

// If there is no featured-image, print the first image block found.
if (
	! twenty_twenty_one_can_show_post_thumbnail() &&
	has_block( 'core/image', get_the_content() )
) {

	twenty_twenty_one_print_first_instance_of_block( 'core/image', get_the_content() );
}

the_excerpt();
