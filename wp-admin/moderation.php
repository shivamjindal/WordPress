<?php
/**
 * Comment Moderation Administration Screen.
 *
 * Redirects to edit-comments.php?comment_status=moderated.
 *
 * @package WordPress
 * @subpackage Administration
 */
require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/moderation.php' );
if ( '/wp-admin/moderation.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

require_once dirname( __DIR__ ) . '/wp-load.php';
wp_redirect( admin_url( 'edit-comments.php?comment_status=moderated' ) );
exit;
