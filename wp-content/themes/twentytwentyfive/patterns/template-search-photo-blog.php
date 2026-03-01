<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/template-search-photo-blog.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/template-search-photo-blog.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Photo blog search results
 * Slug: twentytwentyfive/template-search-photo-blog
 * Template Types: search
 * Viewport width: 1400
 * Inserter: no
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>
<!-- wp:template-part {"slug":"header"} /-->

<!-- wp:group {"tagName":"main","style":{"spacing":{"margin":{"top":"var:preset|spacing|60"}}},"layout":{"type":"constrained"}} -->
<main class="wp-block-group" style="margin-top:var(--wp--preset--spacing--60)">
	<!-- wp:query-title {"type":"search","textAlign":"center","align":"wide"} /-->
	<!-- wp:group {"align":"wide","layout":{"type":"default"}} -->
	<div class="wp-block-group alignwide">
		<!-- wp:pattern {"slug":"twentytwentyfive/hidden-search"} /-->
	</div>
	<!-- /wp:group -->
	<!-- wp:pattern {"slug":"twentytwentyfive/template-query-loop-photo-blog"} /-->
</main>
<!-- /wp:group -->

<!-- wp:template-part {"slug":"footer"} /-->
