<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfive/patterns/hidden-search.php' );
if ( '/wp-content/themes/twentytwentyfive/patterns/hidden-search.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Search
 * Slug: twentytwentyfive/hidden-search
 * Inserter: no
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_Five
 * @since Twenty Twenty-Five 1.0
 */

?>

<!-- wp:search {"label":"<?php echo esc_html_x( 'Search', 'Search form label.', 'twentytwentyfive' ); ?>","showLabel":false,"placeholder":"<?php echo esc_attr_x( 'Type here...', 'Search input field placeholder text.', 'twentytwentyfive' ); ?>","buttonText":"<?php echo esc_attr_x( 'Search', 'Button text. Verb.', 'twentytwentyfive' ); ?>"} /-->
