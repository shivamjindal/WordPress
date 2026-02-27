<?php
/**
 * Locale API
 *
 * @package WordPress
 * @subpackage i18n
 * @since 1.2.0
 * @deprecated 4.7.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/locale.php' ) === '/wp-includes/locale.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/locale.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '4.7.0' );
