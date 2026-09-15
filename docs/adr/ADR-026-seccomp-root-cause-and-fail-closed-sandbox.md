# ADR-026: Seccomp Root Cause (fstatfs), Container Lifecycle, Fail-Closed Sandbox Modes, Durable Audit

**Date:** 2026-09-15 · **Status:** Accepted · **Supersedes:** ADR-025 §G2 hypothesis

## Context
`docker_tests` failed on the production path (`HardenedConfig::default()`, seccomp ON) with:
`reopen exec fifo: get safe /proc/thread-self/fd handle: fstatfs ...: operation not permitted`.
ADR-025 attributed the CI failure to a missing `clone3`. Direct isolation testing refuted this.

## Evidence (Docker 29.6.1 snap, runc 1.3.4, libseccomp 2.5.5, WSL2)
- `--network none` alone: PASS. `no-new-privileges` alone: PASS.
- `no-new-privileges` + old profile: FAIL with `fstatfs` EPERM → **root cause is the profile**, not snap, not clone3.
- Old profile lacked `statfs`/`fstatfs`, which runc needs after applying seccomp early under `no-new-privileges`.
- The `netns bind-mount` error was a secondary symptom of failed container init.

## Decisions
1. **Profile**: allowlist rebuilt (~200 syscalls) incl. `fstatfs/statfs`, `sigaltstack`, `capset/setgroups`, `*at` variants; explicit denylist for `ptrace`, `mount`, `setns`, `unshare`, `pivot_root`, `bpf`, `kexec_*`, `name_to_handle_at`, `open_by_handle_at`, `io_uring_*`.
   *Open item:* `fsopen/fsconfig/fsmount/move_mount/open_tree/mount_setattr` are allowed but were **not proven required** (tests passed before they were added); candidate for removal under least-privilege.
2. **Container lifecycle**: every run gets `--name ec-sbx-<uuid>`; on timeout or IO error `docker rm -f` is issued. Verified `leftover_ec_sbx=0` after full docker runs. *Open item:* dedicated timeout-path test.
3. **Fail-closed modes**: `SandboxMode::Local` rejected by `validate()` always; `SandboxMode::Simulated` rejected in release builds; the artifact-id name-oracle compiles only under `debug_assertions`.
4. **Durable audit**: `AppState::open` propagates `load_proposals` errors; audit restored via `AuditLog::from_entries`; handlers persist audit entries; storage failures are surfaced. Covered by `persistence_gate.rs`.
5. **Image parity**: `rust:1.96-slim` aligned with `rust-toolchain.toml`.
6. **Deps**: `anyhow` 1.0.104 (RUSTSEC-2026-0190).

## Test parity (honest statement)
Production executor path runs with seccomp ON (136 sandbox + 107 app docker tests green).
`HardenedDockerRunner::for_testing()` still uses `without_seccomp()` for escape-vector tests — **not at parity yet**; tracked as follow-up.

## Verification
`cargo test --workspace`: all green, 0 failed (46 feature-gated ignored). Clippy and rustdoc clean under `-D warnings`.
