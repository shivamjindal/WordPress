<?php
/**
 * WordPress Diff bastard child of old MediaWiki Diff Formatter.
 *
 * Basically all that remains is the table structure and some method names.
 *
 * @package WordPress
 * @subpackage Diff
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/wp-diff.php' ) === '/wp-includes/wp-diff.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/wp-diff.php' ) ) {
	exit;
}

// Don't load directly.
if ( ! defined( 'ABSPATH' ) ) {
	die( '-1' );
}

if ( ! class_exists( 'Text_Diff', false ) ) {
	/** Text_Diff class */
	require ABSPATH . WPINC . '/Text/Diff.php';
	/** Text_Diff_Renderer class */
	require ABSPATH . WPINC . '/Text/Diff/Renderer.php';
	/** Text_Diff_Renderer_inline class */
	require ABSPATH . WPINC . '/Text/Diff/Renderer/inline.php';
	/** Text_Exception class */
	require ABSPATH . WPINC . '/Text/Exception.php';
}

require ABSPATH . WPINC . '/class-wp-text-diff-renderer-table.php';
require ABSPATH . WPINC . '/class-wp-text-diff-renderer-inline.php';
