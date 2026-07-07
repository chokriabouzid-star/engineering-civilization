#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_constitutional::constitution::{Constitution, ObservedOutcome};
use ec_constitutional::engine::{ConstitutionalEngine, EvaluationContext};
use ec_constitutional::invariant::Invariant;
use ec_constitutional::meta::{OssificationDetector, ValueDriftDetector};
use ec_constitutional::policy::PolicySet;
use ec_constitutional::security::SecurityInvariant;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::decay::{DecayConfig, ExponentialHalfLifeDecay, TemporalDecay};
use ec_epistemic::propagation::{ConservativeCombiner, ConservativePropagation};
use ec_epistemic::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use std::sync::Arc;
use std::time::Duration;

// ─── Helpers ────────────────────────────────────────────────────────

fn build_test_constitution() -> Constitution {
    let invariants: Vec<Arc<dyn Invariant>> = vec![Arc::new(SecurityInvariant::default())];
    Constitution::new(invariants, CatastropheThresholds::default())
}

fn good_fitness() -> FitnessVector {
    FitnessVector {
        security: 0.85,
        reversibility: 0.75,
        test_coverage: 0.80,
        maintainability: 0.70,
        performance: 0.60,
        architectural_stability: 0.65,
    }
}

fn good_epistemic() -> EpistemicState {
    EpistemicState::new(
        0.9,
        Evidence::new(100, 3600, 0.95, 0.98).unwrap(),
        UncertaintyDecomposition::new(0.05, 0.03, 0.02).unwrap(),
        CalibrationState::default(),
    )
    .unwrap()
}

// ─── Scenario 1: EpistemicState → ConstitutionalEvaluation ─────────

#[test]
fn scenario_1_epistemic_flows_through_evaluation() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::new("Scenario 1");
    let fitness = good_fitness();
    let epistemic = good_epistemic();

    let eval = engine.evaluate("s1-artifact", 1, &fitness, &epistemic, &context);

    assert_eq!(eval.epistemic.confidence, epistemic.confidence);
    assert_eq!(
        eval.epistemic.evidence.sample_size,
        epistemic.evidence.sample_size
    );
}

// ─── Scenario 2: Constitution.learn() Updates Understanding ────────

#[test]
fn scenario_2_constitution_learns_from_reality() {
    let constitution = build_test_constitution();
    let fitness = good_fitness();
    let epistemic = good_epistemic();

    let prediction = constitution.evaluate("s2-artifact", &fitness, &epistemic);
    assert!(prediction.is_valid);

    // Reality matches prediction
    let reality_correct = ObservedOutcome {
        correctness: 1.0,
        reproducibility: 0.92,
    };
    let error_correct = constitution.learn(&prediction, &reality_correct);
    assert_eq!(error_correct.validity_error, 0.0);

    // Reality contradicts prediction
    let reality_wrong = ObservedOutcome {
        correctness: 0.0,
        reproducibility: 0.92,
    };
    let error_wrong = constitution.learn(&prediction, &reality_wrong);
    assert_eq!(error_wrong.validity_error, 1.0);
}

// ─── Scenario 3: Temporal Decay Affects Evaluation ─────────────────

#[test]
fn scenario_3_temporal_decay_reduces_confidence() {
    let epistemic = good_epistemic();
    let original_confidence = epistemic.confidence;

    let decayed = ExponentialHalfLifeDecay::decay(
        &epistemic,
        Duration::from_secs(60 * 24 * 60 * 60),
        DecayConfig::default(),
    )
    .unwrap();

    assert!(decayed.confidence < original_confidence);
}

// ─── Scenario 4: Conservative Propagation Across Multiple Evals ────

#[test]
fn scenario_4_conservative_propagation() {
    let epistemic1 = EpistemicState::new(
        0.9,
        Evidence::new(100, 1000, 0.95, 0.98).unwrap(),
        UncertaintyDecomposition::new(0.05, 0.03, 0.02).unwrap(),
        CalibrationState::default(),
    )
    .unwrap();

    let epistemic2 = EpistemicState::new(
        0.8,
        Evidence::new(50, 2000, 0.90, 0.95).unwrap(),
        UncertaintyDecomposition::new(0.10, 0.05, 0.03).unwrap(),
        CalibrationState::default(),
    )
    .unwrap();

    let combined =
        ConservativeCombiner::combine(&[epistemic1.clone(), epistemic2.clone()]).unwrap();

    assert!(combined.confidence <= epistemic1.confidence.min(epistemic2.confidence));
}

