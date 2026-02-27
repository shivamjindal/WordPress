<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/block-library/query/view.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/block-library/query/view.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'module_dependencies' => array(array('id' => '@wordpress/interactivity', 'import' => 'static'), array('id' => '@wordpress/interactivity-router', 'import' => 'dynamic')), 'version' => '7a4ec5bfb61a7137cf4b');