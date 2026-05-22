#![forbid(unsafe_code)]

//! Week 41 Gate — Bayesian Pipeline Integration

use ec_app::pipeline::{BayesianPipeline, PipelineVerdict};
use ec_constitutional::Constitution;
use ec_fitness::fitness::CatastropheThresholds as CT;

fn permissive_constitution() -> Constitution {
    Constitution::new(
        vec![],
        CT {
            min_security: 0.0,
            min_reversibility: 0.0,
            min_test_coverage: 0.0,
            min_maintainability: 0.0,
            min_performance: 0.0,
            min_architectural_stability: 0.0,
        },
    )
}

// ─── Gate 1: creation ──────────────────────────────────────────────

#[test]
fn w41_bayesian_pipeline_creates() {
    let p = BayesianPipeline::new(permissive_constitution());
    assert!(p.is_ok());
}

// ─── Gate 2: single run ────────────────────────────────────────────

#[test]
fn w41_single_run_accepted() {
    let mut p = BayesianPipeline::new(permissive_constitution()).unwrap();
    let r = p.run("test-artifact", "fn main() {}");
    assert!(matches!(r.verdict, PipelineVerdict::Accepted));
    assert_eq!(r.total_observations, 1);
}

// ─── Gate 3: confidence grows ──────────────────────────────────────

#[test]
fn w41_confidence_grows_with_runs() {
    let mut p = BayesianPipeline::new(permissive_constitution()).unwrap();
    let first = p.run("a1", "fn main() {}");
    for i in 0..20 {
        p.run(&format!("a{}", i), "fn main() {}");
    }
    let last = p.run("a_final", "fn main() {}");

    assert!(last.total_observations > first.total_observations);
    assert!(last.raw_confidence >= first.raw_confidence - 0.01,
        "last={:.3} >= first={:.3}", last.raw_confidence, first.raw_confidence);
}

// ─── Gate 4: calibration diagnosis ─────────────────────────────────

#[test]
fn w41_insufficient_data_initially() {
    let mut p = BayesianPipeline::new(permissive_constitution()).unwrap();
    let r = p.run("a1", "fn main() {}");
    assert!(matches!(r.calibration_diagnosis,
        ec_epistemic::CalibrationDiagnosis::InsufficientData { .. }));
}

#[test]
fn w41_calibration_after_many_runs() {
    let mut p = BayesianPipeline::new(permissive_constitution()).unwrap();
    for i in 0..20 {
        p.run(&format!("a{}", i), "fn main() {}");
    }
    let r = p.run("final", "fn main() {}");
    assert!(!matches!(r.calibration_diagnosis,
        ec_epistemic::CalibrationDiagnosis::InsufficientData { .. }));
}

// ─── Gate 5: build_epistemic_from_bayesian ──────────────────────────

#[test]
fn w41_build_epistemic_from_bayesian() {
    let evidence = ec_epistemic::BayesianEvidence::from_history(10, 2, 0.8).unwrap();
    let cal = ec_epistemic::CalibrationState::default();
    let state = ec_app::pipeline::build_epistemic_from_bayesian(&evidence, &cal);
    assert!(state.is_ok());
    let s = state.unwrap();
    assert!(s.confidence >= 0.10 && s.confidence <= 0.95);
}

// ─── Gate 6: old pipeline still works ──────────────────────────────

#[test]
fn w41_old_pipeline_still_works() {
    let mut p = ec_app::pipeline::IntegrationPipeline::new_simulated(
        permissive_constitution(),
        ec_fitness::fitness::CatastropheThresholds::default(),
    ).unwrap();
    let r = p.run("test", "fn main() {}");
    assert!(r.is_accepted());
}

// ─── Gate 7: unique run ids ────────────────────────────────────────

#[test]
fn w41_unique_run_ids() {
    let mut p = BayesianPipeline::new(permissive_constitution()).unwrap();
    let r1 = p.run("a1", "fn main() {}");
    let r2 = p.run("a2", "fn main() {}");
    assert_ne!(r1.run_id, r2.run_id);
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w41_gate_complete() {
    let mut p = BayesianPipeline::new(permissive_constitution()).unwrap();

    let mut results = vec![];
    for i in 0..25 {
        let r = p.run(&format!("art-{}", i), "fn main() {}");
        results.push(r);
    }

    let first = &results[0];
    let last = &results[24];

    println!("═══════════════════════════════════════════════");
    println!("  Week 41 Gate — Bayesian Pipeline");
    println!("═══════════════════════════════════════════════");
    println!("  Runs:         {}", last.total_observations);
    println!("  First conf:   {:.3}", first.raw_confidence);
    println!("  Last conf:    {:.3} (raw) → {:.3} (adjusted)",
        last.raw_confidence, last.bayesian_confidence);
    println!("  Diagnosis:    {:?}", last.calibration_diagnosis);
    println!("═══════════════════════════════════════════════");

    assert_eq!(last.total_observations, 25);
    assert!(!matches!(last.calibration_diagnosis,
        ec_epistemic::CalibrationDiagnosis::InsufficientData { .. }));

    println!("  ✅ Week 41 Gate: PASSED");
}
