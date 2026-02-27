<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

require_once dirname( dirname( dirname( dirname( __DIR__ ) ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/library/SimplePie/Cache/MySQL.php' );
if ( '/wp-includes/SimplePie/library/SimplePie/Cache/MySQL.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use SimplePie\Cache\MySQL;

class_exists('SimplePie\Cache\MySQL');

// @trigger_error(sprintf('Using the "SimplePie_Cache_MySQL" class is deprecated since SimplePie 1.7.0, use "SimplePie\Cache\MySQL" instead.'), \E_USER_DEPRECATED);

/** @phpstan-ignore-next-line */
if (\false) {
    /** @deprecated since SimplePie 1.7.0, use "SimplePie\Cache\MySQL" instead */
    class SimplePie_Cache_MySQL extends MySQL
    {
    }
}
