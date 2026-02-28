<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/lib/stream-xchacha20.php' );
if ( '/wp-includes/sodium_compat/lib/stream-xchacha20.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

if (!is_callable('sodium_crypto_stream_xchacha20')) {
    /**
     * @see ParagonIE_Sodium_Compat::crypto_stream_xchacha20()
     * @param int $len
     * @param string $nonce
     * @param string $key
     * @return string
     * @throws SodiumException
     * @throws TypeError
     */
    function sodium_crypto_stream_xchacha20(
        $len,
        $nonce,
        #[\SensitiveParameter]
        $key
    ) {
        return ParagonIE_Sodium_Compat::crypto_stream_xchacha20($len, $nonce, $key, true);
    }
}
if (!is_callable('sodium_crypto_stream_xchacha20_keygen')) {
    /**
     * @see ParagonIE_Sodium_Compat::crypto_stream_xchacha20_keygen()
     * @return string
     * @throws Exception
     */
    function sodium_crypto_stream_xchacha20_keygen()
    {
        return ParagonIE_Sodium_Compat::crypto_stream_xchacha20_keygen();
    }
}
if (!is_callable('sodium_crypto_stream_xchacha20_xor')) {
    /**
     * @see ParagonIE_Sodium_Compat::crypto_stream_xchacha20_xor()
     * @param string $message
     * @param string $nonce
     * @param string $key
     * @return string
     * @throws SodiumException
     * @throws TypeError
     */
    function sodium_crypto_stream_xchacha20_xor(
        #[\SensitiveParameter]
        $message,
        $nonce,
        #[\SensitiveParameter]
        $key
    ) {
        return ParagonIE_Sodium_Compat::crypto_stream_xchacha20_xor($message, $nonce, $key, true);
    }
}
if (!is_callable('sodium_crypto_stream_xchacha20_xor_ic')) {
    /**
     * @see ParagonIE_Sodium_Compat::crypto_stream_xchacha20_xor_ic()
     * @param string $message
     * @param string $nonce
     * @param int $counter
     * @param string $key
     * @return string
     * @throws SodiumException
     * @throws TypeError
     */
    function sodium_crypto_stream_xchacha20_xor_ic(
        #[\SensitiveParameter]
        $message,
        $nonce,
        $counter,
        #[\SensitiveParameter]
        $key
    ) {
        return ParagonIE_Sodium_Compat::crypto_stream_xchacha20_xor_ic($message, $nonce, $counter, $key, true);
    }
}
