<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/customize-widgets.min.asset.php' );
if ( '/wp-includes/js/dist/customize-widgets.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-block-editor', 'wp-block-library', 'wp-blocks', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-dom', 'wp-element', 'wp-hooks', 'wp-i18n', 'wp-is-shallow-equal', 'wp-keyboard-shortcuts', 'wp-keycodes', 'wp-media-utils', 'wp-preferences', 'wp-primitives', 'wp-private-apis', 'wp-widgets'), 'version' => 'af2f36bd7afd2843306a');