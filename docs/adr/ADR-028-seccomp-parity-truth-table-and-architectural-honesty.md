# ADR-028: seccomp parity truth table & architectural honesty

Status: Accepted

## Context (T1.2)
`HardenedConfig::without_seccomp()` sets `seccomp_profile: None`.
Docker command omits `--security-opt seccomp=...`; daemon applies
BUILT-IN default profile (`moby/default.json`). Confirmed by:
`docker info --format '...Seccomp=profile=builtin...'`

## Measured truth table (2026-09-20, Docker 29.1.3, x86_64)

| Probe | ARM A (daemon default) | ARM B (EC custom) | Type |
|---|---|---|---|
| pidfd_open_self | ret=3 errno=0 (SUCCESS) | ret=-1 errno=1 | **Primary discriminator** |
| process_vm_readv_zero | ret=0 errno=0 (SUCCESS) | ret=-1 errno=1 | **Secondary** |
| ptrace_attach_bogus | ret=-1 errno=3 (ESRCH) | ret=-1 errno=1 | Tertiary |
| pidfd_open_bogus | ret=-1 errno=3 (ESRCH) | ret=-1 errno=1 | Tertiary |
| unshare_newuser | ret=-1 errno=1 | ret=-1 errno=1 | Defense-in-depth |
| add_key_null | ret=-1 errno=1 | ret=-1 errno=1 | Defense-in-depth |
| keyctl_null | ret=-1 errno=1 | ret=-1 errno=1 | Defense-in-depth |
| perf_event_open_null | ret=-1 errno=1 | ret=-1 errno=1 | Defense-in-depth |

## Decisions
1. Gate asserts 4 discriminating + 4 defense probes; pre/post orphan check.
2. `without_seccomp()` is misnamed — it means "Docker default profile",
   not "unconfined". Rename to `without_custom_seccomp()` is a follow-up.
3. Exact errno assertions (ESRCH=3) lock Docker default behavior.
   If Docker changes, rerun probe and update — never relax silently.
4. T1.3 (least-privilege matrix) investigates `clone` args filter
   and the 6 mount API syscalls overlapping with `--cap-drop ALL`.

## Verification
`cargo test -p ec-sandbox --features docker_tests --test seccomp_parity_gate`
