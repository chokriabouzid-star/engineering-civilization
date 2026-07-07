#![forbid(unsafe_code)]

//! عقدة القرار — قلب الذاكرة السببية.

use crate::types::{ArtifactSnapshot, NodeId, RejectionReason, RetrospectiveAssessment};
use chrono::{DateTime, Utc};
use ec_fitness::fitness::FitnessVector;
use serde::{Deserialize, Serialize};

/// نتيجة sandbox مُبسطة (لا تخترق Truth≠Fitness).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SandboxOutcome {
    /// هل نجحت الاختبارات؟
    pub correctness: f64,
    /// قابلية التكرار.
    pub reproducibility: f64,
    /// الثقة التجريبية.
    pub empirical_confidence: f64,
}

/// بديل مرفوض — الطريق الذي لم نسلكه.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectedAlternative {
    /// معرف البديل.
    pub id: NodeId,
    /// الكود المرفوض.
    pub artifact: ArtifactSnapshot,
    /// التقييم الدستوري.
    pub fitness: FitnessVector,
    /// سبب الرفض.
    pub reason: RejectionReason,
    /// متى رُفض.
    pub rejected_at: DateTime<Utc>,
    /// تقييمات استعادية (append-only log).
    pub retrospective: Vec<RetrospectiveAssessment>,
}

impl RejectedAlternative {
    /// إنشاء بديل مرفوض.
    pub fn new(
        artifact: ArtifactSnapshot,
        fitness: FitnessVector,
        reason: RejectionReason,
    ) -> Self {
        Self {
            id: NodeId::new(),
            artifact,
            fitness,
            reason,
            rejected_at: Utc::now(),
            retrospective: Vec::new(),
        }
    }

    /// إضافة تقييم استعادي.
    pub fn add_retrospective(&mut self, assessment: RetrospectiveAssessment) {
        self.retrospective.push(assessment);
    }
}

/// عقدة قرار — immutable ما عدا retrospective log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionNode {
    /// معرف فريد.
    pub id: NodeId,
    /// متى اُتخذ القرار.
    pub created_at: DateTime<Utc>,

    // ─── What ────────────────────────────────────────────────────
    /// معرف الـ artifact (للعرض).
    pub artifact_id: String,
    /// الكود.
    pub artifact: ArtifactSnapshot,
    /// التقييم الدستوري.
    pub fitness: FitnessVector,
    /// الحكم الدستوري.
    pub constitutional_valid: bool,
    /// نتيجة التنفيذ (إن وُجدت).
    pub sandbox_outcome: Option<SandboxOutcome>,

    // ─── Why ─────────────────────────────────────────────────────
    /// البدائل المرفوضة.
    pub alternatives: Vec<RejectedAlternative>,

    // ─── Where From ──────────────────────────────────────────────
    /// العقد الوالدة (causal parents).
    pub causal_parents: Vec<NodeId>,

    // ─── Mutable Interpretation ──────────────────────────────────
    /// تقييمات استعادية (append-only log).
    pub retrospective: Vec<RetrospectiveAssessment>,
}

impl DecisionNode {
    /// إنشاء عقدة جديدة.
    ///
    /// **pub(crate)** — لا يُنشأ خارج ec-memory.
    /// الاستخدام الخارجي عبر `DecisionNodeBuilder`.
    pub(crate) fn new(
        artifact_id: impl Into<String>,
        artifact: ArtifactSnapshot,
        fitness: FitnessVector,
        constitutional_valid: bool,
        sandbox_outcome: Option<SandboxOutcome>,
        causal_parents: Vec<NodeId>,
    ) -> Self {
        Self {
            id: NodeId::new(),
            created_at: Utc::now(),
            artifact_id: artifact_id.into(),
            artifact,
            fitness,
            constitutional_valid,
            sandbox_outcome,
            alternatives: Vec::new(),
            causal_parents,
            retrospective: Vec::new(),
        }
    }

    /// إضافة تقييم استعادي.
    pub fn add_retrospective(&mut self, assessment: RetrospectiveAssessment) {
        self.retrospective.push(assessment);
    }

    /// إضافة بديل مرفوض.
    pub fn add_alternative(&mut self, alternative: RejectedAlternative) {
        self.alternatives.push(alternative);
    }
}

// ═══════════════════════════════════════════════════════════════════
// DecisionNodeBuilder — الاستخدام الخارجي الوحيد لإنشاء القرارات
// ═══════════════════════════════════════════════════════════════════

/// باني القرار — الطريقة الوحيدة لإنشاء قرار من خارج ec-memory.
///
/// **Design Invariant:**
/// - `id` و `created_at` لا يُحددهما المستخدم
/// - الذاكرة تتحكم فيهما عبر `CausalMemoryGraph::record_from_builder()`
#[derive(Debug, Clone)]
pub struct DecisionNodeBuilder {
    /// معرف الـ artifact.
    pub artifact_id: String,
    /// الكود.
    pub artifact: ArtifactSnapshot,
    /// اللياقة.
    pub fitness: FitnessVector,
    /// هل اجتاز الدستور؟
    pub constitutional_valid: bool,
    /// نتيجة sandbox.
    pub sandbox_outcome: Option<SandboxOutcome>,
    /// الآباء السببيون.
    pub causal_parents: Vec<NodeId>,
    /// البدائل المرفوضة.
    pub alternatives: Vec<RejectedAlternative>,
}

