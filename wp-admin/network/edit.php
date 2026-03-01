<?php
/**
 * Action handler for Multisite administration panels.
 *
 * @package WordPress
 * @subpackage Multisite
 * @since 3.0.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/network/edit.php' );
if ( '/wp-admin/network/edit.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

$action = $_GET['action'] ?? '';

if ( empty( $action ) ) {
	wp_redirect( network_admin_url() );
	exit;
}

/**
 * Fires just before the action handler in several Network Admin screens.
 *
 * This hook fires on multiple screens in the Multisite Network Admin,
 * including Users, Network Settings, and Site Settings.
 *
 * @since 3.0.0
 */
do_action( 'wpmuadminedit' );

/**
 * Fires the requested handler action.
 *
 * The dynamic portion of the hook name, `$action`, refers to the name
 * of the requested action derived from the `GET` request.
 *
 * @since 3.1.0
 */
do_action( "network_admin_edit_{$action}" );

wp_redirect( network_admin_url() );
exit;
