#![deny(warnings)]
#![forbid(unsafe_code)]

//! Week 17 Integration Tests (updated for Week 19)
//!
//! Tests the full pipeline:
//! Code → Sandbox → Reality → Constitution → Decision

use ec_app::pipeline::{IntegrationPipeline, PipelineVerdict};
use ec_constitutional::constitution::Constitution;
use ec_constitutional::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use std::sync::Arc;

// ─── Helpers ─────────────────────────────────────────────────────────

#[derive(Debug)]
struct AlwaysAccept;
impl Invariant for AlwaysAccept {
    fn name(&self) -> &'static str { "AlwaysAccept" }
    fn check(&self, _: &FitnessVector, _: &EpistemicState)
        -> Result<(), ViolationReport> { Ok(()) }
}

#[derive(Debug)]
struct AlwaysReject;
impl Invariant for AlwaysReject {
    fn name(&self) -> &'static str { "AlwaysReject" }
    fn check(&self, _: &FitnessVector, _: &EpistemicState)
        -> Result<(), ViolationReport> {
        Err(ViolationReport::new(
            "AlwaysReject", "Always rejected", 0.0, 1.0, true,
        ))
    }
}

fn permissive_constitution() -> Constitution {
    Constitution::new(
        vec![Arc::new(AlwaysAccept)],
        CatastropheThresholds {
            min_security: 0.0,
            min_reversibility: 0.0,
            min_test_coverage: 0.0,
            min_maintainability: 0.0,
            min_performance: 0.0,
            min_architectural_stability: 0.0,
        },
    )
}

fn strict_constitution() -> Constitution {
    Constitution::new(
        vec![Arc::new(AlwaysReject)],
        CatastropheThresholds::default(),
    )
}

fn simulated_pipeline() -> IntegrationPipeline {
    IntegrationPipeline::new_simulated(
        permissive_constitution(),
        CatastropheThresholds::default(),
    )
    .unwrap()
}

// ─── Test 1: Pipeline يعمل ───────────────────────────────────────────

#[test]
fn pipeline_creates_successfully() {
    assert!(IntegrationPipeline::new_simulated(
        permissive_constitution(),
        CatastropheThresholds::default(),
    )
    .is_ok());
}

// ─── Test 2: Simulated mode ──────────────────────────────────────────

#[test]
fn pipeline_simulated_normal_code_accepted() {
    let mut p = simulated_pipeline();
    let result = p.run("test-artifact", "fn main() {}");

    assert!(result.is_accepted(), "verdict: {:?}", result.verdict);
    assert!(result.execution.success);
    assert!(result.execution.reality.is_some());
}

#[test]
fn pipeline_simulated_fail_code_rejected() {
    let mut p = simulated_pipeline();
    let result = p.run("fail-artifact", "");

    assert!(!result.is_accepted());
    assert!(matches!(
        result.verdict,
        PipelineVerdict::RejectedByReality { .. }
    ));
}

#[test]
fn pipeline_constitutional_rejection() {
    let mut p = IntegrationPipeline::new_simulated(
        strict_constitution(),
        CatastropheThresholds::default(),
    )
    .unwrap();

    let result = p.run("test", "fn main() {}");

    assert!(!result.is_accepted());
    assert!(matches!(
        result.verdict,
        PipelineVerdict::RejectedByConstitution { .. }
    ));
}

// ─── Test 3: PipelineResult ──────────────────────────────────────────

#[test]
fn pipeline_result_has_unique_run_id() {
    let mut p = simulated_pipeline();
    let r1 = p.run("a", "fn main() {}");
    let r2 = p.run("b", "fn main() {}");

    assert_ne!(r1.run_id, r2.run_id);
}

#[test]
fn pipeline_result_contains_code() {
    let mut p = simulated_pipeline();
    let code = "fn main() { println!(\"hello\"); }";
    let result = p.run("test", code);

    assert_eq!(result.code, code);
}

#[test]
fn pipeline_result_summary_is_informative() {
    let mut p = simulated_pipeline();
    let result = p.run("test", "fn main() {}");

    let summary = result.summary();
    assert!(!summary.is_empty());
    assert!(summary.contains("Pipeline"));
}

// ─── Test 4: Prediction Error ────────────────────────────────────────

#[test]
fn pipeline_prediction_error_calculated() {
    let mut p = simulated_pipeline();
    let result = p.run("test", "fn main() {}");

    assert!((0.0..=1.0).contains(&result.prediction_error.validity_error));
    assert!((0.0..=1.0).contains(&result.prediction_error.confidence_gap));
}

#[test]
fn pipeline_correct_prediction_has_zero_error() {
    let mut p = simulated_pipeline();
    let result = p.run("test", "fn main() {}");

    assert_eq!(
        result.prediction_error.validity_error, 0.0,
        "Correct prediction should have zero validity error"
    );
}

// ─── Test 5: Feedback Learning ───────────────────────────────────────

