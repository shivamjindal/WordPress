<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/library/SimplePie/HTTP/Parser.php' );
if ( '/wp-includes/SimplePie/library/SimplePie/HTTP/Parser.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use SimplePie\HTTP\Parser;

class_exists('SimplePie\HTTP\Parser');

// @trigger_error(sprintf('Using the "SimplePie_HTTP_Parser" class is deprecated since SimplePie 1.7.0, use "SimplePie\HTTP\Parser" instead.'), \E_USER_DEPRECATED);

/** @phpstan-ignore-next-line */
if (\false) {
    /**
     * @deprecated since SimplePie 1.7.0, use "SimplePie\HTTP\Parser" instead
     * @template Psr7Compatible of bool
     * @extends Parser<Psr7Compatible>
     */
    class SimplePie_HTTP_Parser extends Parser
    {
    }
}
