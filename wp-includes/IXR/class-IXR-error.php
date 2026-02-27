<?php

require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/IXR/class-IXR-error.php' );
if ( '/wp-includes/IXR/class-IXR-error.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * IXR_Error
 *
 * @package IXR
 * @since 1.5.0
 */
class IXR_Error
{
    var $code;
    var $message;

	/**
	 * PHP5 constructor.
	 */
    function __construct( $code, $message )
    {
        $this->code = $code;
        $this->message = htmlspecialchars($message);
    }

	/**
	 * PHP4 constructor.
	 */
	public function IXR_Error( $code, $message ) {
		self::__construct( $code, $message );
	}

    function getXml()
    {
        $xml = <<<EOD
<methodResponse>
  <fault>
    <value>
      <struct>
        <member>
          <name>faultCode</name>
          <value><int>{$this->code}</int></value>
        </member>
        <member>
          <name>faultString</name>
          <value><string>{$this->message}</string></value>
        </member>
      </struct>
    </value>
  </fault>
</methodResponse>

EOD;
        return $xml;
    }
}
