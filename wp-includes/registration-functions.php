<?php
/**
 * Deprecated. No longer needed.
 *
 * @package WordPress
 * @deprecated 2.1.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/registration-functions.php' ) === '/wp-includes/registration-functions.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/registration-functions.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '2.1.0', '', __( 'This file no longer needs to be included.' ) );
