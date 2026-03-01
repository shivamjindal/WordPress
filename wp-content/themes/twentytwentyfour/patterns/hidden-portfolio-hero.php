<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentyfour/patterns/hidden-portfolio-hero.php' );
if ( '/wp-content/themes/twentytwentyfour/patterns/hidden-portfolio-hero.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Title: Portfolio hero
 * Slug: twentytwentyfour/hidden-portfolio-hero
 * Inserter: no
 */
?>

<!-- wp:spacer {"height":"var:preset|spacing|50","style":{"layout":{}}} -->
<div style="height:var(--wp--preset--spacing--50)" aria-hidden="true" class="wp-block-spacer"></div>
<!-- /wp:spacer -->

<!-- wp:group {"align":"wide","layout":{"type":"constrained"}} -->
<div class="wp-block-group alignwide">
	<!-- wp:heading {"level":1,"align":"wide","style":{"typography":{"lineHeight":"1.2"}},"fontSize":"xx-large"} -->
	<h1 class="wp-block-heading alignwide has-xx-large-font-size" style="line-height:1.2"><?php echo wp_kses_post( __( 'I’m <em>Leia Acosta</em>, a passionate photographer who finds inspiration in capturing the fleeting beauty of life.', 'twentytwentyfour' ) ); ?></h1>
	<!-- /wp:heading -->
</div>
<!-- /wp:group -->
