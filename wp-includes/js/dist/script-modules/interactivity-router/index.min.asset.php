<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/interactivity-router/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/interactivity-router/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'module_dependencies' => array(array('id' => '@wordpress/a11y', 'import' => 'dynamic'), array('id' => '@wordpress/interactivity', 'import' => 'static')), 'version' => 'bb48ce8e3364d5290463');