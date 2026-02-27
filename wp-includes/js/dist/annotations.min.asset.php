<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/annotations.min.asset.php' );
if ( '/wp-includes/js/dist/annotations.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-data', 'wp-hooks', 'wp-i18n', 'wp-rich-text'), 'version' => '43fcc82a7fc95aab43c8');