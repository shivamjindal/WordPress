<?php
/**
 * View Transitions API.
 *
 * @package WordPress
 * @subpackage View Transitions
 * @since 7.0.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/view-transitions.php' ) === '/wp-includes/view-transitions.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/view-transitions.php' ) ) {
	exit;
}

/**
 * Enqueues View Transitions CSS for the admin.
 *
 * @since 7.0.0
 */
function wp_enqueue_view_transitions_admin_css(): void {
	wp_enqueue_style( 'wp-view-transitions-admin' );
}

/**
 * Gets the CSS for View Transitions in the admin.
 *
 * @since 7.0.0
 *
 * @return string The CSS.
 */
function wp_get_view_transitions_admin_css(): string {
	$affix = SCRIPT_DEBUG ? '' : '.min';
	$path  = ABSPATH . "wp-admin/css/view-transitions{$affix}.css";
	return file_get_contents( $path );
}
