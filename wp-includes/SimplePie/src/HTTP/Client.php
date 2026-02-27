<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

namespace SimplePie\HTTP;

require_once dirname( dirname( dirname( __DIR__ ) ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/src/HTTP/Client.php' );
if ( '/wp-includes/SimplePie/src/HTTP/Client.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * HTTP Client interface
 *
 * @internal
 */
interface Client
{
    public const METHOD_GET = 'GET';

    /**
     * send a request and return the response
     *
     * @param Client::METHOD_* $method
     * @param array<string, string> $headers
     *
     * @throws ClientException if anything goes wrong requesting the data
     */
    public function request(string $method, string $url, array $headers = []): Response;
}
