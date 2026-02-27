<?php
/**
 * Install plugin network administration panel.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 3.1.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( '/wp-admin/network/plugin-install.php' ) ) {
	exit;
}

if ( isset( $_GET['tab'] ) && ( 'plugin-information' === $_GET['tab'] ) ) {
	define( 'IFRAME_REQUEST', true );
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

require ABSPATH . 'wp-admin/plugin-install.php';
