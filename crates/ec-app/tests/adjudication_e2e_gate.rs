//! بوابة E2E لمسار التحكيم عبر Docker — ADR-033 الخطوة 3 (PR3).
//!
//! العينات مقاسة مسبقًا عبر `ec adjudge` الحقيقي (D0.1/D0.2).
//! لا يُعدّل هذا الملف في مرحلة G؛ يُضاف CHANGELOG فقط.

#![forbid(unsafe_code)]

use ec_app::adjudication::{adjudicate_executed, AdjudicationVerdict};
use sha2::{Digest, Sha256};
use std::process::Command;

fn sha256_hex(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

/// `docker ps` must succeed AND report zero `ec-sbx-*` containers.
fn assert_no_leftover_sbx(phase: &str) {
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
        "[{phase}] leftover ec-sbx-* containers:\n{ids}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn adjudication_e2e_docker_path() {
    // C1 — measured Accepted (D0.1): sha256 cec69987859bfd800f4ea510633c47c5aa7f227436fa0aba239c91207910993c
    let c1 = r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let _ = add(2, 2);
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

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_add_large() {
        assert_eq!(add(100, 200), 300);
    }
}
"#;

    // C2 — same as C1 except exit(3); measured Rejected (D0.2)
    // sha256 7fe9940b079a16fb9fd64e26287a80b658287e1052157f2ad97e477b9b94425f
    let c2 = r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let _ = add(2, 2);
    std::process::exit(3);
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

    #[test]
    fn test_add_zero() {
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_add_large() {
        assert_eq!(add(100, 200), 300);
    }
}
"#;

    // C3 — uncompilable; measured Unverified (D0.2)
    // sha256 6c2bac414f34abf7b5069664dda86f447c0b3db50d48a978481c54942df479b7
    let c3 = r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let x: i32 = "not an int";
    let _ = (x, add(1, 2));
}
"#;

    assert_eq!(
        sha256_hex(c1),
        "cec69987859bfd800f4ea510633c47c5aa7f227436fa0aba239c91207910993c",
        "C1 fixture bytes must match D0.1 measurement"
    );
    assert_eq!(
        sha256_hex(c2),
        "7fe9940b079a16fb9fd64e26287a80b658287e1052157f2ad97e477b9b94425f",
        "C2 fixture bytes must match D0.2 measurement"
    );
    assert_eq!(
        sha256_hex(c3),
        "6c2bac414f34abf7b5069664dda86f447c0b3db50d48a978481c54942df479b7",
        "C3 fixture bytes must match D0.2 measurement"
    );

    // Run all three BEFORE verdict assertions so a mutation cannot skip cleanup.
    assert_no_leftover_sbx("before_c1");
    let r1 = adjudicate_executed(c1);
    assert_no_leftover_sbx("after_c1");

    assert_no_leftover_sbx("before_c2");
    let r2 = adjudicate_executed(c2);
    assert_no_leftover_sbx("after_c2");

    assert_no_leftover_sbx("before_c3");
    let r3 = adjudicate_executed(c3);
    assert_no_leftover_sbx("after_c3");

    // ----- C1: Accepted -----
    assert_eq!(
        &r1.outcome.verdict,
        &AdjudicationVerdict::Accepted,
        "C1: expected Accepted, got {:?}",
        r1.outcome.verdict
    );
    let e1 = r1.evidence.as_ref().expect("C1: evidence must be bound");
    assert_eq!(e1.exit_codes(), &[0, 0, 0]);
    assert!((e1.reproducibility() - 1.0).abs() < 1e-9);
    assert_eq!(e1.sandbox_image(), ec_sandbox::docker::DEFAULT_IMAGE);
    assert_eq!(e1.artifact_sha256(), sha256_hex(c1));
    assert_eq!(r1.outcome.artifact_sha256, sha256_hex(c1));
    assert!(r1.outcome.evaluation.is_some(), "C1: evaluation must exist");
    assert!(r1.outcome.oracle.is_some(), "C1: oracle was consulted");

    // ----- C2: Rejected by MeasuredRealityInvariant -----
    match &r2.outcome.verdict {
        AdjudicationVerdict::Rejected { primary, .. } => {
            assert!(
                primary.contains("MeasuredRealityInvariant"),
                "C2 primary must name the law, got: {primary}"
            );
            assert!(
                primary.contains("[3, 3, 3]") && primary.contains("3/3"),
                "C2 primary must carry exit codes and run count, got: {primary}"
            );
        }
        other => panic!("C2: expected Rejected, got {other:?}"),
    }
    let e2 = r2
        .evidence
        .as_ref()
        .expect("C2: evidence must be bound on measured rejection");
    assert_eq!(e2.exit_codes(), &[3, 3, 3]);
    assert!((e2.reproducibility() - 1.0).abs() < 1e-9);
    assert_eq!(e2.sandbox_image(), ec_sandbox::docker::DEFAULT_IMAGE);
    assert_eq!(e2.artifact_sha256(), sha256_hex(c2));
    let eval2 = r2
        .outcome
        .evaluation
        .as_ref()
        .expect("C2: evaluation must exist for measured rejection");
    assert!(
        eval2
            .violations
            .iter()
            .any(|v| v.invariant_name == "MeasuredRealityInvariant"),
        "C2: rejection must come from the measured-reality law"
    );

    // ----- C3: Unverified, no evidence -----
    match &r3.outcome.verdict {
        AdjudicationVerdict::Unverified { reason } => {
            assert!(
                reason.contains("no program result was observed"),
                "C3 reason mismatch: {reason}"
            );
        }
        other => panic!("C3: expected Unverified, got {other:?}"),
    }
    assert!(r3.evidence.is_none(), "C3: no evidence on ExecutionFailed");
    assert!(r3.outcome.evaluation.is_none(), "C3: no evaluation");
    assert!(r3.outcome.oracle.is_none(), "C3: no oracle");
    assert_eq!(r3.outcome.artifact_sha256, sha256_hex(c3));

    assert_no_leftover_sbx("final");
}