#[test]
fn pipeline_feedback_improves_over_runs() {
    let mut p = simulated_pipeline();

    for i in 0..20 {
        p.run(&format!("artifact-{}", i), "fn main() {}");
    }

    assert!(p.is_improving());
    assert!(!p.needs_review());
}

#[test]
fn pipeline_mean_error_low_for_good_predictions() {
    let mut p = simulated_pipeline();

    for i in 0..10 {
        p.run(&format!("artifact-{}", i), "fn main() {}");
    }

    assert!(p.mean_validity_error() < 0.1);
}

// ─── Test 6: Multiple Artifacts ──────────────────────────────────────

#[test]
fn pipeline_handles_multiple_artifacts() {
    let mut p = simulated_pipeline();
    let mut accepted = 0;
    let mut rejected = 0;

    for i in 0..10 {
        let artifact_id = if i % 3 == 0 {
            format!("fail-{}", i)
        } else {
            format!("good-{}", i)
        };
        let result = p.run(&artifact_id, "fn main() {}");

        if result.is_accepted() {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }

    assert!(accepted > 0, "some should be accepted");
    assert!(rejected > 0, "some should be rejected (fail-*)");
}

// ─── Test 7: Fitness from Code (Week 19) ─────────────────────────────

#[test]
fn fitness_comes_from_code_not_reality() {
    let mut p = simulated_pipeline();
    let result = p.run("test", "fn main() {}");

    // FitnessVector يُقاس من الكود، لا من RealityVector
    assert!(result.evaluation.fitness.security > 0.0);
    // sandbox code without tests → coverage = 0.0
    assert_eq!(result.evaluation.fitness.test_coverage, 0.0);
}

#[test]
fn unsafe_code_gets_low_security_fitness() {
    let mut p = simulated_pipeline();
    let result = p.run("unsafe-code", "fn main() { unsafe { } }");

    // unsafe code should have security < 1.0
    assert!(result.evaluation.fitness.security < 1.0);
}

// ─── Test 8: EpistemicState from Reality ─────────────────────────────

#[test]
fn epistemic_built_from_reality() {
    use ec_app::pipeline::build_epistemic_from_reality;
    use ec_sandbox::reality::RealityVector;

    let reality = RealityVector::test_fixture(1.0, 0.98, 0.95, 3);
    let epistemic = build_epistemic_from_reality(&reality);

    assert!(epistemic.is_ok());
    let ep = epistemic.unwrap();
    assert!(ep.confidence > 0.5, "confidence: {}", ep.confidence);
}

// ─── Test 9: Security Violation Handling ─────────────────────────────

#[test]
fn pipeline_handles_escape_attempt() {
    let mut p = simulated_pipeline();
    let result = p.run("escape-artifact", "fn main() {}");

    if !result.execution.is_secure() {
        assert!(matches!(
            result.verdict,
            PipelineVerdict::ExecutionFailed { .. }
        ));
    }
}

// ─── Test 10: Sandbox Mode ───────────────────────────────────────────

#[test]
fn pipeline_reports_sandbox_mode() {
    use ec_sandbox::config::SandboxMode;
    let p = simulated_pipeline();
    assert!(matches!(p.sandbox_mode(), SandboxMode::Simulated));
}

// ─── Week 17 Final Gate ──────────────────────────────────────────────

#[test]
fn week17_gate_full_pipeline_integration() {
    let mut pipeline = IntegrationPipeline::new_simulated(
        permissive_constitution(),
        CatastropheThresholds::default(),
    )
    .unwrap();

    // ─── Scenario 1: Good code → Accepted
    let r1 = pipeline.run("good-code", "fn main() { println!(\"ok\"); }");
    assert!(r1.is_accepted(), "Good code should be accepted: {:?}", r1.verdict);
    assert_eq!(r1.prediction_error.validity_error, 0.0);

    // ─── Scenario 2: Fail code → Rejected by Reality
    let r2 = pipeline.run("fail-code", "fn main() {}");
    assert!(!r2.is_accepted());

    // ─── Scenario 3: Feedback learns
    for i in 0..10 {
        pipeline.run(&format!("learn-{}", i), "fn main() {}");
    }
    assert!(pipeline.is_improving());
    assert!(pipeline.mean_validity_error() < 0.1);

    // ─── Scenario 4: Result has all fields
    let r4 = pipeline.run("final", "fn main() {}");
    assert!(!r4.run_id.is_nil());
    assert!(!r4.code.is_empty());
    assert!(r4.execution.reality.is_some());
    assert!(!r4.summary().is_empty());

    // ─── Scenario 5: Fitness from code (Week 19)
    assert!(r4.evaluation.fitness.security > 0.0);
    assert!(r4.evaluation.fitness.validate().is_ok());

    println!("✅ Week 17 Gate: PASSED");
    println!("   - Full pipeline: Constitution → Sandbox → Feedback → Analysis");
    println!("   - Correct predictions: zero error");
    println!("   - Feedback: improving after 10 runs");
    println!("   - FitnessVector: from code analysis, not reality");
}
