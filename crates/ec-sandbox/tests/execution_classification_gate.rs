#![forbid(unsafe_code)]

//! بوابة تصنيف نتائج التنفيذ — PR2.

use ec_sandbox::compiler::{CompilationResult, RunOutput};
use ec_sandbox::docker::DockerError;
use ec_sandbox::execution_outcome::{
    classify, measure_reproducibility_strict, ExecutionFailure, SandboxOutcome,
};
use std::time::Duration;

const IMAGE: &str = "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523";

fn run(exit_code: i32, stdout: &str, stderr: &str) -> RunOutput {
    RunOutput {
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
        exit_code,
        elapsed: Duration::from_millis(1),
    }
}

fn success(runs: Vec<RunOutput>) -> Result<CompilationResult, DockerError> {
    Ok(CompilationResult::Success { runs })
}

fn assert_program_runs(outcome: &SandboxOutcome, expected: &[i32], repro: f64) {
    match outcome {
        SandboxOutcome::ProgramRuns {
            exit_codes,
            reproducibility,
            image,
        } => {
            assert_eq!(exit_codes, expected, "exit codes mismatch");
            assert!(
                (reproducibility - repro).abs() < 1e-9,
                "reproducibility {} != {}",
                reproducibility,
                repro
            );
            assert_eq!(image, IMAGE, "image must be the one that ran");
        }
        other => panic!("expected ProgramRuns, got {:?}", other),
    }
}

fn assert_failure(outcome: &SandboxOutcome, expected: &ExecutionFailure) {
    match outcome {
        SandboxOutcome::ExecutionFailed { failure } => {
            assert_eq!(failure, expected);
        }
        other => panic!("expected ExecutionFailed, got {:?}", other),
    }
}

#[test]
fn ok_runs_classify_as_program_runs() {
    let out = classify(
        success(vec![
            run(0, "out", ""),
            run(0, "out", ""),
            run(0, "out", ""),
        ]),
        IMAGE.to_string(),
    );
    assert_program_runs(&out, &[0, 0, 0], 1.0);
}

#[test]
fn nonzero_exit_classifies_as_program_runs() {
    let out = classify(
        success(vec![
            run(3, "out", ""),
            run(3, "out", ""),
            run(3, "out", ""),
        ]),
        IMAGE.to_string(),
    );
    assert_program_runs(&out, &[3, 3, 3], 1.0);
}

#[test]
fn exit_101_classifies_as_program_runs() {
    let out = classify(
        success(vec![
            run(101, "", "observed"),
            run(101, "", "observed"),
            run(101, "", "observed"),
        ]),
        IMAGE.to_string(),
    );
    assert_program_runs(&out, &[101, 101, 101], 1.0);
}

#[test]
fn exit_137_classifies_as_program_runs() {
    let out = classify(
        success(vec![
            run(137, "", "observed"),
            run(137, "", "observed"),
            run(137, "", "observed"),
        ]),
        IMAGE.to_string(),
    );
    assert_program_runs(&out, &[137, 137, 137], 1.0);
}

#[test]
fn no_program_result_is_execution_failed() {
    let out = classify(
        Ok(CompilationResult::Failed {
            stderr: "error[E0308]".to_string(),
        }),
        IMAGE.to_string(),
    );
    assert_failure(&out, &ExecutionFailure::NoProgramResult);
}

#[test]
fn docker_unavailable_is_execution_failed() {
    let out = classify(
        Err(DockerError::NotAvailable("no daemon".to_string())),
        IMAGE.to_string(),
    );
    assert_failure(&out, &ExecutionFailure::DockerUnavailable);
}

#[test]
fn timeout_is_execution_failed() {
    let out = classify(
        Err(DockerError::Timeout { duration_secs: 60 }),
        IMAGE.to_string(),
    );
    assert_failure(
        &out,
        &ExecutionFailure::ContainerTimeout { duration_secs: 60 },
    );
}

#[test]
fn io_error_is_execution_failed() {
    let out = classify(
        Err(DockerError::Io(std::io::Error::other("boom"))),
        IMAGE.to_string(),
    );
    assert_failure(&out, &ExecutionFailure::RunnerIo);
}

#[test]
fn daemon_error_is_execution_failed() {
    let out = classify(
        Err(DockerError::DaemonError {
            exit_code: 125,
            stderr: "daemon refused".to_string(),
        }),
        IMAGE.to_string(),
    );
    assert_failure(&out, &ExecutionFailure::DaemonError);
}

#[test]
fn minus_one_is_execution_failed() {
    let out = classify(success(vec![run(-1, "", "")]), IMAGE.to_string());
    assert_failure(&out, &ExecutionFailure::InvalidExitCode { code: -1 });
}

#[test]
fn out_of_range_256_is_execution_failed() {
    let out = classify(success(vec![run(256, "", "")]), IMAGE.to_string());
    assert_failure(&out, &ExecutionFailure::InvalidExitCode { code: 256 });
}

#[test]
fn empty_runs_is_execution_failed() {
    let out = classify(success(vec![]), IMAGE.to_string());
    assert_failure(&out, &ExecutionFailure::EmptyRuns);
}

#[test]
fn strict_reproducibility_counts_stderr() {
    let runs = vec![run(0, "same", "err-a"), run(0, "same", "err-b")];
    assert!(
        (measure_reproducibility_strict(&runs) - 0.5).abs() < 1e-9,
        "stderr divergence must lower reproducibility"
    );
}

#[test]
fn strict_reproducibility_full_when_identical() {
    let runs = vec![
        run(0, "same", "err"),
        run(0, "same", "err"),
        run(0, "same", "err"),
    ];
    assert!((measure_reproducibility_strict(&runs) - 1.0).abs() < 1e-9);
}
