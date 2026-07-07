#![deny(warnings)]
#![forbid(unsafe_code)]

//! Week 20 Gate — Causal Memory Foundation
//!
//! Gate Criteria:
//! ✓ Append-only enforced (no delete methods)
//! ✓ Retrospective: only mutable field
//! ✓ causal_chain() works
//! ✓ DAG validation (no cycles)
//! ✓ Builder pattern for external construction

use ec_fitness::fitness::FitnessVector;
use ec_memory::*;

fn dummy_artifact(code: &str) -> ArtifactSnapshot {
    ArtifactSnapshot::new(code)
}

fn dummy_fitness() -> FitnessVector {
    FitnessVector::default()
}

fn dummy_builder(id_str: &str, parents: Vec<NodeId>) -> DecisionNodeBuilder {
    DecisionNodeBuilder::new(id_str, dummy_artifact("fn main() {}"), dummy_fitness())
        .constitutional_valid(true)
        .causal_parents(parents)
}

// ─── Gate 1: Memory Creates ──────────────────────────────────────────

#[test]
fn gate_memory_creates() {
    let mem = CausalMemoryGraph::new();
    assert_eq!(mem.len(), 0);
    assert!(mem.is_empty());
}

// ─── Gate 2: Append-only ─────────────────────────────────────────────

#[test]
fn gate_append_only_no_delete() {
    let mut mem = CausalMemoryGraph::new();
    let id = mem
        .record_from_builder(dummy_builder("test", vec![]))
        .unwrap();

    // ❌ هذا لن يُترجم — لا توجد delete()
    // mem.delete(id);  // compile error

    assert_eq!(mem.len(), 1);
    assert!(mem.get(id).is_some());
}

#[test]
fn gate_record_increases_len() {
    let mut mem = CausalMemoryGraph::new();
    assert_eq!(mem.len(), 0);

    mem.record_from_builder(dummy_builder("a", vec![])).unwrap();
    assert_eq!(mem.len(), 1);

    mem.record_from_builder(dummy_builder("b", vec![])).unwrap();
    assert_eq!(mem.len(), 2);
}

// ─── Gate 3: Retrospective Only ──────────────────────────────────────

#[test]
fn gate_retrospective_is_mutable() {
    let mut mem = CausalMemoryGraph::new();
    let id = mem
        .record_from_builder(dummy_builder("test", vec![]))
        .unwrap();

    let assessment = RetrospectiveAssessment::new(true, 0.9, "better").unwrap();
    mem.update_retrospective(id, assessment).unwrap();

    let node = mem.get(id).unwrap();
    assert_eq!(node.retrospective.len(), 1);
}

#[test]
fn gate_retrospective_is_append_only_log() {
    let mut mem = CausalMemoryGraph::new();
    let id = mem
        .record_from_builder(dummy_builder("test", vec![]))
        .unwrap();

    let a1 = RetrospectiveAssessment::new(true, 0.8, "first").unwrap();
    let a2 = RetrospectiveAssessment::new(false, 0.9, "second").unwrap();

    mem.update_retrospective(id, a1).unwrap();
    mem.update_retrospective(id, a2).unwrap();

    let node = mem.get(id).unwrap();
    assert_eq!(node.retrospective.len(), 2);
}

// ─── Gate 4: Causal Chain ────────────────────────────────────────────

#[test]
fn gate_causal_chain_works() {
    let mut mem = CausalMemoryGraph::new();
    let id1 = mem.record_from_builder(dummy_builder("a", vec![])).unwrap();
    let id2 = mem
        .record_from_builder(dummy_builder("b", vec![id1]))
        .unwrap();
    let id3 = mem
        .record_from_builder(dummy_builder("c", vec![id2]))
        .unwrap();

    let chain = mem.causal_chain(id3);
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0].id, id3);
    assert_eq!(chain[1].id, id2);
    assert_eq!(chain[2].id, id1);
}

#[test]
fn gate_causal_chain_single_node() {
    let mut mem = CausalMemoryGraph::new();
    let id = mem
        .record_from_builder(dummy_builder("solo", vec![]))
        .unwrap();

    let chain = mem.causal_chain(id);
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].id, id);
}

// ─── Gate 5: Decisions for Artifact ──────────────────────────────────

#[test]
fn gate_decisions_for_artifact() {
    let mut mem = CausalMemoryGraph::new();
    mem.record_from_builder(dummy_builder("test-v1", vec![]))
        .unwrap();
    mem.record_from_builder(dummy_builder("test-v2", vec![]))
        .unwrap();
    mem.record_from_builder(dummy_builder("other", vec![]))
        .unwrap();

    let decisions = mem.decisions_for_artifact("test-v1");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].artifact_id, "test-v1");
}

// ─── Gate 6: Latest N ────────────────────────────────────────────────

#[test]
fn gate_latest_n() {
    let mut mem = CausalMemoryGraph::new();
    for i in 0..10 {
        mem.record_from_builder(dummy_builder(&format!("node-{}", i), vec![]))
            .unwrap();
    }

    let latest = mem.latest_n(3);
    assert_eq!(latest.len(), 3);
    assert_eq!(latest[0].artifact_id, "node-7");
    assert_eq!(latest[2].artifact_id, "node-9");
}

