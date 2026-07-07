#![forbid(unsafe_code)]

//! Week 38 Gate — BayesianQuery (Analogies + Confidence)

use ec_fitness::FitnessVector;
use ec_memory::*;

fn make_graph_with_outcomes() -> (CausalMemoryGraph, SqliteStorage) {
    let graph = CausalMemoryGraph::new();
    let storage = SqliteStorage::in_memory_with_outcomes().unwrap();
    (graph, storage)
}

fn sample_fitness(sec: f64, cov: f64) -> FitnessVector {
    FitnessVector {
        security: sec,
        reversibility: 0.8,
        test_coverage: cov,
        maintainability: 0.7,
        performance: 0.8,
        architectural_stability: 0.7,
    }
}

fn add_node(graph: &mut CausalMemoryGraph, artifact: &str, fitness: FitnessVector) -> NodeId {
    graph
        .record_from_builder(crate::node::DecisionNodeBuilder::new(
            artifact,
            crate::types::ArtifactSnapshot::new("fn f() {}"),
            fitness,
        ))
        .unwrap()
}

// ─── Gate 1: empty graph ────────────────────────────────────────────

#[test]
fn w38_empty_graph_no_results() {
    let (graph, storage) = make_graph_with_outcomes();
    let q = BayesianQuery::new(&graph, &storage);
    let target = sample_fitness(0.9, 0.9);
    let results = q.find_similar_with_confidence(&target, 5).unwrap();
    assert!(results.is_empty());
}

// ─── Gate 2: with nodes, no outcomes ────────────────────────────────

#[test]
fn w38_nodes_without_outcomes_low_confidence() {
    let (mut graph, storage) = make_graph_with_outcomes();
    add_node(&mut graph, "a1", sample_fitness(0.9, 0.9));

    let q = BayesianQuery::new(&graph, &storage);
    let target = sample_fitness(0.9, 0.9);
    let results = q.find_similar_with_confidence(&target, 5).unwrap();

    assert_eq!(results.len(), 1);
    assert!(
        results[0].bayesian_confidence < 0.50,
        "got: {}",
        results[0].bayesian_confidence
    );
    assert!(results[0].total_observations == 0);
}

// ─── Gate 3: with outcomes ──────────────────────────────────────────

#[test]
fn w38_with_outcomes_higher_confidence() {
    let (mut graph, storage) = make_graph_with_outcomes();
    add_node(&mut graph, "a1", sample_fitness(0.9, 0.9));

    for _ in 0..15 {
        storage.record_outcome("a1", true, 0.9).unwrap();
    }

    let q = BayesianQuery::new(&graph, &storage);
    let target = sample_fitness(0.9, 0.9);
    let results = q.find_similar_with_confidence(&target, 5).unwrap();

    assert_eq!(results.len(), 1);
    assert!(
        results[0].bayesian_confidence > 0.45,
        "after 15 successes: {}",
        results[0].bayesian_confidence
    );
    assert_eq!(results[0].total_observations, 15);
}

// ─── Gate 4: combined = min(similarity, confidence) ────────────────

#[test]
fn w38_combined_is_min() {
    let (mut graph, storage) = make_graph_with_outcomes();
    add_node(&mut graph, "a1", sample_fitness(0.9, 0.9));
    storage.record_outcome("a1", true, 0.9).unwrap();

    let q = BayesianQuery::new(&graph, &storage);
    let target = sample_fitness(0.5, 0.5);
    let results = q.find_similar_with_confidence(&target, 5).unwrap();

    assert!(!results.is_empty());
    let r = &results[0];
    assert!(
        (r.combined - r.similarity.min(r.bayesian_confidence)).abs() < 0.001,
        "combined={:.3}, min({:.3},{:.3})",
        r.combined,
        r.similarity,
        r.bayesian_confidence
    );
}

// ─── Gate 5: ranked by similarity ──────────────────────────────────

#[test]
fn w38_ranked_by_similarity() {
    let (mut graph, storage) = make_graph_with_outcomes();
    add_node(&mut graph, "a1", sample_fitness(0.9, 0.9));
    add_node(&mut graph, "a2", sample_fitness(0.3, 0.3));

    for _ in 0..5 {
        storage.record_outcome("a1", true, 0.9).unwrap();
        storage.record_outcome("a2", true, 0.9).unwrap();
    }

    let q = BayesianQuery::new(&graph, &storage);
    let target = sample_fitness(0.9, 0.9);
    let results = q.find_similar_with_confidence(&target, 5).unwrap();

    assert!(results.len() >= 2);
    assert!(
        results[0].similarity >= results[1].similarity,
        "first={:.3} >= second={:.3}",
        results[0].similarity,
        results[1].similarity
    );
}

// ─── Gate 6: best_by_confidence ─────────────────────────────────────

#[test]
fn w38_best_by_confidence() {
    let (mut graph, storage) = make_graph_with_outcomes();
    add_node(&mut graph, "a1", sample_fitness(0.9, 0.9));
    add_node(&mut graph, "a2", sample_fitness(0.5, 0.5));

    for _ in 0..10 {
        storage.record_outcome("a1", true, 0.9).unwrap();
    }
    for _ in 0..10 {
        storage.record_outcome("a2", false, 0.1).unwrap();
    }

    let q = BayesianQuery::new(&graph, &storage);
    let best = q.best_by_confidence(2).unwrap();

    assert_eq!(best.len(), 2);
    assert!(best[0].bayesian_confidence >= best[1].bayesian_confidence);
    assert_eq!(best[0].artifact_id, "a1");
}

// ─── Gate 7: old query still works ──────────────────────────────────

#[test]
fn w38_old_query_still_works() {
    let (graph, _) = make_graph_with_outcomes();
    let q = MemoryQuery::new(&graph);
    let target = sample_fitness(0.9, 0.9);
    let results = q.find_similar(&target, 5);
    assert!(results.is_empty());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w38_gate_complete() {
    let (mut graph, storage) = make_graph_with_outcomes();
    add_node(&mut graph, "a1", sample_fitness(0.9, 0.9));
    add_node(&mut graph, "a2", sample_fitness(0.7, 0.7));
    add_node(&mut graph, "a3", sample_fitness(0.3, 0.3));

    for _ in 0..10 {
        storage.record_outcome("a1", true, 0.9).unwrap();
    }
    for _ in 0..5 {
        storage.record_outcome("a2", true, 0.8).unwrap();
    }
    for _ in 0..10 {
        storage.record_outcome("a3", false, 0.1).unwrap();
    }

    let q = BayesianQuery::new(&graph, &storage);
    let target = sample_fitness(0.9, 0.9);
    let results = q.find_similar_with_confidence(&target, 3).unwrap();

    println!("═══════════════════════════════════════════════");
    println!("  Week 38 Gate — BayesianQuery");
    println!("═══════════════════════════════════════════════");
    for r in &results {
        println!(
            "  {} sim={:.3} bayes={:.3} combined={:.3} obs={}",
            r.artifact_id, r.similarity, r.bayesian_confidence, r.combined, r.total_observations
        );
    }
    println!("═══════════════════════════════════════════════");

    assert_eq!(results.len(), 3);
    assert!(results[0].similarity >= results[1].similarity);

    println!("  ✅ Week 38 Gate: PASSED");
}
