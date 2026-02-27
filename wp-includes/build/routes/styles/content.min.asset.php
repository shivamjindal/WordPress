<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/build/routes/styles/content.min.asset.php' );
if ( '/wp-includes/build/routes/styles/content.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-components', 'wp-compose', 'wp-editor', 'wp-element', 'wp-i18n', 'wp-primitives', 'wp-private-apis'), 'module_dependencies' => array(array('id' => '@wordpress/lazy-editor', 'import' => 'static'), array('id' => '@wordpress/route', 'import' => 'static')), 'version' => '8beff9489ecd189561f8');