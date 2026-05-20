#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstitutionalVerdict {
    Accepted,
    Rejected {
        violations: Vec<ViolationReport>,
        has_non_compensable: bool,
    },
}

impl ConstitutionalVerdict {
    pub fn evaluate(
        invariants: &[Arc<dyn Invariant>],
        fitness: &FitnessVector,
        epistemic: &EpistemicState,
    ) -> Self {
        let mut violations = Vec::new();
        let mut has_non_compensable = false;

        for inv in invariants {
            if let Err(violation) = inv.check(fitness, epistemic) {
                if violation.is_non_compensable {
                    has_non_compensable = true;
                }
                violations.push(violation);
            }
        }

        if violations.is_empty() {
            Self::Accepted
        } else {
            Self::Rejected {
                violations,
                has_non_compensable,
            }
        }
    }
}
