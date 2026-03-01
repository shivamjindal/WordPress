<?php
require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/wp-includes/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-content/themes/twentytwentytwo/index.php' );
if ( '/wp-content/themes/twentytwentytwo/index.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

// There is nothing output here because block themes do not use php templates.
// There is a core ticket discussing removing this requirement for block themes:
// https://core.trac.wordpress.org/ticket/54272.
