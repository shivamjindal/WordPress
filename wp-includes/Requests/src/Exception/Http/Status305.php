<?php
/**
 * Exception for 305 Use Proxy responses
 *
 * @package Requests\Exceptions
 */

namespace WpOrg\Requests\Exception\Http;

require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/src/Exception/Http/Status305.php' );
if ( '/wp-includes/Requests/src/Exception/Http/Status305.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use WpOrg\Requests\Exception\Http;

/**
 * Exception for 305 Use Proxy responses
 *
 * @package Requests\Exceptions
 */
final class Status305 extends Http {
	/**
	 * HTTP status code
	 *
	 * @var integer
	 */
	protected $code = 305;

	/**
	 * Reason phrase
	 *
	 * @var string
	 */
	protected $reason = 'Use Proxy';
}
