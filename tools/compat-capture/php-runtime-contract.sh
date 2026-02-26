#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
php_pid=""
rust_pid=""
php_port="${WP_RUNTIME_CONTRACT_PHP_PORT:-$((12000 + RANDOM % 2000))}"
rust_port="${WP_RUNTIME_CONTRACT_RUST_PORT:-$((php_port + 1))}"
php_base_url="http://127.0.0.1:${php_port}"
rust_base_url="http://127.0.0.1:${rust_port}"

cleanup() {
  if [[ -n "${php_pid}" ]]; then
    kill "${php_pid}" >/dev/null 2>&1 || true
    wait "${php_pid}" >/dev/null 2>&1 || true
  fi
  if [[ -n "${rust_pid}" ]]; then
    kill "${rust_pid}" >/dev/null 2>&1 || true
    wait "${rust_pid}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "Starting Rust server..."
(
  cd "${repo_root}/rust" && \
  WP_RUST_SERVER_LISTEN="127.0.0.1:${rust_port}" cargo run -p wp-rs-server >/tmp/wp-compat-runtime-rust.log 2>&1
) &
rust_pid=$!

echo "Starting PHP server with php-runtime compatibility mode..."
(
  cd "${repo_root}" && \
  WP_RUST_GATEWAY_ENABLED=1 \
  WP_RUST_GATEWAY_BACKEND_URL="${rust_base_url}" \
  WP_RUST_ENDPOINT_ALLOWLIST="*" \
  WP_RUST_METHOD_ALLOWLIST="GET,HEAD,POST" \
  WP_RUST_PLUGIN_COMPAT_MODE="php-runtime" \
  php -S "127.0.0.1:${php_port}" >/tmp/wp-compat-runtime-php.log 2>&1
) &
php_pid=$!

wait_for_endpoint() {
  local url="$1"
  local attempts=0
  while (( attempts < 20 )); do
    if curl -sS "${url}" >/dev/null 2>&1; then
      return 0
    fi
    attempts=$((attempts + 1))
    sleep 1
  done
  echo "Timed out waiting for ${url}" >&2
  return 1
}

wait_for_endpoint "${rust_base_url}/__wp_rust/health"
wait_for_endpoint "${php_base_url}/wp-login.php"

capture_headers() {
  local route="$1"
  local method="$2"
  local content_type="$3"
  local payload="$4"
  local header_file="$5"
  local body_file="$6"

  local curl_args=(
    -sS
    -X "${method}"
    -D "${header_file}"
    -o "${body_file}"
  )

  if [[ -n "${content_type}" ]]; then
    curl_args+=( -H "Content-Type: ${content_type}" )
  fi
  if [[ -n "${payload}" ]]; then
    curl_args+=( --data "${payload}" )
  fi

  curl "${curl_args[@]}" "${php_base_url}${route}"
}

rust_handled_header() {
  local header_file="$1"
  awk 'tolower($1)=="x-wp-rust-handled:"{sub(/\r$/, "", $2); print $2; exit}' "${header_file}"
}

assert_rust_handled() {
  local route="$1"
  local expected="$2"
  local method="${3:-GET}"
  local content_type="${4:-}"
  local payload="${5:-}"
  local header_file
  local body_file
  header_file="$(mktemp)"
  body_file="$(mktemp)"
  capture_headers "${route}" "${method}" "${content_type}" "${payload}" "${header_file}" "${body_file}"
  local handled
  handled="$(rust_handled_header "${header_file}")"
  rm -f "${header_file}" "${body_file}"

  if [[ "${expected}" == "yes" && "${handled}" != "1" ]]; then
    echo "Expected Rust handling for ${route}, got '${handled:-<none>}'" >&2
    exit 1
  fi
  if [[ "${expected}" == "no" && -n "${handled}" ]]; then
    echo "Expected PHP handling for ${route}, got Rust handled header '${handled}'" >&2
    exit 1
  fi
}

assert_rust_handled "/wp-login.php" "yes"
assert_rust_handled "/wp-admin/install.php" "yes"
assert_rust_handled "/wp-admin/setup-config.php" "yes"
assert_rust_handled "/wp-admin/install-helper.php" "yes"
assert_rust_handled "/wp-admin/options.php" "yes"
assert_rust_handled "/wp-admin/options-general.php" "yes"
assert_rust_handled "/wp-admin/options-writing.php" "yes"
assert_rust_handled "/wp-admin/options-reading.php" "yes"
assert_rust_handled "/wp-admin/options-discussion.php" "yes"
assert_rust_handled "/wp-admin/options-media.php" "yes"
assert_rust_handled "/wp-admin/options-permalink.php" "yes"
assert_rust_handled "/wp-admin/options-privacy.php" "yes"
assert_rust_handled "/wp-admin/options-privacy.php?tab=policyguide" "yes"
assert_rust_handled "/wp-admin/privacy-policy-guide.php" "yes"
assert_rust_handled "/wp-admin/tools.php" "yes"
assert_rust_handled "/wp-admin/tools.php?wp-privacy-policy-guide=1" "yes"
assert_rust_handled "/wp-admin/tools.php?page=export_personal_data" "yes"
assert_rust_handled "/wp-admin/tools.php?page=remove_personal_data" "yes"
assert_rust_handled "/wp-admin/site-health.php" "yes"
assert_rust_handled "/wp-admin/site-health.php?tab=debug" "yes"
assert_rust_handled "/wp-admin/admin.php" "yes"
assert_rust_handled "/wp-login.php" "yes" "POST" "application/x-www-form-urlencoded" "log=admin&pwd=secret"
assert_rust_handled "/wp-comments-post.php" "yes" "POST" "application/x-www-form-urlencoded" "comment_post_ID=123&comment=hello"
assert_rust_handled "/wp-admin/setup-config.php" "yes" "POST" "application/x-www-form-urlencoded" "dbname=wordpress&uname=wp_user&pwd=secret"
assert_rust_handled "/wp-content/plugins/hello.php" "no"
assert_rust_handled "/wp-content/plugins/hello.php" "no" "POST" "application/x-www-form-urlencoded" "foo=bar"
assert_rust_handled "/wp-content/themes/twentytwentyfive/style.css" "no"

echo "php-runtime compatibility contract checks passed."
