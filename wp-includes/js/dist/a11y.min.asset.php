<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/a11y.min.asset.php' );
if ( '/wp-includes/js/dist/a11y.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-dom-ready', 'wp-i18n'), 'version' => '879fa8fbccbc34cefa51');