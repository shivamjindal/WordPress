<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentyeleven/404.php' );
if ( '/wp-content/themes/twentyeleven/404.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Template for displaying 404 pages (Not Found)
 *
 * @package WordPress
 * @subpackage Twenty_Eleven
 * @since Twenty Eleven 1.0
 */

get_header(); ?>

	<div id="primary">
		<div id="content" role="main">

			<article id="post-0" class="post error404 not-found">
				<header class="entry-header">
					<h1 class="entry-title"><?php _e( 'This is somewhat embarrassing, isn&rsquo;t it?', 'twentyeleven' ); ?></h1>
				</header>

				<div class="entry-content">
					<p><?php _e( 'It seems we can&rsquo;t find what you&rsquo;re looking for. Perhaps searching, or one of the links below, can help.', 'twentyeleven' ); ?></p>

					<?php get_search_form(); ?>

					<?php the_widget( 'WP_Widget_Recent_Posts', array( 'number' => 10 ), array( 'widget_id' => '404' ) ); ?>

					<div class="widget">
						<h2 class="widgettitle"><?php _e( 'Most Used Categories', 'twentyeleven' ); ?></h2>
						<ul>
						<?php
						wp_list_categories(
							array(
								'orderby'    => 'count',
								'order'      => 'DESC',
								'show_count' => 1,
								'title_li'   => '',
								'number'     => 10,
							)
						);
						?>
						</ul>
					</div>

					<?php
					/* translators: %s: Smilie. */
					$archive_content = '<p>' . sprintf( __( 'Try looking in the monthly archives. %s', 'twentyeleven' ), convert_smilies( ':)' ) ) . '</p>';
					the_widget(
						'WP_Widget_Archives',
						array(
							'count'    => 0,
							'dropdown' => 1,
						),
						array( 'after_title' => '</h2>' . $archive_content )
					);
					?>

					<?php the_widget( 'WP_Widget_Tag_Cloud' ); ?>

				</div><!-- .entry-content -->
			</article><!-- #post-0 -->

		</div><!-- #content -->
	</div><!-- #primary -->

<?php get_footer(); ?>
