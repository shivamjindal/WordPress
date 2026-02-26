#!/usr/bin/env bash
set -euo pipefail

target="${1:-}"
base_url="${2:-}"

if [[ -z "${target}" ]]; then
  echo "Usage: $0 <php|rust> [base_url]" >&2
  exit 1
fi

case "${target}" in
  php)
    base_url="${base_url:-http://127.0.0.1:8080}"
    ;;
  rust)
    base_url="${base_url:-http://127.0.0.1:8088}"
    ;;
  *)
    echo "Target must be php or rust" >&2
    exit 1
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
routes_file="${repo_root}/tests/compat/baseline_routes.txt"
payloads_file="${repo_root}/tests/compat/request_payloads.tsv"
output_dir="${repo_root}/tests/compat/artifacts/${target}"

mkdir -p "${output_dir}"

echo "Capturing ${target} responses from ${base_url}"

lookup_request_payload() {
  local request_method="$1"
  local request_path="$2"

  if [[ ! -f "${payloads_file}" ]]; then
    return 1
  fi

  awk -F'\t' -v method="${request_method}" -v path="${request_path}" '
    $0 !~ /^#/ && NF >= 4 && $1 == method && $2 == path {
      print $3 "\t" $4
      found = 1
      exit
    }
    END {
      if (!found) {
        exit 1
      }
    }
  ' "${payloads_file}"
}

while IFS= read -r route || [[ -n "${route}" ]]; do
  entry="$(echo "${route}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
  [[ -z "${entry}" ]] && continue
  [[ "${entry}" =~ ^# ]] && continue

  method="GET"
  path="${entry}"
  if [[ "${entry}" =~ ^(GET|POST|PUT|PATCH|DELETE|HEAD)[[:space:]]+(.+)$ ]]; then
    method="${BASH_REMATCH[1]}"
    path="${BASH_REMATCH[2]}"
  fi

  slug="${method,,}_$(echo "${path}" | sed 's#[?&=/]#_#g' | sed 's#^_##' | sed 's#_*$##')"
  [[ -z "${slug}" ]] && slug="root"

  url="${base_url}${path}"
  headers_file="$(mktemp)"
  body_file="$(mktemp)"
  request_content_type=""
  request_body=""
  if payload_entry="$(lookup_request_payload "${method}" "${path}" 2>/dev/null)"; then
    request_content_type="${payload_entry%%$'\t'*}"
    request_body="${payload_entry#*$'\t'}"
  fi

  curl_args=(
    -sS
    -X "${method}"
    -o "${body_file}"
    -D "${headers_file}"
    -w "%{http_code}"
  )

  if [[ -n "${request_content_type}" ]]; then
    curl_args+=( -H "Content-Type: ${request_content_type}" )
  fi
  if [[ -n "${request_body}" ]]; then
    curl_args+=( --data "${request_body}" )
  fi

  status_code="$(curl "${curl_args[@]}" "${url}" || true)"
  content_type="$(awk -F': ' 'tolower($1)=="content-type"{sub(/\r$/, "", $2); print $2; exit}' "${headers_file}")"
  location_header="$(awk -F': ' 'tolower($1)=="location"{sub(/\r$/, "", $2); print $2; exit}' "${headers_file}")"
  rust_handled="$(awk -F': ' 'tolower($1)=="x-wp-rust-handled"{sub(/\r$/, "", $2); print $2; exit}' "${headers_file}")"
  rust_latency_ms="$(awk -F': ' 'tolower($1)=="x-wp-rust-latency-ms"{sub(/\r$/, "", $2); print $2; exit}' "${headers_file}")"
  rust_latency_us="$(awk -F': ' 'tolower($1)=="x-wp-rust-latency-micros"{sub(/\r$/, "", $2); print $2; exit}' "${headers_file}")"
  set_cookie_count="$(awk 'tolower($1)=="set-cookie:"{count++} END{print count+0}' "${headers_file}")"
  body_sha256="$(sha256sum "${body_file}" | awk '{print $1}')"

  STATUS_CODE="${status_code}" \
  METHOD="${method}" \
  CONTENT_TYPE="${content_type}" \
  LOCATION_HEADER="${location_header}" \
  RUST_HANDLED="${rust_handled}" \
  RUST_LATENCY_MS="${rust_latency_ms}" \
  RUST_LATENCY_US="${rust_latency_us}" \
  SET_COOKIE_COUNT="${set_cookie_count}" \
  BODY_SHA256="${body_sha256}" \
  BODY_FILE="${body_file}" \
  ROUTE="${path}" \
  python3 - <<'PY' > "${output_dir}/${slug}.json"
import json
import os

body_file = os.environ["BODY_FILE"]
with open(body_file, "rb") as handle:
    body = handle.read()

preview = body.decode("utf-8", errors="replace")[:400]
document = {
    "method": os.environ["METHOD"],
    "route": os.environ["ROUTE"],
    "status_code": int(os.environ["STATUS_CODE"]) if os.environ["STATUS_CODE"].isdigit() else -1,
    "content_type": os.environ["CONTENT_TYPE"],
    "location": os.environ["LOCATION_HEADER"],
    "rust_handled": os.environ["RUST_HANDLED"],
    "rust_latency_ms": int(os.environ["RUST_LATENCY_MS"]) if os.environ["RUST_LATENCY_MS"].isdigit() else None,
    "rust_latency_us": int(os.environ["RUST_LATENCY_US"]) if os.environ["RUST_LATENCY_US"].isdigit() else None,
    "rust_latency_header_present": bool(os.environ["RUST_LATENCY_MS"] or os.environ["RUST_LATENCY_US"]),
    "set_cookie_count": int(os.environ["SET_COOKIE_COUNT"]),
    "body_sha256": os.environ["BODY_SHA256"],
    "body_preview": preview,
}
print(json.dumps(document, indent=2, sort_keys=True))
PY

  rm -f "${headers_file}" "${body_file}"
  echo "Captured ${method} ${path} -> ${output_dir}/${slug}.json"
done < "${routes_file}"

echo "Done."
