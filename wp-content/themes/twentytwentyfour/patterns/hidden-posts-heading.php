<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/hidden-posts-heading.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/hidden-posts-heading.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Posts heading
 * Slug: twentytwentyfour/hidden-posts-heading
 * Categories: hidden
 * Inserter: no
 */
?>
<!-- wp:heading {"level":1,"align":"wide","style":{"spacing":{"padding":{"top":"var:preset|spacing|50"}}}} -->
<h1 class="wp-block-heading alignwide" style="padding-top:var(--wp--preset--spacing--50)"><?php esc_html_e( 'Posts', 'twentytwentyfour' ); ?></h1>
<!-- /wp:heading -->
