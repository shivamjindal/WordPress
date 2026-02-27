<?php

// SPDX-FileCopyrightText: 2004-2023 Ryan Parman, Sam Sneddon, Ryan McCue
// SPDX-License-Identifier: BSD-3-Clause

declare(strict_types=1);

namespace SimplePie;

require_once dirname( dirname( __DIR__ ) ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/SimplePie/src/Rating.php' );
if ( '/wp-includes/SimplePie/src/Rating.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}

/**
 * Handles `<media:rating>` or `<itunes:explicit>` tags as defined in Media RSS and iTunes RSS respectively
 *
 * Used by {@see \SimplePie\Enclosure::get_rating()} and {@see \SimplePie\Enclosure::get_ratings()}
 *
 * This class can be overloaded with {@see \SimplePie\SimplePie::set_rating_class()}
 */
class Rating
{
    /**
     * Rating scheme
     *
     * @var ?string
     * @see get_scheme()
     */
    public $scheme;

    /**
     * Rating value
     *
     * @var ?string
     * @see get_value()
     */
    public $value;

    /**
     * Constructor, used to input the data
     *
     * For documentation on all the parameters, see the corresponding
     * properties and their accessors
     */
    public function __construct(
        ?string $scheme = null,
        ?string $value = null
    ) {
        $this->scheme = $scheme;
        $this->value = $value;
    }

    /**
     * String-ified version
     *
     * @return string
     */
    public function __toString()
    {
        // There is no $this->data here
        return md5(serialize($this));
    }

    /**
     * Get the organizational scheme for the rating
     *
     * @return string|null
     */
    public function get_scheme()
    {
        if ($this->scheme !== null) {
            return $this->scheme;
        }

        return null;
    }

    /**
     * Get the value of the rating
     *
     * @return string|null
     */
    public function get_value()
    {
        if ($this->value !== null) {
            return $this->value;
        }

        return null;
    }
}

class_alias('SimplePie\Rating', 'SimplePie_Rating');
