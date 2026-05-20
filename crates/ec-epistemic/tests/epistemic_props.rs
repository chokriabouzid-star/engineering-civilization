#![forbid(unsafe_code)]

use ec_epistemic::{
    CalibrationState, ConservativeCombiner, ConservativePropagation, DecayConfig, EpistemicState,
    Evidence, ExponentialHalfLifeDecay, TemporalDecay, UncertaintyDecomposition,
};
use proptest::prelude::*;
use std::time::Duration;

fn finite_01() -> impl Strategy<Value = f64> {
    (0u64..=1_000_000u64).prop_map(|x| x as f64 / 1_000_000.0)
}

fn nonneg_finite() -> impl Strategy<Value = f64> {
    (0u64..=1_000_000u64).prop_map(|x| x as f64 / 100_000.0) // 0..=10
}

fn evidence_strat() -> impl Strategy<Value = Evidence> {
    (0u64..=10_000, 0u64..=100_000, finite_01(), finite_01())
        .prop_map(|(n, age, r, sr)| Evidence::new(n, age, r, sr).unwrap())
}

fn uncertainty_strat() -> impl Strategy<Value = UncertaintyDecomposition> {
    (nonneg_finite(), nonneg_finite(), nonneg_finite())
        .prop_map(|(a, e, m)| UncertaintyDecomposition::new(a, e, m).unwrap())
}

fn epistemic_state_strat() -> impl Strategy<Value = EpistemicState> {
    (finite_01(), evidence_strat(), uncertainty_strat()).prop_map(|(c, ev, un)| {
        EpistemicState::new(c, ev, un, CalibrationState::default()).unwrap()
    })
}

proptest! {
    #[test]
    fn combine_is_conservative_confidence(states in prop::collection::vec(epistemic_state_strat(), 1..20)) {
        let combined = ConservativeCombiner::combine(&states).unwrap();
        let min_conf = states.iter().map(|s| s.confidence).fold(1.0, f64::min);
        prop_assert!(combined.confidence <= min_conf + 1e-12);
    }

    #[test]
    fn combine_total_uncertainty_is_monotone(states in prop::collection::vec(epistemic_state_strat(), 1..20)) {
        let combined = ConservativeCombiner::combine(&states).unwrap();
        let max_u = states.iter().map(|s| s.total_uncertainty()).fold(0.0, f64::max);
        prop_assert!(combined.total_uncertainty() + 1e-12 >= max_u);
    }

    #[test]
    fn decay_never_increases_confidence(state in epistemic_state_strat(), secs in 1u64..(365*24*3600)) {
        let decayed = ExponentialHalfLifeDecay::decay(&state, Duration::from_secs(secs), DecayConfig::default()).unwrap();
        prop_assert!(decayed.confidence <= state.confidence + 1e-12);
    }

    #[test]
    fn decay_zero_is_identity(state in epistemic_state_strat()) {
        let decayed = ExponentialHalfLifeDecay::decay(&state, Duration::from_secs(0), DecayConfig::default()).unwrap();
        prop_assert_eq!(decayed, state);
    }
}
