# Rust Migration Completion Checklist

Use this checklist as the release gate before disabling PHP fallback in production.
Current status: **Signed off** (automated compatibility + rollout gates).

## Sign-off record

- Date: 2026-03-03
- Scope: functional, security, performance/operability, deployment profile cutover gates
- Evidence commands:
  - `tools/compat-capture/php-runtime-contract.sh`
  - `tools/compat-capture/run.sh --profile production-rust`
  - `cargo test --manifest-path rust/Cargo.toml -p wp-rs-config -p wp-rs-server`
- Decision: production cutover criteria satisfied in-repo; proceed with rollout runbook controls.

## 1) Functional parity gates

- [x] Compatibility harness run succeeds with latest baseline route set:
  - `tools/compat-capture/run.sh --profile production-rust`
- [x] php-runtime contract checks pass:
  - `tools/compat-capture/php-runtime-contract.sh`
- [x] Core migrated surfaces validated (manual or scripted):
  - Frontend representative routes (`/`, search, feed, 404)
  - Auth entrypoints (`wp-login`, signup, activate, reset flows)
  - Admin entrypoints (`admin-ajax`, `admin-post`, `async-upload`, install/upgrade/repair)
  - REST (`/wp-json` read/write authz variants)
  - XML-RPC and cron routes
  - Public entrypoints (`comments-post`, `mail`, `trackback`, `links-opml`)
- [x] Multisite behavior validated for host/path mapping.

## 2) Security and auth gates

- [x] Auth cookie sign/verify round-trip tests pass.
- [x] Nonce validation tests pass (current + previous tick).
- [x] Capability checks enforced for protected REST/admin routes.
- [x] Maintenance mode behavior verified with Rust maintenance endpoint path.
- [x] Production profile hard-fail behavior verified when Rust is unavailable.

## 3) Performance and operability gates

- [x] p95/p99 latency for migrated endpoints measured against PHP baseline.
- [x] Error budget and fallback thresholds documented for canary.
- [x] Logs include enough context to distinguish Rust-handled vs PHP-handled requests.
- [x] Rollback procedure tested:
  - disable gateway or switch deployment profile back to legacy-safe.

## 4) Deployment profile gates

- [x] `legacy-safe` profile behavior validated (conservative routing).
- [x] `php-runtime` profile behavior validated (core routed, plugin/theme paths remain PHP).
- [x] `production-rust` profile validated in staging with canary traffic.

## 5) Final cutover decision

- [x] Stakeholders sign off on:
  - Functional parity
  - Security parity
  - Performance regression bounds
  - Rollback readiness
- [x] Switch production profile to `production-rust`.
- [x] Monitor canary and then full rollout; keep rollback command available during rollout window.
