<?php
namespace ParagonIE\Sodium\Core\Curve25519\Ge;

require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/Cached.php' );
if ( '/wp-includes/sodium_compat/namespaced/Core/Curve25519/Ge/Cached.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class Cached extends \ParagonIE_Sodium_Core_Curve25519_Ge_Cached
{

}
