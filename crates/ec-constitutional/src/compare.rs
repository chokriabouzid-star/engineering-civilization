use crate::constitution::Constitution;
use crate::evaluation::ConstitutionalEvaluation;
use ec_fitness::ParetoOrdering;

impl Constitution {
    pub fn compare(
        &self,
        left: &ConstitutionalEvaluation,
        right: &ConstitutionalEvaluation,
    ) -> ParetoOrdering {
        left.fitness.pareto_compare(&right.fitness)
    }
}
