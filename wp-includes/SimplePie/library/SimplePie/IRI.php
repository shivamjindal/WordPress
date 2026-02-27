<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/library/SimplePie/IRI.php' );
if ( '/wp-includes/SimplePie/library/SimplePie/IRI.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use SimplePie\IRI;

class_exists('SimplePie\IRI');

// @trigger_error(sprintf('Using the "SimplePie_IRI" class is deprecated since SimplePie 1.7.0, use "SimplePie\IRI" instead.'), \E_USER_DEPRECATED);

/** @phpstan-ignore-next-line */
if (\false) {
    /** @deprecated since SimplePie 1.7.0, use "SimplePie\IRI" instead */
    class SimplePie_IRI extends IRI
    {
    }
}
