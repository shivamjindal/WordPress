<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/core-data.min.asset.php' );
if ( '/wp-includes/js/dist/core-data.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-api-fetch', 'wp-block-editor', 'wp-blocks', 'wp-compose', 'wp-data', 'wp-deprecated', 'wp-element', 'wp-hooks', 'wp-html-entities', 'wp-i18n', 'wp-private-apis', 'wp-rich-text', 'wp-undo-manager', 'wp-url', 'wp-warning'), 'version' => '3b0026fbf1b7f47d1169');