<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/edit-widgets.min.asset.php' );
if ( '/wp-includes/js/dist/edit-widgets.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-api-fetch', 'wp-block-editor', 'wp-block-library', 'wp-blocks', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-deprecated', 'wp-dom', 'wp-element', 'wp-hooks', 'wp-i18n', 'wp-keyboard-shortcuts', 'wp-keycodes', 'wp-media-utils', 'wp-notices', 'wp-patterns', 'wp-plugins', 'wp-preferences', 'wp-primitives', 'wp-private-apis', 'wp-url', 'wp-viewport', 'wp-widgets'), 'module_dependencies' => array(array('id' => '@wordpress/route', 'import' => 'static')), 'version' => '0a3c1a7b25581214ceb0');