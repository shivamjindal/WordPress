<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/page-business-home.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/page-business-home.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Business homepage
 * Slug: twentytwentyfive/page-business-home
 * Categories: twentytwentyfive_page, featured
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A business homepage pattern.
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>

<!-- wp:pattern {"slug":"twentytwentyfive/cta-centered-heading"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/overlapped-images"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/services-3-col"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/testimonials-large"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/pricing-2-col"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/cta-newsletter"} /-->
