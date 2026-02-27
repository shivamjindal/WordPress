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
assert_rust_handled "/wp-admin/profile.php" "yes"
assert_rust_handled "/wp-admin/profile.php?updated=true" "yes"
assert_rust_handled "/wp-admin/user-edit.php?user_id=2" "yes"
assert_rust_handled "/wp-admin/user-edit.php?user_id=2&updated=true" "yes"
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
assert_rust_handled "/wp-admin/about.php" "yes"
assert_rust_handled "/wp-admin/credits.php" "yes"
assert_rust_handled "/wp-admin/contribute.php" "yes"
assert_rust_handled "/wp-admin/freedoms.php" "yes"
assert_rust_handled "/wp-admin/freedoms.php?privacy-notice=1" "yes"
assert_rust_handled "/wp-admin/privacy.php" "yes"
assert_rust_handled "/wp-admin/plugin-install.php" "yes"
assert_rust_handled "/wp-admin/plugin-install.php?tab=plugin-information" "yes"
assert_rust_handled "/wp-admin/plugin-editor.php" "yes"
assert_rust_handled "/wp-admin/plugin-editor.php?plugin=hello.php&file=hello.php" "yes"
assert_rust_handled "/wp-admin/theme-install.php" "yes"
assert_rust_handled "/wp-admin/theme-install.php?tab=theme-information" "yes"
assert_rust_handled "/wp-admin/theme-editor.php" "yes"
assert_rust_handled "/wp-admin/theme-editor.php?theme=twentytwentyfive&file=style.css" "yes"
assert_rust_handled "/wp-admin/plugins.php" "yes"
assert_rust_handled "/wp-admin/plugins.php?action=activate&plugin=hello.php" "yes"
assert_rust_handled "/wp-admin/themes.php" "yes"
assert_rust_handled "/wp-admin/themes.php?action=activate&stylesheet=twentytwentyfive" "yes"
assert_rust_handled "/wp-admin/users.php" "yes"
assert_rust_handled "/wp-admin/users.php?action=delete&id=2" "yes"
assert_rust_handled "/wp-admin/tools.php" "yes"
assert_rust_handled "/wp-admin/tools.php?wp-privacy-policy-guide=1" "yes"
assert_rust_handled "/wp-admin/tools.php?page=export_personal_data" "yes"
assert_rust_handled "/wp-admin/tools.php?page=remove_personal_data" "yes"
assert_rust_handled "/wp-admin/site-health.php" "yes"
assert_rust_handled "/wp-admin/site-health.php?tab=debug" "yes"
assert_rust_handled "/wp-admin/export.php" "yes"
assert_rust_handled "/wp-admin/export.php?download=true&content=all" "yes"
assert_rust_handled "/wp-admin/import.php" "yes"
assert_rust_handled "/wp-admin/import.php?invalid=movabletype" "yes"
assert_rust_handled "/wp-admin/export-personal-data.php" "yes"
assert_rust_handled "/wp-admin/erase-personal-data.php" "yes"
assert_rust_handled "/wp-admin/network.php" "yes"
assert_rust_handled "/wp-admin/network/setup.php" "yes"
assert_rust_handled "/wp-admin/network/" "yes"
assert_rust_handled "/wp-admin/network/index.php" "yes"
assert_rust_handled "/wp-admin/network/sites.php" "yes"
assert_rust_handled "/wp-admin/network/sites.php?action=confirm&action2=deleteblog&id=2" "yes"
assert_rust_handled "/wp-admin/network/users.php" "yes"
assert_rust_handled "/wp-admin/network/users.php?action=deleteuser&id=2" "yes"
assert_rust_handled "/wp-admin/network/themes.php" "yes"
assert_rust_handled "/wp-admin/network/themes.php?action=enable&theme=twentytwentyfive" "yes"
assert_rust_handled "/wp-admin/network/plugins.php" "yes"
assert_rust_handled "/wp-admin/network/plugins.php?action=activate&plugin=hello.php" "yes"
assert_rust_handled "/wp-admin/network/settings.php" "yes"
assert_rust_handled "/wp-admin/network/settings.php?network_admin_hash=confirm-rust-admin-email" "yes"
assert_rust_handled "/wp-admin/network/settings.php?dismiss=new_network_admin_email" "yes"
assert_rust_handled "/wp-admin/network/site-new.php" "yes"
assert_rust_handled "/wp-admin/network/site-new.php?update=added&id=2" "yes"
assert_rust_handled "/wp-admin/network/site-info.php?id=2" "yes"
assert_rust_handled "/wp-admin/network/site-info.php?update=updated&id=2" "yes"
assert_rust_handled "/wp-admin/network/site-settings.php?id=2" "yes"
assert_rust_handled "/wp-admin/network/site-settings.php?update=updated&id=2" "yes"
assert_rust_handled "/wp-admin/network/site-users.php?id=2" "yes"
assert_rust_handled "/wp-admin/network/site-users.php?id=2&update=adduser" "yes"
assert_rust_handled "/wp-admin/network/site-themes.php?id=2" "yes"
assert_rust_handled "/wp-admin/network/site-themes.php?id=2&enabled=1" "yes"
assert_rust_handled "/wp-admin/network/user-new.php" "yes"
assert_rust_handled "/wp-admin/network/user-new.php?update=added&user_id=2" "yes"
assert_rust_handled "/wp-admin/network/edit.php?action=siteoptions" "yes"
assert_rust_handled "/wp-admin/network/update.php?action=update-selected" "yes"
assert_rust_handled "/wp-admin/network/update-core.php" "yes"
assert_rust_handled "/wp-admin/network/update-core.php?action=do-plugin-upgrade" "yes"
assert_rust_handled "/wp-admin/network/plugin-install.php" "yes"
assert_rust_handled "/wp-admin/network/plugin-install.php?tab=plugin-information" "yes"
assert_rust_handled "/wp-admin/network/plugin-editor.php" "yes"
assert_rust_handled "/wp-admin/network/plugin-editor.php?plugin=hello.php&file=hello.php" "yes"
assert_rust_handled "/wp-admin/network/theme-editor.php" "yes"
assert_rust_handled "/wp-admin/network/theme-editor.php?theme=twentytwentyfive&file=style.css" "yes"
assert_rust_handled "/wp-admin/network/privacy.php" "yes"
assert_rust_handled "/wp-admin/network/privacy.php?updated=true" "yes"
assert_rust_handled "/wp-admin/network/about.php" "yes"
assert_rust_handled "/wp-admin/network/credits.php" "yes"
assert_rust_handled "/wp-admin/network/contribute.php" "yes"
assert_rust_handled "/wp-admin/network/freedoms.php" "yes"
assert_rust_handled "/wp-admin/network/profile.php" "yes"
assert_rust_handled "/wp-admin/network/profile.php?updated=true" "yes"
assert_rust_handled "/wp-admin/network/user-edit.php?user_id=2" "yes"
assert_rust_handled "/wp-admin/network/user-edit.php?user_id=2&updated=true" "yes"
assert_rust_handled "/wp-admin/network/upgrade.php" "yes"
assert_rust_handled "/wp-admin/network/upgrade.php?action=upgrade&n=0" "yes"
assert_rust_handled "/wp-admin/network/theme-install.php" "yes"
assert_rust_handled "/wp-admin/network/theme-install.php?tab=theme-information" "yes"
assert_rust_handled "/wp-admin/ms-delete-site.php" "yes"
assert_rust_handled "/wp-admin/ms-delete-site.php?h=confirm-rust-delete" "yes"
assert_rust_handled "/wp-admin/index.php" "yes"
assert_rust_handled "/wp-admin/update.php" "yes"
assert_rust_handled "/wp-admin/update.php?action=update-selected" "yes"
assert_rust_handled "/wp-admin/update-core.php" "yes"
assert_rust_handled "/wp-admin/update-core.php?action=do-plugin-upgrade" "yes"
assert_rust_handled "/wp-admin/admin.php" "yes"
assert_rust_handled "/wp-login.php" "yes" "POST" "application/x-www-form-urlencoded" "log=admin&pwd=secret"
assert_rust_handled "/wp-comments-post.php" "yes" "POST" "application/x-www-form-urlencoded" "comment_post_ID=123&comment=hello"
assert_rust_handled "/wp-admin/setup-config.php" "yes" "POST" "application/x-www-form-urlencoded" "dbname=wordpress&uname=wp_user&pwd=secret"
assert_rust_handled "/wp-admin/profile.php" "yes" "POST" "application/x-www-form-urlencoded" "action=update-user&nickname=rustadmin"
assert_rust_handled "/wp-admin/user-edit.php?user_id=2" "yes" "POST" "application/x-www-form-urlencoded" "action=update&user_id=2&email=admin%40example.com"
assert_rust_handled "/wp-admin/plugin-install.php" "yes" "POST" "application/x-www-form-urlencoded" "s=akismet&tab=search&type=term"
assert_rust_handled "/wp-admin/plugin-editor.php" "yes" "POST" "application/x-www-form-urlencoded" "action=update&plugin=hello.php&file=hello.php"
assert_rust_handled "/wp-admin/theme-install.php" "yes" "POST" "application/x-www-form-urlencoded" "s=twentytwentyfive&tab=search&type=term"
assert_rust_handled "/wp-admin/theme-editor.php" "yes" "POST" "application/x-www-form-urlencoded" "action=update&theme=twentytwentyfive&file=style.css"
assert_rust_handled "/wp-admin/plugins.php" "yes" "POST" "application/x-www-form-urlencoded" "action=activate-selected&checked=hello.php"
assert_rust_handled "/wp-admin/themes.php" "yes" "POST" "application/x-www-form-urlencoded" "action=enable-auto-update&stylesheet=twentytwentyfive"
assert_rust_handled "/wp-admin/users.php" "yes" "POST" "application/x-www-form-urlencoded" "action=delete&users=2"
assert_rust_handled "/wp-admin/update.php?action=update-selected" "yes" "POST" "application/x-www-form-urlencoded" "checked=hello.php"
assert_rust_handled "/wp-admin/update-core.php?action=do-plugin-upgrade" "yes" "POST" "application/x-www-form-urlencoded" "checked=hello.php"
assert_rust_handled "/wp-admin/export-personal-data.php" "yes" "POST" "application/x-www-form-urlencoded" "username_or_email_for_privacy_request=person%40example.com&action=add_request"
assert_rust_handled "/wp-admin/erase-personal-data.php" "yes" "POST" "application/x-www-form-urlencoded" "username_or_email_for_privacy_request=person%40example.com&action=add_request"
assert_rust_handled "/wp-admin/network.php" "yes" "POST" "application/x-www-form-urlencoded" "sitename=Rust+Network&email=admin%40example.com&subdomain_install=0"
assert_rust_handled "/wp-admin/network/setup.php" "yes" "POST" "application/x-www-form-urlencoded" "sitename=Rust+Network&email=admin%40example.com&subdomain_install=0"
assert_rust_handled "/wp-admin/network/sites.php" "yes" "POST" "application/x-www-form-urlencoded" "action=allblogs&site_ids=2"
assert_rust_handled "/wp-admin/network/users.php" "yes" "POST" "application/x-www-form-urlencoded" "action=allusers&bulk_action=spam&allusers=2"
assert_rust_handled "/wp-admin/network/themes.php" "yes" "POST" "application/x-www-form-urlencoded" "action=enable-selected&checked=twentytwentyfive"
assert_rust_handled "/wp-admin/network/plugins.php" "yes" "POST" "application/x-www-form-urlencoded" "action=deactivate-selected&checked=hello.php"
assert_rust_handled "/wp-admin/network/settings.php" "yes" "POST" "application/x-www-form-urlencoded" "site_name=Rust+Network&new_admin_email=admin%40example.com&registration=all"
assert_rust_handled "/wp-admin/network/site-new.php?action=add-site" "yes" "POST" "application/x-www-form-urlencoded" "blog%5Bdomain%5D=rustsite&blog%5Btitle%5D=Rust+Site&blog%5Bemail%5D=admin%40example.com"
assert_rust_handled "/wp-admin/network/site-info.php?action=update-site" "yes" "POST" "application/x-www-form-urlencoded" "id=2&blog%5Burl%5D=https%3A%2F%2Fexample.com%2Frustsite%2F"
assert_rust_handled "/wp-admin/network/site-settings.php?action=update-site" "yes" "POST" "application/x-www-form-urlencoded" "id=2&option%5Bblogname%5D=Rust+Site+2&option%5Badmin_email%5D=admin2%40example.com"
assert_rust_handled "/wp-admin/network/site-users.php?action=adduser" "yes" "POST" "application/x-www-form-urlencoded" "id=2&newuser=admin&new_role=subscriber"
assert_rust_handled "/wp-admin/network/site-themes.php?action=enable-selected" "yes" "POST" "application/x-www-form-urlencoded" "id=2&checked=twentytwentyfive"
assert_rust_handled "/wp-admin/network/user-new.php?action=add-user" "yes" "POST" "application/x-www-form-urlencoded" "user%5Busername%5D=rustuser&user%5Bemail%5D=rustuser%40example.com"
assert_rust_handled "/wp-admin/network/edit.php?action=siteoptions" "yes" "POST" "application/x-www-form-urlencoded" "id=2"
assert_rust_handled "/wp-admin/network/update.php?action=update-selected" "yes" "POST" "application/x-www-form-urlencoded" "checked=hello.php"
assert_rust_handled "/wp-admin/network/update-core.php?action=do-plugin-upgrade" "yes" "POST" "application/x-www-form-urlencoded" "checked=hello.php"
assert_rust_handled "/wp-admin/network/plugin-install.php" "yes" "POST" "application/x-www-form-urlencoded" "s=akismet&tab=search&type=term"
assert_rust_handled "/wp-admin/network/plugin-editor.php" "yes" "POST" "application/x-www-form-urlencoded" "action=update&plugin=hello.php&file=hello.php"
assert_rust_handled "/wp-admin/network/theme-editor.php" "yes" "POST" "application/x-www-form-urlencoded" "action=update&theme=twentytwentyfive&file=style.css"
assert_rust_handled "/wp-admin/network/privacy.php" "yes" "POST" "application/x-www-form-urlencoded" "page_for_privacy_policy=2"
assert_rust_handled "/wp-admin/network/profile.php" "yes" "POST" "application/x-www-form-urlencoded" "action=update-user&nickname=networkadmin"
assert_rust_handled "/wp-admin/network/user-edit.php?user_id=2" "yes" "POST" "application/x-www-form-urlencoded" "action=update&user_id=2&email=admin%40example.com"
assert_rust_handled "/wp-admin/network/upgrade.php" "yes" "POST" "application/x-www-form-urlencoded" "action=upgrade&n=0"
assert_rust_handled "/wp-admin/network/theme-install.php" "yes" "POST" "application/x-www-form-urlencoded" "s=twentytwentyfive&tab=search&type=term"
assert_rust_handled "/wp-admin/ms-delete-site.php" "yes" "POST" "application/x-www-form-urlencoded" "action=deleteblog&confirmdelete=1"
assert_rust_handled "/wp-content/plugins/hello.php" "no"
assert_rust_handled "/wp-content/plugins/hello.php" "no" "POST" "application/x-www-form-urlencoded" "foo=bar"
assert_rust_handled "/wp-content/themes/twentytwentyfive/style.css" "no"

echo "php-runtime compatibility contract checks passed."
