<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/abilities/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/abilities/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-data', 'wp-i18n'), 'version' => 'bd07cd6be9d3678c2a45');