<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/commands.min.asset.php' );
if ( '/wp-includes/js/dist/commands.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-dom', 'react-jsx-runtime', 'wp-components', 'wp-data', 'wp-element', 'wp-i18n', 'wp-keyboard-shortcuts', 'wp-primitives', 'wp-private-apis'), 'version' => 'd7a1e27135c422b68ab8');