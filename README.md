# Engineering Civilization

Constitutional software evaluation kernel with hardened sandbox execution,
multi-dimensional fitness governance, and durable audit trails.

## Status (honest)

- Branch: `master`, post-ADR-026 (seccomp root-cause, container lifecycle, durable audit).
- Rust: `1.96.0` pinned via `rust-toolchain.toml`; `rust-version = "1.96"` inherited by all crates.
- License: MIT (`LICENSE`), inherited via `license.workspace = true`.
- At ADR-026 merge (`0debf1c`): CI green; tests 666 passed / 0 failed / 46 feature-gated ignored.
- Current status: trust GitHub Actions on HEAD, not numbers written here.

This is an engineering snapshot, not a permanent guarantee.

## What this is

- Static analysis of Rust into a six-dimensional fitness vector.
- Constitutional evaluation with catastrophic, non-compensable thresholds.
- Epistemic uncertainty modeling and calibration primitives.
- Hardened Docker/seccomp execution of untrusted Rust with measured reality.
- Causal memory and governance with SQLite persistence.
- Thin REST API and CLI adapters over the kernel crates.

## What this is not

- Not a formal security boundary against kernel or container-runtime vulnerabilities.
- Not a general-purpose code execution platform.
- Not externally calibrated yet (see Known limitations).

## Quick start

```bash
cargo test --workspace --locked --no-fail-fast

cargo test -p ec-sandbox --locked --features docker_tests -- --test-threads=1
cargo test -p ec-app     --locked --features docker_tests -- --test-threads=1

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

EC_API_KEY=secret EC_DB=ec.db EC_BIND_ADDR=127.0.0.1:8080 \
  cargo run -p ec-api --bin ec-server
GET /api/v1/health is public; other endpoints require x-api-key.
Workspace structure
text

crates/
  ec-fitness         6D fitness vector + Pareto
  ec-epistemic       uncertainty, Bayesian, calibration
  ec-constitutional  evaluation engine, thresholds, meta
  ec-analysis        static analysis (syn-based)
  ec-sandbox         Docker/seccomp execution, RealityVector
  ec-memory          causal decision graph (SQLite)
  ec-codegen         template generation
  ec-governance      proposals, audit, storage
  ec-app             Integration / Iterative / Bayesian pipelines
  ec-api             Axum REST adapter
  ec-cli             CLI adapter (bin: ec)

docs/adr/  docs/proofs/  docs/specs/
Security & hardening (implemented)
Docker execution uses --network none, --cap-drop ALL, --read-only,
--user 1000:1000, --pids-limit 256, no-new-privileges,
seccomp=crates/ec-sandbox/profiles/rust-sandbox.json, tmpfs workspace,
per-run --name ec-sbx-<uuid> with docker rm -f on timeout/IO.

Fail-closed: SandboxMode::Local rejected in validate(); Simulated
disabled in release; test-only name behavior isolated to #[cfg(test)].

Durable governance: AppState::open() restores proposals and audit log;
storage errors propagated. Covered by ec-api/tests/persistence_gate.rs.

Kernel purity: ec-constitutional/tests/kernel_purity.rs fails the build if
the kernel gains a runtime dependency on tokio/async-trait (ADR-023/ADR-027).

Root-cause note: ADR-025 hypothesized missing clone3; isolation testing
proved the real cause was missing fstatfs/statfs under runc 1.3.
ADR-026 supersedes ADR-025 on this point.

Known limitations (tracked, not hidden)
Some escape-vector tests still run via for_testing() without seccomp;
production path uses seccomp ON. Parity suite pending.
No dedicated timeout-orphan regression test yet.
Some seccomp allowlist syscalls (fsopen/fsconfig/fsmount/move_mount/
open_tree/mount_setattr) are least-privilege removal candidates.
API key comparison is not constant-time; no rate/body limits yet.
CPU-bound analysis and SQLite run inside async handlers without spawn_blocking.
Heuristic dimensions reversibility and architectural_stability are
uncalibrated and currently produce false positives on legitimate code
(e.g. CLI println!, high-import modules). Treat as advisory until calibrated.
Self-check CI job is advisory (continue-on-error) until its
false-positive rate is proven acceptable — see limitation 6.
Verification
Bash

git log -1 --oneline
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
docker ps -aq --filter name=ec-sbx- | wc -l   # expect 0 after docker tests
Key ADRs
ADR-021: slow tests policy.
ADR-023: kernel purity and coupling fix.
ADR-024: multi-model audit resolutions (with inline corrections).
ADR-025: post-Phase-4 remediation (seccomp hypothesis superseded).
ADR-026: seccomp root cause, lifecycle, fail-closed, durable audit.
ADR-027: kernel purity gate (structural, deterministic).
License
MIT — see LICENSE.
