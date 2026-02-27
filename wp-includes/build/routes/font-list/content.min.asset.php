<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/build/routes/font-list/content.min.asset.php' );
if ( '/wp-includes/build/routes/font-list/content.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-api-fetch', 'wp-block-editor', 'wp-blocks', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-date', 'wp-editor', 'wp-element', 'wp-i18n', 'wp-keycodes', 'wp-primitives', 'wp-private-apis', 'wp-style-engine'), 'module_dependencies' => array(array('id' => '@wordpress/a11y', 'import' => 'static'), array('id' => '@wordpress/route', 'import' => 'static')), 'version' => '992ce63a7329ac2506e7');