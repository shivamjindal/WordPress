# Rust Migration Completion Checklist

Use this checklist as the release gate before disabling PHP fallback in production.

## 1) Functional parity gates

- [ ] Compatibility harness run succeeds with latest baseline route set:
  - `tools/compat-capture/run.sh --profile production-rust`
- [ ] php-runtime contract checks pass:
  - `tools/compat-capture/php-runtime-contract.sh`
- [ ] Core migrated surfaces validated (manual or scripted):
  - Frontend representative routes (`/`, search, feed, 404)
  - Auth entrypoints (`wp-login`, signup, activate, reset flows)
  - Admin entrypoints (`admin-ajax`, `admin-post`, `async-upload`, install/upgrade/repair)
  - REST (`/wp-json` read/write authz variants)
  - XML-RPC and cron routes
  - Public entrypoints (`comments-post`, `mail`, `trackback`, `links-opml`)
- [ ] Multisite behavior validated for host/path mapping.

## 2) Security and auth gates

- [ ] Auth cookie sign/verify round-trip tests pass.
- [ ] Nonce validation tests pass (current + previous tick).
- [ ] Capability checks enforced for protected REST/admin routes.
- [ ] Maintenance mode behavior verified with Rust maintenance endpoint path.
- [ ] Production profile hard-fail behavior verified when Rust is unavailable.

## 3) Performance and operability gates

- [ ] p95/p99 latency for migrated endpoints measured against PHP baseline.
- [ ] Error budget and fallback thresholds documented for canary.
- [ ] Logs include enough context to distinguish Rust-handled vs PHP-handled requests.
- [ ] Rollback procedure tested:
  - disable gateway or switch deployment profile back to legacy-safe.

## 4) Deployment profile gates

- [ ] `legacy-safe` profile behavior validated (conservative routing).
- [ ] `php-runtime` profile behavior validated (core routed, plugin/theme paths remain PHP).
- [ ] `production-rust` profile validated in staging with canary traffic.

## 5) Final cutover decision

- [ ] Stakeholders sign off on:
  - Functional parity
  - Security parity
  - Performance regression bounds
  - Rollback readiness
- [ ] Switch production profile to `production-rust`.
- [ ] Monitor canary and then full rollout; keep rollback command available during rollout window.
