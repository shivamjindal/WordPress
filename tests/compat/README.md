# PHP vs Rust compatibility harness

This directory stores fixtures and route manifests used to compare legacy PHP
responses against Rust gateway responses.

## Files

- `baseline_routes.txt`: list of routes to capture for both runtimes.
- `request_payloads.tsv`: request body fixtures used for POST route captures.
- `migration-checklist.md`: production cutover release checklist.
- `artifacts/php/`: captured normalized PHP responses.
- `artifacts/rust/`: captured normalized Rust responses.
- `artifacts/report/latest.diff`: latest generated parity report.
- `seed/wordpress_seed.sql`: deterministic fixture data for `wp_options`.

## Workflow

1. (Optional) run `tools/compat-capture/seed-db.sh` to reset fixture data.
2. Run `tools/compat-capture/run.sh` for one-command capture + diff report.
3. If you want strict parity checking, use `tools/compat-capture/run.sh --strict`.
4. To compare with a Rust-cutover profile, use:
   - `tools/compat-capture/run.sh --profile production-rust`
   - combine with strict mode when needed:
     `tools/compat-capture/run.sh --profile production-rust --strict`

You can still run individual steps manually:

1. Start legacy PHP server and Rust server.
2. Run `tools/compat-capture/capture.sh php`
3. Run `tools/compat-capture/capture.sh rust`
4. Run `tools/compat-capture/diff.sh` (or `--strict`).

## php-runtime compatibility checks

Run `tools/compat-capture/php-runtime-contract.sh` to validate that:
- migrated core endpoints are handled by Rust in `php-runtime` mode, and
- representative plugin/theme file paths remain handled by PHP (for both GET and POST probes).
