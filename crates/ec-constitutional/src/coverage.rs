#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;

/// Enforces minimum test coverage.
///
/// Unlike SecurityInvariant, this does not check epistemic confidence
/// separately — test coverage IS a form of epistemic evidence.
/// Low coverage means low evidence, which is already reflected
/// in the EpistemicState's evidence fields.
#[derive(Debug, Clone)]
pub struct TestCoverageInvariant {
    /// Minimum acceptable test coverage score. Default: 0.30
    pub min_coverage: f64,
}

impl Default for TestCoverageInvariant {
    fn default() -> Self {
        Self { min_coverage: 0.30 }
    }
}

impl Invariant for TestCoverageInvariant {
    fn name(&self) -> &'static str {
        "TestCoverageInvariant"
    }

    fn check(
        &self,
        fitness: &FitnessVector,
        _epistemic: &EpistemicState, // coverage already reflects evidence
    ) -> Result<(), ViolationReport> {
        if fitness.test_coverage < self.min_coverage {
            return Err(ViolationReport::new(
                self.name(),
                format!(
                    "test coverage {:.2} below minimum {:.2}",
                    fitness.test_coverage, self.min_coverage
                ),
                fitness.test_coverage,
                self.min_coverage,
                false, // compensable: can be improved without systemic risk
            ));
        }
        Ok(())
    }
}
