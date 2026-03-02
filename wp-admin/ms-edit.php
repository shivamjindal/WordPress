<?php
/**
 * Action handler for Multisite administration panels.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 3.0.0
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/ms-edit.php' );
if ( '/wp-admin/ms-edit.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

require_once __DIR__ . '/admin.php';

wp_redirect( network_admin_url() );
exit;
