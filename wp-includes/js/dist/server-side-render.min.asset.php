<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/server-side-render.min.asset.php' );
if ( '/wp-includes/js/dist/server-side-render.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-api-fetch', 'wp-blocks', 'wp-components', 'wp-compose', 'wp-data', 'wp-element', 'wp-i18n', 'wp-url'), 'version' => '3283cbe3e651b5676910');