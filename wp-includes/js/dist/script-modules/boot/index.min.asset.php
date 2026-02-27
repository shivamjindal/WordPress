<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/boot/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/boot/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-commands', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-editor', 'wp-element', 'wp-html-entities', 'wp-i18n', 'wp-keyboard-shortcuts', 'wp-keycodes', 'wp-primitives', 'wp-private-apis', 'wp-theme', 'wp-url'), 'module_dependencies' => array(array('id' => '@wordpress/a11y', 'import' => 'static'), array('id' => '@wordpress/lazy-editor', 'import' => 'dynamic'), array('id' => '@wordpress/route', 'import' => 'static')), 'version' => '5d918eb16ecdac63b2b1');