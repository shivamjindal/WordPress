<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/format-library.min.asset.php' );
if ( '/wp-includes/js/dist/format-library.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react-jsx-runtime', 'wp-a11y', 'wp-block-editor', 'wp-components', 'wp-compose', 'wp-data', 'wp-element', 'wp-html-entities', 'wp-i18n', 'wp-primitives', 'wp-private-apis', 'wp-rich-text', 'wp-url'), 'module_dependencies' => array(array('id' => '@wordpress/latex-to-mathml', 'import' => 'dynamic')), 'version' => 'd75d3264e34a3a1ee358');