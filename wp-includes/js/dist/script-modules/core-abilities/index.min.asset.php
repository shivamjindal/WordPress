<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/core-abilities/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/core-abilities/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-api-fetch', 'wp-url'), 'module_dependencies' => array(array('id' => '@wordpress/abilities', 'import' => 'static')), 'version' => '336043fa59033fb5e9b0');