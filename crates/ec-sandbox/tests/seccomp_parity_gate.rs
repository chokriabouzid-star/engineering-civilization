//! T1.2 / ADR-026 C4 — custom seccomp enforcement + escape parity on the
//! production path.
//!
//! ARMS (measured 2026-09-20, Docker 29.1.3, `SecurityOptions=[seccomp,profile=builtin]`):
//!   A = `HardenedDockerRunner::for_testing()`     -> no `--security-opt seccomp`
//!       flag is passed, so the DAEMON DEFAULT policy applies. NOT unconfined.
//!   B = `HardenedDockerRunner::default_hardened()` -> the real production
//!       constructor, loading our custom profile (defaultAction=SCMP_ACT_ERRNO).
//!
//! INDICATOR = (ret, errno), never `is_err()`. SCMP_ACT_ERRNO returns EPERM(1)
//! and kills nothing; exit_code is 0 in BOTH arms.
//!
//! FIVE ABI-CONTROLLED CASES (every numeric argument is c_long-wide; the 6th
//! argument travels on the stack on x86_64, so width matters):
//!   ptrace_attach_bogus            A=(-1, ESRCH 3)  B=(-1, EPERM 1)
//!   pidfd_open_bogus               A=(-1, ESRCH 3)  B=(-1, EPERM 1)
//!   pidfd_open_self                A=(fd>=0, 0)     B=(-1, EPERM 1)  <- success flips
//!   process_vm_readv_zero_iov      A=(0, 0)         B=(-1, EPERM 1)  <- success flips
//!   process_vm_readv_invalid_flags A=(-1, EINVAL 22) B=(-1, EPERM 1)
//!
//! SCOPE — what this gate does NOT claim:
//!   * NOT "EC is globally stricter than Docker default". Only these five
//!     cases are measured.
//!   * `pidfd_open(self)` and a zero-length `process_vm_readv` are benign
//!     operations; neither is a container escape.
//!   * Completeness / least-privilege of the profile is T1.3. Known open item:
//!     `clone`/`clone3` are ALLOW with no args filter in our profile.
//!
//! MEASURED NON-DISCRIMINATING (kept out on purpose, documented in ADR-028):
//!   unshare(CLONE_NEWUSER), add_key, keyctl, perf_event_open = EPERM in BOTH
//!   arms (daemon default + `--cap-drop ALL`). They cannot prove anything here.
//!
//! LEGACY VECTORS: `test_*_escape` report `escaped=false` even on a Docker
//! error (`is_contained()` is just `!escaped`). So containment alone is NOT
//! accepted: we also require the runtime marker printed AFTER `---OUTPUT---`,
//! proving the program actually ran under the production profile.
//!
//! NEGATIVE CONTROL (mandatory before commit):
//!   1. change ONLY the `production` binding below to `for_testing()`
//!   2. rerun: the ARM B `(-1, EPERM)` assertions MUST fail with (-1,3)/(3,0)/(0,0)
//!   3. restore `default_hardened()`, rerun -> green
//!
//! ARCH: x86_64 syscall numbers (101/434/310); the probe refuses to compile
//! on anything else inside the container.
//!
//! Run:
//!   cargo test -p ec-sandbox --locked --features docker_tests \
//!     --test seccomp_parity_gate -- --test-threads=1 --nocapture

use std::collections::HashMap;
use std::process::Command;

use ec_sandbox::hardened::{EscapeTestResult, HardenedDockerRunner};

const EPERM: i32 = 1;
const ESRCH: i32 = 3;
const EINVAL: i32 = 22;

const PROBE_NAMES: [&str; 5] = [
    "ptrace_attach_bogus",
    "pidfd_open_bogus",
    "pidfd_open_self",
    "process_vm_readv_zero_iov",
    "process_vm_readv_invalid_flags",
];

/// Verbatim from the ABI-controlled survey run on 2026-09-20. Do not retype
/// the syscall numbers or argument order: these exact values produced the
/// measurements asserted below.
const PROBE_SRC: &str = r#"
#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("This probe requires Linux x86_64 inside the container.");

use std::os::raw::{c_int, c_long};

extern "C" {
    fn syscall(number: c_long, ...) -> c_long;
    fn close(fd: c_int) -> c_int;
}

