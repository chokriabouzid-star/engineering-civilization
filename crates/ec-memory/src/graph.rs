#![forbid(unsafe_code)]

//! Causal Memory Graph — append-only history.

use crate::node::{DecisionNode, DecisionNodeBuilder};
use crate::types::{NodeId, RetrospectiveAssessment};
use serde::{Deserialize, Serialize};

/// أخطاء الذاكرة.
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    /// عقدة غير موجودة.
    #[error("Node not found: {0}")]
    NodeNotFound(NodeId),

    /// محاولة إنشاء دورة.
    #[error("Cycle detected: {node} → parents")]
    CycleDetected {
        /// العقدة التي ستُنشئ الدورة.
        node: NodeId,
    },
}

/// Causal Memory Graph — الذاكرة السببية.
///
/// **Design Invariant:**
/// - Append-only: لا delete(), لا update()
/// - Vec بدل HashMap: الترتيب الزمني مهم
/// - DAG enforced: لا دورات
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalMemoryGraph {
    /// العقد (append-only).
    nodes: Vec<DecisionNode>,
}

impl CausalMemoryGraph {
    /// إنشاء ذاكرة جديدة فارغة.
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// إنشاء من عقد محملة (pub(crate) — للتخزين فقط).
    ///
    /// يتجاوز validate_acyclic لأن البيانات خُزنت بعد التحقق.
    pub(crate) fn from_nodes(nodes: Vec<DecisionNode>) -> Self {
        Self { nodes }
    }

    /// عدد القرارات المُسجلة.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// هل الذاكرة فارغة؟
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// ✅ المسموح: تسجيل قرار جديد.
    ///
    /// **Validates acyclic property.**
    pub fn record(&mut self, node: DecisionNode) -> Result<NodeId, MemoryError> {
        self.validate_acyclic(&node)?;

        let id = node.id;
        self.nodes.push(node);
        Ok(id)
    }

    /// ✅ المسموح: تسجيل قرار من Builder.
    ///
    /// الطريقة الموصىى بها لإنشاء قرارات من خارج ec-memory.
    /// id و created_at يُنشآن هنا — لا يتحكم فيهما المستخدم.
    pub fn record_from_builder(
        &mut self,
        builder: DecisionNodeBuilder,
    ) -> Result<NodeId, MemoryError> {
        let node = builder.build();
        self.record(node)
    }

    /// ✅ المسموح: التحديث الاستعادي فقط.
    pub fn update_retrospective(
        &mut self,
        id: NodeId,
        assessment: RetrospectiveAssessment,
    ) -> Result<(), MemoryError> {
        self.nodes
            .iter_mut()
            .find(|n| n.id == id)
            .ok_or(MemoryError::NodeNotFound(id))?
            .add_retrospective(assessment);
        Ok(())
    }

    /// ✅ المسموح: تحديث البديل المرفوض.
    pub fn update_alternative_retrospective(
        &mut self,
        node_id: NodeId,
        alternative_id: NodeId,
        assessment: RetrospectiveAssessment,
    ) -> Result<(), MemoryError> {
        let node = self
            .nodes
            .iter_mut()
            .find(|n| n.id == node_id)
            .ok_or(MemoryError::NodeNotFound(node_id))?;

        let alt = node
            .alternatives
            .iter_mut()
            .find(|a| a.id == alternative_id)
            .ok_or(MemoryError::NodeNotFound(alternative_id))?;

        alt.add_retrospective(assessment);
        Ok(())
    }

    // ❌ الممنوع: لا توجد هذه الـ methods
    // fn delete(&mut self, id: NodeId) { }
    // fn update_fitness(&mut self, ...) { }
    // fn clear(&mut self) { }

    /// ✅ القراءة: الحصول على عقدة.
    pub fn get(&self, id: NodeId) -> Option<&DecisionNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// ✅ القراءة: كل العقد.
    pub fn all(&self) -> &[DecisionNode] {
        &self.nodes
    }

    /// ✅ القراءة: السلسلة السببية.
    pub fn causal_chain(&self, id: NodeId) -> Vec<&DecisionNode> {
        let mut chain = Vec::new();
        let mut current_id = Some(id);

        while let Some(cid) = current_id {
            if let Some(node) = self.get(cid) {
                chain.push(node);
                current_id = node.causal_parents.first().copied();
            } else {
                break;
            }
        }

        chain
    }

    /// ✅ القراءة: قرارات artifact محدد.
    pub fn decisions_for_artifact(&self, artifact_id: &str) -> Vec<&DecisionNode> {
        self.nodes
            .iter()
            .filter(|n| n.artifact_id == artifact_id)
            .collect()
    }

    /// ✅ القراءة: آخر N قرار.
    pub fn latest_n(&self, n: usize) -> &[DecisionNode] {
        let start = self.nodes.len().saturating_sub(n);
        &self.nodes[start..]
    }

    /// التحقق من عدم وجود دورات (improved).
    fn validate_acyclic(&self, node: &DecisionNode) -> Result<(), MemoryError> {
        // حالة 1: node.id موجود بالفعل (تسجيل مكرر)
        if self.get(node.id).is_some() {
            return Err(MemoryError::CycleDetected { node: node.id });
        }

        for parent_id in &node.causal_parents {
            // حالة 2: العقدة تشير لنفسها
            if *parent_id == node.id {
                return Err(MemoryError::CycleDetected { node: node.id });
            }

            // حالة 3: الـ parent غير موجود في الـ graph
            if self.get(*parent_id).is_none() {
                return Err(MemoryError::NodeNotFound(*parent_id));
            }
        }
        Ok(())
    }
}

