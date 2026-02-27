<?php
/**
 * Event dispatcher
 *
 * @package Requests\EventDispatcher
 */

namespace WpOrg\Requests;

require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/Requests/src/HookManager.php' );
if ( '/wp-includes/Requests/src/HookManager.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Event dispatcher
 *
 * @package Requests\EventDispatcher
 */
interface HookManager {
	/**
	 * Register a callback for a hook
	 *
	 * @param string $hook Hook name
	 * @param callable $callback Function/method to call on event
	 * @param int $priority Priority number. <0 is executed earlier, >0 is executed later
	 */
	public function register($hook, $callback, $priority = 0);

	/**
	 * Dispatch a message
	 *
	 * @param string $hook Hook name
	 * @param array $parameters Parameters to pass to callbacks
	 * @return boolean Successfulness
	 */
	public function dispatch($hook, $parameters = []);
}
