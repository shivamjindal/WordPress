#!/usr/bin/env bash
set -euo pipefail

strict_mode="false"
if [[ "${1:-}" == "--strict" ]]; then
  strict_mode="true"
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
php_dir="${repo_root}/tests/compat/artifacts/php"
rust_dir="${repo_root}/tests/compat/artifacts/rust"
report_dir="${repo_root}/tests/compat/artifacts/report"
report_file="${report_dir}/latest.diff"

if [[ ! -d "${php_dir}" ]]; then
  echo "Missing PHP artifacts directory: ${php_dir}" >&2
  exit 1
fi

if [[ ! -d "${rust_dir}" ]]; then
  echo "Missing Rust artifacts directory: ${rust_dir}" >&2
  exit 1
fi

mkdir -p "${report_dir}"

echo "Comparing ${php_dir} vs ${rust_dir}"
if diff -ru "${php_dir}" "${rust_dir}" > "${report_file}"; then
  echo "No differences found."
  exit 0
fi

echo "Differences found. Report written to ${report_file}"
if [[ "${strict_mode}" == "true" ]]; then
  cat "${report_file}"
  exit 1
fi
