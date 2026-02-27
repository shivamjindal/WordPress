<?php
/**
 * Loads the old Requests class file when the autoloader
 * references the original PSR-0 Requests class.
 *
 * @deprecated 6.2.0
 * @package WordPress
 * @subpackage Requests
 * @since 6.2.0
 */

require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/library/Requests.php' );
if ( '/wp-includes/Requests/library/Requests.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

include_once ABSPATH . WPINC . '/class-requests.php';
