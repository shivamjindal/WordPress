<?php
/**
 * Exception for 413 Request Entity Too Large responses
 *
 * @package Requests\Exceptions
 */

namespace WpOrg\Requests\Exception\Http;

require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/src/Exception/Http/Status413.php' );
if ( '/wp-includes/Requests/src/Exception/Http/Status413.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use WpOrg\Requests\Exception\Http;

/**
 * Exception for 413 Request Entity Too Large responses
 *
 * @package Requests\Exceptions
 */
final class Status413 extends Http {
	/**
	 * HTTP status code
	 *
	 * @var integer
	 */
	protected $code = 413;

	/**
	 * Reason phrase
	 *
	 * @var string
	 */
	protected $reason = 'Request Entity Too Large';
}
