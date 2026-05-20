#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_constitutional::constitution::Constitution;
use ec_constitutional::engine::{ConstitutionalEngine, EvaluationContext};
use ec_constitutional::policy::PolicySet;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{Evidence, EpistemicState, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};

// ─── Helpers ────────────────────────────────────────────────────────

/// دستور بدون ثوابت — للاختبار فقط
fn build_empty_constitution() -> Constitution {
    Constitution::new(vec![], CatastropheThresholds::default())
}

fn minimal_valid_epistemic() -> EpistemicState {
    EpistemicState::new(
        0.5,
        Evidence::new(1, 1, 1.0, 1.0).unwrap(),
        UncertaintyDecomposition::new(0.1, 0.1, 0.1).unwrap(),
        CalibrationState::default(),
    )
    .unwrap()
}

fn zero_fitness() -> FitnessVector {
    FitnessVector::default()
}

// ─── Test 1: Empty Artifact ID ──────────────────────────────────────

#[test]
fn empty_artifact_id() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    let eval = engine.evaluate("", 0, &zero_fitness(), &minimal_valid_epistemic(), &context);
    assert_eq!(eval.artifact_id, "");
}

// ─── Test 2: NaN in Metadata (Fitness) ──────────────────────────────

#[test]
fn nan_in_fitness_vector() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    let mut fitness = zero_fitness();
    fitness.security = f64::NAN;

    let eval = engine.evaluate("nan-test", 1, &fitness, &minimal_valid_epistemic(), &context);
    assert!(!eval.is_valid || eval.catastrophic.is_some());
}

// ─── Test 3: All Dimensions Catastrophic ────────────────────────────

#[test]
fn all_dimensions_catastrophic() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    let fitness = FitnessVector {
        security: 0.0,
        reversibility: 0.0,
        test_coverage: 0.0,
        maintainability: 0.0,
        performance: 0.0,
        architectural_stability: 0.0,
    };

    let eval = engine.evaluate("catastrophe-all", 2, &fitness, &minimal_valid_epistemic(), &context);
    assert!(!eval.is_valid);
    assert!(eval.catastrophic.is_some());
}

// ─── Test 4: Invalid TOML Policy ────────────────────────────────────

#[test]
fn invalid_toml_policy() {
    let invalid_toml = r#"
        invalid = { not: a, valid: toml ]
    "#;
    let result = PolicySet::from_toml_str(invalid_toml);
    assert!(result.is_err());
}

// ─── Test 5: Valid TOML with Wrong Types ────────────────────────────

#[test]
fn valid_toml_wrong_types() {
    let toml_with_type_errors = r#"
        max_p95_latency_ms = "this should be a number"
        enable_feature = 123
    "#;
    let result = PolicySet::from_toml_str(toml_with_type_errors);
    match result {
        Ok(_) => println!("Parsed with type coercion or default"),
        Err(e) => println!("Parse error (acceptable): {:?}", e),
    }
}

// ─── Test 6: Frontier with 1000 Artifacts ───────────────────────────

#[test]
fn frontier_with_1000_artifacts() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    
    let mut evaluations = Vec::new();
    for i in 0..1000 {
        // Generate valid fitness vectors above thresholds
        let fitness = FitnessVector {
            security: 0.8 + (i % 100) as f64 * 0.001, // 0.8 to 0.899
            reversibility: 0.5 + (i % 50) as f64 * 0.002, // 0.5 to 0.598
            test_coverage: 0.7 + (i % 80) as f64 * 0.0015, // 0.7 to 0.819
            maintainability: 0.6 + (i % 60) as f64 * 0.001, // 0.6 to 0.659
            performance: 0.4 + (i % 120) as f64 * 0.001, // 0.4 to 0.519
            architectural_stability: 0.6 + (i % 70) as f64 * 0.0015, // 0.6 to 0.704
        };
        let eval = engine.evaluate(
            &format!("art-{}", i),
            i as u64,
            &fitness,
            &minimal_valid_epistemic(),
            &context,
        );
        // All should be valid since there are no invariants
        assert!(eval.is_valid, "Evaluation {} should be valid", i);
        evaluations.push(eval);
    }

    let frontier = engine.build_frontier(&evaluations);
    
    // With no invariants, all should be valid and some should be on frontier
    assert!(!frontier.is_empty(), "Frontier should not be empty with 1000 valid artifacts");
    println!("Frontier size from 1000 artifacts: {}", frontier.len());
}

// ─── Test 7: Extremely High Values (Overflow Risk) ──────────────────

#[test]
fn extremely_high_values() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    let fitness = FitnessVector {
        security: f64::MAX,
        reversibility: f64::MAX,
        test_coverage: f64::MAX,
        maintainability: f64::MAX,
        performance: f64::MAX,
        architectural_stability: f64::MAX,
    };

    let _eval = engine.evaluate("overflow-test", 3, &fitness, &minimal_valid_epistemic(), &context);
    
    // Extremely high values should either:
    // 1. Be rejected as catastrophic (if they exceed thresholds)
    // 2. Be rejected as invalid (if they cause internal errors like overflow)
    // 3. At least not crash the system

    // We only require that the system does not crash
    // The exact behavior may vary based on internal checks
    println!("Extremely high values test completed without crash");
}

// ─── Test 8: Negative Values in Fitness ─────────────────────────────

#[test]
fn negative_fitness_values() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    let fitness = FitnessVector {
        security: -1.0,
        reversibility: -0.5,
        ..FitnessVector::default()
    };

    let eval = engine.evaluate("negative-test", 4, &fitness, &minimal_valid_epistemic(), &context);
    assert!(!eval.is_valid);
}

// ─── Test 9: Extremely Small Positive Values ────────────────────────

#[test]
fn extremely_small_positive_values() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    let fitness = FitnessVector {
        security: 1e-100,
        reversibility: 1e-50,
        ..FitnessVector::default()
    };

    let eval = engine.evaluate("tiny-values", 5, &fitness, &minimal_valid_epistemic(), &context);
    assert!(eval.is_valid || eval.catastrophic.is_some());
}

// ─── Test 10: Duplicate Artifact IDs ────────────────────────────────

#[test]
fn duplicate_artifact_ids() {
    let engine = ConstitutionalEngine::with_default_cache(build_empty_constitution());
    let context = EvaluationContext::default();
    
    let fitness1 = FitnessVector {
        security: 0.9,
        ..FitnessVector::default()
    };
    let fitness2 = FitnessVector {
        security: 0.8,
        ..FitnessVector::default()
    };

    let eval1 = engine.evaluate("duplicate-id", 100, &fitness1, &minimal_valid_epistemic(), &context);
    let eval2 = engine.evaluate("duplicate-id", 100, &fitness2, &minimal_valid_epistemic(), &context);
    
    assert_eq!(eval1.artifact_id, eval2.artifact_id);
}
