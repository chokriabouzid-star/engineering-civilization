# ADR-028: seccomp parity truth table & architectural honesty

Status: Accepted

## Context (T1.2 / ADR-026 C4)
`HardenedConfig::without_seccomp()` (used by `for_testing()`) sets
`seccomp_profile: None`. This omits `--security-opt seccomp=...` from the
Docker invocation; the daemon then applies its BUILT-IN default policy
(confirmed: `docker info` → `SecurityOptions=[name=seccomp,profile=builtin]`).
It is NOT unconfined.

## Measured truth table (2026-09-20, Docker 29.1.3, x86_64)

| Probe | A (daemon default) | B (EC custom) | Role |
|---|---|---|---|
| ptrace_attach_bogus | ret=-1 errno=3 (ESRCH) | ret=-1 errno=1 (EPERM) | Discriminating |
| pidfd_open_bogus | ret=-1 errno=3 (ESRCH) | ret=-1 errno=1 (EPERM) | Discriminating |
| pidfd_open_self | ret≥0 errno=0 (SUCCESS) | ret=-1 errno=1 (EPERM) | **Primary discriminating** |
| process_vm_readv_zero_iov | ret=0 errno=0 (SUCCESS) | ret=-1 errno=1 (EPERM) | Discriminating |
| process_vm_readv_invalid_flags | ret=-1 errno=22 (EINVAL) | ret=-1 errno=1 (EPERM) | Discriminating |

None of the above demonstrate a container escape by themselves; they show
that syscalls reach the kernel under Docker default and are intercepted by
seccomp under the EC custom profile.

## Escape vector parity (all under `default_hardened()`)
All 5 legacy vectors ran on the production constructor and returned their
runtime containment marker (not merely a Docker/compiler error):
- proc_sysrq      → BLOCKED: Permission denied
- dev_mem         → BLOCKED: No such file or directory
- ptrace_proc_mem → BLOCKED: Permission denied
- mount_syscall   → BLOCKED
- fork_bomb       → CONTAINED: pids-limit stopped fork bomb at 254 threads

## Decision
1. Gate = `seccomp_parity_gate.rs`: asserts exact (ret, errno) pairs for A,
   exact EPERM for B, runtime markers for all 5 legacy vectors, PRE/POST
   orphan check, and a `SRC_UNTOUCHED` guard (zero src/ changes).
2. Legacy escape helpers historically treated any error (including Docker
   infra errors) as "contained" via `is_contained()`. This gate additionally
   requires the vector's own runtime marker, closing a false-pass risk.
3. `without_seccomp()`/`for_testing()` naming is misleading; documented here.
   Rename to `without_custom_seccomp()` is a follow-up, out of T1.2 scope.
4. Exact errno assertions on ARM A lock current Docker default behavior.
   If Docker changes it, rerun the gate and update this ADR — never relax
   the assertion silently.

## Verification
cargo test -p ec-sandbox --locked --features docker_tests \
  --test seccomp_parity_gate -- --test-threads=1 --nocapture
