<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/library/SimplePie/gzdecode.php' );
if ( '/wp-includes/SimplePie/library/SimplePie/gzdecode.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use SimplePie\Gzdecode;

class_exists('SimplePie\Gzdecode');

// @trigger_error(sprintf('Using the "SimplePie_gzdecode" class is deprecated since SimplePie 1.7.0, use "SimplePie\Gzdecode" instead.'), \E_USER_DEPRECATED);

/** @phpstan-ignore-next-line */
if (\false) {
    /** @deprecated since SimplePie 1.7.0, use "SimplePie\Gzdecode" instead */
    class SimplePie_gzdecode extends Gzdecode
    {
    }
}
