<?php
/**
 * Network Contribute administration panel.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 6.3.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( '/wp-admin/network/contribute.php' ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

require ABSPATH . 'wp-admin/contribute.php';
