<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentytwo/inc/patterns/header-centered-logo-black-background.php' );
if ( '/wp-content/themes/twentytwentytwo/inc/patterns/header-centered-logo-black-background.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Header with centered logo and black background
 */
return array(
	'title'      => __( 'Header with centered logo and background', 'twentytwentytwo' ),
	'categories' => array( 'header' ),
	'blockTypes' => array( 'core/template-part/header' ),
	'content'    => '<!-- wp:group {"align":"full","style":{"spacing":{"padding":{"bottom":"var(--wp--custom--spacing--small, 1.25rem)","top":"var(--wp--custom--spacing--small, 1.25rem)"}},"elements":{"link":{"color":{"text":"var:preset|color|background"}}}},"backgroundColor":"foreground","textColor":"background","layout":{"type":"flex","justifyContent":"center"}} -->
					<div class="wp-block-group alignfull has-background-color has-foreground-background-color has-text-color has-background has-link-color" style="padding-top:var(--wp--custom--spacing--small, 1.25rem);padding-bottom:var(--wp--custom--spacing--small, 1.25rem)"><!-- wp:navigation {"layout":{"type":"flex","setCascadingProperties":true,"justifyContent":"right"}} -->
					<!-- wp:navigation-link {"isTopLevelLink":true} /-->

					<!-- wp:navigation-link {"isTopLevelLink":true} /-->

					<!-- wp:site-logo {"width":90} /-->

					<!-- wp:navigation-link {"isTopLevelLink":true} /-->

					<!-- wp:navigation-link {"isTopLevelLink":true} /-->
					<!-- /wp:navigation --></div>
					<!-- /wp:group -->',
);
