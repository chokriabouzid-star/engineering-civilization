use ec_constitutional::constitution::Constitution;
use ec_constitutional::coverage::TestCoverageInvariant;
use ec_constitutional::invariant::Invariant;
use ec_constitutional::security::SecurityInvariant;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use ec_fitness::ParetoOrdering;
use std::sync::Arc;

fn create_evidence() -> Evidence {
    Evidence::new(100, 3600, 0.9, 0.9).unwrap()
}

fn create_uncertainty() -> UncertaintyDecomposition {
    UncertaintyDecomposition::new(0.1, 0.1, 0.1).unwrap()
}

fn create_healthy_epistemic() -> EpistemicState {
    EpistemicState::new(
        0.9,
        create_evidence(),
        create_uncertainty(),
        CalibrationState::default(),
    )
    .unwrap()
}

fn create_test_constitution() -> Constitution {
    let invariants: Vec<Arc<dyn Invariant>> = vec![
        Arc::new(SecurityInvariant {
            min_security: 0.7,
            min_confidence: 0.5,
        }),
        Arc::new(TestCoverageInvariant { min_coverage: 0.6 }),
    ];

    let thresholds = CatastropheThresholds::default();

    Constitution::new(invariants, thresholds)
}

#[test]
fn safe_artifact_should_pass() {
    let constitution = create_test_constitution();
    let fitness = FitnessVector {
        security: 0.9,
        reversibility: 0.8,
        test_coverage: 0.85,
        maintainability: 0.7,
        performance: 0.6,
        architectural_stability: 0.75,
    };
    let epistemic = create_healthy_epistemic();

    let evaluation = constitution.evaluate("test-artifact-1", &fitness, &epistemic);

    assert!(evaluation.is_valid);
    assert!(evaluation.violations.is_empty());
    assert!(evaluation.catastrophic.is_none());
}

#[test]
fn catastrophic_artifact_should_fail() {
    let constitution = create_test_constitution();
    let fitness = FitnessVector {
        security: 0.1,
        reversibility: 0.8,
        test_coverage: 0.85,
        maintainability: 0.7,
        performance: 0.6,
        architectural_stability: 0.75,
    };
    let epistemic = create_healthy_epistemic();

    let evaluation = constitution.evaluate("test-artifact-bad", &fitness, &epistemic);

    assert!(!evaluation.is_valid);
    assert!(evaluation.catastrophic.is_some());
}

#[test]
fn uncertainty_should_propagate() {
    let constitution = create_test_constitution();
    let fitness = FitnessVector {
        security: 0.9,
        reversibility: 0.8,
        test_coverage: 0.85,
        maintainability: 0.7,
        performance: 0.6,
        architectural_stability: 0.75,
    };

    let low_evidence = Evidence::new(10, 7200, 0.6, 0.6).unwrap();
    let high_uncertainty = UncertaintyDecomposition::new(0.3, 0.3, 0.3).unwrap();

    let epistemic = EpistemicState::new(
        0.6,
        low_evidence,
        high_uncertainty,
        CalibrationState::default(),
    )
    .unwrap();

    let evaluation = constitution.evaluate("test-uncertainty", &fitness, &epistemic);

    assert_eq!(evaluation.epistemic.confidence, 0.6);
    assert_eq!(evaluation.epistemic.evidence.reproducibility, 0.6);
}

#[test]
fn pareto_comparison_should_work() {
    let constitution = create_test_constitution();
    let epistemic = create_healthy_epistemic();

    let fitness_a = FitnessVector {
        security: 0.9,
        reversibility: 0.9,
        test_coverage: 0.9,
        maintainability: 0.9,
        performance: 0.9,
        architectural_stability: 0.9,
    };

    let fitness_b = FitnessVector {
        security: 0.8,
        reversibility: 0.8,
        test_coverage: 0.8,
        maintainability: 0.8,
        performance: 0.8,
        architectural_stability: 0.8,
    };

    let eval_a = constitution.evaluate("a", &fitness_a, &epistemic);
    let eval_b = constitution.evaluate("b", &fitness_b, &epistemic);

    let ordering = constitution.compare(&eval_a, &eval_b);
    assert!(matches!(ordering, ParetoOrdering::Dominates));
}

#[test]
fn frontier_should_exclude_dominated() {
    let constitution = create_test_constitution();
    let epistemic = create_healthy_epistemic();

    let dominated = FitnessVector {
        security: 0.7,
        reversibility: 0.7,
        test_coverage: 0.7,
        maintainability: 0.7,
        performance: 0.7,
        architectural_stability: 0.7,
    };

    let dominant = FitnessVector {
        security: 0.9,
        reversibility: 0.9,
        test_coverage: 0.9,
        maintainability: 0.9,
        performance: 0.9,
        architectural_stability: 0.9,
    };

    let eval_dom = constitution.evaluate("dom", &dominated, &epistemic);
    let eval_dominant = constitution.evaluate("dominant", &dominant, &epistemic);

    let frontier = constitution.build_frontier(&[eval_dom, eval_dominant]);

    assert_eq!(frontier.len(), 1);
    assert_eq!(frontier[0].artifact_id, "dominant");
}
