<?php
require_once dirname( __DIR__ ) . '/rust-gateway.php';
$rust_gateway_request_path = wp_rust_gateway_current_request_path( '/wp-includes/sodium_compat/autoload-php7.php' );
if ( '/wp-includes/sodium_compat/autoload-php7.php' === $rust_gateway_request_path
	&& wp_rust_gateway_try_proxy( $rust_gateway_request_path ) ) {
	exit;
}
/*
 This file should only ever be loaded on PHP 7+
 */
if (PHP_VERSION_ID < 70000) {
    return;
}

spl_autoload_register(function ($class) {
    $namespace = 'ParagonIE_Sodium_';
    // Does the class use the namespace prefix?
    $len = strlen($namespace);
    if (strncmp($namespace, $class, $len) !== 0) {
        // no, move to the next registered autoloader
        return false;
    }

    // Get the relative class name
    $relative_class = substr($class, $len);

    // Replace the namespace prefix with the base directory, replace namespace
    // separators with directory separators in the relative class name, append
    // with .php
    $file = dirname(__FILE__) . '/src/' . str_replace('_', '/', $relative_class) . '.php';
    // if the file exists, require it
    if (file_exists($file)) {
        require_once $file;
        return true;
    }
    return false;
});
