#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_constitutional::evaluation::ConstitutionalEvaluation;
use ec_constitutional::meta::{
    OssificationDetector, ReviewReason, ValueDriftDetector,
};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;

fn zero_fitness() -> FitnessVector {
    FitnessVector {
        security: 0.0,
        reversibility: 0.0,
        test_coverage: 0.0,
        maintainability: 0.0,
        performance: 0.0,
        architectural_stability: 0.0,
    }
}

// Helper to create a dummy evaluation
fn dummy_eval(id: &str, is_valid: bool) -> ConstitutionalEvaluation {
    ConstitutionalEvaluation {
        artifact_id: id.to_string(),
        fitness: zero_fitness(),
        epistemic: EpistemicState::new(
            0.5,
            ec_epistemic::state::Evidence {
                sample_size: 1,
                age_seconds: 1,
                reproducibility: 1.0,
                source_reliability: 1.0,
            },
            ec_epistemic::state::UncertaintyDecomposition {
                aleatoric: 0.0,
                epistemic: 0.0,
                model: 0.0,
            },
            ec_epistemic::calibration::CalibrationState::default(),
        )
        .unwrap(),
        violations: vec![],
        catastrophic: None,
        is_valid,
        explanation: String::new(),
    }
}

// ─── OssificationDetector Tests ─────────────────────────────────────

#[test]
fn ossification_no_evaluations() {
    let detector = OssificationDetector::new();
    assert_eq!(detector.rejection_rate(), 0.0);
    assert!(detector.needs_review().is_none());
}

#[test]
fn ossification_all_accepted() {
    let mut detector = OssificationDetector::new();
    for i in 0..100 {
        detector.record(&dummy_eval(&format!("id-{}", i), true));
    }
    assert_eq!(detector.rejection_rate(), 0.0);
    assert!(detector.needs_review().is_none());
}

#[test]
fn ossification_all_rejected() {
    let mut detector = OssificationDetector::new();
    for i in 0..100 {
        detector.record(&dummy_eval(&format!("id-{}", i), false));
    }
    assert_eq!(detector.rejection_rate(), 1.0);
    assert!(matches!(
        detector.needs_review(),
        Some(ReviewReason::HighRejectionRate(1.0))
    ));
}

#[test]
fn ossification_mixed_rate() {
    let mut detector = OssificationDetector::new();
    for i in 0..100 {
        let is_valid = i % 4 == 0; // 25% accepted, 75% rejected
        detector.record(&dummy_eval(&format!("id-{}", i), is_valid));
    }
    assert_eq!(detector.rejection_rate(), 0.75);
    assert!(detector.needs_review().is_none());
}

#[test]
fn ossification_triggers_above_threshold() {
    let mut detector = OssificationDetector::new();
    // 91 rejected, 9 accepted
    for i in 0..100 {
        detector.record(&dummy_eval(&format!("id-{}", i), i >= 91));
    }
    assert_eq!(detector.rejection_rate(), 0.91);
    assert!(detector.needs_review().is_some());
}

// ─── ValueDriftDetector Tests ───────────────────────────────────────

#[test]
fn drift_no_samples() {
    let baseline = FitnessVector {
        security: 1.0,
        ..zero_fitness()
    };
    let detector = ValueDriftDetector::new(baseline, 10);
    assert!(detector.drift_degrees().is_none());
    assert!(detector.needs_review().is_none());
}

#[test]
fn drift_no_drift_when_identical() {
    let baseline = FitnessVector {
        security: 0.8,
        performance: 0.2,
        ..zero_fitness()
    };
    let mut detector = ValueDriftDetector::new(baseline.clone(), 10);
    for _ in 0..10 {
        detector.record(&baseline);
    }
    let degrees = detector.drift_degrees().unwrap();
    assert!(degrees < 1e-6, "Expected no drift, got {} degrees", degrees);
    assert!(detector.needs_review().is_none());
}

#[test]
fn drift_90_degrees_simple() {
    let baseline = FitnessVector {
        security: 1.0,
        ..zero_fitness()
    };
    let current = FitnessVector {
        performance: 1.0,
        ..zero_fitness()
    };

    let mut detector = ValueDriftDetector::new(baseline, 10);
    for _ in 0..10 {
        detector.record(&current);
    }

    let degrees = detector.drift_degrees().unwrap();
    assert!((degrees - 90.0).abs() < 1e-6, "Expected 90 degrees, got {}", degrees);
    assert!(detector.needs_review().is_some());
}

#[test]
fn drift_45_degrees_simple() {
    let baseline = FitnessVector {
        security: 1.0,
        ..zero_fitness()
    };
    let current = FitnessVector {
        security: 1.0,
        performance: 1.0,
        ..zero_fitness()
    };

    let mut detector = ValueDriftDetector::new(baseline, 10);
    for _ in 0..10 {
        detector.record(&current);
    }

    let degrees = detector.drift_degrees().unwrap();
    assert!((degrees - 45.0).abs() < 1e-6, "Expected 45 degrees, got {}", degrees);
    assert!(detector.needs_review().is_some());
}

#[test]
fn drift_complex_vector_triggers_review() {
    let baseline = FitnessVector {
        security: 0.9,
        reversibility: 0.8,
        test_coverage: 0.7,
        maintainability: 0.6,
        performance: 0.1,
        architectural_stability: 0.5,
    };
    let current = FitnessVector {
        security: 0.2, // انخفاض حاد في الأمان
        reversibility: 0.8,
        test_coverage: 0.7,
        maintainability: 0.6,
        performance: 1.0, // ارتفاع حاد في الأداء
        architectural_stability: 0.5,
    };

    let mut detector = ValueDriftDetector::new(baseline, 10);
    for _ in 0..10 {
        detector.record(&current);
    }

    let degrees = detector.drift_degrees().unwrap();
    assert!(degrees > 30.0, "Expected >30 degrees drift, got {}", degrees);
    assert!(detector.needs_review().is_some());
}
