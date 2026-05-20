#![deny(warnings)]
#![forbid(unsafe_code)]

pub mod fitness;
pub mod pareto;

pub use fitness::{CatastropheThresholds, CatastrophicDimension, FitnessVector};
pub use pareto::ParetoOrdering;
