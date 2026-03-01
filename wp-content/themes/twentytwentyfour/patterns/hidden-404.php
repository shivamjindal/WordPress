<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/hidden-404.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/hidden-404.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: 404
 * Slug: twentytwentyfour/hidden-404
 * Inserter: no
 */
?>

<!-- wp:heading {"level":1} -->
<h1 class="wp-block-heading" id="page-not-found"><?php echo esc_html_x( 'Page Not Found', 'Heading for a webpage that is not found', 'twentytwentyfour' ); ?></h1>
<!-- /wp:heading -->
<!-- wp:paragraph -->
<p><?php echo esc_html_x( 'The page you are looking for does not exist, or it has been moved. Please try searching using the form below.', 'Message to convey that a webpage could not be found', 'twentytwentyfour' ); ?></p>
<!-- /wp:paragraph -->
<!-- wp:pattern {"slug":"twentytwentyfour/hidden-search"} /-->
