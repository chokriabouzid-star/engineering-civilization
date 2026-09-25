
Changelog
All notable changes are documented here. Format loosely follows Keep a Changelog.

[Unreleased]
Added
- **ADR-031**: Pin the sandbox image to the measured Rust 1.96 image by
  digest (`rust:1.96-slim`; rustc 1.96.1, glibc 2.41, Debian trixie;
  linux/amd64). Docker suites with the unchanged seccomp profile: 259
  passed, 0 failed, 2 ignored across `ec-sandbox` and `ec-app`.
- **F1**: `ec-analysis::isolation` — the analyzer now runs in a separate
  process (`EC-ANALYZE-WORKER-V1` stdin/stdout protocol) with a wall-clock
  budget and a recursive-spawn barrier. A dead worker is reported as a
  structured `IsolationError`, not as an abort of the caller. The worker
  starts with a cleared environment (no `EC_API_KEY`, no `EC_DB`).
- **F1**: Two permanent regression gate files (three tests). `f1_server_survival.rs` spawns a
  real `ec-server` and asserts the hostile payload yields HTTP 502 while
  `/health` stays 200 afterwards. `f1_cli_survival.rs` asserts `ec analyze`
  and `ec check` exit cleanly with 2 and 1 respectively (signal termination
  rejected) and that `ec check` reports an `analysis_failed` violation.
  `isolation.rs` unit tests cover foreign stdout, a hung worker killed at its
  deadline, non-zero exit, signal death, spawn failure, and the cleared
  environment.
- **T1.3**: Static seccomp policy gate (`seccomp_policy_gate.rs`, 11 tests) with 9 negative-control mutations enforces the least-privilege contract of `rust-sandbox.json` (ADR-029).
- **T1.3**: Compatibility gate (`least_privilege_compat_gate.rs`) proves hello-world, thread spawn/join, and child-process execution still work under `default_hardened()` with the tightened profile.
- **T1.3**: ADR-029 records the decision to restrict `clone` with namespace-bit mask (`0x7e020000`) and redirect `clone3` to `ENOSYS` (38) for glibc fallback.
T1.1 sandbox timeout cleanup regression gate (`timeout_gate.rs`) proves
forced container removal on timeout and zero `ec-sbx-*` orphans (ADR-026 C1).
T1.2 sandbox seccomp parity gate (`seccomp_parity_gate.rs`) verifies five
ABI-controlled syscall observations against the daemon-default baseline,
runs all five legacy escape vectors through `default_hardened()`, and requires
their runtime `BLOCKED`/`CONTAINED` markers rather than accepting infrastructure
errors as proof of containment (ADR-026 C4, ADR-028).
Kernel purity gate (ec-constitutional/tests/kernel_purity.rs) — fails if the
constitutional kernel gains a runtime dependency on tokio/async-trait
(ADR-023/ADR-027). Verified with a negative control.
LICENSE (MIT) and rust-version = "1.96" inherited across all 11 crates.
README.md, SECURITY.md, CHANGELOG.md, ADR-027.
Fixed
- **F1**: Deeply nested generic types (`Vec<Vec<...>>`, 600 levels) in
  `POST /api/v1/analyze`, `ec analyze`, and `ec check` aborted the process
  with SIGABRT (exit 134). The lexical guard added in 4c0635c counted `{([`
  and the unary run `&*!` only, so the payload passed it. After this change:
  `/analyze` returns 502 with an `error` body (504 on worker timeout, 500 if
  the worker cannot be located or started) and the server stays up;
  `ec analyze` exits 2 with a diagnostic on stderr; `ec check` records an
  `analysis_failed` violation and counts the file as failed (fail-closed).

Changed
- PROJECT-REFERENCE.md refreshed to v1.9.6 against 93950e8: ADR count 22 -> 26; workspace and `ec check` figures re-measured on that commit; Docker figures tied to an identical tree; seccomp status corrected to ADR-026/028/029 evidence; ec-analysis no longer labelled a pure kernel (subprocess isolation, ADR-030).
PROJECT-REFERENCE.md: corrected ADR count (19 -> 22).
ADR-024: inline correction notes for unverified figures (713 tests, 18 ADRs)
without rewriting history.
- **ADR-030**: correction addendum. The original claim ("zero exit 134 across
  all deep-nesting test cases") was scoped to `{([` and unary runs; the
  hostile payload touches neither counter. `depth_guard` is retained as a
  cheap first filter but is no longer claimed sufficient on its own.
[0.3.0] - ADR-026 (merge 0debf1c)
Security
Seccomp profile rebuilt for runc 1.3 / glibc 2.39 (root cause: fstatfs/statfs).
Container lifecycle: --name ec-sbx-<uuid> + docker rm -f on timeout/IO
(zero orphan containers).
Fail-closed: SandboxMode::Local rejected; Simulated disabled in release.
anyhow updated to 1.0.104 (RUSTSEC-2026-0190).
Fixed
Governance audit persistence: restored on startup, persisted on write;
storage errors propagated. Covered by persistence_gate.rs.
Stable artifact hash (SHA-256) replacing DefaultHasher in ec-app.
Rustdoc HTML/link escapes; float epsilon comparison in ec-constitutional.
Removed
14 tracked junk files (one-off scripts, patches, recon reports).
