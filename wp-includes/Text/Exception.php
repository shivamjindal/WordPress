<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Text/Exception.php' );
if ( '/wp-includes/Text/Exception.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * Exception for errors from the Text_Diff package.
 *
 * {@internal This is a WP native addition to the external Text_Diff package.}
 *
 * @package WordPress
 * @subpackage Text_Diff
 */

class Text_Exception extends Exception {}
