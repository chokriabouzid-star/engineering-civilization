#![forbid(unsafe_code)]

//! Memory Query — "ماذا لو اخترنا البديل؟"
//!
//! Week 23 — Counterfactuals + Retrospective
//!
//! يُجيب على أسئلة مثل:
//! - هل البديل المرفوض كان سيكون أفضل؟
//! - كيف تطورت اللياقة عبر الـ iterations؟
//! - أي القرارات أشبه بلياقة معينة؟

use crate::graph::CausalMemoryGraph;
use crate::node::RejectedAlternative;
use crate::types::NodeId;
use chrono::{DateTime, Utc};
use ec_fitness::fitness::FitnessVector;
use ec_fitness::ParetoOrdering;
use serde::{Deserialize, Serialize};

/// كائن الاستعلام — يقرأ الذاكرة بدون تعديلها.
#[derive(Debug)]
pub struct MemoryQuery<'a> {
    graph: &'a CausalMemoryGraph,
}

impl<'a> MemoryQuery<'a> {
    /// إنشاء كائن استعلام من الذاكرة.
    pub fn new(graph: &'a CausalMemoryGraph) -> Self {
        Self { graph }
    }

    /// أفضل بديل مرفوض لقرار معين.
    ///
    /// يُرجع البديل ذا أعلى مجموع (security + test_coverage).
    pub fn best_rejected_alternative(&self, node_id: NodeId) -> Option<&RejectedAlternative> {
        let node = self.graph.get(node_id)?;
        node.alternatives.iter().max_by(|a, b| {
            let a_score = a.fitness.security + a.fitness.test_coverage;
            let b_score = b.fitness.security + b.fitness.test_coverage;
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// هل البديل المرفوض كان سيكون أفضل؟
    ///
    /// يُقارن بديل بمُختار عبر Pareto ordering.
    pub fn counterfactual_gain(
        &self,
        chosen: &FitnessVector,
        alternative: &FitnessVector,
    ) -> CounterfactualGain {
        match alternative.pareto_compare(chosen) {
            ParetoOrdering::Dominates => CounterfactualGain::AlternativeWasBetter {
                dimensions_better: count_better_dimensions(alternative, chosen),
            },
            ParetoOrdering::Dominated => CounterfactualGain::ChoiceWasCorrect,
            ParetoOrdering::Equal => CounterfactualGain::NoMeaningfulDifference,
            ParetoOrdering::NonDominated => CounterfactualGain::TradeoffDependent,
        }
    }

    /// تطور الـ fitness عبر الـ iterations لـ artifact معين.
    ///
    /// يُرجع لقطة لكل قرار مُسجّل لهذا الـ artifact_id،
    /// مرتبة ترتيباً زمنياً (حسب الإدراج).
    pub fn fitness_evolution(&self, artifact_id: &str) -> Vec<FitnessSnapshot> {
        self.graph
            .decisions_for_artifact(artifact_id)
            .iter()
            .enumerate()
            .map(|(i, n)| FitnessSnapshot {
                iteration: i + 1,
                fitness: n.fitness.clone(),
                was_accepted: n
                    .sandbox_outcome
                    .as_ref()
                    .map(|o| o.correctness > 0.99)
                    .unwrap_or(false),
                created_at: n.created_at,
            })
            .collect()
    }

    /// أشبه القرارات بـ fitness معين (بـ cosine similarity).
    ///
    /// لا يحتاج vector DB — حساب بسيط كافٍ للـ MVP.
    pub fn find_similar(&self, target: &FitnessVector, k: usize) -> Vec<SimilarDecision> {
        let mut scored: Vec<(f64, &crate::node::DecisionNode)> = self
            .graph
            .all()
            .iter()
            .map(|n| (n.fitness.cosine_similarity(target), n))
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(k)
            .map(|(similarity, node)| SimilarDecision {
                node_id: node.id,
                similarity,
                fitness: node.fitness.clone(),
                was_accepted: node
                    .sandbox_outcome
                    .as_ref()
                    .map(|o| o.correctness > 0.99)
                    .unwrap_or(false),
            })
            .collect()
    }

    /// Cosine similarity بين متجهي لياقة (delegate to FitnessVector).
    ///
    /// 1.0 = متطابقان، 0.0 = متعامدان، -1.0 = متعاكسان.
    pub fn cosine_similarity(a: &FitnessVector, b: &FitnessVector) -> f64 {
        a.cosine_similarity(b)
    }
}

// ─── الأنواع المُصدَّرة ────────────────────────────────────────────

/// نتيجة المقارنة الاستعادية — هل البديل كان أفضل؟
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CounterfactualGain {
    /// البديل كان أفضل (يُسيطر باريتو).
    AlternativeWasBetter {
        /// عدد الأبعاد الأفضل.
        dimensions_better: usize,
    },
    /// الاختيار كان صحيحاً (البديل مُسيطَر عليه).
    ChoiceWasCorrect,
    /// لا فرق ذو معنى.
    NoMeaningfulDifference,
    /// يعتمد على المفاضلة (non-dominated).
    TradeoffDependent,
}

/// لقطة لياقة في نقطة زمنية.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitnessSnapshot {
    /// رقم الـ iteration (1-based).
    pub iteration: usize,
    /// اللياقة في هذه النقطة.
    pub fitness: FitnessVector,
    /// هل قُبل هذا الـ iteration؟
    pub was_accepted: bool,
    /// متى أُنشئ.
    pub created_at: DateTime<Utc>,
}

/// قرار مشابه (من find_similar).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimilarDecision {
    /// معرف العقدة.
    pub node_id: NodeId,
    /// درجة التشابه.
    pub similarity: f64,
    /// اللياقة.
    pub fitness: FitnessVector,
    /// هل قُبل؟
    pub was_accepted: bool,
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn count_better_dimensions(a: &FitnessVector, b: &FitnessVector) -> usize {
    let mut count = 0;
    if a.security > b.security {
        count += 1;
    }
    if a.reversibility > b.reversibility {
        count += 1;
    }
    if a.test_coverage > b.test_coverage {
        count += 1;
    }
    if a.maintainability > b.maintainability {
        count += 1;
    }
    if a.performance > b.performance {
        count += 1;
    }
    if a.architectural_stability > b.architectural_stability {
        count += 1;
    }
    count
}
