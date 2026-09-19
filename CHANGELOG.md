
Changelog
All notable changes are documented here. Format loosely follows Keep a Changelog.

[Unreleased]
Added
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
