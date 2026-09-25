# ADR-031: Sandbox image upgraded to Rust 1.96 (pinned by digest)

## Status
Accepted

## Context
The workspace toolchain file requests Rust `1.96.0`, while the sandbox
previously used `rust:1.75-slim` (Debian 12 bookworm, glibc 2.36, rustc
1.75.0). This was a material version mismatch. Updating the image also changes
the userspace distribution and therefore warranted compatibility testing
against the existing seccomp profile.

## Decision
Pin `ec-sandbox::DEFAULT_IMAGE` to this immutable image reference:

`rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523`

On the measured host, Docker reported that `rust:1.96-slim` and the pinned
digest resolved to the same image ID (`TAG_AND_DIGEST_SAME_IMAGE_ID=YES`).

### Measured image properties

| Property | Observed value |
|---|---|
| OS / architecture | `linux/amd64` |
| Tag observed locally | `rust:1.96-slim` |
| rustc bundled in image | `1.96.1 (31fca3adb 2026-06-26)` |
| cargo bundled in image | `1.96.1 (356927216 2026-06-26)` |
| glibc | `2.41 (Debian GLIBC 2.41-12+deb13u3)` |
| Distribution | Debian 13 (trixie) |

### Toolchain-version scope

The image contains Rust `1.96.1`; the repository toolchain file requests
`1.96.0`. This decision aligns the image with the same `1.96.x` series; it
does **not** claim patch-level identity.

A separate invocation with the repository's exact toolchain selection active
caused rustup to attempt downloading `1.96.0`. That download failed with a DNS
resolution error in the measured environment. The Docker compatibility results
below establish successful behavior for the tested sandbox workloads; they do
not establish that exact `1.96.0` can be installed or selected offline inside
the image.

## Verification (measured)

The Docker suites were run with the existing, unmodified seccomp profile and
the pinned image reference:

| Suite | Result |
|---|---|
| `ec-sandbox` | 152 passed, 0 failed, 1 ignored |
| `ec-app` Docker-feature suite | 107 passed, 0 failed, 1 ignored |
| Total across both suites | 259 passed, 0 failed, 2 ignored |
| `least_privilege_compat_gate` | hello-world, thread spawn/join, and child process all exited 0 |
| `seccomp_parity_gate` | 1 passed |
| `seccomp_policy_gate` | 11 passed |
| `timeout_gate` | 1 passed |
| `ec-sbx-*` containers | 0 before and 0 after |

The measured results support compatibility of the existing profile with the
workloads covered by those tests on the observed `linux/amd64` image. They are
not a claim that every possible program or syscall has been verified.

## Consequences
- `DEFAULT_IMAGE` is pinned by digest; changes to the mutable tag do not
  silently change this reference.
- The observed image is `linux/amd64`. Compatibility on other architectures
  has not been measured here.
- Advancing the digest requires rerunning the Docker compatibility suites
  against the candidate image before changing the pin.

## How to verify automatically
Run these commands from the repository root on a host with Docker available.
They exercise the image configured by `DEFAULT_IMAGE`:

~~~bash
cargo test -p ec-sandbox --locked --features docker_tests -- --test-threads=1
cargo test -p ec-app --locked --features docker_tests -- --test-threads=1
~~~
