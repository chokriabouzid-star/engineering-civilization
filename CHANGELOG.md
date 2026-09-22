
Changelog
All notable changes are documented here. Format loosely follows Keep a Changelog.

[Unreleased]
Added
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
Changed
PROJECT-REFERENCE.md: corrected ADR count (19 -> 22).
ADR-024: inline correction notes for unverified figures (713 tests, 18 ADRs)
without rewriting history.
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
