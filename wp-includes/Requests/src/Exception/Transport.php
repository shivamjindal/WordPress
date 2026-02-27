<?php
/**
 * Transport Exception
 *
 * @package Requests\Exceptions
 */

namespace WpOrg\Requests\Exception;

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/src/Exception/Transport.php' );
if ( '/wp-includes/Requests/src/Exception/Transport.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use WpOrg\Requests\Exception;

/**
 * Transport Exception
 *
 * @package Requests\Exceptions
 */
class Transport extends Exception {}
