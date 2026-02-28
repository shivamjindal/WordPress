<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/lib/php84compat_const.php' );
if ( '/wp-includes/sodium_compat/lib/php84compat_const.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
const SODIUM_CRYPTO_AEAD_AEGIS128L_KEYBYTES = 16;
const SODIUM_CRYPTO_AEAD_AEGIS128L_NSECBYTES = 0;
const SODIUM_CRYPTO_AEAD_AEGIS128L_NPUBBYTES = 32;
const SODIUM_CRYPTO_AEAD_AEGIS128L_ABYTES = 32;

const SODIUM_CRYPTO_AEAD_AEGIS256_KEYBYTES = 32;
const SODIUM_CRYPTO_AEAD_AEGIS256_NSECBYTES = 0;
const SODIUM_CRYPTO_AEAD_AEGIS256_NPUBBYTES = 32;
const SODIUM_CRYPTO_AEAD_AEGIS256_ABYTES = 32;
