#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;

#[derive(Debug, Clone)]
pub struct ReversibilityInvariant {
    pub min_reversibility: f64,
}

impl Default for ReversibilityInvariant {
    fn default() -> Self {
        Self {
            min_reversibility: 0.3,
        }
    }
}

impl Invariant for ReversibilityInvariant {
    fn name(&self) -> &'static str {
        "ReversibilityInvariant"
    }

    fn check(
        &self,
        fitness: &FitnessVector,
        _epistemic: &EpistemicState,
    ) -> Result<(), ViolationReport> {
        if fitness.reversibility < self.min_reversibility {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "reversibility {:.2} below minimum {:.2}",
                    fitness.reversibility, self.min_reversibility
                ),
                fitness.reversibility,
                self.min_reversibility,
                true,
            ));
        }
        Ok(())
    }
}
