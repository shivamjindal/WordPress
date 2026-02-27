<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/build/routes/pattern-list/content.min.asset.php' );
if ( '/wp-includes/build/routes/pattern-list/content.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('react', 'react-dom', 'react-jsx-runtime', 'wp-block-editor', 'wp-components', 'wp-compose', 'wp-core-data', 'wp-data', 'wp-date', 'wp-deprecated', 'wp-editor', 'wp-element', 'wp-i18n', 'wp-keycodes', 'wp-patterns', 'wp-preferences', 'wp-primitives', 'wp-private-apis', 'wp-warning'), 'module_dependencies' => array(array('id' => '@wordpress/lazy-editor', 'import' => 'static'), array('id' => '@wordpress/route', 'import' => 'static')), 'version' => 'd5302b4d21f9ed18ccff');