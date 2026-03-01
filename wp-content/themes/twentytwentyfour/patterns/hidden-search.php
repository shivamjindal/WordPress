<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/hidden-search.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/hidden-search.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Search
 * Slug: twentytwentyfour/hidden-search
 * Inserter: no
 */
?>

<!-- wp:search {"label":"<?php echo esc_attr_x( 'Search', 'search form label', 'twentytwentyfour' ); ?>","showLabel":false,"buttonText":"<?php echo esc_attr_x( 'Search', 'search button text', 'twentytwentyfour' ); ?>","fontSize":"medium"} /-->
