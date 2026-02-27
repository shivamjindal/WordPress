<?php
/**
 * WordPress Version
 *
 * Contains version information for the current WordPress release.
 *
 * @package WordPress
 * @since 1.2.0
 */

require_once __DIR__ . '/rust-gateway.php';
if ( wp_rust_gateway_current_request_path( '/wp-includes/version.php' ) === '/wp-includes/version.php'
	&& wp_rust_gateway_try_proxy( '/wp-includes/version.php' ) ) {
	exit;
}

/**
 * The WordPress version string.
 *
 * Holds the current version number for WordPress core. Used to bust caches
 * and to enable development mode for scripts when running from the /src directory.
 *
 * @global string $wp_version
 */
$wp_version = '7.0-alpha-61604';

/**
 * Holds the WordPress DB revision, increments when changes are made to the WordPress DB schema.
 *
 * @global int $wp_db_version
 */
$wp_db_version = 60717;

/**
 * Holds the TinyMCE version.
 *
 * @global string $tinymce_version
 */
$tinymce_version = '49110-20250317';

/**
 * Holds the minimum required PHP version.
 *
 * @global string $required_php_version
 */
$required_php_version = '7.4';

/**
 * Holds the names of required PHP extensions.
 *
 * @global string[] $required_php_extensions
 */
$required_php_extensions = array(
	'json',
	'hash',
);

/**
 * Holds the minimum required MySQL version.
 *
 * @global string $required_mysql_version
 */
$required_mysql_version = '5.5.5';
