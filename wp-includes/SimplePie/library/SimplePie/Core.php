<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/library/SimplePie/Core.php' );
if ( '/wp-includes/SimplePie/library/SimplePie/Core.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * SimplePie class.
 *
 * Class for backward compatibility.
 *
 * @deprecated Use {@see SimplePie} directly
 */
class SimplePie_Core extends SimplePie
{
}
