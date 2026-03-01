<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/page-landing-podcast.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/page-landing-podcast.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Landing page for podcast
 * Slug: twentytwentyfive/page-landing-podcast
 * Categories: twentytwentyfive_page, featured
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A landing page for the podcast with a hero section, description, logos, grid with videos and newsletter signup.
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>

<!-- wp:pattern {"slug":"twentytwentyfive/hero-podcast"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/heading-and-paragraph-with-image"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/logos"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/grid-videos"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/cta-newsletter"} /-->
