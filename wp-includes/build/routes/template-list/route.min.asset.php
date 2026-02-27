<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/build/routes/template-list/route.min.asset.php' );
if ( '/wp-includes/build/routes/template-list/route.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-core-data', 'wp-data', 'wp-element', 'wp-i18n', 'wp-preferences'), 'version' => 'a0d92e3963c72fb877e2');