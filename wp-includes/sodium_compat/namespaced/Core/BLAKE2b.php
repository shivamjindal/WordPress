<?php
namespace ParagonIE\Sodium\Core;

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/Core/BLAKE2b.php' );
if ( '/wp-includes/sodium_compat/namespaced/Core/BLAKE2b.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class BLAKE2b extends \ParagonIE_Sodium_Core_BLAKE2b
{

}
