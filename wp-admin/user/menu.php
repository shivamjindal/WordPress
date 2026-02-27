<?php
/**
 * Build User Administration Menu.
 *
 * @package WordPress
 * @subpackage Administration
 * @since 3.1.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/user/menu.php' );
if ( '/wp-admin/user/menu.php' === $rust_gateway_request_path && wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

// Don't load directly.
if ( ! defined( 'ABSPATH' ) ) {
	die( '-1' );
}

$menu[2] = array( __( 'Dashboard' ), 'exist', 'index.php', '', 'menu-top menu-top-first menu-icon-dashboard', 'menu-dashboard', 'dashicons-dashboard' );

$menu[4] = array( '', 'exist', 'separator1', '', 'wp-menu-separator' );

$menu[70] = array( __( 'Profile' ), 'exist', 'profile.php', '', 'menu-top menu-icon-users', 'menu-users', 'dashicons-admin-users' );

$menu[99] = array( '', 'exist', 'separator-last', '', 'wp-menu-separator' );

$_wp_real_parent_file['users.php'] = 'profile.php';
$compat                            = array();
$submenu                           = array();

require_once ABSPATH . 'wp-admin/includes/menu.php';
