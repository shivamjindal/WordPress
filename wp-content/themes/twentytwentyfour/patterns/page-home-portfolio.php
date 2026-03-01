<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/page-home-portfolio.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/page-home-portfolio.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Portfolio home with post featured images
 * Slug: twentytwentyfour/page-home-portfolio
 * Categories: twentytwentyfour_page
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A portfolio home page with a description and a 4-column post section with only feature images.
 */
?>

<!-- wp:pattern {"slug":"twentytwentyfour/hidden-portfolio-hero"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/posts-images-only-offset-4-col"} /-->
