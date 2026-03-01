<?php
/**
 * Multisite sites administration panel.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 3.0.0
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/ms-sites.php' );
if ( '/wp-admin/ms-sites.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

require_once __DIR__ . '/admin.php';

wp_redirect( network_admin_url( 'sites.php' ) );
exit;
