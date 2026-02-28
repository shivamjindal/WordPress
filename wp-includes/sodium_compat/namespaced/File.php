<?php
namespace ParagonIE\Sodium;

require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/namespaced/File.php' );
if ( '/wp-includes/sodium_compat/namespaced/File.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

class File extends \ParagonIE_Sodium_File
{

}
