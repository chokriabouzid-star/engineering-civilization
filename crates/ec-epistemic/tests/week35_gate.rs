#![forbid(unsafe_code)]

//! Week 35 Gate — BayesianEvidence

use ec_epistemic::BayesianEvidence;

// ─── Gate 1: initial_prior غير متحيز ───────────────────────────────

#[test]
fn w35_initial_prior_unbiased() {
    let p = BayesianEvidence::initial_prior().unwrap();
    assert_eq!(p.successes, 0);
    assert_eq!(p.failures, 0);
    assert!(
        (p.mean_score - 0.5).abs() < 0.01,
        "Prior يجب أن يكون غير متحيز: {}",
        p.mean_score
    );
}

#[test]
fn w35_initial_prior_zero_observations() {
    let p = BayesianEvidence::initial_prior().unwrap();
    assert_eq!(p.total_observations(), 0);
}

// ─── Gate 2: credible_confidence يرتفع مع النجاحات ────────────────

#[test]
fn w35_confidence_grows_with_successes() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    let init = e.credible_confidence();

    for _ in 0..15 {
        e = e.update_with_outcome(true, 0.9).unwrap();
    }

    assert!(
        e.credible_confidence() > init,
        "بعد 15 نجاح: {} يجب أن يكون أعلى من {}",
        e.credible_confidence(),
        init
    );
}

#[test]
fn w35_confidence_low_with_few_samples() {
    let e = BayesianEvidence::initial_prior().unwrap();
    assert!(
        e.credible_confidence() < 0.50,
        "بدون بيانات: confidence يجب أن تكون منخفضة، got: {}",
        e.credible_confidence()
    );
}

// ─── Gate 3: نتائج مختلطة ──────────────────────────────────────────

#[test]
fn w35_mixed_outcomes_moderate_confidence() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    for i in 0..10 {
        let ok = i % 2 == 0;
        e = e
            .update_with_outcome(ok, if ok { 0.9 } else { 0.1 })
            .unwrap();
    }
    let c = e.credible_confidence();
    assert!(
        (0.30..0.80).contains(&c),
        "نتائج مختلطة → ثقة معتدلة: {}",
        c
    );
}

// ─── Gate 4: فشل يخفض الثقة ────────────────────────────────────────

#[test]
fn w35_failures_lower_confidence() {
    let mut e_fail = BayesianEvidence::initial_prior().unwrap();
    for _ in 0..10 {
        e_fail = e_fail.update_with_outcome(false, 0.1).unwrap();
    }
    let mut e_ok = BayesianEvidence::initial_prior().unwrap();
    for _ in 0..10 {
        e_ok = e_ok.update_with_outcome(true, 0.9).unwrap();
    }
    assert!(
        e_fail.credible_confidence() < e_ok.credible_confidence(),
        "فشل={:.3}  نجاح={:.3}",
        e_fail.credible_confidence(),
        e_ok.credible_confidence()
    );
}

// ─── Gate 5: update يُراكم بشكل صحيح ───────────────────────────────

#[test]
fn w35_update_accumulates_correctly() {
    let mut e = BayesianEvidence::initial_prior().unwrap();
    e = e.update_with_outcome(true, 0.8).unwrap();
    e = e.update_with_outcome(true, 0.9).unwrap();
    e = e.update_with_outcome(false, 0.2).unwrap();

    assert_eq!(e.successes, 2);
    assert_eq!(e.failures, 1);
    assert_eq!(e.total_observations(), 3);
    assert!(e.mean_score > 0.0 && e.mean_score < 1.0);
}

// ─── Gate 6: Evidence القديم لا يتكسر ──────────────────────────────

#[test]
fn w35_old_evidence_still_works() {
    let e = ec_epistemic::Evidence::new(5, 100, 0.8, 0.9).unwrap();
    assert_eq!(e.sample_size, 5);
    assert_eq!(e.age_seconds, 100);
}

// ─── Gate 7: from_history ───────────────────────────────────────────

#[test]
fn w35_from_history() {
    let e = BayesianEvidence::from_history(8, 2, 0.75).unwrap();
    assert_eq!(e.successes, 8);
    assert_eq!(e.failures, 2);
    assert_eq!(e.total_observations(), 10);
}

#[test]
fn w35_from_history_rejects_invalid() {
    let r = BayesianEvidence::from_history(5, 5, 1.5);
    assert!(r.is_err());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w35_gate_complete() {
    let p = BayesianEvidence::initial_prior().unwrap();
    let mut e = p.clone();
    for _ in 0..5 {
        e = e.update_with_outcome(true, 0.9).unwrap();
    }

    println!("═══════════════════════════════════════════════");
    println!("  Week 35 Gate — BayesianEvidence");
    println!("═══════════════════════════════════════════════");
    println!(
        "  Prior:       s={} f={} mean={:.2}",
        p.successes, p.failures, p.mean_score
    );
    println!(
        "  After 5 ok:  s={} f={} mean={:.2} conf={:.3}",
        e.successes,
        e.failures,
        e.mean_score,
        e.credible_confidence()
    );
    println!("═══════════════════════════════════════════════");

    assert_eq!(p.successes, 0);
    assert_eq!(e.successes, 5);
    assert!(e.credible_confidence() > p.credible_confidence());

    println!("  ✅ Week 35 Gate: PASSED");
}