impl Default for CausalMemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::types::ArtifactSnapshot;
    use ec_fitness::fitness::FitnessVector;

    fn dummy_node(id_str: &str, parents: Vec<NodeId>) -> DecisionNode {
        DecisionNode::new(
            id_str,
            ArtifactSnapshot::new("fn main() {}"),
            FitnessVector::default(),
            true,
            None,
            parents,
        )
    }

    #[test]
    fn graph_starts_empty() {
        let g = CausalMemoryGraph::new();
        assert_eq!(g.len(), 0);
        assert!(g.is_empty());
    }

    #[test]
    fn graph_record_adds_node() {
        let mut g = CausalMemoryGraph::new();
        let node = dummy_node("test", vec![]);
        let id = g.record(node).unwrap();

        assert_eq!(g.len(), 1);
        assert!(g.get(id).is_some());
    }

    #[test]
    fn graph_record_multiple_nodes() {
        let mut g = CausalMemoryGraph::new();
        let id1 = g.record(dummy_node("a", vec![])).unwrap();
        let id2 = g.record(dummy_node("b", vec![id1])).unwrap();
        let id3 = g.record(dummy_node("c", vec![id2])).unwrap();

        assert_eq!(g.len(), 3);
        assert!(g.get(id1).is_some());
        assert!(g.get(id2).is_some());
        assert!(g.get(id3).is_some());
    }

    #[test]
    fn graph_prevents_cycles_self_reference() {
        let mut g = CausalMemoryGraph::new();
        let node = dummy_node("first", vec![]);
        let node_id = node.id;
        g.record(node).unwrap();

        // محاولة إنشاء عقدة تشير لنفسها
        let mut self_ref_node = dummy_node("self-ref", vec![node_id]);
        self_ref_node.id = node_id; // نفس الـ id

        let result = g.record(self_ref_node);
        assert!(result.is_err());
        assert!(matches!(result, Err(MemoryError::CycleDetected { .. })));
    }

    #[test]
    fn graph_prevents_missing_parent() {
        let mut g = CausalMemoryGraph::new();
        let fake_parent = NodeId::new(); // لا يوجد في الـ graph
        let node = dummy_node("orphan", vec![fake_parent]);

        let result = g.record(node);
        assert!(result.is_err());
        assert!(matches!(result, Err(MemoryError::NodeNotFound(_))));
    }

    #[test]
    fn graph_prevents_duplicate_registration() {
        let mut g = CausalMemoryGraph::new();
        let node = dummy_node("test", vec![]);
        let node_clone = node.clone();

        g.record(node).unwrap();

        // محاولة تسجيل نفس الـ id مرة أخرى
        let result = g.record(node_clone);
        assert!(result.is_err());
        assert!(matches!(result, Err(MemoryError::CycleDetected { .. })));
    }

    #[test]
    fn graph_get_returns_none_for_missing() {
        let g = CausalMemoryGraph::new();
        let id = NodeId::new();
        assert!(g.get(id).is_none());
    }

    #[test]
    fn graph_update_retrospective() {
        let mut g = CausalMemoryGraph::new();
        let id = g.record(dummy_node("test", vec![])).unwrap();

        let assessment = RetrospectiveAssessment::new(true, 0.9, "better").unwrap();
        assert!(g.update_retrospective(id, assessment).is_ok());

        let node = g.get(id).unwrap();
        assert_eq!(node.retrospective.len(), 1);
    }

    #[test]
    fn graph_update_retrospective_missing_node() {
        let mut g = CausalMemoryGraph::new();
        let id = NodeId::new();
        let assessment = RetrospectiveAssessment::new(true, 0.9, "test").unwrap();

        assert!(matches!(
            g.update_retrospective(id, assessment),
            Err(MemoryError::NodeNotFound(_))
        ));
    }

    #[test]
    fn graph_causal_chain() {
        let mut g = CausalMemoryGraph::new();
        let id1 = g.record(dummy_node("a", vec![])).unwrap();
        let id2 = g.record(dummy_node("b", vec![id1])).unwrap();
        let id3 = g.record(dummy_node("c", vec![id2])).unwrap();

        let chain = g.causal_chain(id3);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].id, id3);
        assert_eq!(chain[1].id, id2);
        assert_eq!(chain[2].id, id1);
    }

    #[test]
    fn graph_decisions_for_artifact() {
        let mut g = CausalMemoryGraph::new();
        g.record(dummy_node("test-1", vec![])).unwrap();
        g.record(dummy_node("test-2", vec![])).unwrap();
        g.record(dummy_node("other", vec![])).unwrap();

        let decisions = g.decisions_for_artifact("test-1");
        assert_eq!(decisions.len(), 1);
    }

    #[test]
    fn graph_latest_n() {
        let mut g = CausalMemoryGraph::new();
        for i in 0..10 {
            g.record(dummy_node(&format!("node-{}", i), vec![]))
                .unwrap();
        }

        let latest = g.latest_n(3);
        assert_eq!(latest.len(), 3);
        assert_eq!(latest[0].artifact_id, "node-7");
        assert_eq!(latest[2].artifact_id, "node-9");
    }

    #[test]
    fn graph_all_returns_slice() {
        let mut g = CausalMemoryGraph::new();
        g.record(dummy_node("a", vec![])).unwrap();
        g.record(dummy_node("b", vec![])).unwrap();

        assert_eq!(g.all().len(), 2);
    }
}
