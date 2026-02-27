<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/workflow/index.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/workflow/index.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-dom', 'react-jsx-runtime', 'wp-components', 'wp-data', 'wp-element', 'wp-i18n', 'wp-keyboard-shortcuts', 'wp-primitives', 'wp-private-apis'), 'module_dependencies' => array(array('id' => '@wordpress/abilities', 'import' => 'static')), 'version' => '632383fbe66eca7e49c5');