fn probe(name: &str, number: c_long, args: [c_long; 6]) -> c_long {
    let ret = unsafe {
        syscall(
            number,
            args[0], args[1], args[2],
            args[3], args[4], args[5],
        )
    };
    let errno = if ret == -1 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(-1)
    } else {
        0
    };
    println!("PROBE {name} ret={ret} errno={errno}");
    ret
}

fn main() {
    const BOGUS: c_long = 0x7fff_ffff;

    // Linux x86_64 syscall numbers. No real target process is attached.
    probe("ptrace_attach_bogus", 101, [16, BOGUS, 0, 0, 0, 0]);
    probe("pidfd_open_bogus", 434, [BOGUS, 0, 0, 0, 0, 0]);

    // Benign success candidate: obtain a descriptor for this process only.
    let fd = probe(
        "pidfd_open_self",
        434,
        [c_long::from(std::process::id()), 0, 0, 0, 0, 0],
    );
    if fd >= 0 {
        assert_eq!(unsafe { close(fd as c_int) }, 0, "close pidfd");
    }

    // Distinguish a zero-length request from deliberately invalid flags.
    probe("process_vm_readv_zero_iov", 310, [BOGUS, 0, 0, 0, 0, 0]);
    probe("process_vm_readv_invalid_flags", 310, [BOGUS, 0, 0, 0, 0, 1]);
}
"#;

/// Strict: `docker ps` must succeed AND report zero `ec-sbx-*` containers.
/// A failed command with empty stdout is not evidence of cleanliness.
fn assert_no_orphans(phase: &str) {
    let out = Command::new("docker")
        .args(["ps", "-aq", "--filter", "name=ec-sbx-"])
        .output()
        .expect("could not execute docker ps");
    assert!(
        out.status.success(),
        "[{phase}] docker ps failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let ids = String::from_utf8_lossy(&out.stdout);
    assert!(
        ids.trim().is_empty(),
        "[{phase}] orphan containers exist:\n{ids}\n\
         clean with: docker ps -aq --filter name=ec-sbx- | xargs -r docker rm -f"
    );
    println!("[{phase}] leftover=0");
}

/// Execution success is checked separately from the syscall results the
/// program prints: a broken toolchain must never look like containment.
fn run_source(label: &str, runner: &HardenedDockerRunner, source: &str) -> String {
    let out = runner.compile_and_run_hardened(source).unwrap_or_else(|e| {
        panic!(
            "[{label}] execution failed: {e:?}\n\
             cannot distinguish 'seccomp denied' from 'toolchain broken' — FAIL CLOSED."
        )
    });
    println!(
        "\n===== {label} =====\nexit_code={} elapsed={:?}\n{}\nstderr:\n{}",
        out.exit_code, out.elapsed, out.stdout, out.stderr
    );
    assert_eq!(
        out.exit_code, 0,
        "[{label}] program must finish successfully (syscalls fail, process does not)"
    );
    out.stdout
}

/// Exactly one well-formed observation per known case.
fn run_arm(label: &str, runner: &HardenedDockerRunner) -> HashMap<String, (i64, i32)> {
    let stdout = run_source(label, runner, PROBE_SRC);
    let mut samples: HashMap<String, (i64, i32)> = HashMap::new();

    for line in stdout.lines().filter(|line| line.starts_with("PROBE ")) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 4, "[{label}] malformed line: {line}");

        let name = fields[1];
        assert!(
            PROBE_NAMES.contains(&name),
            "[{label}] unknown probe: {name}"
        );
        let ret = fields[2]
            .strip_prefix("ret=")
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_else(|| panic!("[{label}] malformed return value: {line}"));
        let errno = fields[3]
            .strip_prefix("errno=")
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or_else(|| panic!("[{label}] malformed errno: {line}"));

        assert!(
            samples.insert(name.to_owned(), (ret, errno)).is_none(),
            "[{label}] duplicate probe: {name}"
        );
    }

    assert_eq!(
        samples.len(),
        PROBE_NAMES.len(),
        "[{label}] missing observations:\n{stdout}"
    );
    samples
}

