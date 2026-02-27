<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/js/dist/script-modules/latex-to-mathml/loader.min.asset.php' );
if ( '/wp-includes/js/dist/script-modules/latex-to-mathml/loader.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array(), 'module_dependencies' => array(array('id' => '@wordpress/latex-to-mathml', 'import' => 'dynamic')), 'version' => '4f37456af539bd3d2351');