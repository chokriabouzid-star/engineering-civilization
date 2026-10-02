#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_sandbox::docker::DockerOutput;
use ec_sandbox::executor::{ExecutionResult, SandboxExecutor};
use ec_sandbox::hardened::HardenedDockerRunner;
use ec_sandbox::{SandboxConfig, SandboxMode};
use std::time::Duration;

const OUTPUT_MARKER: &str = "---OUTPUT---";

const OOM_SOURCE: &str = r#"
fn main() {
    let mut values: Vec<u8> = Vec::new();
    let chunk = 32 * 1024 * 1024;

    for i in 0..64 {
        let next_len = values.len() + chunk;
        values.resize(next_len, 7u8);
        println!("chunk {} len {}", i, values.len());
    }

    println!("SBX_OOM_SURVIVED {}", values.len());
}
"#;

fn run_raw(source: &str) -> DockerOutput {
    HardenedDockerRunner::default_hardened()
        .expect("create default hardened runner with seccomp enabled")
        .compile_and_run_hardened(source)
        .expect("run source through HardenedDockerRunner")
}

fn run_executor(source: &str) -> ExecutionResult {
    let mut config = SandboxConfig::new(SandboxMode::Docker);
    config.runs_for_reproducibility = 1;
    config.limits.max_memory_mb = 512;
    config.limits.max_execution_time = Duration::from_secs(30);

    SandboxExecutor::new(config)
        .expect("create Docker SandboxExecutor")
        .execute("sandbox-truth-gate", source)
}

fn assert_runtime_failure_with_low_correctness(label: &str, result: &ExecutionResult) {
    assert!(
        !result.success,
        "{label}: non-zero program exit was reported as success: {result:?}"
    );

    let correctness = result.reality.as_ref().map(|reality| reality.correctness);
    assert!(
        correctness.is_some_and(|value| value < 1.0),
        "{label}: expected a RealityVector with correctness < 1.0, got {correctness:?}; result={result:?}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn raw_marker_is_emitted_before_program_output() {
    let out = run_raw(r#"fn main() { println!("SBX_RAW_AFTER_MARKER"); }"#);

    assert_eq!(out.exit_code, 0, "raw result: {out:?}");
    let marker_position = out
        .stdout
        .find(OUTPUT_MARKER)
        .expect("runner output marker must be present");
    let program_position = out
        .stdout
        .find("SBX_RAW_AFTER_MARKER")
        .expect("program output must be present");

    assert!(
        marker_position < program_position,
        "runner marker must precede program output: {:?}",
        out.stdout
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn raw_panic_yields_exit_101() {
    let out = run_raw(r#"fn main() { panic!("SBX_RAW_PANIC"); }"#);

    assert_eq!(out.exit_code, 101, "raw result: {out:?}");
    assert!(
        out.stdout.contains(OUTPUT_MARKER),
        "runner marker must precede the panicking program: {out:?}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn raw_explicit_exit_code_is_preserved() {
    let out = run_raw(r#"fn main() { std::process::exit(3); }"#);

    assert_eq!(out.exit_code, 3, "raw result: {out:?}");
    assert!(
        out.stdout.contains(OUTPUT_MARKER),
        "runner marker must be present after successful compilation: {out:?}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn raw_plain_compile_failure_has_no_marker() {
    let out = run_raw(
        r#"
fn main() {
    let value: i32 = "not an int";
    println!("{}", value);
}
"#,
    );

    assert_ne!(
        out.exit_code, 0,
        "compile failure returned success: {out:?}"
    );
    let combined = format!("{}{}", out.stdout, out.stderr);
    assert!(
        !combined.contains(OUTPUT_MARKER),
        "plain compile failure unexpectedly contains the marker: {combined}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn raw_compile_failure_can_echo_marker_from_source() {
    let out = run_raw(
        r#"
fn main() {
    let value: i32 = "---OUTPUT---";
    println!("{}", value);
}
"#,
    );

    assert_ne!(
        out.exit_code, 0,
        "compile failure returned success: {out:?}"
    );
    let combined = format!("{}{}", out.stdout, out.stderr);
    assert!(
        combined.contains(OUTPUT_MARKER),
        "compiler diagnostics did not echo the marker from the source: {combined}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn raw_oom_is_killed_with_nonzero_exit() {
    let out = run_raw(OOM_SOURCE);

    assert_ne!(
        out.exit_code, 0,
        "OOM program unexpectedly succeeded: {out:?}"
    );
    assert!(
        out.stdout.contains(OUTPUT_MARKER),
        "the marker should show compilation completed before the OOM: {out:?}"
    );
    assert!(
        !out.stdout.contains("SBX_OOM_SURVIVED"),
        "the OOM probe unexpectedly reached its final statement: {out:?}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn executor_must_not_report_panic_as_success() {
    let result = run_executor(r#"fn main() { panic!("SBX_EXECUTOR_PANIC"); }"#);
    assert_runtime_failure_with_low_correctness("panic", &result);
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn executor_must_not_report_exit_3_as_success() {
    let result = run_executor(r#"fn main() { std::process::exit(3); }"#);
    assert_runtime_failure_with_low_correctness("exit(3)", &result);
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn executor_must_not_report_marker_compile_failure_as_success() {
    let result = run_executor(
        r#"
fn main() {
    let value: i32 = "---OUTPUT---";
    println!("{}", value);
}
"#,
    );

    assert!(
        !result.success,
        "uncompilable source containing the marker was reported as successful: {result:?}"
    );

    let error_message = result
        .error_message
        .as_deref()
        .unwrap_or("<no error message>");
    assert!(
        error_message.contains("Compilation failed"),
        "expected a compilation-failure message, got {error_message:?}; result={result:?}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn executor_must_not_report_oom_as_success() {
    let result = run_executor(OOM_SOURCE);
    assert_runtime_failure_with_low_correctness("OOM", &result);
}
