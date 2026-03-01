<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/binding-format.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/binding-format.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Post format name
 * Slug: twentytwentyfive/binding-format
 * Categories: twentytwentyfive_post-format
 * Description: Prints the name of the post format with the help of the Block Bindings API.
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>
<!-- wp:paragraph {"metadata":{"bindings":{"content":{"source":"twentytwentyfive/format"}}},"fontSize":"small"} -->
<p class="has-small-font-size"></p>
<!-- /wp:paragraph -->
