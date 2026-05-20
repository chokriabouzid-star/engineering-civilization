use ec_constitutional::coverage::TestCoverageInvariant;
use ec_constitutional::invariant::Invariant;
use ec_constitutional::reversibility::ReversibilityInvariant;
use ec_constitutional::security::SecurityInvariant;
use ec_constitutional::type_safety::TypeSafetyInvariant;
use ec_constitutional::verdict::ConstitutionalVerdict;
use ec_epistemic::state::EpistemicState;
use ec_epistemic::state::Evidence;
use ec_epistemic::state::UncertaintyDecomposition;
use ec_fitness::fitness::FitnessVector;
use std::sync::Arc;

fn good_epistemic() -> EpistemicState {
    EpistemicState::new(
        0.95,
        Evidence::new(100, 60, 0.95, 0.95).unwrap(),
        UncertaintyDecomposition::new(0.05, 0.05, 0.05).unwrap(),
        Default::default(),
    )
    .unwrap()
}

fn invariants() -> Vec<Arc<dyn Invariant>> {
    vec![
        Arc::new(SecurityInvariant {
            min_security: 0.70,
            min_confidence: 0.50,
        }),
        Arc::new(TestCoverageInvariant { min_coverage: 0.60 }),
        Arc::new(ReversibilityInvariant {
            min_reversibility: 0.30,
        }),
        Arc::new(TypeSafetyInvariant {
            allow_unsafe: false,
        }),
    ]
}

#[test]
fn secure_artifact_should_be_accepted() {
    let fitness = FitnessVector {
        security: 0.95,
        reversibility: 0.90,
        test_coverage: 0.92,
        maintainability: 0.85,
        performance: 0.70,
        architectural_stability: 0.88,
    };

    let verdict = ConstitutionalVerdict::evaluate(&invariants(), &fitness, &good_epistemic());

    match verdict {
        ConstitutionalVerdict::Accepted => {}
        _ => panic!("expected Accepted verdict"),
    }
}

#[test]
fn catastrophic_security_should_be_rejected() {
    let fitness = FitnessVector {
        security: 0.05,
        reversibility: 1.0,
        test_coverage: 1.0,
        maintainability: 1.0,
        performance: 1.0,
        architectural_stability: 1.0,
    };

    let verdict = ConstitutionalVerdict::evaluate(&invariants(), &fitness, &good_epistemic());

    match verdict {
        ConstitutionalVerdict::Rejected { .. } => {}
        _ => panic!("expected Rejected verdict"),
    }
}

#[test]
fn low_coverage_should_be_rejected() {
    let fitness = FitnessVector {
        security: 0.95,
        reversibility: 0.90,
        test_coverage: 0.10,
        maintainability: 0.90,
        performance: 0.90,
        architectural_stability: 0.90,
    };

    let verdict = ConstitutionalVerdict::evaluate(&invariants(), &fitness, &good_epistemic());

    match verdict {
        ConstitutionalVerdict::Rejected { .. } => {}
        _ => panic!("expected Rejected verdict"),
    }
}

#[test]
fn epistemic_uncertainty_should_not_crash() {
    let epistemic = EpistemicState::new(
        0.10,
        Evidence::new(1, 31_536_000, 0.10, 0.10).unwrap(),
        UncertaintyDecomposition::new(0.90, 0.90, 0.90).unwrap(),
        Default::default(),
    )
    .unwrap();

    let fitness = FitnessVector {
        security: 0.90,
        reversibility: 0.90,
        test_coverage: 0.90,
        maintainability: 0.90,
        performance: 0.90,
        architectural_stability: 0.90,
    };

    let verdict = ConstitutionalVerdict::evaluate(&invariants(), &fitness, &epistemic);

    let _ = verdict;
}
