<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/customize/class-wp-customize-upload-control.php' );
if ( '/wp-includes/customize/class-wp-customize-upload-control.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * Customize API: WP_Customize_Upload_Control class
 *
 * @package WordPress
 * @subpackage Customize
 * @since 4.4.0
 */

/**
 * Customize Upload Control Class.
 *
 * @since 3.4.0
 *
 * @see WP_Customize_Media_Control
 */
class WP_Customize_Upload_Control extends WP_Customize_Media_Control {
	/**
	 * Control type.
	 *
	 * @since 3.4.0
	 * @var string
	 */
	public $type = 'upload';

	/**
	 * Media control mime type.
	 *
	 * @since 4.1.0
	 * @var string
	 */
	public $mime_type = '';

	/**
	 * Button labels.
	 *
	 * @since 4.1.0
	 * @var array
	 */
	public $button_labels = array();

	public $removed = '';         // Unused.
	public $context;              // Unused.
	public $extensions = array(); // Unused.

	/**
	 * Refresh the parameters passed to the JavaScript via JSON.
	 *
	 * @since 3.4.0
	 *
	 * @uses WP_Customize_Media_Control::to_json()
	 */
	public function to_json() {
		parent::to_json();

		$value = $this->value();
		if ( $value ) {
			// Get the attachment model for the existing file.
			$attachment_id = attachment_url_to_postid( $value );
			if ( $attachment_id ) {
				$this->json['attachment'] = wp_prepare_attachment_for_js( $attachment_id );
			}
		}
	}
}
