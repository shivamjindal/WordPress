<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/page-landing-book.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/page-landing-book.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Landing page for book
 * Slug: twentytwentyfive/page-landing-book
 * Categories: twentytwentyfive_page, featured
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A landing page for the book with a hero section, pre-order links, locations, FAQs and newsletter signup.
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>

<!-- wp:pattern {"slug":"twentytwentyfive/hero-book"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/cta-book-links"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/banner-about-book"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/cta-book-locations"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/text-faqs"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/cta-newsletter"} /-->
