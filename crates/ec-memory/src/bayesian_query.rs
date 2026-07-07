#![forbid(unsafe_code)]

//! Bayesian Query — استعلامات مشابهة مع ثقة Bayesian
//! Week 38 — إضافة فقط

use crate::graph::CausalMemoryGraph;
use crate::storage::StorageError;
use crate::types::NodeId;
use crate::OutcomeStorage;
use ec_fitness::FitnessVector;

/// استعلامات مدعومة بـ Bayesian evidence
pub struct BayesianQuery<'a, S: OutcomeStorage> {
    graph: &'a CausalMemoryGraph,
    storage: &'a S,
}

impl<'a, S: OutcomeStorage> BayesianQuery<'a, S> {
    /// إنشاء كائن استعلام
    pub fn new(graph: &'a CausalMemoryGraph, storage: &'a S) -> Self {
        Self { graph, storage }
    }

    /// أشبه القرارات مع Bayesian confidence
    pub fn find_similar_with_confidence(
        &self,
        target: &FitnessVector,
        k: usize,
    ) -> Result<Vec<BayesianSimilarDecision>, StorageError> {
        let mut scored: Vec<(f64, &crate::node::DecisionNode)> = self
            .graph
            .all()
            .iter()
            .map(|n| (n.fitness.cosine_similarity(target), n))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut results = Vec::new();
        for (similarity, node) in scored.into_iter().take(k) {
            let evidence = self.storage.load_evidence(&node.artifact_id)?;
            let bayesian_conf = evidence.credible_confidence();
            let combined = similarity.min(bayesian_conf);

            results.push(BayesianSimilarDecision {
                node_id: node.id,
                artifact_id: node.artifact_id.clone(),
                similarity,
                bayesian_confidence: bayesian_conf,
                combined,
                fitness: node.fitness.clone(),
                was_accepted: node
                    .sandbox_outcome
                    .as_ref()
                    .map(|o| o.correctness > 0.99)
                    .unwrap_or(false),
                total_observations: evidence.total_observations(),
            });
        }

        Ok(results)
    }

    /// أفضل القرارات حسب Bayesian confidence
    pub fn best_by_confidence(
        &self,
        k: usize,
    ) -> Result<Vec<BayesianSimilarDecision>, StorageError> {
        let nodes: Vec<_> = self.graph.all().to_vec();

        let mut scored = Vec::new();
        for node in &nodes {
            let evidence = self.storage.load_evidence(&node.artifact_id)?;
            let conf = evidence.credible_confidence();
            let was_accepted = node
                .sandbox_outcome
                .as_ref()
                .map(|o| o.correctness > 0.99)
                .unwrap_or(false);
            scored.push((
                conf,
                node.id,
                node.artifact_id.clone(),
                node.fitness.clone(),
                was_accepted,
                evidence.total_observations(),
            ));
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scored
            .into_iter()
            .take(k)
            .map(
                |(bayesian_conf, node_id, artifact_id, fitness, was_accepted, total_obs)| {
                    BayesianSimilarDecision {
                        node_id,
                        artifact_id: artifact_id.clone(),
                        similarity: 1.0,
                        bayesian_confidence: bayesian_conf,
                        combined: bayesian_conf,
                        fitness,
                        was_accepted,
                        total_observations: total_obs,
                    }
                },
            )
            .collect())
    }
}

/// قرار مشابه مع Bayesian confidence
#[derive(Debug, Clone)]
pub struct BayesianSimilarDecision {
    /// معرف العقدة
    pub node_id: NodeId,
    /// معرف الـ artifact
    pub artifact_id: String,
    /// تشابه cosine
    pub similarity: f64,
    /// ثقة Bayesian
    pub bayesian_confidence: f64,
    /// min(similarity, bayesian_confidence)
    pub combined: f64,
    /// اللياقة
    pub fitness: FitnessVector,
    /// هل قُبل؟
    pub was_accepted: bool,
    /// عدد المشاهدات
    pub total_observations: u32,
}
