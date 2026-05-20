#![forbid(unsafe_code)]

pub mod compare;
pub mod constitution;
pub mod coverage;
pub mod evaluation;
pub mod frontier;
pub mod invariant;
pub mod reversibility;
pub mod security;
pub mod type_safety;
pub mod verdict;
pub mod engine;
pub mod meta;
pub mod policy;

// ─── Public Exports ─────────────────────────────────────────────────

pub use constitution::Constitution;
pub use engine::{ConstitutionalEngine, EvaluationContext};
pub use evaluation::ConstitutionalEvaluation;
pub use invariant::Invariant;
pub use security::SecurityInvariant;
pub use coverage::TestCoverageInvariant;
pub use reversibility::ReversibilityInvariant;
pub use type_safety::TypeSafetyInvariant;
pub use verdict::ConstitutionalVerdict;
pub use meta::{OssificationDetector, ValueDriftDetector};
