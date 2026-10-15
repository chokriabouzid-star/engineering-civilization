#![forbid(unsafe_code)]

//! بوابة مجال رموز الخروج في MeasuredEvidence — PR2.

use ec_epistemic::{ExecutionProvenance, MeasuredEvidence, OracleId};

const IMAGE: &str = "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523";

fn build(exit_codes: Vec<i32>) -> Result<MeasuredEvidence, ec_epistemic::EpistemicError> {
    MeasuredEvidence::new(
        ExecutionProvenance::Docker,
        "a".repeat(64),
        IMAGE.to_string(),
        exit_codes,
        0.99,
        OracleId::ExitCodeZero,
    )
}

#[test]
fn accepts_zero() {
    assert!(build(vec![0]).is_ok(), "0 is a valid program exit");
}

#[test]
fn accepts_255() {
    assert!(build(vec![255]).is_ok(), "255 is the upper bound");
}

#[test]
fn rejects_minus_one() {
    assert!(
        build(vec![-1]).is_err(),
        "-1 means the runner returned no status"
    );
}

#[test]
fn rejects_minus_two() {
    assert!(
        build(vec![-2]).is_err(),
        "no negative value is a program exit"
    );
}

#[test]
fn rejects_256() {
    assert!(build(vec![256]).is_err(), "256 is outside 0..=255");
}

#[test]
fn rejects_i32_max() {
    assert!(
        build(vec![i32::MAX]).is_err(),
        "i32::MAX is outside 0..=255"
    );
}

#[test]
fn deserialization_rejects_out_of_range() {
    let json = r#"{
        "provenance": "Docker",
        "artifact_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "sandbox_image": "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523",
        "exit_codes": [256],
        "reproducibility": 1.0,
        "oracle": "ExitCodeZero"
    }"#;
    let res: Result<MeasuredEvidence, _> = serde_json::from_str(json);
    assert!(
        res.is_err(),
        "deserialization must route through the constructor"
    );
}
