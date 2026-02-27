<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/nux.min.asset.php' );
if ( '/wp-includes/js/dist/nux.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-components', 'wp-compose', 'wp-data', 'wp-deprecated', 'wp-element', 'wp-i18n', 'wp-primitives'), 'version' => 'fa9377877c3600a8e1d3');