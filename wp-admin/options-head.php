<?php
/**
 * WordPress Options Header.
 *
 * Displays updated message, if updated variable is part of the URL query.
 *
 * @package WordPress
 * @subpackage Administration
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/options-head.php' );
if ( '/wp-admin/options-head.php' === $rust_gateway_request_path && wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

// Don't load directly.
if ( ! defined( 'ABSPATH' ) ) {
	die( '-1' );
}

$action = ! empty( $_REQUEST['action'] ) ? sanitize_text_field( $_REQUEST['action'] ) : '';

if ( isset( $_GET['updated'] ) && isset( $_GET['page'] ) ) {
	// For back-compat with plugins that don't use the Settings API and just set updated=1 in the redirect.
	add_settings_error( 'general', 'settings_updated', __( 'Settings saved.' ), 'success' );
}

settings_errors();
