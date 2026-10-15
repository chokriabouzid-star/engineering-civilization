#![forbid(unsafe_code)]

//! بوابة ربط نتيجة التنفيذ بالتحكيم — PR2.

use ec_app::adjudication::{adjudicate, adjudicate_from_outcome, AdjudicationVerdict};
use ec_epistemic::{ExecutionProvenance, MeasuredEvidence, OracleId};
use ec_sandbox::execution_outcome::{ExecutionFailure, SandboxOutcome};
use sha2::{Digest, Sha256};

const IMAGE: &str = "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523";

const HEALTHY_CODE: &str = r#"/// A pure add function.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-2, 2), 0);
    }
}
"#;

fn sha256_hex(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

fn program_runs(exit_codes: Vec<i32>) -> SandboxOutcome {
    SandboxOutcome::ProgramRuns {
        exit_codes,
        reproducibility: 0.99,
        image: IMAGE.to_string(),
    }
}

#[test]
fn program_runs_exit_3_is_rejected_by_law() {
    let outcome = program_runs(vec![3, 3, 3]);
    let ex = adjudicate_from_outcome(HEALTHY_CODE, &outcome);

    assert!(
        matches!(&ex.outcome.verdict, AdjudicationVerdict::Rejected { .. }),
        "expected Rejected, got {:?}",
        ex.outcome.verdict
    );

    let evidence = ex.evidence.as_ref().expect("evidence must be bound");
    assert_eq!(evidence.exit_codes(), &[3, 3, 3]);
    assert_eq!(evidence.sandbox_image(), IMAGE);

    let eval = ex
        .outcome
        .evaluation
        .as_ref()
        .expect("evaluation must exist for a measured failure");
    assert!(
        eval.violations
            .iter()
            .any(|v| v.invariant_name == "MeasuredRealityInvariant"),
        "rejection must come from the measured-reality law, not a pre-rule"
    );

    if let AdjudicationVerdict::Rejected { primary, .. } = &ex.outcome.verdict {
        assert!(
            primary.contains('3'),
            "reason must carry the exit code: {}",
            primary
        );
    }
}

#[test]
fn program_runs_all_zero_is_accepted() {
    let outcome = program_runs(vec![0, 0, 0]);
    let ex = adjudicate_from_outcome(HEALTHY_CODE, &outcome);
    assert_eq!(&ex.outcome.verdict, &AdjudicationVerdict::Accepted);
    let evidence = ex.evidence.as_ref().expect("evidence must be bound");
    assert_eq!(evidence.exit_codes(), &[0, 0, 0]);
}

#[test]
fn program_runs_exit_137_is_rejected() {
    let outcome = program_runs(vec![137, 137, 137]);
    let ex = adjudicate_from_outcome(HEALTHY_CODE, &outcome);
    assert!(
        matches!(&ex.outcome.verdict, AdjudicationVerdict::Rejected { .. }),
        "an observed non-zero exit must be rejected, got {:?}",
        ex.outcome.verdict
    );
    assert!(
        ex.evidence.is_some(),
        "evidence must be bound for observed exit codes"
    );
}

#[test]
fn execution_failed_is_unverified() {
    let outcome = SandboxOutcome::ExecutionFailed {
        failure: ExecutionFailure::DockerUnavailable,
    };
    let ex = adjudicate_from_outcome(HEALTHY_CODE, &outcome);

    if let AdjudicationVerdict::Unverified { reason } = &ex.outcome.verdict {
        assert!(
            reason.contains("docker unavailable"),
            "reason must name the classified cause: {}",
            reason
        );
    } else {
        panic!("expected Unverified, got {:?}", ex.outcome.verdict);
    }

    assert!(ex.evidence.is_none(), "no evidence may be bound on failure");
    assert!(ex.outcome.evaluation.is_none(), "no evaluation on failure");
    assert!(ex.outcome.oracle.is_none(), "no oracle was consulted");
}

#[test]
fn control_hostile_input_survives_core_adjudication() {
    let hostile = "(".repeat(5000);
    let evidence = MeasuredEvidence::new(
        ExecutionProvenance::Docker,
        sha256_hex(&hostile),
        IMAGE.to_string(),
        vec![0, 0, 0],
        0.99,
        OracleId::ExitCodeZero,
    )
    .expect("valid evidence");

    let outcome = adjudicate(&hostile, Some(&evidence));
    assert!(
        !outcome.artifact_sha256.is_empty(),
        "hostile input must not abort the core"
    );
}

#[test]
fn control_pipeline_file_frozen() {
    const PIPELINE: &str = include_str!("../src/pipeline.rs");
    assert_eq!(
        PIPELINE.lines().count(),
        992,
        "pipeline.rs must remain untouched"
    );
    let hex = format!("{:x}", Sha256::digest(PIPELINE.as_bytes()));
    assert_eq!(
        hex, "d00fd0fd4523117e541cde191bcd90f86f0604e8f80af5a893c704d65b745a97",
        "pipeline.rs content must remain byte-identical"
    );
}
