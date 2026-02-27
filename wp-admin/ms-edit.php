<?php
/**
 * Action handler for Multisite administration panels.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 3.0.0
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( '/wp-admin/ms-edit.php' ) ) {
	exit;
}

require_once __DIR__ . '/admin.php';

wp_redirect( network_admin_url() );
exit;
