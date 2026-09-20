//! T1.1 — ADR-026 / C1 regression gate: timeout must leave zero orphan containers.
//!
//! What it proves: when `compile_and_run_hardened` falls into the `recv_timeout`
//! branch, the named container `ec-sbx-<uuid>` is force-removed (`docker rm -f`).
//! `--rm` alone is NOT enough: it only fires when the CLI process exits, which
//! never happens on a hung container.
//!
//! NEGATIVE CONTROL (mandatory — a gate without one is a claim, not a gate):
//!   1. comment out `Self::force_remove_container(&container_name);` in
//!      `crates/ec-sandbox/src/hardened.rs` (Timeout branch, ~line 232)
//!   2. this test MUST fail on the POST assertion (orphan leaked)
//!   3. restore the line, `docker rm -f` the leak, re-run -> PASS
//!
//! Run:
//!   cargo test -p ec-sandbox --locked --features docker_tests \
//!     --test timeout_gate -- --test-threads=1 --nocapture

use std::process::Command;
use std::time::{Duration, Instant};

use ec_sandbox::docker::DockerError;
use ec_sandbox::hardened::{HardenedConfig, HardenedDockerRunner};

/// Compiles cleanly, then never exits. Forces the recv_timeout path.
const HANGING_SOURCE: &str = "fn main() { loop {} }";

/// IDs of every container whose name starts with `ec-sbx-` (running or exited).
fn orphan_ids() -> Vec<String> {
    let out = Command::new("docker")
        .args(["ps", "-aq", "--filter", "name=ec-sbx-"])
        .output()
        .expect("`docker ps` failed — is the daemon running?");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

/// One scenario: hang a container, demand Timeout, demand zero leftovers.
fn assert_timeout_leaves_no_orphan(label: &str, runner: HardenedDockerRunner) {
    let pre = orphan_ids();
    assert!(
        pre.is_empty(),
        "[{label}] PRE-CONDITION: {} orphan(s) already present: {pre:?}\n\
         clean with: docker ps -aq --filter name=ec-sbx- | xargs -r docker rm -f",
        pre.len()
    );

    let started = Instant::now();
    let result = runner.compile_and_run_hardened(HANGING_SOURCE);
    let elapsed = started.elapsed();

    match &result {
        Err(DockerError::Timeout { duration_secs }) => {
            println!("[{label}] OK: Timeout after {duration_secs}s (wall {elapsed:?})");
        }
        other => panic!(
            "[{label}] expected Err(DockerError::Timeout), got {other:?} after {elapsed:?}.\n\
             If this is Ok(..), the container self-terminated before recv_timeout \
             and the C1 path was never exercised — this gate would be vacuous."
        ),
    }

    let post = orphan_ids();
    assert!(
        post.is_empty(),
        "[{label}] POST-CONDITION: container leaked! {} orphan(s): {post:?}\n\
         force_remove_container did not run on the Timeout branch.",
        post.len()
    );
    println!("[{label}] OK: leftover=0");
}

/// Construct a hardened runner with a short timeout using public APIs only.
///
/// This intentionally does not use the private `clone_with_timeout` helper:
/// integration tests must exercise the public construction contract.
fn make_runner(timeout_secs: u64, hardened: HardenedConfig) -> HardenedDockerRunner {
    let base = ec_sandbox::docker::DockerRunner::new(
        ec_sandbox::docker::DEFAULT_IMAGE,
        512,
        0.5,
        Duration::from_secs(timeout_secs),
    );
    HardenedDockerRunner::new(base, hardened).expect("hardened runner with custom timeout")
}

/// Sequential on purpose: two scenarios in one `#[test]` so no intra-binary
/// parallelism can pollute the global `ec-sbx-` orphan count.
#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests + a running docker daemon"
)]
fn timeout_kills_orphan_container() {
    // (a) testing profile (seccomp off) — isolates the lifecycle question.
    assert_timeout_leaves_no_orphan(
        "for_testing",
        make_runner(2, HardenedConfig::without_seccomp()),
    );

    // (b) production profile (seccomp on) — rule #6: the shipped path must be
    // the tested path. A failure here is a real finding for T1.2/T1.3, not noise.
    assert_timeout_leaves_no_orphan(
        "default_hardened",
        make_runner(2, HardenedConfig::default()),
    );
}
