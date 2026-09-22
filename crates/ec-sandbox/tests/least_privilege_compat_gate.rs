//! T1.3 — functional compatibility under the least-privilege seccomp profile.
//!
//! These tests use the production constructor `default_hardened()`.
//!
//! H1: filtered `clone` does not break standard threads or child processes.
//! H2: `clone3 -> ENOSYS(38)` permits the libc fallback to `clone`.

use ec_sandbox::hardened::HardenedDockerRunner;

fn run_source(label: &str, source: &str) -> String {
    let runner = HardenedDockerRunner::default_hardened()
        .unwrap_or_else(|e| panic!("[{label}] cannot create runner: {e:?}"));

    let output = runner
        .compile_and_run_hardened(source)
        .unwrap_or_else(|e| panic!("[{label}] Docker execution failed: {e:?}"));

    println!(
        "\n===== {label} =====\n\
         exit_code={}\n\
         elapsed={:?}\n\
         stdout:\n{}\n\
         stderr:\n{}",
        output.exit_code, output.elapsed, output.stdout, output.stderr
    );

    assert_eq!(
        output.exit_code, 0,
        "[{label}] expected exit code 0, got {}. stderr: {}",
        output.exit_code,
        output.stderr
    );

    output.stdout
}

#[test]
#[cfg_attr(not(feature = "docker_tests"), ignore)]
fn compat_hello_world() {
    let stdout = run_source(
        "hello_world",
        r#"
        fn main() {
            println!("T13_HELLO_OK");
        }
        "#,
    );

    assert!(
        stdout.contains("T13_HELLO_OK"),
        "hello marker missing from stdout: {stdout}"
    );
}

#[test]
#[cfg_attr(not(feature = "docker_tests"), ignore)]
fn compat_thread_spawn_join() {
    let stdout = run_source(
        "thread_spawn_join",
        r#"
        use std::thread;

        fn main() {
            let handle = thread::spawn(|| 40 + 2);
            let result = handle.join().expect("thread join failed");
            assert_eq!(result, 42);
            println!("T13_THREAD_OK");
        }
        "#,
    );

    assert!(
        stdout.contains("T13_THREAD_OK"),
        "thread marker missing from stdout: {stdout}"
    );
}

#[test]
#[cfg_attr(not(feature = "docker_tests"), ignore)]
fn compat_child_process() {
    let stdout = run_source(
        "child_process",
        r#"
        use std::process::Command;

        fn main() {
            let status = Command::new("/bin/true")
                .status()
                .expect("child process spawn failed");

            assert!(status.success());
            println!("T13_PROCESS_OK");
        }
        "#,
    );

    assert!(
        stdout.contains("T13_PROCESS_OK"),
        "process marker missing from stdout: {stdout}"
    );
}
