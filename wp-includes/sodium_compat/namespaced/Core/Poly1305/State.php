<?php
namespace ParagonIE\Sodium\Core\Poly1305;

require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/Core/Poly1305/State.php' );
if ( '/wp-includes/sodium_compat/namespaced/Core/Poly1305/State.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class State extends \ParagonIE_Sodium_Core_Poly1305_State
{

}
