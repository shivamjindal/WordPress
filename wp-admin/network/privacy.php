<?php
/**
 * Network Privacy administration panel.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 4.9.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/network/privacy.php' );
if ( '/wp-admin/network/privacy.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

require ABSPATH . 'wp-admin/privacy.php';
