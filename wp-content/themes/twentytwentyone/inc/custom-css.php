<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyone/inc/custom-css.php' );
if ( '/wp-content/themes/twentytwentyone/inc/custom-css.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Custom CSS
 *
 * @package WordPress
 * @subpackage Twenty_Twenty_One
 * @since Twenty Twenty-One 1.0
 */

/**
 * Generates CSS.
 *
 * @since Twenty Twenty-One 1.0
 *
 * @param string $selector The CSS selector.
 * @param string $style    The CSS style.
 * @param string $value    The CSS value.
 * @param string $prefix   The CSS prefix.
 * @param string $suffix   The CSS suffix.
 * @param bool   $display  Print the styles.
 * @return string Generated CSS.
 */
function twenty_twenty_one_generate_css( $selector, $style, $value, $prefix = '', $suffix = '', $display = true ) {

	// Bail early if there is no $selector elements or properties and $value.
	if ( ! $value || ! $selector ) {
		return '';
	}

	$css = sprintf( '%s { %s: %s; }', $selector, $style, $prefix . $value . $suffix );

	if ( $display ) {
		/*
		 * Note to reviewers: $css contains auto-generated CSS.
		 * It is included inside <style> tags and can only be interpreted as CSS on the browser.
		 * Using wp_strip_all_tags() here is sufficient escaping to avoid
		 * malicious attempts to close </style> and open a <script>.
		 */
		echo wp_strip_all_tags( $css ); // phpcs:ignore WordPress.Security.EscapeOutput
	}
	return $css;
}
