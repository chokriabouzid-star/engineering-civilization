#![forbid(unsafe_code)]

//! Week 36 Gate — BayesianTracker

use ec_sandbox::BayesianTracker;

// ─── Gate 1: creation ──────────────────────────────────────────────

#[test]
fn w36_new_tracker_unbiased() {
    let t = BayesianTracker::new();
    assert_eq!(t.evidence().successes, 0);
    assert_eq!(t.evidence().failures, 0);
    assert_eq!(t.total_observations(), 0);
}

#[test]
fn w36_default_same_as_new() {
    let t1 = BayesianTracker::new();
    let t2 = BayesianTracker::default();
    assert_eq!(t1.evidence().successes, t2.evidence().successes);
    assert_eq!(t1.evidence().failures, t2.evidence().failures);
}

// ─── Gate 2: recording ─────────────────────────────────────────────

#[test]
fn w36_record_success() {
    let mut t = BayesianTracker::new();
    t.record(true, 0.9);
    assert_eq!(t.evidence().successes, 1);
    assert_eq!(t.evidence().failures, 0);
    assert_eq!(t.total_observations(), 1);
}

#[test]
fn w36_record_failure() {
    let mut t = BayesianTracker::new();
    t.record(false, 0.1);
    assert_eq!(t.evidence().successes, 0);
    assert_eq!(t.evidence().failures, 1);
}

#[test]
fn w36_record_mixed() {
    let mut t = BayesianTracker::new();
    t.record(true, 0.9);
    t.record(false, 0.2);
    t.record(true, 0.8);
    assert_eq!(t.evidence().successes, 2);
    assert_eq!(t.evidence().failures, 1);
    assert_eq!(t.total_observations(), 3);
}

// ─── Gate 3: confidence grows ──────────────────────────────────────

#[test]
fn w36_confidence_grows_with_successes() {
    let mut t = BayesianTracker::new();
    let init = t.credible_confidence();
    for _ in 0..15 {
        t.record(true, 0.9);
    }
    assert!(t.credible_confidence() > init,
        "after 15 successes: {} > {}", t.credible_confidence(), init);
}

#[test]
fn w36_confidence_low_without_data() {
    let t = BayesianTracker::new();
    assert!(t.credible_confidence() < 0.50);
}

// ─── Gate 4: sufficient data ───────────────────────────────────────

#[test]
fn w36_no_sufficient_data_initially() {
    let t = BayesianTracker::new();
    assert!(!t.has_sufficient_data());
}

#[test]
fn w36_sufficient_after_5() {
    let mut t = BayesianTracker::new();
    for _ in 0..5 {
        t.record(true, 0.9);
    }
    assert!(t.has_sufficient_data());
}

// ─── Gate 5: failures lower confidence ─────────────────────────────

#[test]
fn w36_failures_lower_than_successes() {
    let mut t_fail = BayesianTracker::new();
    for _ in 0..10 {
        t_fail.record(false, 0.1);
    }
    let mut t_ok = BayesianTracker::new();
    for _ in 0..10 {
        t_ok.record(true, 0.9);
    }
    assert!(t_fail.credible_confidence() < t_ok.credible_confidence(),
        "failures={:.3} < successes={:.3}",
        t_fail.credible_confidence(), t_ok.credible_confidence());
}

// ─── Gate 6: old RealityFeedback still works ────────────────────────

#[test]
fn w36_old_feedback_still_works() {
    let fb = ec_sandbox::RealityFeedback::new();
    assert_eq!(fb.mean_validity_error(), 0.0);
    assert!(fb.is_improving());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w36_gate_complete() {
    let mut t = BayesianTracker::new();
    for i in 0..20 {
        let ok = i < 15; // 15 نجاح + 5 فشل
        t.record(ok, if ok { 0.9 } else { 0.1 });
    }

    println!("═══════════════════════════════════════════════");
    println!("  Week 36 Gate — BayesianTracker");
    println!("═══════════════════════════════════════════════");
    println!("  Observations: {}", t.total_observations());
    println!("  Successes:    {}", t.evidence().successes);
    println!("  Failures:     {}", t.evidence().failures);
    println!("  Mean score:   {:.2}", t.evidence().mean_score);
    println!("  Confidence:   {:.3}", t.credible_confidence());
    println!("  Sufficient:   {}", t.has_sufficient_data());
    println!("═══════════════════════════════════════════════");

    assert_eq!(t.total_observations(), 20);
    assert!(t.has_sufficient_data());
    assert!(t.credible_confidence() > 0.45);

    println!("  ✅ Week 36 Gate: PASSED");
}
