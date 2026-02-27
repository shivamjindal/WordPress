<?php
/**
 * Query: Standard.
 *
 * @package WordPress
 */

require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/block-patterns/query-standard-posts.php' );
if ( '/wp-includes/block-patterns/query-standard-posts.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

return array(
	'title'      => _x( 'Standard', 'Block pattern title' ),
	'blockTypes' => array( 'core/query' ),
	'categories' => array( 'query' ),
	'content'    => '<!-- wp:query {"query":{"perPage":3,"pages":0,"offset":0,"postType":"post","order":"desc","orderBy":"date","author":"","search":"","exclude":[],"sticky":"","inherit":false}} -->
					<div class="wp-block-query">
					<!-- wp:post-template -->
					<!-- wp:post-title {"isLink":true} /-->
					<!-- wp:post-featured-image  {"isLink":true,"align":"wide"} /-->
					<!-- wp:post-excerpt /-->
					<!-- wp:separator -->
					<hr class="wp-block-separator"/>
					<!-- /wp:separator -->
					<!-- wp:post-date /-->
					<!-- /wp:post-template -->
					</div>
					<!-- /wp:query -->',
);
