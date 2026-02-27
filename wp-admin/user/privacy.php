<?php
/**
 * User Dashboard Privacy administration panel.
 *
 * @package WordPress
 * @subpackage Administration
 * @since 4.9.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( '/wp-admin/user/privacy.php' ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

require ABSPATH . 'wp-admin/privacy.php';
