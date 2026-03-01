<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/template-index-portfolio.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/template-index-portfolio.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Portfolio index template
 * Slug: twentytwentyfour/template-index-portfolio
 * Template Types: index
 * Viewport width: 1400
 * Inserter: no
 */
?>

<!-- wp:template-part {"slug":"header","area":"header","tagName":"header"} /-->

<!-- wp:group {"tagName":"main","align":"full","layout":{"type":"constrained"}} -->
<main class="wp-block-group alignfull">
	<!-- wp:heading {"level":1,"align":"wide","style":{"spacing":{"padding":{"top":"var:preset|spacing|50"}}}} -->
	<h1 class="wp-block-heading alignwide" style="padding-top:var(--wp--preset--spacing--50)"><?php esc_html_e( 'Posts', 'twentytwentyfour' ); ?></h1>
	<!-- /wp:heading -->

	<!-- wp:pattern {"slug":"twentytwentyfour/posts-images-only-offset-4-col"} /-->

</main>
<!-- /wp:group -->

<!-- wp:template-part {"slug":"footer","area":"footer","tagName":"footer"} /-->
