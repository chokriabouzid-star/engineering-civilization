#![forbid(unsafe_code)]

use ec_epistemic::CalibrationState;
use proptest::prelude::*;

fn finite_01() -> impl Strategy<Value = f64> {
    (0u64..=1_000_000u64).prop_map(|x| x as f64 / 1_000_000.0)
}

proptest! {
    #[test]
    fn ece_in_range(samples in prop::collection::vec((finite_01(), prop_oneof![Just(0.0f64), Just(1.0f64)]), 0..2000)) {
        let mut c = CalibrationState::default();
        for (p, a) in samples {
            c.record(p, a).unwrap();
        }
        let ece = c.ece();
        prop_assert!(ece >= 0.0 && ece <= 1.0);
    }

    #[test]
    fn perfect_calibration_has_zero_ece(preds in prop::collection::vec(finite_01(), 1..2000)) {
        let mut c = CalibrationState::default();
        for p in preds {
            let p2 = if p < 0.5 { 0.0 } else { 1.0 };
            c.record(p2, p2).unwrap();
        }
        prop_assert!(c.ece() <= 1e-12);
    }

    #[test]
    fn worst_case_is_high_ece(n in 1u64..2000) {
        let mut c = CalibrationState::default();
        for _ in 0..n {
            c.record(1.0, 0.0).unwrap();
        }
        prop_assert!(c.ece() > 0.80);
    }
}
