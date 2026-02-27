<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/core-commands.min.asset.php' );
if ( '/wp-includes/js/dist/core-commands.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-commands', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-element', 'wp-html-entities', 'wp-i18n', 'wp-primitives', 'wp-private-apis', 'wp-router', 'wp-url'), 'version' => 'c4f08cdbaa4f757ccb4f');