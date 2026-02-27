<?php
/**
 * Back-compat placeholder for the base embed template
 *
 * @package WordPress
 * @subpackage oEmbed
 * @since 4.4.0
 * @deprecated 4.5.0 Moved to wp-includes/theme-compat/embed.php
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/embed-template.php' ) === '/wp-includes/embed-template.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/embed-template.php' ) ) {
	exit;
}

_deprecated_file( basename( __FILE__ ), '4.5.0', WPINC . '/theme-compat/embed.php' );

require ABSPATH . WPINC . '/theme-compat/embed.php';
