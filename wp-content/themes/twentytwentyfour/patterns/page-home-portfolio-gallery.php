<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/page-home-portfolio-gallery.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/page-home-portfolio-gallery.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Portfolio home image gallery
 * Slug: twentytwentyfour/page-home-gallery
 * Categories: twentytwentyfour_page
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A portfolio home page that features a gallery.
 */
?>

<!-- wp:pattern {"slug":"twentytwentyfour/hidden-portfolio-hero"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/gallery-offset-images-grid-4-col"} /-->
