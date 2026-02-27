<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/library/SimplePie/Copyright.php' );
if ( '/wp-includes/SimplePie/library/SimplePie/Copyright.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use SimplePie\Copyright;

class_exists('SimplePie\Copyright');

// @trigger_error(sprintf('Using the "SimplePie_Copyright" class is deprecated since SimplePie 1.7.0, use "SimplePie\Copyright" instead.'), \E_USER_DEPRECATED);

/** @phpstan-ignore-next-line */
if (\false) {
    /** @deprecated since SimplePie 1.7.0, use "SimplePie\Copyright" instead */
    class SimplePie_Copyright extends Copyright
    {
    }
}
