<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/media-utils.min.asset.php' );
if ( '/wp-includes/js/dist/media-utils.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-dom', 'react-jsx-runtime', 'wp-api-fetch', 'wp-blob', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-date', 'wp-deprecated', 'wp-element', 'wp-i18n', 'wp-keycodes', 'wp-primitives', 'wp-private-apis', 'wp-url', 'wp-warning'), 'version' => 'c9a0129d6b272cb10b9a');