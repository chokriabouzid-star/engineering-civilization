use crate::invariant::ViolationReport;
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::CatastrophicDimension;
use ec_fitness::fitness::FitnessVector;

#[derive(Debug, Clone)]
pub struct ConstitutionalEvaluation {
    pub artifact_id: String,
    pub fitness: FitnessVector,
    pub epistemic: EpistemicState,
    pub violations: Vec<ViolationReport>,
    pub catastrophic: Option<CatastrophicDimension>,
    pub is_valid: bool,
    pub explanation: String,
}
