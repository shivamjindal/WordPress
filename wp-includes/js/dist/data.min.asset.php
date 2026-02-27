<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/data.min.asset.php' );
if ( '/wp-includes/js/dist/data.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-compose', 'wp-deprecated', 'wp-element', 'wp-is-shallow-equal', 'wp-priority-queue', 'wp-private-apis', 'wp-redux-routine'), 'version' => '8dc0164cad146febaf2d');