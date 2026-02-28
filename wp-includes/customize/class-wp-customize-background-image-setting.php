<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/customize/class-wp-customize-background-image-setting.php' );
if ( '/wp-includes/customize/class-wp-customize-background-image-setting.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * Customize API: WP_Customize_Background_Image_Setting class
 *
 * @package WordPress
 * @subpackage Customize
 * @since 4.4.0
 */

/**
 * Customizer Background Image Setting class.
 *
 * @since 3.4.0
 *
 * @see WP_Customize_Setting
 */
final class WP_Customize_Background_Image_Setting extends WP_Customize_Setting {

	/**
	 * Unique string identifier for the setting.
	 *
	 * @since 3.4.0
	 * @var string
	 */
	public $id = 'background_image_thumb';

	/**
	 * @since 3.4.0
	 *
	 * @param mixed $value The value to update. Not used.
	 */
	public function update( $value ) {
		remove_theme_mod( 'background_image_thumb' );
	}
}
