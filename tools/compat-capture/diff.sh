#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
php_dir="${repo_root}/tests/compat/artifacts/php"
rust_dir="${repo_root}/tests/compat/artifacts/rust"

if [[ ! -d "${php_dir}" ]]; then
  echo "Missing PHP artifacts directory: ${php_dir}" >&2
  exit 1
fi

if [[ ! -d "${rust_dir}" ]]; then
  echo "Missing Rust artifacts directory: ${rust_dir}" >&2
  exit 1
fi

echo "Comparing ${php_dir} vs ${rust_dir}"
diff -ru "${php_dir}" "${rust_dir}"
echo "No differences found."
