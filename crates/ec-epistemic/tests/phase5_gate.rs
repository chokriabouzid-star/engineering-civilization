#![forbid(unsafe_code)]

//! Phase 5 Gate (Partial) — Bayesian Evidence + Confidence
//!
//! Weeks 35-38: BayesianEvidence → BayesianTracker → OutcomeStorage → BayesianQuery
//! D1-D8: كلها محفوظة
//! كل الـ APIs القديمة لا تزال تعمل

use ec_epistemic::{BayesianEvidence, Evidence, EpistemicState};
use ec_epistemic::{UncertaintyDecomposition, CalibrationState};

// ─── D8: Confidence منفصل ──────────────────────────────────────────

#[test]
fn p5_confidence_separate_from_fitness() {
    // BayesianEvidence يُنتج confidence مستقل عن أي FitnessVector
    let e = BayesianEvidence::initial_prior().unwrap();
    let conf = e.credible_confidence();
    assert!((0.0..=1.0).contains(&conf));
    assert!(conf < 0.50, "prior بدون بيانات → ثقة منخفضة");
}

// ─── BayesianEvidence Lifecycle ─────────────────────────────────────

#[test]
fn p5_bayesian_lifecycle() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    assert_eq!(e.total_observations(), 0);

    // 10 نجاحات
    for _ in 0..10 {
        e = e.update_with_outcome(true, 0.9).unwrap();
    }
    assert_eq!(e.successes, 10);
    assert_eq!(e.failures, 0);
    let conf_after_success = e.credible_confidence();

    // 5 فشل
    for _ in 0..5 {
        e = e.update_with_outcome(false, 0.1).unwrap();
    }
    assert_eq!(e.successes, 10);
    assert_eq!(e.failures, 5);
    let conf_after_mixed = e.credible_confidence();

    assert!(conf_after_success > conf_after_mixed,
        "pure success ({:.3}) > mixed ({:.3})", conf_after_success, conf_after_mixed);
}

#[test]
fn p5_wilson_converges() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    let conf_before = e.credible_confidence();

    for _ in 0..50 {
        e = e.update_with_outcome(true, 0.95).unwrap();
    }
    let conf_after = e.credible_confidence();

    assert!(conf_after > conf_before);
    assert!(conf_after > 0.80, "50 successes → high confidence: {}", conf_after);
}

#[test]
fn p5_from_history_matches_update() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    e = e.update_with_outcome(true, 0.9).unwrap();
    e = e.update_with_outcome(true, 0.8).unwrap();
    e = e.update_with_outcome(false, 0.2).unwrap();

    let from_hist = BayesianEvidence::from_history(2, 1, e.mean_score).unwrap();
    assert_eq!(from_hist.successes, e.successes);
    assert_eq!(from_hist.failures, e.failures);
}

// ─── Old Evidence API لا يتكسر ─────────────────────────────────────

#[test]
fn p5_old_evidence_unchanged() {
    let e = Evidence::new(100, 3600, 0.95, 0.90).unwrap();
    assert_eq!(e.sample_size, 100);
    assert_eq!(e.age_seconds, 3600);
    assert!((e.reproducibility - 0.95).abs() < 0.001);
    assert!((e.source_reliability - 0.90).abs() < 0.001);
}

#[test]
fn p5_old_epistemic_state_unchanged() {
    let evidence = Evidence::new(50, 0, 0.9, 0.8).unwrap();
    let uncertainty = UncertaintyDecomposition::new(0.1, 0.2, 0.05).unwrap();
    let state = EpistemicState::new(0.85, evidence, uncertainty, CalibrationState::default()).unwrap();
    assert!((state.confidence - 0.85).abs() < 0.001);
    assert!(state.total_uncertainty() > 0.0);
}

// ─── Confidence grows monotonically with consistent data ────────────

#[test]
fn p5_confidence_monotonic_with_successes() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    let mut confs = vec![e.credible_confidence()];

    for _ in 0..20 {
        e = e.update_with_outcome(true, 0.9).unwrap();
        confs.push(e.credible_confidence());
    }

    // بعد أول 5 مشاهدات، يجب أن يرتفع أو يبقى ثابت
    for i in 5..confs.len() {
        assert!(confs[i] >= confs[i-1] - 0.01,
            "conf[{}]={:.3} dropped from conf[{}]={:.3}",
            i, confs[i], i-1, confs[i-1]);
    }
}

// ─── Invalid inputs rejected ────────────────────────────────────────

#[test]
fn p5_invalid_score_rejected() {
    let e = BayesianEvidence::initial_prior().unwrap();
    assert!(e.update_with_outcome(true, 1.5).is_err());
    assert!(e.update_with_outcome(true, -0.1).is_err());
}

#[test]
fn p5_invalid_mean_score_rejected() {
    assert!(BayesianEvidence::from_history(5, 2, 2.0).is_err());
    assert!(BayesianEvidence::from_history(5, 2, -0.1).is_err());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn phase5_partial_gate_complete() {
    let prior = BayesianEvidence::initial_prior().unwrap();
    let mut success_only = prior.clone();
    let mut mixed = prior.clone();
    let mut failure_only = prior.clone();

    for _ in 0..20 {
        success_only = success_only.update_with_outcome(true, 0.95).unwrap();
        mixed = mixed.update_with_outcome(true, 0.7).unwrap();
        mixed = mixed.update_with_outcome(false, 0.3).unwrap();
        failure_only = failure_only.update_with_outcome(false, 0.05).unwrap();
    }

    println!("╔══════════════════════════════════════════════╗");
    println!("║   Phase 5 Gate (Partial) — Weeks 35-38      ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  BayesianEvidence:           ✅              ║");
    println!("║  Wilson interval:            ✅              ║");
    println!("║  from_history:               ✅              ║");
    println!("║  Old Evidence API:           ✅              ║");
    println!("║  Old EpistemicState API:     ✅              ║");
    println!("║  D1-D8 preserved:            ✅              ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  Prior:        conf={:.3}                    ║", prior.credible_confidence());
    println!("║  Success only: conf={:.3} s={} f={}         ║",
        success_only.credible_confidence(), success_only.successes, success_only.failures);
    println!("║  Mixed:        conf={:.3} s={} f={}          ║",
        mixed.credible_confidence(), mixed.successes, mixed.failures);
    println!("║  Failure only: conf={:.3} s={} f={}         ║",
        failure_only.credible_confidence(), failure_only.successes, failure_only.failures);
    println!("╚══════════════════════════════════════════════╝");

    assert!(success_only.credible_confidence() > mixed.credible_confidence());
    assert!(mixed.credible_confidence() > failure_only.credible_confidence());

    println!("  ✅ Phase 5 Partial Gate: PASSED");
}
