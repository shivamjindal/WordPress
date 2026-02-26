#!/usr/bin/env bash
set -euo pipefail

strict_flag=""
deployment_profile="${WP_RUST_DEPLOYMENT_PROFILE:-legacy-safe}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict)
      strict_flag="--strict"
      shift
      ;;
    --profile)
      if [[ $# -lt 2 ]]; then
        echo "--profile requires a value" >&2
        exit 1
      fi
      deployment_profile="$2"
      shift 2
      ;;
    *)
      echo "Usage: $0 [--strict] [--profile <legacy-safe|production-rust|custom>]" >&2
      exit 1
      ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
php_pid=""
rust_pid=""

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

echo "Starting local PHP and Rust servers for compatibility capture..."
echo "Using WP_RUST_DEPLOYMENT_PROFILE=${deployment_profile}"
(cd "${repo_root}" && WP_RUST_DEPLOYMENT_PROFILE="${deployment_profile}" php -S 127.0.0.1:8080 >/tmp/wp-compat-php.log 2>&1) &
php_pid=$!

(cd "${repo_root}/rust" && WP_RUST_DEPLOYMENT_PROFILE="${deployment_profile}" cargo run -p wp-rs-server >/tmp/wp-compat-rust.log 2>&1) &
rust_pid=$!

sleep 2

echo "Capturing PHP baseline..."
"${repo_root}/tools/compat-capture/capture.sh" php "http://127.0.0.1:8080"
echo "Capturing Rust baseline..."
"${repo_root}/tools/compat-capture/capture.sh" rust "http://127.0.0.1:8088"

echo "Generating diff report..."
if [[ "${strict_flag}" == "--strict" ]]; then
  "${repo_root}/tools/compat-capture/diff.sh" --strict
else
  "${repo_root}/tools/compat-capture/diff.sh"
fi

echo "Compatibility capture completed."
