<?php
/**
 * Edit user network administration panel.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 3.1.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( '/wp-admin/network/user-edit.php' ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

require ABSPATH . 'wp-admin/user-edit.php';
