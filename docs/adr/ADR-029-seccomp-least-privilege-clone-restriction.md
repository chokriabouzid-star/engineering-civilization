# ADR-029: Seccomp Profile Least-Privilege - clone/clone3 Restriction

## Status

Accepted (2026-09-21)

## Context

At `94eba48`, the custom seccomp profile (`rust-sandbox.json`) contained `clone`
and `clone3` in the unconditional `SCMP_ACT_ALLOW` group (203 syscalls).
Comparison against upstream Moby
(`moby/profiles@245180c51918481c0525424b3ee025d2b435d46c`, content SHA-256
`785b2429264afba4d594320337cb17f144f3c7d51585f9805eef72e28f4f9334`) revealed
two divergences:

1. **`clone`** - upstream restricts namespace-creation flags via
   `SCMP_CMP_MASKED_EQ` on argument 0 with mask `0x7e020000`. This mask covers
   `CLONE_NEWNS`, `CLONE_NEWCGROUP`, `CLONE_NEWUTS`, `CLONE_NEWIPC`,
   `CLONE_NEWUSER`, `CLONE_NEWPID`, `CLONE_NEWNET`. Our profile allowed these
   flags unconditionally.

2. **`clone3`** - upstream returns `ENOSYS(38)` to trigger the glibc fallback
   to `clone`. Our profile allowed `clone3` unconditionally. Using `EPERM(1)`
   instead would break thread/process creation in glibc >= 2.34 because the
   fallback triggers only on `ENOSYS`.

Measured environment: Docker client `29.1.3` / server `29.8.0`,
`rust:1.75-slim` image (glibc `2.36-9+deb12u4`), `x86_64` host and container.

## Decision

1. Remove `clone` and `clone3` from the unconditional `SCMP_ACT_ALLOW` group.
2. Add a conditional rule for `clone`: `SCMP_ACT_ALLOW` with `args[0]` filtered
   via `SCMP_CMP_MASKED_EQ` (mask `0x7e020000`, comparison `0`, `valueTwo: 0`
   written explicitly). Permits thread/process creation while blocking namespace
   creation.
3. Add a rule for `clone3`: `SCMP_ACT_ERRNO` with `errnoRet: 38` (ENOSYS).
   Triggers glibc fallback to the filterable `clone` path.
4. Enforce via static policy gate (`seccomp_policy_gate.rs`) with 11 tests.

## Consequences

- `clone` with namespace flags is now blocked at seccomp level, closing the
  last remaining namespace-creation vector (`unshare` and `setns` were already
  denied).
- `clone3` returns `ENOSYS` which glibc 2.36 interprets as "not available" and
  falls back to `clone`.
- Functional verification: 3 compat tests pass under `docker_tests`
  (`compat_exit=0`), 149-test full suite passes, fork-bomb creates 254 threads
  before pids-limit stop.
- The six mount-API syscalls are already denied (ERRNO group); no functional
  change. See ADR-026 addendum.

## How compliance is automatically verified

`crates/ec-sandbox/tests/seccomp_policy_gate.rs` runs in standard CI (no Docker)
and enforces:

- `clone` appears in exactly one `SCMP_ACT_ALLOW` rule with exact mask filter.
- `clone3` appears in exactly one `SCMP_ACT_ERRNO` rule with `errnoRet=38`.
- No escape vector appears in any `SCMP_ACT_ALLOW` rule.
- `statfs`/`fstatfs` remain in `SCMP_ACT_ALLOW` (regression guard for ADR-026 C3).
- No duplicate syscall names across rules.
- 9 negative-control mutations verify the gate rejects weakened profiles.
- In-memory compliant fixture proves the gate is not trivially green.

Reference profile: `moby/profiles@245180c51918481c0525424b3ee025d2b435d46c`.
Reference content SHA-256: `785b2429264afba4d594320337cb17f144f3c7d51585f9805eef72e28f4f9334`.
Patched profile SHA-256: `c8e83ba7557f7bc862a077f0a40a16613bcd4064556b4e8d15744a98b562f88d`.
