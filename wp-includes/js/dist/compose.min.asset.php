<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/compose.min.asset.php' );
if ( '/wp-includes/js/dist/compose.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-jsx-runtime', 'wp-deprecated', 'wp-dom', 'wp-element', 'wp-is-shallow-equal', 'wp-keycodes', 'wp-priority-queue', 'wp-undo-manager'), 'version' => '056e828be7433d31ea43');