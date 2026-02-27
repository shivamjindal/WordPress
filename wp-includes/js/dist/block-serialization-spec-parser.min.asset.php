<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/block-serialization-spec-parser.min.asset.php' );
if ( '/wp-includes/js/dist/block-serialization-spec-parser.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'version' => '9ebc5e95e1de1cabd1e6');