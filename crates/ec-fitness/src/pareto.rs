use crate::fitness::FitnessVector;

#[derive(Debug, PartialEq)]
pub enum ParetoOrdering {
    Dominates,
    Dominated,
    NonDominated,
    Equal,
}

impl FitnessVector {
    pub fn pareto_compare(&self, other: &FitnessVector) -> ParetoOrdering {
        let self_better = self.security >= other.security
            && self.reversibility >= other.reversibility
            && self.test_coverage >= other.test_coverage
            && self.maintainability >= other.maintainability
            && self.performance >= other.performance
            && self.architectural_stability >= other.architectural_stability;

        let other_better = other.security >= self.security
            && other.reversibility >= self.reversibility
            && other.test_coverage >= self.test_coverage
            && other.maintainability >= self.maintainability
            && other.performance >= self.performance
            && other.architectural_stability >= self.architectural_stability;

        if self_better && other_better {
            ParetoOrdering::Equal
        } else if self_better {
            ParetoOrdering::Dominates
        } else if other_better {
            ParetoOrdering::Dominated
        } else {
            ParetoOrdering::NonDominated
        }
    }
}