// ─── Scenario 5: OssificationDetector Catches High Rejection ───────

#[test]
fn scenario_5_ossification_detector() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();
    let mut detector = OssificationDetector::new();

    for i in 0..100 {
        let fitness = FitnessVector {
            security: 0.1,
            ..FitnessVector::default()
        };
        let epistemic = good_epistemic();
        let eval = engine.evaluate(&format!("ossif-{}", i), i, &fitness, &epistemic, &context);
        detector.record(&eval);
    }

    assert!(detector.rejection_rate() > 0.90);
    assert!(detector.needs_review().is_some());
}

// ─── Scenario 6: ValueDriftDetector Catches Priority Shift ─────────

#[test]
fn scenario_6_value_drift_detector() {
    let baseline = FitnessVector {
        security: 0.9,
        performance: 0.3,
        ..FitnessVector::default()
    };

    let mut detector = ValueDriftDetector::new(baseline, 10);

    let drifted = FitnessVector {
        security: 0.5,
        performance: 0.9,
        ..FitnessVector::default()
    };

    for _ in 0..10 {
        detector.record(&drifted);
    }

    let degrees = detector.drift_degrees().unwrap();
    assert!(
        degrees > 30.0,
        "Expected significant drift, got {} degrees",
        degrees
    );
    assert!(detector.needs_review().is_some());
}

// ─── Scenario 7: PolicySet Loads and Applies ───────────────────────

#[test]
fn scenario_7_policy_set_loads_from_toml() {
    let toml = r#"
        max_latency_ms = 500
        min_security_score = 0.8
        deployment_env = "production"
        enable_experimental = false
    "#;

    let policy = PolicySet::from_toml_str(toml).unwrap();

    assert_eq!(policy.get_float("max_latency_ms"), Some(500.0));
    assert_eq!(policy.get_float("min_security_score"), Some(0.8));
    assert_eq!(policy.get_string("deployment_env"), Some("production"));
}

// ─── Scenario 8: Cache + Frontier + Pareto Together ────────────────

#[test]
fn scenario_8_cache_frontier_pareto_integration() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();
    let epistemic = good_epistemic();

    let mut evaluations = Vec::new();

    for i in 0..20 {
        let fitness = FitnessVector {
            security: 0.7 + (i % 5) as f64 * 0.05,
            reversibility: 0.6 + (i % 4) as f64 * 0.05,
            test_coverage: 0.7 + (i % 6) as f64 * 0.03,
            maintainability: 0.5 + (i % 3) as f64 * 0.1,
            performance: 0.4 + (i % 7) as f64 * 0.02,
            architectural_stability: 0.6 + (i % 5) as f64 * 0.04,
        };
        let eval = engine.evaluate(&format!("combo-{}", i), i, &fitness, &epistemic, &context);
        evaluations.push(eval);
    }

    let frontier = engine.build_frontier(&evaluations);

    assert!(!frontier.is_empty());
    assert_eq!(engine.cache_len(), 20);
}

// ─── Scenario 9: Full Pipeline End-to-End ──────────────────────────

#[test]
fn scenario_9_full_pipeline_intent_to_learning() {
    let constitution = build_test_constitution();
    let engine = ConstitutionalEngine::with_default_cache(constitution.clone());

    let context = EvaluationContext::new("Build secure API with high coverage");

    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let prediction = engine.evaluate("pipeline-artifact", 999, &fitness, &epistemic, &context);

    assert!(prediction.is_valid);

    let reality = ObservedOutcome {
        correctness: 1.0,
        reproducibility: 0.93,
    };

    let error = constitution.learn(&prediction, &reality);

    assert_eq!(error.validity_error, 0.0);
}

// ─── Scenario 10: Multi-Artifact Comparison ────────────────────────

#[test]
fn scenario_10_multi_artifact_comparison() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();
    let epistemic = good_epistemic();

    let fitness_a = FitnessVector {
        security: 0.9,
        reversibility: 0.7,
        ..FitnessVector::default()
    };

    let fitness_b = FitnessVector {
        security: 0.8,
        reversibility: 0.9,
        ..FitnessVector::default()
    };

    let eval_a = engine.evaluate("artifact-a", 1, &fitness_a, &epistemic, &context);
    let eval_b = engine.evaluate("artifact-b", 2, &fitness_b, &epistemic, &context);

    let ordering = engine.compare(&eval_a, &eval_b);

    assert!(matches!(ordering, ec_fitness::ParetoOrdering::NonDominated));
}
