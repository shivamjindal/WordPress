<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/blocks.min.asset.php' );
if ( '/wp-includes/js/dist/blocks.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-autop', 'wp-blob', 'wp-block-serialization-default-parser', 'wp-data', 'wp-deprecated', 'wp-dom', 'wp-element', 'wp-hooks', 'wp-html-entities', 'wp-i18n', 'wp-is-shallow-equal', 'wp-private-apis', 'wp-rich-text', 'wp-shortcode', 'wp-warning'), 'version' => 'df3ea4980b2830f1957a');