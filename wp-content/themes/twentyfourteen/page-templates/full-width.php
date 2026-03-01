<?php
require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentyfourteen/page-templates/full-width.php' );
if ( '/wp-content/themes/twentyfourteen/page-templates/full-width.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Template Name: Full Width Page
 *
 * @package WordPress
 * @subpackage Twenty_Fourteen
 * @since Twenty Fourteen 1.0
 */

get_header(); ?>

<div id="main-content" class="main-content">

<?php
if ( is_front_page() && twentyfourteen_has_featured_posts() ) {
	// Include the featured content template.
	get_template_part( 'featured-content' );
}
?>

	<div id="primary" class="content-area">
		<div id="content" class="site-content" role="main">
			<?php
			// Start the Loop.
			while ( have_posts() ) :
				the_post();

				// Include the page content template.
				get_template_part( 'content', 'page' );

				// If comments are open or we have at least one comment, load up the comment template.
				if ( comments_open() || get_comments_number() ) {
					comments_template();
				}
				endwhile;
			?>
		</div><!-- #content -->
	</div><!-- #primary -->
</div><!-- #main-content -->

<?php
get_sidebar();
get_footer();
