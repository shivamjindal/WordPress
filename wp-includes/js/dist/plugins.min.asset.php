<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/plugins.min.asset.php' );
if ( '/wp-includes/js/dist/plugins.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-compose', 'wp-deprecated', 'wp-element', 'wp-hooks', 'wp-is-shallow-equal', 'wp-primitives'), 'version' => '7144b5b613cb5942d347');