// ─── Gate 7: ArtifactSnapshot Sharing ────────────────────────────────

#[test]
fn gate_artifact_snapshot_shares_code() {
    let snap1 = ArtifactSnapshot::new("fn main() {}");
    let snap2 = snap1.clone();

    assert!(std::sync::Arc::ptr_eq(&snap1.code, &snap2.code));
}

#[test]
fn gate_artifact_snapshot_same_code_same_hash() {
    let s1 = ArtifactSnapshot::new("fn main() {}");
    let s2 = ArtifactSnapshot::new("fn main() {}");

    assert_eq!(s1.hash, s2.hash);
}

// ─── Gate 8: NodeId Uniqueness ───────────────────────────────────────

#[test]
fn gate_node_id_is_unique() {
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    assert_ne!(id1, id2);
}

// ─── Gate 9: RejectedAlternative ─────────────────────────────────────

#[test]
fn gate_rejected_alternative_creates() {
    let alt = RejectedAlternative::new(
        dummy_artifact("bad code"),
        dummy_fitness(),
        RejectionReason::CatastrophicFailure {
            dimension: "security".to_string(),
        },
    );

    assert!(alt.retrospective.is_empty());
}

#[test]
fn gate_rejected_alternative_retrospective() {
    let mut alt = RejectedAlternative::new(
        dummy_artifact("code"),
        dummy_fitness(),
        RejectionReason::ParetoDominated {
            dominated_by: NodeId::new(),
        },
    );

    let assessment = RetrospectiveAssessment::new(false, 0.7, "worse").unwrap();
    alt.add_retrospective(assessment);

    assert_eq!(alt.retrospective.len(), 1);
}

// ─── Gate 10: Alternative Retrospective Update ───────────────────────

#[test]
fn gate_update_alternative_retrospective() {
    let mut mem = CausalMemoryGraph::new();

    let alt = RejectedAlternative::new(
        dummy_artifact("alt"),
        dummy_fitness(),
        RejectionReason::SandboxFailure { correctness: 0.3 },
    );
    let alt_id = alt.id;

    let builder = dummy_builder("test", vec![]).add_alternative(alt);
    let node_id = mem.record_from_builder(builder).unwrap();

    let assessment = RetrospectiveAssessment::new(true, 0.85, "was better").unwrap();
    mem.update_alternative_retrospective(node_id, alt_id, assessment)
        .unwrap();

    let updated_node = mem.get(node_id).unwrap();
    assert_eq!(updated_node.alternatives[0].retrospective.len(), 1);
}

// ─── Final Gate ──────────────────────────────────────────────────────

#[test]
fn week20_gate_complete() {
    println!("═══════════════════════════════════════════");
    println!("  Week 20 Gate — Causal Memory Foundation");
    println!("═══════════════════════════════════════════");

    // 1. Memory creates
    let mut mem = CausalMemoryGraph::new();
    assert_eq!(mem.len(), 0);
    println!("✅ Gate 1: Memory creates");

    // 2. Append-only
    let id1 = mem.record_from_builder(dummy_builder("a", vec![])).unwrap();
    let id2 = mem
        .record_from_builder(dummy_builder("b", vec![id1]))
        .unwrap();
    assert_eq!(mem.len(), 2);
    println!("✅ Gate 2: Append-only (no delete)");

    // 3. Retrospective
    let assessment = RetrospectiveAssessment::new(true, 0.9, "test").unwrap();
    mem.update_retrospective(id1, assessment).unwrap();
    assert_eq!(mem.get(id1).unwrap().retrospective.len(), 1);
    println!("✅ Gate 3: Retrospective mutable");

    // 4. Causal chain
    let id3 = mem
        .record_from_builder(dummy_builder("c", vec![id2]))
        .unwrap();
    let chain = mem.causal_chain(id3);
    assert_eq!(chain.len(), 3);
    println!("✅ Gate 4: Causal chain works");

    // 5. ArtifactSnapshot sharing
    let snap1 = ArtifactSnapshot::new("fn main() {}");
    let snap2 = snap1.clone();
    assert!(std::sync::Arc::ptr_eq(&snap1.code, &snap2.code));
    println!("✅ Gate 5: ArtifactSnapshot shares Arc");

    // 6. Decisions for artifact
    mem.record_from_builder(dummy_builder("test-v1", vec![]))
        .unwrap();
    mem.record_from_builder(dummy_builder("test-v2", vec![]))
        .unwrap();
    let decisions = mem.decisions_for_artifact("test-v1");
    assert_eq!(decisions.len(), 1);
    println!("✅ Gate 6: Decisions for artifact");

    // 7. Latest N
    for i in 0..10 {
        mem.record_from_builder(dummy_builder(&format!("node-{}", i), vec![]))
            .unwrap();
    }
    let latest = mem.latest_n(5);
    assert_eq!(latest.len(), 5);
    println!("✅ Gate 7: Latest N");

    println!();
    println!("═══════════════════════════════════════════");
    println!("  ✅ Week 20 Gate: PASSED");
    println!("  Append-only memory with causal tracking");
    println!("═══════════════════════════════════════════");
}
