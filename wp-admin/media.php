<?php
/**
 * Media management action handler.
 *
 * This file is deprecated, use 'wp-admin/upload.php' instead.
 *
 * @deprecated 6.3.0
 * @package WordPress
 * @subpackage Administration
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/media.php' );
if ( '/wp-admin/media.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap. */
require_once __DIR__ . '/admin.php';

$parent_file  = 'upload.php';
$submenu_file = 'upload.php';

$action = ! empty( $_REQUEST['action'] ) ? sanitize_text_field( $_REQUEST['action'] ) : '';

switch ( $action ) {
	case 'editattachment':
	case 'edit':
		if ( empty( $_GET['attachment_id'] ) ) {
			wp_redirect( admin_url( 'upload.php?error=deprecated' ) );
			exit;
		}
		$att_id = (int) $_GET['attachment_id'];

		wp_redirect( admin_url( "upload.php?item={$att_id}&error=deprecated" ) );
		exit;

	default:
		wp_redirect( admin_url( 'upload.php?error=deprecated' ) );
		exit;
}
