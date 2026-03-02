<?php
/**
 * Add Link Administration Screen.
 *
 * @package WordPress
 * @subpackage Administration
 */

require_once dirname( __DIR__ ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-admin/link-add.php' );
if ( '/wp-admin/link-add.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/** Load WordPress Administration Bootstrap */
require_once __DIR__ . '/admin.php';

if ( ! current_user_can( 'manage_links' ) ) {
	wp_die( __( 'Sorry, you are not allowed to add links to this site.' ) );
}

// Used in the HTML title tag.
$title       = __( 'Add Link' );
$parent_file = 'link-manager.php';

$action  = ! empty( $_REQUEST['action'] ) ? sanitize_text_field( $_REQUEST['action'] ) : '';
$cat_id  = ! empty( $_REQUEST['cat_id'] ) ? absint( $_REQUEST['cat_id'] ) : 0;
$link_id = ! empty( $_REQUEST['link_id'] ) ? absint( $_REQUEST['link_id'] ) : 0;

wp_enqueue_script( 'link' );
wp_enqueue_script( 'xfn' );

if ( wp_is_mobile() ) {
	wp_enqueue_script( 'jquery-touch-punch' );
}

$link = get_default_link_to_edit();
require ABSPATH . 'wp-admin/edit-link-form.php';

require_once ABSPATH . 'wp-admin/admin-footer.php';
