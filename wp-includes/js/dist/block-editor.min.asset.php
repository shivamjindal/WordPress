<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/block-editor.min.asset.php' );
if ( '/wp-includes/js/dist/block-editor.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-dom', 'react-jsx-runtime', 'wp-a11y', 'wp-api-fetch', 'wp-blob', 'wp-block-serialization-default-parser', 'wp-blocks', 'wp-commands', 'wp-components', 'wp-compose', 'wp-data', 'wp-date', 'wp-deprecated', 'wp-dom', 'wp-element', 'wp-hooks', 'wp-html-entities', 'wp-i18n', 'wp-is-shallow-equal', 'wp-keyboard-shortcuts', 'wp-keycodes', 'wp-notices', 'wp-preferences', 'wp-primitives', 'wp-priority-queue', 'wp-private-apis', 'wp-rich-text', 'wp-style-engine', 'wp-token-list', 'wp-url', 'wp-warning'), 'version' => '4b383afbd1e5227e4044');