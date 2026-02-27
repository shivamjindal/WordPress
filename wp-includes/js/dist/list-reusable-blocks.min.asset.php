<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/list-reusable-blocks.min.asset.php' );
if ( '/wp-includes/js/dist/list-reusable-blocks.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-api-fetch', 'wp-blob', 'wp-components', 'wp-compose', 'wp-element', 'wp-i18n'), 'version' => '823d3e15c6efc27b72cd');