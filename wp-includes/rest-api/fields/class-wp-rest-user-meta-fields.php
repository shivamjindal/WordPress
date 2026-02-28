<?php
require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/rest-api/fields/class-wp-rest-user-meta-fields.php' );
if ( '/wp-includes/rest-api/fields/class-wp-rest-user-meta-fields.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * REST API: WP_REST_User_Meta_Fields class
 *
 * @package WordPress
 * @subpackage REST_API
 * @since 4.7.0
 */

/**
 * Core class used to manage meta values for users via the REST API.
 *
 * @since 4.7.0
 *
 * @see WP_REST_Meta_Fields
 */
class WP_REST_User_Meta_Fields extends WP_REST_Meta_Fields {

	/**
	 * Retrieves the user meta type.
	 *
	 * @since 4.7.0
	 *
	 * @return string The user meta type.
	 */
	protected function get_meta_type() {
		return 'user';
	}

	/**
	 * Retrieves the user meta subtype.
	 *
	 * @since 4.9.8
	 *
	 * @return string 'user' There are no subtypes.
	 */
	protected function get_meta_subtype() {
		return 'user';
	}

	/**
	 * Retrieves the type for register_rest_field().
	 *
	 * @since 4.7.0
	 *
	 * @return string The user REST field type.
	 */
	public function get_rest_field_type() {
		return 'user';
	}
}
