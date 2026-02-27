<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/edit-post.min.asset.php' );
if ( '/wp-includes/js/dist/edit-post.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('media-models', 'media-views', 'postbox', 'react-jsx-runtime', 'wp-api-fetch', 'wp-block-editor', 'wp-block-library', 'wp-blocks', 'wp-commands', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-deprecated', 'wp-editor', 'wp-element', 'wp-hooks', 'wp-html-entities', 'wp-i18n', 'wp-keyboard-shortcuts', 'wp-keycodes', 'wp-notices', 'wp-plugins', 'wp-preferences', 'wp-primitives', 'wp-private-apis', 'wp-style-engine', 'wp-url', 'wp-widgets'), 'module_dependencies' => array(array('id' => '@wordpress/route', 'import' => 'static')), 'version' => '093004e1308aa5aa7fd9');