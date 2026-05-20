#![forbid(unsafe_code)]
use crate::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use crate::{CalibrationState, EpistemicError, EpistemicResult};

/// واجهة للانتشار المحافظ لعدم اليقين.
pub trait ConservativePropagation {
    /// دمج حالات معرفية متعددة.
    fn combine(states: &[EpistemicState]) -> EpistemicResult<EpistemicState>;
}

/// مدمج محافظ.
pub struct ConservativeCombiner;

impl ConservativePropagation for ConservativeCombiner {
    fn combine(states: &[EpistemicState]) -> EpistemicResult<EpistemicState> {
        if states.is_empty() {
            return Err(EpistemicError::OutOfRange {
                field: "states",
                value: 0.0,
                min: 1.0,
                max: f64::INFINITY,
            });
        }
        for s in states {
            s.validate()?;
        }
        let confidence = states.iter().map(|s| s.confidence).fold(1.0, f64::min);
        let evidence = Evidence::weakest(
            &states
                .iter()
                .map(|s| s.evidence.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        let aleatoric = states
            .iter()
            .map(|s| s.uncertainty.aleatoric.powi(2))
            .sum::<f64>()
            .sqrt();
        let epistemic = states
            .iter()
            .map(|s| s.uncertainty.epistemic.powi(2))
            .sum::<f64>()
            .sqrt();
        let model = states
            .iter()
            .map(|s| s.uncertainty.model.powi(2))
            .sum::<f64>()
            .sqrt();
        let calibration = CalibrationState::merge_all(
            &states
                .iter()
                .map(|s| s.calibration.clone())
                .collect::<Vec<_>>(),
        );
        EpistemicState::new(
            confidence,
            evidence,
            UncertaintyDecomposition::new(aleatoric, epistemic, model)?,
            calibration,
        )
    }
}
