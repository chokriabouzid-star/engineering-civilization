#![forbid(unsafe_code)]
use crate::state::{EpistemicState, UncertaintyDecomposition};
use crate::{ensure_in_range, EpistemicResult};
use std::time::Duration;

/// إعدادات التلاشي الزمني.
#[derive(Debug, Clone, Copy)]
pub struct DecayConfig {
    /// عمر النصف.
    pub half_life: Duration,
    /// سقف زيادة عدم اليقين.
    pub epistemic_increase_cap: f64,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            half_life: Duration::from_secs(30 * 24 * 60 * 60),
            epistemic_increase_cap: 0.5,
        }
    }
}

/// سلوك التلاشي الزمني.
pub trait TemporalDecay {
    /// تطبيق التلاشي على الحالة.
    fn decay(
        state: &EpistemicState,
        elapsed: Duration,
        cfg: DecayConfig,
    ) -> EpistemicResult<EpistemicState>;
}

/// تلاشي أسي بناءً على عمر النصف.
pub struct ExponentialHalfLifeDecay;

impl TemporalDecay for ExponentialHalfLifeDecay {
    fn decay(
        state: &EpistemicState,
        elapsed: Duration,
        cfg: DecayConfig,
    ) -> EpistemicResult<EpistemicState> {
        state.validate()?;
        if elapsed.is_zero() {
            return Ok(state.clone());
        }
        let decay_factor = 0.5_f64.powf(elapsed.as_secs_f64() / cfg.half_life.as_secs_f64());
        let new_confidence = (state.confidence * decay_factor).clamp(0.0, 1.0);
        ensure_in_range("confidence", new_confidence, 0.0, 1.0)?;
        let new_epistemic = (state.uncertainty.epistemic
            + (1.0 - decay_factor) * cfg.epistemic_increase_cap)
            .min(1.0);
        let new_uncertainty = UncertaintyDecomposition::new(
            state.uncertainty.aleatoric,
            new_epistemic,
            state.uncertainty.model,
        )?;
        let mut new_state = state.clone();
        new_state.confidence = new_confidence;
        new_state.uncertainty = new_uncertainty;
        new_state.evidence.age_seconds = new_state
            .evidence
            .age_seconds
            .saturating_add(elapsed.as_secs());
        new_state.validate()?;
        Ok(new_state)
    }
}
