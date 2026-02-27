<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/components.min.asset.php' );
if ( '/wp-includes/js/dist/components.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-dom', 'react-jsx-runtime', 'wp-a11y', 'wp-compose', 'wp-date', 'wp-deprecated', 'wp-dom', 'wp-element', 'wp-escape-html', 'wp-hooks', 'wp-html-entities', 'wp-i18n', 'wp-is-shallow-equal', 'wp-keycodes', 'wp-primitives', 'wp-private-apis', 'wp-rich-text', 'wp-warning'), 'version' => 'c8e16b0453cccb5aa4f4');