<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/page-portfolio-overview.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/page-portfolio-overview.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Portfolio project overview
 * Slug: twentytwentyfour/page-portfolio-overview
 * Categories: twentytwentyfour_page, featured
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A full portfolio page with a section for project description, project details, a full screen image, and a gallery section with two images.
 */
?>

<!-- wp:pattern {"slug":"twentytwentyfour/banner-project-description"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/text-project-details"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/gallery-full-screen-image"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/text-centered-statement"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/gallery-project-layout"} /-->
