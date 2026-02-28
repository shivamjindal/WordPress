<?php
namespace ParagonIE\Sodium\Core\Curve25519\Ge;

require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/P2.php' );
if ( '/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/P2.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class P2 extends \ParagonIE_Sodium_Core_Curve25519_Ge_P2
{

}
