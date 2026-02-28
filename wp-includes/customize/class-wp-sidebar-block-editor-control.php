<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/customize/class-wp-sidebar-block-editor-control.php' );
if ( '/wp-includes/customize/class-wp-sidebar-block-editor-control.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * Customize API: WP_Sidebar_Block_Editor_Control class.
 *
 * @package WordPress
 * @subpackage Customize
 * @since 5.8.0
 */

/**
 * Core class used to implement the widgets block editor control in the
 * customizer.
 *
 * @since 5.8.0
 *
 * @see WP_Customize_Control
 */
class WP_Sidebar_Block_Editor_Control extends WP_Customize_Control {
	/**
	 * The control type.
	 *
	 * @since 5.8.0
	 *
	 * @var string
	 */
	public $type = 'sidebar_block_editor';

	/**
	 * Render the widgets block editor container.
	 *
	 * @since 5.8.0
	 */
	public function render_content() {
		// Render an empty control. The JavaScript in
		// @wordpress/customize-widgets will do the rest.
	}
}
