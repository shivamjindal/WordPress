<?php
/**
 * Polyfill for SPL autoload feature. This file is separate to prevent compiler notices
 * on the deprecated __autoload() function.
 *
 * See https://core.trac.wordpress.org/ticket/41134
 *
 * @deprecated 5.3.0 No longer needed as the minimum PHP requirement has moved beyond PHP 5.3.
 *
 * @package PHP
 * @access private
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/spl-autoload-compat.php' ) === '/wp-includes/spl-autoload-compat.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/spl-autoload-compat.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '5.3.0', '', 'SPL can no longer be disabled as of PHP 5.3.' );
