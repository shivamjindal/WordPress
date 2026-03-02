<?php
/**
 * User Dashboard Freedoms administration panel.
 *
 * @package WordPress
 * @subpackage Administration
 * @since 3.4.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/user/freedoms.php' );
if ( '/wp-admin/user/freedoms.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

require ABSPATH . 'wp-admin/freedoms.php';
