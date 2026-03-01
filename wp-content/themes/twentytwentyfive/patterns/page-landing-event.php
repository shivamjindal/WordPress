<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/page-landing-event.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/page-landing-event.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Landing page for event
 * Slug: twentytwentyfive/page-landing-event
 * Categories: twentytwentyfive_page, featured
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A landing page for the event with a hero section, description, FAQs and call to action.
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>

<!-- wp:pattern {"slug":"twentytwentyfive/hero-full-width-image"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/heading-and-paragraph-with-image"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/banner-description-images-grid"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/text-faqs"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/contact-centered-social-link"} /-->
