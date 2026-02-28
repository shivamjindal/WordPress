<?php
namespace ParagonIE\Sodium\Core;

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/Core/SipHash.php' );
if ( '/wp-includes/sodium_compat/namespaced/Core/SipHash.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class SipHash extends \ParagonIE_Sodium_Core_SipHash
{

}
