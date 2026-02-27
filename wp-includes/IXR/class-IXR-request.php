<?php

require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/IXR/class-IXR-request.php' );
if ( '/wp-includes/IXR/class-IXR-request.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * IXR_Request
 *
 * @package IXR
 * @since 1.5.0
 */
class IXR_Request
{
    var $method;
    var $args;
    var $xml;

	/**
	 * PHP5 constructor.
	 */
    function __construct($method, $args)
    {
        $this->method = $method;
        $this->args = $args;
        $this->xml = <<<EOD
<?xml version="1.0"?>
<methodCall>
<methodName>{$this->method}</methodName>
<params>

EOD;
        foreach ($this->args as $arg) {
            $this->xml .= '<param><value>';
            $v = new IXR_Value($arg);
            $this->xml .= $v->getXml();
            $this->xml .= "</value></param>\n";
        }
        $this->xml .= '</params></methodCall>';
    }

	/**
	 * PHP4 constructor.
	 */
	public function IXR_Request( $method, $args ) {
		self::__construct( $method, $args );
	}

    function getLength()
    {
        return strlen($this->xml);
    }

    function getXml()
    {
        return $this->xml;
    }
}
