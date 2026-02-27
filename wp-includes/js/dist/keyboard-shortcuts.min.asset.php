<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/keyboard-shortcuts.min.asset.php' );
if ( '/wp-includes/js/dist/keyboard-shortcuts.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-data', 'wp-element', 'wp-keycodes'), 'version' => '3940e84bc76dca71a245');