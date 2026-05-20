use ec_constitutional::constitution::Constitution;
use ec_constitutional::coverage::TestCoverageInvariant;
use ec_constitutional::invariant::Invariant;
use ec_constitutional::reversibility::ReversibilityInvariant;
use ec_constitutional::security::SecurityInvariant;
use ec_constitutional::type_safety::TypeSafetyInvariant;
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

#[test]
fn phase_0_gate_test() {
    // 1. Build full constitution
    let invariants: Vec<Arc<dyn Invariant>> = vec![
        Arc::new(SecurityInvariant {
            min_security: 0.7,
            min_confidence: 0.5,
        }),
        Arc::new(TestCoverageInvariant { min_coverage: 0.6 }),
        Arc::new(ReversibilityInvariant {
            min_reversibility: 0.3,
        }),
        Arc::new(TypeSafetyInvariant {
            allow_unsafe: false,
        }),
    ];

    let thresholds = CatastropheThresholds::default();
    let constitution = Constitution::new(invariants, thresholds);

    // 2. Test constitutional evaluation
    let healthy_fitness = FitnessVector {
        security: 0.9,
        reversibility: 0.8,
        test_coverage: 0.85,
        maintainability: 0.7,
        performance: 0.6,
        architectural_stability: 0.75,
    };

    let healthy_epistemic = create_healthy_epistemic();

    let eval_healthy =
        constitution.evaluate("healthy-artifact", &healthy_fitness, &healthy_epistemic);
    assert!(eval_healthy.is_valid);
    assert!(eval_healthy.violations.is_empty());
    assert!(eval_healthy.catastrophic.is_none());

    // 3. Test catastrophic detection
    let catastrophic_fitness = FitnessVector {
        security: 0.1,
        reversibility: 0.8,
        test_coverage: 0.85,
        maintainability: 0.7,
        performance: 0.6,
        architectural_stability: 0.75,
    };

    let eval_catastrophic = constitution.evaluate(
        "catastrophic-artifact",
        &catastrophic_fitness,
        &healthy_epistemic,
    );
    assert!(!eval_catastrophic.is_valid);
    assert!(eval_catastrophic.catastrophic.is_some());

    // 4. Test epistemic propagation
    let low_evidence = Evidence::new(10, 7200, 0.6, 0.6).unwrap();
    let high_uncertainty = UncertaintyDecomposition::new(0.3, 0.3, 0.3).unwrap();
    let uncertain_epistemic = EpistemicState::new(
        0.6,
        low_evidence,
        high_uncertainty,
        CalibrationState::default(),
    )
    .unwrap();

    let eval_uncertain =
        constitution.evaluate("uncertain-artifact", &healthy_fitness, &uncertain_epistemic);
    assert_eq!(eval_uncertain.epistemic.confidence, 0.6);

    // 5. Test Pareto comparison
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

    let eval_a = constitution.evaluate("a", &fitness_a, &healthy_epistemic);
    let eval_b = constitution.evaluate("b", &fitness_b, &healthy_epistemic);

    let ordering = constitution.compare(&eval_a, &eval_b);
    assert!(matches!(ordering, ParetoOrdering::Dominates));

    // 6. Test frontier construction
    let frontier = constitution.build_frontier(&[eval_a.clone(), eval_b.clone()]);
    assert_eq!(frontier.len(), 1);
    assert_eq!(frontier[0].artifact_id, "a");
}
