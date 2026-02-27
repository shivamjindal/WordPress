<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/a11y/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/a11y/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'version' => '1c371cb517a97cdbcb9f');