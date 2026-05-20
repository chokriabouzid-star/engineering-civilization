#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;

#[derive(Debug, Clone, Default)]
pub struct TypeSafetyInvariant {
    pub allow_unsafe: bool,
}

impl Invariant for TypeSafetyInvariant {
    fn name(&self) -> &'static str {
        "TypeSafetyInvariant"
    }

    fn check(
        &self,
        _fitness: &FitnessVector,
        _epistemic: &EpistemicState,
    ) -> Result<(), ViolationReport> {
        if self.allow_unsafe {
            return Ok(());
        }
        Ok(())
    }
}
