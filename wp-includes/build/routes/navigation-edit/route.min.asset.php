<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/build/routes/navigation-edit/route.min.asset.php' );
if ( '/wp-includes/build/routes/navigation-edit/route.min.asset.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
return array('dependencies' => array('wp-core-data', 'wp-data', 'wp-html-entities', 'wp-i18n'), 'version' => '7895afc23d927ea21948');