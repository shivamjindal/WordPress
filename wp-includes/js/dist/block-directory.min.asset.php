<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/block-directory.min.asset.php' );
if ( '/wp-includes/js/dist/block-directory.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-a11y', 'wp-api-fetch', 'wp-block-editor', 'wp-blocks', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-editor', 'wp-element', 'wp-hooks', 'wp-html-entities', 'wp-i18n', 'wp-notices', 'wp-plugins', 'wp-primitives', 'wp-url'), 'version' => 'b478308aab5c12b3182d');