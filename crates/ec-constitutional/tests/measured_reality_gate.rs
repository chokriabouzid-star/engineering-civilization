//! بوابة قانون الواقع المقاس — PR1 / المرحلة R.
//!
//! الأسماء عقد بين R وG: لا تُعاد صياغتها، ولا يتغيّر ترتيبها.

use ec_constitutional::{
    Invariant, MeasuredRealityInvariant, MIN_REPRODUCIBILITY_FOR_AUTHORITY, MIN_RUNS_FOR_AUTHORITY,
};
use ec_epistemic::{
    CalibrationState, EpistemicState, Evidence, ExecutionProvenance, MeasuredEvidence, OracleId,
    UncertaintyDecomposition,
};
use ec_fitness::FitnessVector;

/// الصورة المثبَّتة المعتمدة — نفس `ec_sandbox::docker::DEFAULT_IMAGE`.
const IMAGE: &str = "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523";

fn healthy_fitness() -> FitnessVector {
    FitnessVector {
        security: 0.95,
        reversibility: 0.95,
        test_coverage: 0.95,
        maintainability: 0.95,
        performance: 0.95,
        architectural_stability: 0.95,
    }
}

fn state_with(measured: Option<MeasuredEvidence>) -> EpistemicState {
    let mut state = EpistemicState::new(
        0.9,
        Evidence::new(1, 0, 0.95, 0.8).unwrap(),
        UncertaintyDecomposition::new(0.1, 0.2, 0.1).unwrap(),
        CalibrationState::default(),
    )
    .unwrap();
    state.measured = measured;
    state
}

fn evidence(
    provenance: ExecutionProvenance,
    exit_codes: Vec<i32>,
    reproducibility: f64,
) -> MeasuredEvidence {
    MeasuredEvidence::new(
        provenance,
        "a".repeat(64),
        IMAGE.to_string(),
        exit_codes,
        reproducibility,
        OracleId::ExitCodeZero,
    )
    .unwrap()
}

#[test]
fn no_evidence_is_err() {
    let inv = MeasuredRealityInvariant;
    let res = inv.check(&healthy_fitness(), &state_with(None));
    assert!(
        res.is_err(),
        "absence of measured evidence must be an error"
    );
    assert_eq!(
        res.unwrap_err().invariant_name,
        "MeasuredRealityInvariant",
        "the violation must be attributed to the measured-reality law"
    );
}

#[test]
fn non_docker_provenance_is_err() {
    let inv = MeasuredRealityInvariant;
    let ev = evidence(ExecutionProvenance::Simulated, vec![0, 0, 0], 0.99);
    let res = inv.check(&healthy_fitness(), &state_with(Some(ev)));
    assert!(
        res.is_err(),
        "Simulated provenance must confer no authority"
    );
    assert!(
        res.unwrap_err().reason.contains("Docker"),
        "reason must name the required provenance"
    );
}

#[test]
fn non_zero_exit_is_err() {
    let inv = MeasuredRealityInvariant;
    let ev = evidence(ExecutionProvenance::Docker, vec![3], 0.99);
    let res = inv.check(&healthy_fitness(), &state_with(Some(ev)));
    assert!(res.is_err(), "a measured non-zero exit must be rejected");
    let reason = res.unwrap_err().reason;
    assert!(
        reason.contains('3'),
        "reason must carry the exit code: {}",
        reason
    );
    assert!(
        reason.contains("1/1"),
        "reason must carry the run ratio: {}",
        reason
    );
}

#[test]
fn insufficient_runs_is_err() {
    let inv = MeasuredRealityInvariant;
    let ev = evidence(ExecutionProvenance::Docker, vec![0, 0], 0.99);
    let res = inv.check(&healthy_fitness(), &state_with(Some(ev)));
    assert!(res.is_err(), "two successful runs cannot confer authority");
    assert!(
        res.unwrap_err()
            .reason
            .contains(&MIN_RUNS_FOR_AUTHORITY.to_string()),
        "reason must carry the required run count"
    );
}

#[test]
fn low_reproducibility_is_err() {
    let inv = MeasuredRealityInvariant;
    let ev = evidence(ExecutionProvenance::Docker, vec![0, 0, 0], 0.90);
    let res = inv.check(&healthy_fitness(), &state_with(Some(ev)));
    assert!(
        res.is_err(),
        "reproducibility 0.90 is below the inherited bar"
    );
    assert!(
        res.unwrap_err()
            .reason
            .contains(&MIN_REPRODUCIBILITY_FOR_AUTHORITY.to_string()),
        "reason must carry the required reproducibility"
    );
}

#[test]
fn reproducibility_at_threshold_is_rejected() {
    // ت8: الحدّ بلا بوابة يُنسى. `is_reproducible` الموروثة حصرية (reality.rs:134)،
    // فـ0.95 بالضبط لا يمنح سلطة.
    let inv = MeasuredRealityInvariant;
    let ev = evidence(ExecutionProvenance::Docker, vec![0, 0, 0], 0.95);
    let res = inv.check(&healthy_fitness(), &state_with(Some(ev)));
    assert!(
        res.is_err(),
        "reproducibility exactly at the threshold must NOT confer authority \
         (inherited bar is exclusive: > 0.95)"
    );
}

// ─── الضوابط (تبقى خضراء في R) ──────────────────────────────────────────────

#[test]
fn valid_evidence_is_ok() {
    let inv = MeasuredRealityInvariant;
    let ev = evidence(ExecutionProvenance::Docker, vec![0, 0, 0], 0.99);
    let res = inv.check(&healthy_fitness(), &state_with(Some(ev)));
    assert!(res.is_ok(), "valid measured evidence must pass the law");
}

#[test]
fn control_deserialization_enforces_validation() {
    let bad = r#"{
        "provenance": "Docker",
        "artifact_sha256": "not-a-sha256",
        "sandbox_image": "rust:latest",
        "exit_codes": [0],
        "reproducibility": 1.0,
        "oracle": "ExitCodeZero"
    }"#;
    let res: Result<MeasuredEvidence, _> = serde_json::from_str(bad);
    assert!(
        res.is_err(),
        "deserialization must route through the constructor"
    );

    let bad_image_repo = r#"{
        "provenance": "Docker",
        "artifact_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sandbox_image": "@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "exit_codes": [0],
        "reproducibility": 1.0,
        "oracle": "ExitCodeZero"
    }"#;
    let res2: Result<MeasuredEvidence, _> = serde_json::from_str(bad_image_repo);
    assert!(res2.is_err(), "an empty image repository must be rejected");
}