impl DecisionNodeBuilder {
    /// إنشاء باني جديد.
    pub fn new(
        artifact_id: impl Into<String>,
        artifact: ArtifactSnapshot,
        fitness: FitnessVector,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            artifact,
            fitness,
            constitutional_valid: false,
            sandbox_outcome: None,
            causal_parents: Vec::new(),
            alternatives: Vec::new(),
        }
    }

    /// تعيين الحكم الدستوري.
    pub fn constitutional_valid(mut self, valid: bool) -> Self {
        self.constitutional_valid = valid;
        self
    }

    /// تعيين نتيجة sandbox.
    pub fn sandbox_outcome(mut self, outcome: Option<SandboxOutcome>) -> Self {
        self.sandbox_outcome = outcome;
        self
    }

    /// تعيين الآباء السببيين.
    pub fn causal_parents(mut self, parents: Vec<NodeId>) -> Self {
        self.causal_parents = parents;
        self
    }

    /// إضافة بديل مرفوض.
    pub fn add_alternative(mut self, alt: RejectedAlternative) -> Self {
        self.alternatives.push(alt);
        self
    }

    /// بناء العقدة (pub(crate) — تستدعيها record_from_builder فقط).
    pub(crate) fn build(self) -> DecisionNode {
        let mut node = DecisionNode::new(
            self.artifact_id,
            self.artifact,
            self.fitness,
            self.constitutional_valid,
            self.sandbox_outcome,
            self.causal_parents,
        );
        for alt in self.alternatives {
            node.add_alternative(alt);
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_artifact() -> ArtifactSnapshot {
        ArtifactSnapshot::new("fn main() {}")
    }

    fn dummy_fitness() -> FitnessVector {
        FitnessVector {
            security: 0.8,
            reversibility: 0.7,
            test_coverage: 0.6,
            maintainability: 0.7,
            performance: 0.5,
            architectural_stability: 0.6,
        }
    }

    #[test]
    fn decision_node_creates() {
        let node = DecisionNode::new(
            "test",
            dummy_artifact(),
            dummy_fitness(),
            true,
            None,
            vec![],
        );
        assert_eq!(node.artifact_id, "test");
        assert!(node.constitutional_valid);
        assert!(node.alternatives.is_empty());
        assert!(node.retrospective.is_empty());
    }

    #[test]
    fn decision_node_has_unique_id() {
        let n1 = DecisionNode::new("a", dummy_artifact(), dummy_fitness(), true, None, vec![]);
        let n2 = DecisionNode::new("b", dummy_artifact(), dummy_fitness(), true, None, vec![]);
        assert_ne!(n1.id, n2.id);
    }

    #[test]
    fn decision_node_add_retrospective() {
        let mut node = DecisionNode::new(
            "test",
            dummy_artifact(),
            dummy_fitness(),
            true,
            None,
            vec![],
        );
        let assessment = RetrospectiveAssessment::new(true, 0.9, "better").unwrap();

        node.add_retrospective(assessment.clone());
        assert_eq!(node.retrospective.len(), 1);
        assert_eq!(node.retrospective[0].confidence, 0.9);
    }

    #[test]
    fn rejected_alternative_creates() {
        let alt = RejectedAlternative::new(
            dummy_artifact(),
            dummy_fitness(),
            RejectionReason::CatastrophicFailure {
                dimension: "security".to_string(),
            },
        );
        assert!(alt.retrospective.is_empty());
    }

    #[test]
    fn rejected_alternative_add_retrospective() {
        let mut alt = RejectedAlternative::new(
            dummy_artifact(),
            dummy_fitness(),
            RejectionReason::ParetoDominated {
                dominated_by: NodeId::new(),
            },
        );

        let assessment = RetrospectiveAssessment::new(false, 0.8, "worse").unwrap();
        alt.add_retrospective(assessment);
        assert_eq!(alt.retrospective.len(), 1);
    }

    #[test]
    fn builder_creates_node() {
        let builder = DecisionNodeBuilder::new("test-builder", dummy_artifact(), dummy_fitness())
            .constitutional_valid(true)
            .sandbox_outcome(Some(SandboxOutcome {
                correctness: 1.0,
                reproducibility: 0.95,
                empirical_confidence: 0.85,
            }))
            .causal_parents(vec![]);

        let node = builder.build();
        assert_eq!(node.artifact_id, "test-builder");
        assert!(node.constitutional_valid);
        assert!(node.sandbox_outcome.is_some());
        assert!(node.causal_parents.is_empty());
    }

    #[test]
    fn builder_with_alternative() {
        let alt = RejectedAlternative::new(
            dummy_artifact(),
            dummy_fitness(),
            RejectionReason::SandboxFailure { correctness: 0.5 },
        );

        let builder = DecisionNodeBuilder::new("with-alt", dummy_artifact(), dummy_fitness())
            .add_alternative(alt);

        let node = builder.build();
        assert_eq!(node.alternatives.len(), 1);
    }

    #[test]
    fn builder_id_is_controlled_by_memory() {
        // id و created_at لا يُحددهما المستخدم
        let b1 = DecisionNodeBuilder::new("a", dummy_artifact(), dummy_fitness()).build();
        let b2 = DecisionNodeBuilder::new("a", dummy_artifact(), dummy_fitness()).build();
        assert_ne!(b1.id, b2.id, "Each build() should produce a unique id");
    }
}
