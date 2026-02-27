<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/data-controls.min.asset.php' );
if ( '/wp-includes/js/dist/data-controls.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-api-fetch', 'wp-data', 'wp-deprecated'), 'version' => '9864b9a790f21e251b90');