<?php
/**
 * User Profile Administration Screen.
 *
 * @package WordPress
 * @subpackage Administration
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
if ( wp_rust_gateway_try_proxy( '/wp-admin/profile.php' ) ) {
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
