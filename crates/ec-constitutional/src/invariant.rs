#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;
use serde::{Deserialize, Serialize};

/// A single constitutional violation — why an artifact was rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViolationReport {
    pub invariant_name: String,
    pub reason: String,
    pub actual_value: f64,
    pub required_value: f64,
    pub is_non_compensable: bool,
}

impl ViolationReport {
    pub fn new(
        invariant_name: &'static str,
        reason: impl Into<String>,
        actual_value: f64,
        required_value: f64,
        is_non_compensable: bool,
    ) -> Self {
        Self {
            invariant_name: invariant_name.to_string(),
            reason: reason.into(),
            actual_value,
            required_value,
            is_non_compensable,
        }
    }
}

/// The constitutional contract between artifacts and the governance kernel.
pub trait Invariant: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;

    fn check(
        &self,
        fitness: &FitnessVector,
        epistemic: &EpistemicState,
    ) -> Result<(), ViolationReport>;
}
