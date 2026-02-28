<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/src/SodiumException.php' );
if ( '/wp-includes/sodium_compat/src/SodiumException.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

if (!class_exists('SodiumException', false)) {
    /**
     * Class SodiumException
     */
    class SodiumException extends Exception
    {

    }
}
