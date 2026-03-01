<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/hidden-no-results.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/hidden-no-results.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: No results
 * Slug: twentytwentyfour/hidden-no-results
 * Inserter: no
 */
?>
<!-- wp:paragraph -->
<p><?php echo esc_html_x( 'No posts were found.', 'Message explaining that there are no results returned from a search', 'twentytwentyfour' ); ?></p>
<!-- /wp:paragraph -->