/// Containment must be backed by a runtime marker emitted after the
/// `---OUTPUT---` separator; a Docker error or timeout is not proof that the
/// vector was exercised under the production profile.
fn assert_legacy_contained(result: EscapeTestResult, expected_marker: &str) {
    println!("\n[parity:{}]\n{}", result.vector, result.output);
    assert!(
        result.is_contained(),
        "[parity:{}] escape reported:\n{}",
        result.vector,
        result.output
    );

    let (_, runtime_output) = result.output.split_once("---OUTPUT---").unwrap_or_else(|| {
        panic!(
            "[parity:{}] no `---OUTPUT---` marker: the program never ran \
             (compile error / docker error / timeout). That is not containment:\n{}",
            result.vector, result.output
        )
    });

    assert!(
        runtime_output
            .lines()
            .any(|line| line.trim_start().starts_with(expected_marker)),
        "[parity:{}] missing runtime marker {expected_marker:?}:\n{}",
        result.vector,
        result.output
    );
}

/// One sequential test: no intra-binary parallelism may pollute the global
/// `ec-sbx-` orphan count (same pattern as `timeout_gate`).
#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests + a running docker daemon"
)]
fn custom_seccomp_enforcement_and_escape_parity() {
    assert_no_orphans("PRE");

    let baseline = HardenedDockerRunner::for_testing().expect("baseline runner");

    // NEGATIVE CONTROL: swap ONLY this line to `for_testing()` -> ARM B must fail.
    let production = HardenedDockerRunner::default_hardened().expect("production runner");

    let a = run_arm("A_daemon_default", &baseline);
    let b = run_arm("B_ec_custom", &production);

    // ── ARM A: exact baseline. Drift = investigate + update ADR-028, never relax.
    for name in ["ptrace_attach_bogus", "pidfd_open_bogus"] {
        assert_eq!(
            a[name],
            (-1, ESRCH),
            "[A] {name}: daemon default baseline changed; rerun the survey"
        );
    }
    let (fd, errno) = a["pidfd_open_self"];
    assert!(
        fd >= 0 && errno == 0,
        "[A] pidfd_open_self must succeed under daemon default: ret={fd} errno={errno}"
    );
    assert_eq!(
        a["process_vm_readv_zero_iov"],
        (0, 0),
        "[A] zero-length request must succeed under daemon default"
    );
    assert_eq!(
        a["process_vm_readv_invalid_flags"],
        (-1, EINVAL),
        "[A] deliberately invalid flags must return EINVAL under daemon default"
    );
    println!("[A_daemon_default] OK: 5/5 baseline observations match");

    // ── ARM B: every case denied by our profile.
    for name in PROBE_NAMES {
        assert_eq!(
            b[name],
            (-1, EPERM),
            "[B] {name}: expected EC seccomp denial (-1, EPERM).\n\
             Seeing ARM A values here means the custom profile was NOT applied \
             (broken `--security-opt seccomp=...` path) — this is the structural \
             negative control firing."
        );
    }
    println!("[B_ec_custom] OK: 5/5 denied (-1, EPERM)");

    // ── The profile must not break innocent code.
    let hello = run_source(
        "hello",
        &production,
        r#"fn main() { println!("hello-seccomp-parity"); }"#,
    );
    assert!(
        hello.lines().any(|l| l.trim() == "hello-seccomp-parity"),
        "[hello] missing runtime output:\n{hello}"
    );
    println!("[hello] OK: rustc works under the custom profile");

    // ── C4: the five legacy vectors, on the PRODUCTION runner.
    // Existing `for_testing()` tests elsewhere stay untouched; these complement.
    assert_legacy_contained(production.test_proc_escape(), "BLOCKED");
    assert_legacy_contained(production.test_dev_mem_escape(), "BLOCKED");
    assert_legacy_contained(production.test_ptrace_escape(), "BLOCKED");
    assert_legacy_contained(production.test_mount_escape(), "BLOCKED");
    assert_legacy_contained(
        production.test_fork_bomb_escape(),
        "CONTAINED: pids-limit stopped fork bomb at ",
    );
    println!("[parity] OK: 5/5 legacy vectors ran and reported containment on default_hardened()");

    assert_no_orphans("POST");
}
