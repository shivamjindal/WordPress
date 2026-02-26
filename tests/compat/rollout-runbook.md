# Rust Gateway Rollout Runbook

This runbook defines how to move between deployment profiles safely:
- `legacy-safe`
- `php-runtime`
- `production-rust`

## 0) Preconditions

Before canary rollout:

1. Run compatibility captures:
   - `tools/compat-capture/run.sh --profile production-rust`
2. Run php-runtime fallback contract checks:
   - `tools/compat-capture/php-runtime-contract.sh`
3. Confirm migration checklist gates in `migration-checklist.md`.

## 1) Profile semantics

- `legacy-safe`
  - Conservative default.
  - Rust gateway typically disabled or narrowly allowlisted.
- `php-runtime`
  - Rust handles migrated core endpoints only.
  - Plugin/theme non-core routes continue in PHP.
- `production-rust`
  - Full Rust cutover profile.
  - Fallback disabled by profile override.

## 2) Canary rollout procedure

### Step A — Enable `php-runtime` canary

Set environment/config:

```bash
WP_RUST_DEPLOYMENT_PROFILE=legacy-safe
WP_RUST_GATEWAY_ENABLED=1
WP_RUST_PLUGIN_COMPAT_MODE=php-runtime
WP_RUST_ENDPOINT_ALLOWLIST=*
WP_RUST_METHOD_ALLOWLIST=GET,HEAD,POST
```

Validate:

1. Core migrated routes return `X-WP-Rust-Handled: 1`.
2. Representative plugin/theme paths do **not** return Rust handled header.
3. Error rate remains within expected bounds.

### Step B — Increase canary traffic

Gradually increase percentage of traffic routed through these settings.
Monitor:

- 5xx rate
- auth/login failure rate
- admin write-path failures
- latency (`X-WP-Rust-Latency-*` metrics)

### Step C — Switch canary to `production-rust`

Set:

```bash
WP_RUST_DEPLOYMENT_PROFILE=production-rust
```

Re-run smoke checks on:

- login/logout/reset,
- admin AJAX/post/upload,
- REST read/write capability checks,
- cron and XML-RPC probes,
- maintenance endpoint behavior.

## 3) Rollback procedure

If canary degrades:

### Immediate rollback (safest)

```bash
WP_RUST_DEPLOYMENT_PROFILE=legacy-safe
WP_RUST_GATEWAY_ENABLED=0
```

### Partial rollback (keep observability, disable strict cutover)

```bash
WP_RUST_DEPLOYMENT_PROFILE=legacy-safe
WP_RUST_GATEWAY_ENABLED=1
WP_RUST_PLUGIN_COMPAT_MODE=php-runtime
WP_RUST_ENDPOINT_ALLOWLIST=/__wp_rust/health,/wp-json/*
WP_RUST_METHOD_ALLOWLIST=GET,HEAD
```

After rollback:

1. Confirm PHP handling restored for impacted routes.
2. Capture failing route artifacts.
3. File incident notes with route, status, request method, and latency headers.

## 4) Exit criteria for full rollout

Proceed from canary to 100% rollout only when:

- No sustained error-rate regressions.
- No auth/capability regressions.
- Latency and throughput within agreed thresholds.
- Rollback path tested and ready.
