# PHP vs Rust compatibility harness

This directory stores fixtures and route manifests used to compare legacy PHP
responses against Rust gateway responses.

## Files

- `baseline_routes.txt`: list of routes to capture for both runtimes.
- `artifacts/php/`: captured normalized PHP responses.
- `artifacts/rust/`: captured normalized Rust responses.

## Workflow

1. Start legacy PHP server and Rust server.
2. Run `tools/compat-capture/capture.sh php`
3. Run `tools/compat-capture/capture.sh rust`
4. Run `tools/compat-capture/diff.sh`

The diff command prints route-level mismatches and exits non-zero when parity
checks fail.
