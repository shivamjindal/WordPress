<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/page-shop-home.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/page-shop-home.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Shop homepage
 * Slug: twentytwentyfive/page-shop-home
 * Categories: twentytwentyfive_page
 * Keywords: starter
 * Block Types: core/post-content
 * Post Types: page, wp_template
 * Viewport width: 1400
 * Description: A shop homepage pattern.
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>

<!-- wp:pattern {"slug":"twentytwentyfive/banner-intro-image"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/grid-with-categories"} /-->
<!-- wp:pattern {"slug":"twentytwentyfive/media-instagram-grid"} /-->
