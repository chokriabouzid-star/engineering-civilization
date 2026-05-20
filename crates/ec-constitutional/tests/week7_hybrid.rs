#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_constitutional::constitution::{Constitution, ObservedOutcome};
use ec_constitutional::evaluation::ConstitutionalEvaluation;
use ec_constitutional::policy::PolicySet;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};

// ─── Helpers ────────────────────────────────────────────────────────

fn dummy_constitution() -> Constitution {
    Constitution::new(vec![], CatastropheThresholds::default())
}

fn dummy_valid_eval() -> ConstitutionalEvaluation {
    ConstitutionalEvaluation {
        artifact_id: "valid".to_string(),
        fitness: FitnessVector::default(),
        epistemic: EpistemicState::new(
            0.9,
            Evidence::new(1, 1, 1.0, 1.0).unwrap(),
            UncertaintyDecomposition::new(0.0, 0.1, 0.0).unwrap(),
            CalibrationState::default(),
        )
        .unwrap(),
        violations: vec![],
        catastrophic: None,
        is_valid: true,
        explanation: "".to_string(),
    }
}

fn dummy_invalid_eval() -> ConstitutionalEvaluation {
    let mut eval = dummy_valid_eval();
    eval.is_valid = false;
    eval.artifact_id = "invalid".to_string();
    eval
}

// ─── TOML Policy Loading Tests ──────────────────────────────────────

#[test]
fn policy_set_loads_from_valid_toml() {
    let toml_str = r#"
        max_p95_latency_ms = 500
        min_auth_score = 0.8
        deployment_target = "production"
        enable_canary = true
    "#;
    let policy_set = PolicySet::from_toml_str(toml_str).unwrap();

    assert_eq!(policy_set.get_float("max_p95_latency_ms"), Some(500.0));
    assert_eq!(policy_set.get_float("min_auth_score"), Some(0.8));
    assert_eq!(policy_set.get_string("deployment_target"), Some("production"));
    assert!(matches!(
        policy_set.values.get("enable_canary"),
        Some(ec_constitutional::policy::PolicyValue::Boolean(true))
    ));
}

#[test]
fn policy_set_fails_on_invalid_toml() {
    let toml_str = "this is not toml {";
    assert!(PolicySet::from_toml_str(toml_str).is_err());
}

// ─── Constitution.learn() Tests ─────────────────────────────────────

#[test]
fn learn_correct_prediction_valid() {
    let constitution = dummy_constitution();
    let prediction = dummy_valid_eval(); // is_valid = true, confidence = 0.9
    let reality = ObservedOutcome {
        correctness: 1.0,
        reproducibility: 0.95,
    };

    let error = constitution.learn(&prediction, &reality);

    assert_eq!(error.validity_error, 0.0);
    assert!((error.confidence_gap - 0.05).abs() < 1e-9); // |0.9 - 0.95|
}

#[test]
fn learn_correct_prediction_invalid() {
    let constitution = dummy_constitution();
    let prediction = dummy_invalid_eval(); // is_valid = false
    let reality = ObservedOutcome {
        correctness: 0.0,
        reproducibility: 0.95,
    };

    let error = constitution.learn(&prediction, &reality);

    assert_eq!(error.validity_error, 0.0);
}

#[test]
fn learn_incorrect_prediction_false_positive() {
    // الدستور قال "صالح" لكن الواقع أثبت أنه "فاسد"
    let constitution = dummy_constitution();
    let prediction = dummy_valid_eval(); // is_valid = true
    let reality = ObservedOutcome {
        correctness: 0.0,
        reproducibility: 0.95,
    };

    let error = constitution.learn(&prediction, &reality);

    assert_eq!(error.validity_error, 1.0, "Should detect a false positive");
}

#[test]
fn learn_incorrect_prediction_false_negative() {
    // الدستور قال "فاسد" لكن الواقع أثبت أنه "صالح"
    let constitution = dummy_constitution();
    let prediction = dummy_invalid_eval(); // is_valid = false
    let reality = ObservedOutcome {
        correctness: 1.0,
        reproducibility: 0.95,
    };

    let error = constitution.learn(&prediction, &reality);

    assert_eq!(error.validity_error, 1.0, "Should detect a false negative");
}
