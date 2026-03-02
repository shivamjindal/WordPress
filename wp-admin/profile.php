<?php
/**
 * User Profile Administration Screen.
 *
 * @package WordPress
 * @subpackage Administration
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/profile.php' );
if ( '/wp-admin/profile.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * This is a profile page.
 *
 * @since 2.5.0
 * @var bool
 */
define( 'IS_PROFILE_PAGE', true );

/** Load User Editing Page */
require_once __DIR__ . '/user-edit.php';
