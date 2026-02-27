<?php
/**
 * Deprecated. No longer needed.
 *
 * @package WordPress
 * @deprecated 3.1.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/registration.php' ) === '/wp-includes/registration.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/registration.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '3.1.0', '', __( 'This file no longer needs to be included.' ) );
