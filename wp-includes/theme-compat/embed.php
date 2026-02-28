<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/theme-compat/embed.php' );
if ( '/wp-includes/theme-compat/embed.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * Contains the post embed base template
 *
 * When a post is embedded in an iframe, this file is used to create the output
 * if the active theme does not include an embed.php template.
 *
 * @package WordPress
 * @subpackage oEmbed
 * @since 4.4.0
 */

get_header( 'embed' );

if ( have_posts() ) :
	while ( have_posts() ) :
		the_post();
		get_template_part( 'embed', 'content' );
	endwhile;
else :
	get_template_part( 'embed', '404' );
endif;

get_footer( 'embed' );
