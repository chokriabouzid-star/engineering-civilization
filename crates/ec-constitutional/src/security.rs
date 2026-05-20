#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;

/// Enforces minimum security score AND minimum epistemic confidence.
///
/// Both conditions must hold. A high security score with low confidence
/// is constitutionally insufficient — we cannot trust the measurement.
#[derive(Debug, Clone)]
pub struct SecurityInvariant {
    /// Minimum acceptable security score. Default: 0.40
    pub min_security: f64,

    /// Minimum confidence required in the epistemic state.
    /// Default: 0.60
    pub min_confidence: f64,
}

impl Default for SecurityInvariant {
    fn default() -> Self {
        Self {
            min_security: 0.40,
            min_confidence: 0.60,
        }
    }
}

impl Invariant for SecurityInvariant {
    fn name(&self) -> &'static str {
        "SecurityInvariant"
    }

    fn check(
        &self,
        fitness: &FitnessVector,
        epistemic: &EpistemicState,
    ) -> Result<(), ViolationReport> {
        // First check: security score must meet threshold
        if fitness.security < self.min_security {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "security score {:.2} below minimum {:.2}",
                    fitness.security, self.min_security
                ),
                fitness.security,
                self.min_security,
                true, // non-compensable: untrusted security = no security
            ));
        }

        // Second check: we must be confident in the measurement
        if epistemic.confidence < self.min_confidence {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "epistemic confidence {:.2} below minimum {:.2} \
                    — cannot trust security measurement",
                    epistemic.confidence, self.min_confidence
                ),
                epistemic.confidence,
                self.min_confidence,
                true, // non-compensable
            ));
        }

        Ok(())
    }
}
