<?php
require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/plugins/index.php' );
if ( '/wp-content/plugins/index.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

// Silence is golden.
