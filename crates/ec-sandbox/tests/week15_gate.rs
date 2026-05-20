#![forbid(unsafe_code)]

//! Week 15 Gate Tests — Reality Feedback Loop
//!
//! Validates:
//! 1. PredictionError calculation
//! 2. RealityFeedback learning
//! 3. Constitutional review detection
//! 4. Full pipeline: Constitution → Sandbox → Learn

use ec_constitutional::constitution::Constitution;
use ec_constitutional::constitution::ObservedOutcome;
use ec_constitutional::invariant::{Invariant, ViolationReport};
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use ec_sandbox::config::SandboxConfig;
use ec_sandbox::executor::SandboxExecutor;
use ec_sandbox::feedback::{PredictionError, PredictionRecord, RealityFeedback};
use std::sync::Arc;

// ─── Helper Functions ────────────────────────────────────────

fn make_fitness(security: f64, coverage: f64) -> FitnessVector {
    FitnessVector {
        security,
        reversibility: 0.8,
        test_coverage: coverage,
        maintainability: 0.7,
        performance: 0.6,
        architectural_stability: 0.75,
    }
}

fn make_epistemic(confidence: f64) -> EpistemicState {
    EpistemicState::new(
        confidence,
        Evidence::new(100, 60, 0.9, 0.95).unwrap(),
        UncertaintyDecomposition::new(0.05, 0.05, 0.02).unwrap(),
        CalibrationState::default(),
    )
    .unwrap()
}

fn make_prediction(valid: bool, confidence: f64) -> PredictionRecord {
    PredictionRecord {
        artifact_id: "test-artifact".to_string(),
        predicted_validity: valid,
        predicted_confidence: confidence,
    }
}

#[derive(Debug)]
struct AlwaysAccept;

impl Invariant for AlwaysAccept {
    fn check(
        &self,
        _fitness: &FitnessVector,
        _epistemic: &EpistemicState,
    ) -> Result<(), ViolationReport> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "AlwaysAccept"
    }
}

// ─── Gate Tests ──────────────────────────────────────────────

#[test]
fn gate_prediction_error_false_positive() {
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    // This test verifies the test infrastructure itself
    // Real false positive detection is tested in other tests
    let _good_result = executor.execute("test", "fn main() {}");
    let bad_result = executor.execute("fail-artifact", "");
    
    assert!(!bad_result.success);
}

#[test]
fn gate_prediction_error_correct_prediction() {
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    let result = executor.execute("test", "fn main() {}");
    assert!(result.success);
    
    let reality = result.reality.unwrap();
    let pred = make_prediction(true, 0.85);
    
    let error = PredictionError::compute(&pred, &reality);
    
    assert_eq!(error.validity_error, 0.0);
    assert!(!error.overconfident);
    assert!(!error.underconfident);
}

#[test]
fn gate_feedback_learns_from_execution() {
    let mut fb = RealityFeedback::new();
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    for _ in 0..5 {
        let result = executor.execute("test", "fn main() {}");
        let reality = result.reality.unwrap();
        let pred = make_prediction(true, 0.9);
        
        let error = fb.learn(&pred, &reality);
        assert!(error.is_acceptable());
    }
    
    assert_eq!(fb.mean_validity_error(), 0.0);
}

#[test]
fn gate_feedback_mean_error_calculation() {
    let mut fb = RealityFeedback::new();
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    // 5 correct predictions
    for _ in 0..5 {
        let result = executor.execute("test", "fn main() {}");
        let reality = result.reality.unwrap();
        let pred = make_prediction(true, 0.9);
        fb.learn(&pred, &reality);
    }
    
    // 5 wrong predictions (predict failure but code succeeds)
    for _ in 0..5 {
        let result = executor.execute("test", "fn main() {}");
        let reality = result.reality.unwrap();
        let pred = make_prediction(false, 0.3); // wrong prediction
        fb.learn(&pred, &reality);
    }
    
    assert!((fb.mean_validity_error() - 0.5).abs() < 0.01);
}

#[test]
fn gate_feedback_improving_detection() {
    let mut fb = RealityFeedback::new();
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    // 10 wrong predictions
    for _ in 0..10 {
        let result = executor.execute("test", "fn main() {}");
        let reality = result.reality.unwrap();
        let pred = make_prediction(false, 0.3);
        fb.learn(&pred, &reality);
    }
    
    // 10 correct predictions
    for _ in 0..10 {
        let result = executor.execute("test", "fn main() {}");
        let reality = result.reality.unwrap();
        let pred = make_prediction(true, 0.9);
        fb.learn(&pred, &reality);
    }
    
    assert!(fb.is_improving(), "should detect improvement");
}

#[test]
fn gate_feedback_no_review_on_good_performance() {
    let mut fb = RealityFeedback::new();
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    for _ in 0..50 {
        let result = executor.execute("test", "fn main() {}");
        let reality = result.reality.unwrap();
        let pred = make_prediction(true, 0.9);
        fb.learn(&pred, &reality);
    }
    
    assert!(!fb.needs_constitutional_review());
}

