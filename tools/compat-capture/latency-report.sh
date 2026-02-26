#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
php_dir="${repo_root}/tests/compat/artifacts/php"
rust_dir="${repo_root}/tests/compat/artifacts/rust"
report_dir="${repo_root}/tests/compat/artifacts/report"
report_file="${report_dir}/latest-latency.json"

mkdir -p "${report_dir}"

PHP_DIR="${php_dir}" \
RUST_DIR="${rust_dir}" \
REPORT_FILE="${report_file}" \
python3 - <<'PY'
import json
import math
import os
from pathlib import Path


def percentile(values, p):
    if not values:
        return None
    if len(values) == 1:
        return float(values[0])
    rank = (len(values) - 1) * p
    low = math.floor(rank)
    high = math.ceil(rank)
    if low == high:
        return float(values[low])
    weight = rank - low
    return float(values[low] * (1 - weight) + values[high] * weight)


def summarize(directory):
    files = sorted(Path(directory).glob("*.json"))
    latencies_ms = []
    latencies_us = []
    present_count = 0

    for file in files:
        try:
            payload = json.loads(file.read_text())
        except json.JSONDecodeError:
            continue

        if payload.get("rust_latency_header_present"):
            present_count += 1
        latency_ms = payload.get("rust_latency_ms")
        latency_us = payload.get("rust_latency_us")
        if isinstance(latency_ms, int):
            latencies_ms.append(latency_ms)
        if isinstance(latency_us, int):
            latencies_us.append(latency_us)

    latencies_ms.sort()
    latencies_us.sort()
    return {
        "captured_routes": len(files),
        "latency_header_present_count": present_count,
        "latency_value_count": len(latencies_ms),
        "latency_value_us_count": len(latencies_us),
        "latency_ms": {
            "min": latencies_ms[0] if latencies_ms else None,
            "max": latencies_ms[-1] if latencies_ms else None,
            "p50": percentile(latencies_ms, 0.50),
            "p95": percentile(latencies_ms, 0.95),
            "mean": (sum(latencies_ms) / len(latencies_ms)) if latencies_ms else None,
        },
        "latency_us": {
            "min": latencies_us[0] if latencies_us else None,
            "max": latencies_us[-1] if latencies_us else None,
            "p50": percentile(latencies_us, 0.50),
            "p95": percentile(latencies_us, 0.95),
            "mean": (sum(latencies_us) / len(latencies_us)) if latencies_us else None,
        },
    }


summary = {
    "php": summarize(os.environ["PHP_DIR"]),
    "rust": summarize(os.environ["RUST_DIR"]),
}

report_path = Path(os.environ["REPORT_FILE"])
report_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
print(report_path)
PY
