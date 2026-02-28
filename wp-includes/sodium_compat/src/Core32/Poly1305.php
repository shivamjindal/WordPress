<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/src/Core32/Poly1305.php' );
if ( '/wp-includes/sodium_compat/src/Core32/Poly1305.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

if (class_exists('ParagonIE_Sodium_Core32_Poly1305', false)) {
    return;
}

/**
 * Class ParagonIE_Sodium_Core32_Poly1305
 */
abstract class ParagonIE_Sodium_Core32_Poly1305 extends ParagonIE_Sodium_Core32_Util
{
    const BLOCK_SIZE = 16;

    /**
     * @internal You should not use this directly from another application
     *
     * @param string $m
     * @param string $key
     * @return string
     * @throws SodiumException
     * @throws TypeError
     */
    public static function onetimeauth($m, $key)
    {
        if (self::strlen($key) < 32) {
            throw new InvalidArgumentException(
                'Key must be 32 bytes long.'
            );
        }
        $state = new ParagonIE_Sodium_Core32_Poly1305_State(
            self::substr($key, 0, 32)
        );
        return $state
            ->update($m)
            ->finish();
    }

    /**
     * @internal You should not use this directly from another application
     *
     * @param string $mac
     * @param string $m
     * @param string $key
     * @return bool
     * @throws SodiumException
     * @throws TypeError
     */
    public static function onetimeauth_verify($mac, $m, $key)
    {
        if (self::strlen($key) < 32) {
            throw new InvalidArgumentException(
                'Key must be 32 bytes long.'
            );
        }
        $state = new ParagonIE_Sodium_Core32_Poly1305_State(
            self::substr($key, 0, 32)
        );
        $calc = $state
            ->update($m)
            ->finish();
        return self::verify_16($calc, $mac);
    }
}
