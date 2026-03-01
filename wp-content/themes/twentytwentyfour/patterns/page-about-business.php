<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/page-about-business.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/page-about-business.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: About
 * Slug: twentytwentyfour/page-about-business
 * Categories: twentytwentyfour_page
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A business about page with a hero section, a text section, a services section, a team section, a clients section, a FAQ section, and a CTA section.
 */
?>

<!-- wp:pattern {"slug":"twentytwentyfour/text-title-left-image-right"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/text-project-details"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/cta-services-image-left"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/team-4-col"} /-->
<!-- wp:pattern {"slug":"clients-section"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/text-faq"} /-->
<!-- wp:pattern {"slug":"twentytwentyfour/cta-content-image-on-right"} /-->
