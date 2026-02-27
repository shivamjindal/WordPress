<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/block-editor/utils/fit-text-frontend.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/block-editor/utils/fit-text-frontend.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'module_dependencies' => array(array('id' => '@wordpress/interactivity', 'import' => 'static')), 'version' => '2f00eb94b5ef309f39eb');