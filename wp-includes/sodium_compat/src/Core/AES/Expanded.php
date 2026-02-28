<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/src/Core/AES/Expanded.php' );
if ( '/wp-includes/sodium_compat/src/Core/AES/Expanded.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

if (class_exists('ParagonIE_Sodium_Core_AES_Expanded', false)) {
    return;
}

/**
 * @internal This should only be used by sodium_compat
 */
class ParagonIE_Sodium_Core_AES_Expanded extends ParagonIE_Sodium_Core_AES_KeySchedule
{
    /** @var bool $expanded */
    protected $expanded = true;
}
