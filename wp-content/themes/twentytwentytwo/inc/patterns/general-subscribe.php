<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentytwo/inc/patterns/general-subscribe.php' );
if ( '/wp-content/themes/twentytwentytwo/inc/patterns/general-subscribe.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Subscribe callout block pattern
 */
return array(
	'title'      => __( 'Subscribe callout', 'twentytwentytwo' ),
	'categories' => array( 'featured', 'buttons' ),
	'content'    => '<!-- wp:columns {"verticalAlignment":"center","align":"wide"} -->
					<div class="wp-block-columns alignwide are-vertically-aligned-center"><!-- wp:column {"verticalAlignment":"center"} -->
					<div class="wp-block-column is-vertically-aligned-center"><!-- wp:heading -->
					<h2>' . wp_kses_post( __( 'Watch birds<br>from your inbox', 'twentytwentytwo' ) ) . '</h2>
					<!-- /wp:heading -->

					<!-- wp:buttons -->
					<div class="wp-block-buttons"><!-- wp:button {"fontSize":"medium"} -->
					<div class="wp-block-button has-custom-font-size has-medium-font-size"><a class="wp-block-button__link">' . esc_html__( 'Join our mailing list', 'twentytwentytwo' ) . '</a></div>
					<!-- /wp:button --></div>
					<!-- /wp:buttons --></div>
					<!-- /wp:column -->

					<!-- wp:column {"verticalAlignment":"center","style":{"spacing":{"padding":{"top":"2rem","bottom":"2rem"}}}} -->
					<div class="wp-block-column is-vertically-aligned-center" style="padding-top:2rem;padding-bottom:2rem"><!-- wp:separator {"color":"primary","className":"is-style-wide"} -->
					<hr class="wp-block-separator has-text-color has-background has-primary-background-color has-primary-color is-style-wide"/>
					<!-- /wp:separator --></div>
					<!-- /wp:column --></div>
					<!-- /wp:columns -->',
);
