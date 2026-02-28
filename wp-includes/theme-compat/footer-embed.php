<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/theme-compat/footer-embed.php' );
if ( '/wp-includes/theme-compat/footer-embed.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/**
 * Contains the post embed footer template
 *
 * When a post is embedded in an iframe, this file is used to create the footer output
 * if the active theme does not include a footer-embed.php template.
 *
 * @package WordPress
 * @subpackage Theme_Compat
 * @since 4.5.0
 */

/**
 * Prints scripts or data before the closing body tag in the embed template.
 *
 * @since 4.4.0
 */
do_action( 'embed_footer' );
?>
</body>
</html>
