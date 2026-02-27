<?php
/**
 * Exception for unknown status responses
 *
 * @package Requests\Exceptions
 */

namespace WpOrg\Requests\Exception\Http;

require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/src/Exception/Http/StatusUnknown.php' );
if ( '/wp-includes/Requests/src/Exception/Http/StatusUnknown.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use WpOrg\Requests\Exception\Http;
use WpOrg\Requests\Response;

/**
 * Exception for unknown status responses
 *
 * @package Requests\Exceptions
 */
final class StatusUnknown extends Http {
	/**
	 * HTTP status code
	 *
	 * @var integer|bool Code if available, false if an error occurred
	 */
	protected $code = 0;

	/**
	 * Reason phrase
	 *
	 * @var string
	 */
	protected $reason = 'Unknown';

	/**
	 * Create a new exception
	 *
	 * If `$data` is an instance of {@see \WpOrg\Requests\Response}, uses the status
	 * code from it. Otherwise, sets as 0
	 *
	 * @param string|null $reason Reason phrase
	 * @param mixed $data Associated data
	 */
	public function __construct($reason = null, $data = null) {
		if ($data instanceof Response) {
			$this->code = (int) $data->status_code;
		}

		parent::__construct($reason, $data);
	}
}