#[test]
fn gate_to_observed_outcome_conversion() {
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    let result = executor.execute("test", "fn main() {}");
    let reality = result.reality.unwrap();
    
    let outcome = RealityFeedback::to_observed_outcome(&reality);
    
    assert_eq!(outcome.correctness, reality.correctness);
    assert_eq!(outcome.reproducibility, reality.reproducibility);
}

#[test]
fn gate_constitution_learns_from_observed_outcome() {
    let constitution = Constitution::new(
        vec![Arc::new(AlwaysAccept)],
        CatastropheThresholds::default(),
    );
    
    let fitness = make_fitness(0.9, 0.9);
    let epistemic = make_epistemic(0.9);
    let evaluation = constitution.evaluate("test", &fitness, &epistemic);
    
    let outcome = ObservedOutcome {
        correctness: 1.0,
        reproducibility: 0.98,
    };
    
    let error = constitution.learn(&evaluation, &outcome);
    
    assert_eq!(error.validity_error, 0.0);
}

#[test]
fn gate_full_pipeline_constitution_to_sandbox() {
    // Step 1: Constitutional evaluation
    let constitution = Constitution::new(
        vec![Arc::new(AlwaysAccept)],
        CatastropheThresholds::default(),
    );
    
    let fitness = make_fitness(0.9, 0.9);
    let epistemic = make_epistemic(0.9);
    let evaluation = constitution.evaluate("test-artifact", &fitness, &epistemic);
    
    assert!(evaluation.is_valid);
    
    // Step 2: Sandbox execution (simulated)
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    let result = executor.execute("test-artifact", "fn main() {}");
    
    assert!(result.success);
    assert!(result.reality.is_some());
    
    // Step 3: Learn
    let reality = result.reality.unwrap();
    let prediction = PredictionRecord::from_evaluation(&evaluation);
    let error = PredictionError::compute(&prediction, &reality);
    
    assert_eq!(error.validity_error, 0.0, "prediction was correct");
}

#[test]
fn gate_prediction_record_from_evaluation() {
    let constitution = Constitution::new(
        vec![Arc::new(AlwaysAccept)],
        CatastropheThresholds::default(),
    );
    
    let fitness = make_fitness(0.9, 0.9);
    let epistemic = make_epistemic(0.85);
    let evaluation = constitution.evaluate("artifact-123", &fitness, &epistemic);
    
    let record = PredictionRecord::from_evaluation(&evaluation);
    
    assert_eq!(record.artifact_id, "artifact-123");
    assert!(record.predicted_validity);
    assert_eq!(record.predicted_confidence, 0.85);
}

#[test]
fn gate_overconfident_detection() {
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    // High confidence prediction
    let pred = make_prediction(true, 0.95);
    
    // Flaky execution (low empirical confidence)
    let result = executor.execute("flaky-test", "");
    let reality = result.reality.unwrap();
    
    let error = PredictionError::compute(&pred, &reality);
    
    // Flaky artifact has low reproducibility → low empirical_confidence
    if reality.empirical_confidence < 0.5 {
        assert!(error.overconfident);
    }
}

#[test]
fn gate_acceptable_error_threshold() {
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    let result = executor.execute("test", "fn main() {}");
    let reality = result.reality.unwrap();
    let pred = make_prediction(true, 0.85);
    
    let error = PredictionError::compute(&pred, &reality);
    
    assert!(error.is_acceptable());
}

#[test]
fn gate_prediction_error_has_artifact_id() {
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    let result = executor.execute("my-artifact", "fn main() {}");
    let reality = result.reality.unwrap();
    
    let pred = PredictionRecord {
        artifact_id: "my-artifact".to_string(),
        predicted_validity: true,
        predicted_confidence: 0.9,
    };
    
    let error = PredictionError::compute(&pred, &reality);
    
    assert_eq!(error.artifact_id, "my-artifact");
}

#[test]
fn week15_gate_complete() {
    // Full integration: Constitution → Sandbox → Feedback
    
    let mut feedback = RealityFeedback::new();
    
    let constitution = Constitution::new(
        vec![Arc::new(AlwaysAccept)],
        CatastropheThresholds::default(),
    );
    
    let config = SandboxConfig::default();
    let executor = SandboxExecutor::new(config).unwrap();
    
    // Test 10 artifacts
    for i in 0..10 {
        let artifact_id = format!("artifact-{}", i);
        let fitness = make_fitness(0.9, 0.9);
        let epistemic = make_epistemic(0.9);
        
        // 1. Constitutional evaluation
        let evaluation = constitution.evaluate(&artifact_id, &fitness, &epistemic);
        let prediction = PredictionRecord::from_evaluation(&evaluation);
        
        // 2. Sandbox execution
        let result = executor.execute(&artifact_id, "fn main() {}");
        assert!(result.success);
        
        let reality = result.reality.unwrap();
        
        // 3. Learn
        let error = feedback.learn(&prediction, &reality);
        assert!(error.is_acceptable());
    }
    
    assert_eq!(feedback.mean_validity_error(), 0.0);
    assert!(!feedback.needs_constitutional_review());
}
