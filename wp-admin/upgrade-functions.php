<?php
/**
 * WordPress Upgrade Functions. Old file, must not be used. Include
 * wp-admin/includes/upgrade.php instead.
 *
 * @deprecated 2.5.0
 * @package WordPress
 * @subpackage Administration
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/upgrade-functions.php' );
if ( '/wp-admin/upgrade-functions.php' === $rust_gateway_request_path && wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '2.5.0', 'wp-admin/includes/upgrade.php' );
require_once ABSPATH . 'wp-admin/includes/upgrade.php';
