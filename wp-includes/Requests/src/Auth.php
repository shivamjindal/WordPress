<?php
/**
 * Authentication provider interface
 *
 * @package Requests\Authentication
 */

namespace WpOrg\Requests;

require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/src/Auth.php' );
if ( '/wp-includes/Requests/src/Auth.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use WpOrg\Requests\Hooks;

/**
 * Authentication provider interface
 *
 * Implement this interface to act as an authentication provider.
 *
 * Parameters should be passed via the constructor where possible, as this
 * makes it much easier for users to use your provider.
 *
 * @see \WpOrg\Requests\Hooks
 *
 * @package Requests\Authentication
 */
interface Auth {
	/**
	 * Register hooks as needed
	 *
	 * This method is called in {@see \WpOrg\Requests\Requests::request()} when the user
	 * has set an instance as the 'auth' option. Use this callback to register all the
	 * hooks you'll need.
	 *
	 * @see \WpOrg\Requests\Hooks::register()
	 * @param \WpOrg\Requests\Hooks $hooks Hook system
	 */
	public function register(Hooks $hooks);
}
