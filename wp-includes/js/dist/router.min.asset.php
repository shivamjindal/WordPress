<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/router.min.asset.php' );
if ( '/wp-includes/js/dist/router.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-compose', 'wp-element', 'wp-private-apis', 'wp-url'), 'version' => 'cafe5ab0f12e75f2b357');