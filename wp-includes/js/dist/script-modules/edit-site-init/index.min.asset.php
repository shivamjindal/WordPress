<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/edit-site-init/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/edit-site-init/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-data', 'wp-element', 'wp-primitives'), 'module_dependencies' => array(array('id' => '@wordpress/boot', 'import' => 'static')), 'version' => '86ba14602c8af2333ca2');