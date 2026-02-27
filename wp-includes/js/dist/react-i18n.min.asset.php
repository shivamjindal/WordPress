<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/react-i18n.min.asset.php' );
if ( '/wp-includes/js/dist/react-i18n.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-element', 'wp-i18n'), 'version' => 'b57b99768728138bd42b');