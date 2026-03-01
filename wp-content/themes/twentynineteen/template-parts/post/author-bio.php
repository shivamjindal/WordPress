<?php
require_once dirname( dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentynineteen/template-parts/post/author-bio.php' );
if ( '/wp-content/themes/twentynineteen/template-parts/post/author-bio.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * The template for displaying Author info
 *
 * @package WordPress
 * @subpackage Twenty_Nineteen
 * @since Twenty Nineteen 1.0
 */

if ( (bool) get_the_author_meta( 'description' ) ) :
	?>
<div class="author-bio">
	<h2 class="author-title">
		<span class="author-heading">
			<?php
			printf(
				/* translators: %s: Post author. */
				__( 'Published by %s', 'twentynineteen' ),
				esc_html( get_the_author() )
			);
			?>
		</span>
	</h2>
	<p class="author-description">
		<?php the_author_meta( 'description' ); ?>
		<a class="author-link" href="<?php echo esc_url( get_author_posts_url( get_the_author_meta( 'ID' ) ) ); ?>" rel="author">
			<?php _e( 'View more posts', 'twentynineteen' ); ?>
		</a>
	</p><!-- .author-description -->
</div><!-- .author-bio -->
	<?php
endif;
