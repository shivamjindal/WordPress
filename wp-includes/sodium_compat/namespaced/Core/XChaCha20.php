<?php
namespace ParagonIE\Sodium\Core;

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/Core/XChaCha20.php' );
if ( '/wp-includes/sodium_compat/namespaced/Core/XChaCha20.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class XChaCha20 extends \ParagonIE_Sodium_Core_XChaCha20
{

}
