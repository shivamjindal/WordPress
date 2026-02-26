#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
seed_sql="${repo_root}/tests/compat/seed/wordpress_seed.sql"

db_host="${DB_HOST:-127.0.0.1}"
db_port="${DB_PORT:-3306}"
db_user="${DB_USER:-root}"
db_password="${DB_PASSWORD:-}"
db_name="${DB_NAME:-wordpress}"

if ! command -v mysql >/dev/null 2>&1; then
  echo "mysql client not found; skipping seed import."
  exit 0
fi

if [[ ! -f "${seed_sql}" ]]; then
  echo "Seed SQL file missing at ${seed_sql}; skipping import."
  exit 0
fi

echo "Ensuring database ${db_name} exists..."
mysql --host="${db_host}" --port="${db_port}" --user="${db_user}" --password="${db_password}" \
  -e "CREATE DATABASE IF NOT EXISTS \`${db_name}\`;"

echo "Importing seed data from ${seed_sql}..."
mysql --host="${db_host}" --port="${db_port}" --user="${db_user}" --password="${db_password}" \
  "${db_name}" < "${seed_sql}"

echo "Seed import completed."
