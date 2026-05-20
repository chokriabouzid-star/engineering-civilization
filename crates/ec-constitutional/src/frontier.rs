use crate::constitution::Constitution;
use crate::evaluation::ConstitutionalEvaluation;
use ec_fitness::ParetoOrdering;

impl Constitution {
    pub fn build_frontier(
        &self,
        evaluations: &[ConstitutionalEvaluation],
    ) -> Vec<ConstitutionalEvaluation> {
        let valid: Vec<&ConstitutionalEvaluation> =
            evaluations.iter().filter(|e| e.is_valid).collect();

        let mut frontier: Vec<ConstitutionalEvaluation> = Vec::new();

        for candidate in valid {
            let is_dominated = frontier.iter().any(|existing| {
                let ordering = self.compare(existing, candidate);
                matches!(ordering, ParetoOrdering::Dominates)
            });

            if !is_dominated {
                frontier.retain(|existing| {
                    let ordering = self.compare(candidate, existing);
                    !matches!(ordering, ParetoOrdering::Dominates)
                });
                frontier.push(candidate.clone());
            }
        }

        frontier
    }
}
