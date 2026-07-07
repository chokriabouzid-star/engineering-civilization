#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_constitutional::constitution::Constitution;
use ec_constitutional::engine::{ConstitutionalEngine, EvaluationContext};
use ec_constitutional::invariant::Invariant;
use ec_constitutional::security::SecurityInvariant;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use proptest::prelude::*;
use std::sync::Arc;

// ─── Helpers ────────────────────────────────────────────────────────

fn build_test_constitution() -> Constitution {
    let invariants: Vec<Arc<dyn Invariant>> = vec![Arc::new(SecurityInvariant::default())];
    Constitution::new(invariants, CatastropheThresholds::default())
}

// ─── Property Tests ─────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn fuzz_fitness_vector_never_panics(
        security in 0.0f64..1.0,
        reversibility in 0.0f64..1.0,
        test_coverage in 0.0f64..1.0,
        maintainability in 0.0f64..1.0,
        performance in 0.0f64..1.0,
        architectural_stability in 0.0f64..1.0,
    ) {
        let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
        let context = EvaluationContext::default();

        let fitness = FitnessVector {
            security,
            reversibility,
            test_coverage,
            maintainability,
            performance,
            architectural_stability,
        };

        let epistemic = EpistemicState::new(
            0.5,
            Evidence::new(1, 1, 1.0, 1.0).unwrap(),
            UncertaintyDecomposition::new(0.1, 0.1, 0.1).unwrap(),
            CalibrationState::default(),
        ).unwrap();

        // Should not panic
        let _eval = engine.evaluate("fuzz", 1, &fitness, &epistemic, &context);
    }

    #[test]
    fn fuzz_epistemic_confidence_never_panics(
        confidence in 0.0f64..1.0,
        sample_size in 1u64..1000,
        age_seconds in 0u64..86400,
    ) {
        let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
        let context = EvaluationContext::default();

        let fitness = FitnessVector {
            security: 0.8,
            reversibility: 0.7,
            test_coverage: 0.7,
            maintainability: 0.6,
            performance: 0.5,
            architectural_stability: 0.6,
        };

        let epistemic = EpistemicState::new(
            confidence,
            Evidence::new(sample_size, age_seconds, 0.9, 0.95).unwrap(),
            UncertaintyDecomposition::new(0.05, 0.05, 0.05).unwrap(),
            CalibrationState::default(),
        ).unwrap();

        // Should not panic
        let _eval = engine.evaluate("fuzz-epistemic", 2, &fitness, &epistemic, &context);
    }

    #[test]
    fn fuzz_artifact_id_never_panics(
        id_len in 0usize..1000,
    ) {
        let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
        let context = EvaluationContext::default();

        let id: String = (0..id_len).map(|_| 'x').collect();

        let fitness = FitnessVector {
            security: 0.8,
            reversibility: 0.7,
            test_coverage: 0.7,
            maintainability: 0.6,
            performance: 0.5,
            architectural_stability: 0.6,
        };

        let epistemic = EpistemicState::new(
            0.8,
            Evidence::new(100, 3600, 0.9, 0.95).unwrap(),
            UncertaintyDecomposition::new(0.05, 0.05, 0.05).unwrap(),
            CalibrationState::default(),
        ).unwrap();

        // Should not panic even with very long IDs
        let _eval = engine.evaluate(&id, 3, &fitness, &epistemic, &context);
    }
}

// ─── Edge Case Tests ────────────────────────────────────────────────

#[test]
fn validate_fitness_vector_catches_invalid() {
    let invalid_nan = FitnessVector {
        security: f64::NAN,
        ..FitnessVector::default()
    };
    assert!(invalid_nan.validate().is_err());

    let invalid_negative = FitnessVector {
        security: -0.1,
        ..FitnessVector::default()
    };
    assert!(invalid_negative.validate().is_err());

    let invalid_above_one = FitnessVector {
        security: 1.1,
        ..FitnessVector::default()
    };
    assert!(invalid_above_one.validate().is_err());

    let valid = FitnessVector {
        security: 0.8,
        reversibility: 0.7,
        test_coverage: 0.7,
        maintainability: 0.6,
        performance: 0.5,
        architectural_stability: 0.6,
    };
    assert!(valid.validate().is_ok());
}
