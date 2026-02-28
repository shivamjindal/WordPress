<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/src/Core32/Util.php' );
if ( '/wp-includes/sodium_compat/src/Core32/Util.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

if (class_exists('ParagonIE_Sodium_Core32_Util', false)) {
    return;
}

/**
 * Class ParagonIE_Sodium_Core_Util
 */
abstract class ParagonIE_Sodium_Core32_Util extends ParagonIE_Sodium_Core_Util
{

}
