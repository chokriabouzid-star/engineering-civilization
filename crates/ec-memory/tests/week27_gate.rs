#![forbid(unsafe_code)]

//! Week 27 Gate — SQLite Persistence

use ec_fitness::fitness::FitnessVector;
use ec_memory::{
    ArtifactSnapshot, CausalMemoryGraph, DecisionNodeBuilder, MemoryStorage,
    RetrospectiveAssessment, SqliteStorage,
};

fn make_builder(name: &str) -> DecisionNodeBuilder {
    let snap = ArtifactSnapshot::new(format!("fn {}() {{}}", name));
    let fitness = FitnessVector {
        security: 0.8,
        ..Default::default()
    };
    DecisionNodeBuilder::new(name, snap, fitness).constitutional_valid(true)
}

fn make_fitness(security: f64, coverage: f64) -> FitnessVector {
    FitnessVector {
        security,
        test_coverage: coverage,
        ..Default::default()
    }
}

// ─── Gate 1: Empty graph roundtrip ──────────────────────
#[test]
fn gate_storage_saves_and_loads_empty_graph() {
    let storage = SqliteStorage::in_memory().unwrap();
    let graph = CausalMemoryGraph::new();
    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();
    assert!(loaded.is_empty());
}

// ─── Gate 2: Persist + reload decisions ──────────────────
#[test]
fn gate_storage_persists_decisions() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();

    let id1 = graph.record_from_builder(make_builder("add")).unwrap();
    let id2 = graph
        .record_from_builder(make_builder("sub").causal_parents(vec![id1]))
        .unwrap();

    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();

    assert_eq!(loaded.len(), 2);
    assert!(loaded.get(id1).is_some(), "id1 must survive roundtrip");
    assert!(loaded.get(id2).is_some(), "id2 must survive roundtrip");
}

// ─── Gate 3: artifact_id preserved ──────────────────────
#[test]
fn gate_storage_preserves_artifact_id() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let id = graph.record_from_builder(make_builder("compute")).unwrap();
    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();
    assert_eq!(loaded.get(id).unwrap().artifact_id, "compute");
}

// ─── Gate 4: fitness preserved with precision ───────────
#[test]
fn gate_storage_preserves_fitness() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let snap = ArtifactSnapshot::new("fn f() {}");
    let fitness = make_fitness(0.9, 0.7);
    let id = graph
        .record_from_builder(DecisionNodeBuilder::new("f", snap, fitness))
        .unwrap();
    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();
    let lf = &loaded.get(id).unwrap().fitness;
    assert!((lf.security - 0.9).abs() < 1e-6);
    assert!((lf.test_coverage - 0.7).abs() < 1e-6);
}

// ─── Gate 5: causal chain after reload ──────────────────
#[test]
fn gate_storage_preserves_causal_chain() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let id1 = graph.record_from_builder(make_builder("a")).unwrap();
    let id2 = graph
        .record_from_builder(make_builder("b").causal_parents(vec![id1]))
        .unwrap();
    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();
    let chain = loaded.causal_chain(id2);
    assert_eq!(chain.len(), 2, "chain must be b → a");
    assert_eq!(chain[0].id, id2);
    assert_eq!(chain[1].id, id1);
}

// ─── Gate 6: append_node is idempotent ──────────────────
#[test]
fn gate_storage_append_node_is_incremental() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let id = graph.record_from_builder(make_builder("first")).unwrap();
    storage.append_node(graph.get(id).unwrap()).unwrap();
    storage.append_node(graph.get(id).unwrap()).unwrap(); // OR IGNORE
    let loaded = storage.load().unwrap();
    assert_eq!(loaded.len(), 1);
}

// ─── Gate 7: retrospective stored and loaded ────────────
#[test]
fn gate_storage_saves_and_loads_retrospective() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let id = graph.record_from_builder(make_builder("fn_x")).unwrap();

    let a1 = RetrospectiveAssessment::new(true, 0.8, "hindsight").unwrap();
    let a2 = RetrospectiveAssessment::new(false, 0.6, "reconsidered").unwrap();
    graph.update_retrospective(id, a1).unwrap();
    graph.update_retrospective(id, a2).unwrap();

    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();

    let node = loaded.get(id).unwrap();
    assert_eq!(node.retrospective.len(), 2);
    assert!(node.retrospective[0].was_better_choice);
    assert!(!node.retrospective[1].was_better_choice);
}

// ─── Gate 8: real file (not :memory:) ──────────────────
#[test]
fn gate_storage_works_with_real_file() {
    let path = "/tmp/ec_memory_test_week27.db";
    let _ = std::fs::remove_file(path);

    {
        let storage = SqliteStorage::new(path).unwrap();
        let mut graph = CausalMemoryGraph::new();
        let _id = graph
            .record_from_builder(make_builder("persist_fn"))
            .unwrap();
        storage.save(&graph).unwrap();
        assert!(std::path::Path::new(path).exists());
    }

    {
        let storage = SqliteStorage::new(path).unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.len(), 1);
        let decisions = loaded.decisions_for_artifact("persist_fn");
        assert_eq!(decisions.len(), 1);
    }

    let _ = std::fs::remove_file(path);
}

// ─── Gate 9: code preserved ────────────────────────────
#[test]
fn gate_storage_preserves_code() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let code = "fn special() -> i32 { 42 }";
    let snap = ArtifactSnapshot::new(code);
    let fitness = FitnessVector::default();
    let id = graph
        .record_from_builder(DecisionNodeBuilder::new("special", snap, fitness))
        .unwrap();
    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();
    let loaded_code = loaded.get(id).unwrap().artifact.code();
    assert_eq!(loaded_code, code);
}

// ─── Gate 10: constitutional_valid preserved ───────────
#[test]
fn gate_storage_preserves_constitutional_valid() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();

    let snap = ArtifactSnapshot::new("fn f() {}");
    let fitness = FitnessVector::default();
    let id_valid = graph
        .record_from_builder(
            DecisionNodeBuilder::new("valid", snap.clone(), fitness.clone())
                .constitutional_valid(true),
        )
        .unwrap();
    let id_invalid = graph
        .record_from_builder(
            DecisionNodeBuilder::new("invalid", snap, fitness).constitutional_valid(false),
        )
        .unwrap();

    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();

    assert!(loaded.get(id_valid).unwrap().constitutional_valid);
    assert!(!loaded.get(id_invalid).unwrap().constitutional_valid);
}

// ─── Final Gate ─────────────────────────────────────────
#[test]
fn week27_gate_complete() {
    let storage = SqliteStorage::in_memory().unwrap();
    let mut graph = CausalMemoryGraph::new();
    let id = graph.record_from_builder(make_builder("gate_fn")).unwrap();

    let a = RetrospectiveAssessment::new(true, 0.9, "all good").unwrap();
    graph.update_retrospective(id, a).unwrap();

    storage.save(&graph).unwrap();
    let loaded = storage.load().unwrap();

    assert_eq!(loaded.len(), 1);
    assert!(loaded.get(id).is_some());
    assert_eq!(loaded.get(id).unwrap().retrospective.len(), 1);

    println!("✅ Week 27: SQLite persistence — PASSED");
    println!("   save + load roundtrip      ✅");
    println!("   fitness precision          ✅");
    println!("   causal chain               ✅");
    println!("   retrospective              ✅");
    println!("   code preserved             ✅");
    println!("   real file + :memory:       ✅");
}
