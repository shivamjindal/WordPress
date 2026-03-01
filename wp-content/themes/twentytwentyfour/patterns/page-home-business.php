<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/page-home-business.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/page-home-business.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Business home
 * Slug: twentytwentyfour/page-home-business
 * Categories: twentytwentyfour_page
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A business home page with a hero section, a text section, a services section, a team section, a clients section, a FAQ section, and a CTA section.
 */
?>

<!-- wp:pattern {"slug":"twentytwentyfour/banner-hero"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/text-feature-grid-3-col"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/text-alternating-images"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/testimonial-centered"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/posts-list"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/cta-subscribe-centered"} /-->
