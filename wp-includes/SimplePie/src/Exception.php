<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

namespace SimplePie;

require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/src/Exception.php' );
if ( '/wp-includes/SimplePie/src/Exception.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

use Exception as NativeException;

/**
 * General SimplePie exception class
 */
class Exception extends NativeException
{
}

class_alias('SimplePie\Exception', 'SimplePie_Exception');
