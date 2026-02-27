<?php
/**
 * Template canvas file to render the current 'wp_template'.
 *
 * @package WordPress
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/template-canvas.php' ) === '/wp-includes/template-canvas.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/template-canvas.php' ) ) {
	exit;
}

/*
 * Get the template HTML.
 * This needs to run before <head> so that blocks can add scripts and styles in wp_head().
 */
$template_html = get_the_block_template_html();
?><!DOCTYPE html>
<html <?php language_attributes(); ?>>
<head>
	<meta charset="<?php bloginfo( 'charset' ); ?>" />
	<?php wp_head(); ?>
</head>

<body <?php body_class(); ?>>
<?php wp_body_open(); ?>

<?php echo $template_html; ?>

<?php wp_footer(); ?>
</body>
</html>
