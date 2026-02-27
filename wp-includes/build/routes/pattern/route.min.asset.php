<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/build/routes/pattern/route.min.asset.php' );
if ( '/wp-includes/build/routes/pattern/route.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'module_dependencies' => array(array('id' => '@wordpress/route', 'import' => 'static')), 'version' => '0ad107785a0d22f8e27c');