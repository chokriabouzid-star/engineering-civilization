#![forbid(unsafe_code)]

//! Week 40 Gate — BayesianCalibration

use ec_epistemic::{BayesianCalibration, BayesianEvidence, CalibrationDiagnosis, CalibrationState};

// ─── Gate 1: insufficient data ──────────────────────────────────────

#[test]
fn w40_insufficient_data_few_samples() {
    let cal = CalibrationState::default();
    let d = BayesianCalibration::diagnose(&cal);
    assert!(matches!(
        d,
        CalibrationDiagnosis::InsufficientData { samples: 0 }
    ));
}

#[test]
fn w40_insufficient_data_9_samples() {
    let mut cal = CalibrationState::default();
    for _ in 0..9 {
        cal.record(0.8, 0.8).unwrap();
    }
    let d = BayesianCalibration::diagnose(&cal);
    assert!(matches!(
        d,
        CalibrationDiagnosis::InsufficientData { samples: 9 }
    ));
}

// ─── Gate 2: well calibrated ────────────────────────────────────────

#[test]
fn w40_well_calibrated_when_predictions_match() {
    let mut cal = CalibrationState::default();
    for _ in 0..20 {
        cal.record(0.8, 0.8).unwrap();
        cal.record(0.5, 0.5).unwrap();
    }
    let d = BayesianCalibration::diagnose(&cal);
    match &d {
        CalibrationDiagnosis::WellCalibrated { ece } => {
            assert!((*ece) < 0.10, "ECE: {}", ece);
        }
        _ => panic!("expected WellCalibrated, got {:?}", d),
    }
}

// ─── Gate 3: overconfident ──────────────────────────────────────────

#[test]
fn w40_overconfident_when_predictions_too_high() {
    let mut cal = CalibrationState::default();
    for _ in 0..30 {
        cal.record(0.9, 0.3).unwrap(); // نتوقع 0.9 لكن الواقع 0.3
    }
    let d = BayesianCalibration::diagnose(&cal);
    match &d {
        CalibrationDiagnosis::Overconfident { ece, avg_gap } => {
            assert!(ece > &0.10);
            assert!(avg_gap > &0.05);
        }
        _ => panic!("expected Overconfident, got {:?}", d),
    }
}

// ─── Gate 4: underconfident ─────────────────────────────────────────

#[test]
fn w40_underconfident_when_predictions_too_low() {
    let mut cal = CalibrationState::default();
    for _ in 0..30 {
        cal.record(0.3, 0.9).unwrap(); // نتوقع 0.3 لكن الواقع 0.9
    }
    let d = BayesianCalibration::diagnose(&cal);
    match &d {
        CalibrationDiagnosis::Underconfident { ece, avg_gap } => {
            assert!(ece > &0.10);
            assert!(avg_gap > &0.05);
        }
        _ => panic!("expected Underconfident, got {:?}", d),
    }
}

// ─── Gate 5: adjustment ─────────────────────────────────────────────

#[test]
fn w40_well_calibrated_no_adjustment() {
    let d = CalibrationDiagnosis::WellCalibrated { ece: 0.05 };
    let adjusted = BayesianCalibration::adjust_confidence(0.80, &d);
    assert!((adjusted - 0.80).abs() < 0.001);
}

#[test]
fn w40_overconfident_reduces_confidence() {
    let d = CalibrationDiagnosis::Overconfident {
        ece: 0.3,
        avg_gap: 0.2,
    };
    let adjusted = BayesianCalibration::adjust_confidence(0.80, &d);
    assert!(adjusted < 0.80, "adjusted={:.3}", adjusted);
}

#[test]
fn w40_underconfident_increases_confidence() {
    let d = CalibrationDiagnosis::Underconfident {
        ece: 0.3,
        avg_gap: 0.2,
    };
    let adjusted = BayesianCalibration::adjust_confidence(0.50, &d);
    assert!(adjusted > 0.50, "adjusted={:.3}", adjusted);
}

#[test]
fn w40_insufficient_data_reduces_slightly() {
    let d = CalibrationDiagnosis::InsufficientData { samples: 3 };
    let adjusted = BayesianCalibration::adjust_confidence(0.80, &d);
    assert!((adjusted - 0.70).abs() < 0.001, "adjusted={:.3}", adjusted);
}

#[test]
fn w40_adjustment_never_below_floor() {
    let d = CalibrationDiagnosis::Overconfident {
        ece: 0.5,
        avg_gap: 0.9,
    };
    let adjusted = BayesianCalibration::adjust_confidence(0.20, &d);
    assert!(adjusted >= 0.10, "floor=0.10, got: {}", adjusted);
}

// ─── Gate 6: adjusted_credible_confidence ───────────────────────────

#[test]
fn w40_adjusted_credible_with_well_calibrated() {
    let mut cal = CalibrationState::default();
    for _ in 0..20 {
        cal.record(0.8, 0.8).unwrap();
    }
    let e = BayesianEvidence::from_history(15, 2, 0.85).unwrap();
    let adjusted = BayesianCalibration::adjusted_credible_confidence(&e, &cal);
    let raw = e.credible_confidence();
    // Well calibrated → لا تعديل
    assert!((adjusted - raw).abs() < 0.01);
}

#[test]
fn w40_adjusted_credible_with_overconfident() {
    let mut cal = CalibrationState::default();
    for _ in 0..30 {
        cal.record(0.9, 0.3).unwrap();
    }
    let e = BayesianEvidence::from_history(15, 2, 0.85).unwrap();
    let adjusted = BayesianCalibration::adjusted_credible_confidence(&e, &cal);
    let raw = e.credible_confidence();
    assert!(adjusted < raw, "adjusted={:.3} < raw={:.3}", adjusted, raw);
}

// ─── Gate 7: old CalibrationState still works ───────────────────────

#[test]
fn w40_old_calibration_still_works() {
    let mut cal = CalibrationState::default();
    cal.record(0.8, 0.8).unwrap();
    assert_eq!(cal.total, 1);
    assert!(cal.ece() < 0.10);
    assert!(cal.is_calibrated());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w40_gate_complete() {
    let mut cal_ok = CalibrationState::default();
    let mut cal_over = CalibrationState::default();
    let mut cal_under = CalibrationState::default();

    for _ in 0..20 {
        cal_ok.record(0.8, 0.8).unwrap();
        cal_over.record(0.9, 0.3).unwrap();
        cal_under.record(0.3, 0.9).unwrap();
    }

    let d_ok = BayesianCalibration::diagnose(&cal_ok);
    let d_over = BayesianCalibration::diagnose(&cal_over);
    let d_under = BayesianCalibration::diagnose(&cal_under);

    println!("═══════════════════════════════════════════════");
    println!("  Week 40 Gate — BayesianCalibration");
    println!("═══════════════════════════════════════════════");
    println!("  Well calibrated: {:?}", d_ok);
    println!("  Overconfident:   {:?}", d_over);
    println!("  Underconfident:  {:?}", d_under);
    println!("═══════════════════════════════════════════════");

    assert!(matches!(d_ok, CalibrationDiagnosis::WellCalibrated { .. }));
    assert!(matches!(d_over, CalibrationDiagnosis::Overconfident { .. }));
    assert!(matches!(
        d_under,
        CalibrationDiagnosis::Underconfident { .. }
    ));

    println!("  ✅ Week 40 Gate: PASSED");
}
