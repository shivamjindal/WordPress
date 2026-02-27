<?php

require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/IXR/class-IXR-base64.php' );
if ( '/wp-includes/IXR/class-IXR-base64.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * IXR_Base64
 *
 * @package IXR
 * @since 1.5.0
 */
class IXR_Base64
{
    var $data;

	/**
	 * PHP5 constructor.
	 */
    function __construct( $data )
    {
        $this->data = $data;
    }

	/**
	 * PHP4 constructor.
	 */
	public function IXR_Base64( $data ) {
		self::__construct( $data );
	}

    function getXml()
    {
        return '<base64>'.base64_encode($this->data).'</base64>';
    }
}
