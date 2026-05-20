#![forbid(unsafe_code)]

//! Week 23 Gate — Counterfactuals + Retrospective
//!
//! يتحقق من:
//! 1. MemoryQuery يقرأ الذاكرة بدون تعديلها
//! 2. counterfactual_gain() للـ 4 حالات
//! 3. fitness_evolution() يتتبع iterations
//! 4. find_similar() بـ cosine similarity
//! 5. cosine_similarity() صحيحة رياضياً

use ec_fitness::fitness::FitnessVector;
use ec_memory::{
    ArtifactSnapshot, CausalMemoryGraph, CounterfactualGain,
    DecisionNodeBuilder, MemoryQuery, NodeId, SandboxOutcome,
};

// ─── Helpers ─────────────────────────────────────────────────────────

fn zero_fitness() -> FitnessVector {
    FitnessVector::default()
}

fn high_security() -> FitnessVector {
    FitnessVector {
        security: 0.95,
        reversibility: 0.9,
        test_coverage: 0.9,
        maintainability: 0.9,
        performance: 0.9,
        architectural_stability: 0.9,
    }
}

fn low_security() -> FitnessVector {
    FitnessVector {
        security: 0.2,
        reversibility: 0.2,
        test_coverage: 0.2,
        maintainability: 0.2,
        performance: 0.2,
        architectural_stability: 0.2,
    }
}

fn make_builder(
    artifact: &str,
    fitness: FitnessVector,
    parent: Option<NodeId>,
) -> DecisionNodeBuilder {
    DecisionNodeBuilder::new(
        artifact,
        ArtifactSnapshot::new("fn main() {}"),
        fitness,
    )
    .constitutional_valid(true)
    .causal_parents(parent.into_iter().collect())
}

fn make_builder_with_outcome(
    artifact: &str,
    fitness: FitnessVector,
    correctness: f64,
    parent: Option<NodeId>,
) -> DecisionNodeBuilder {
    DecisionNodeBuilder::new(
        artifact,
        ArtifactSnapshot::new("fn main() {}"),
        fitness,
    )
    .constitutional_valid(true)
    .sandbox_outcome(Some(SandboxOutcome {
        correctness,
        reproducibility: 0.98,
        empirical_confidence: 0.9,
    }))
    .causal_parents(parent.into_iter().collect())
}

fn build_graph_with_iterations(
    artifact: &str,
    n: usize,
) -> CausalMemoryGraph {
    let mut g = CausalMemoryGraph::new();
    let mut parent: Option<NodeId> = None;
    for i in 0..n {
        let fitness = FitnessVector {
            security: 0.5 + i as f64 * 0.1,
            reversibility: 0.5,
            test_coverage: 0.5,
            maintainability: 0.5,
            performance: 0.5,
            architectural_stability: 0.5,
        };
        let builder = make_builder(artifact, fitness, parent);
        parent = Some(g.record_from_builder(builder).unwrap());
    }
    g
}

// ─── Gate 1: fitness_evolution ───────────────────────────────────────

#[test]
fn query_fitness_evolution_empty_graph() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    assert!(q.fitness_evolution("anything").is_empty());
}

#[test]
fn query_fitness_evolution_tracks_iterations() {
    let g = build_graph_with_iterations("add_fn", 3);
    let q = MemoryQuery::new(&g);
    let evo = q.fitness_evolution("add_fn");
    assert_eq!(evo.len(), 3);
    assert_eq!(evo[0].iteration, 1);
    assert_eq!(evo[2].iteration, 3);
}

#[test]
fn query_fitness_evolution_security_increases() {
    let g = build_graph_with_iterations("fn_x", 3);
    let q = MemoryQuery::new(&g);
    let evo = q.fitness_evolution("fn_x");
    assert!(
        evo[2].fitness.security > evo[0].fitness.security,
        "security should increase: {} → {}",
        evo[0].fitness.security,
        evo[2].fitness.security
    );
}

#[test]
fn query_fitness_evolution_was_accepted() {
    let mut g = CausalMemoryGraph::new();
    // node with correctness 1.0 → accepted
    let b1 = make_builder_with_outcome("fn_a", high_security(), 1.0, None);
    g.record_from_builder(b1).unwrap();
    // node with correctness 0.0 → not accepted
    let b2 = make_builder_with_outcome("fn_a", low_security(), 0.0, None);
    g.record_from_builder(b2).unwrap();

    let q = MemoryQuery::new(&g);
    let evo = q.fitness_evolution("fn_a");
    assert!(evo[0].was_accepted);
    assert!(!evo[1].was_accepted);
}

// ─── Gate 2: find_similar ────────────────────────────────────────────

#[test]
fn query_find_similar_returns_at_most_k() {
    let g = build_graph_with_iterations("fn", 10);
    let q = MemoryQuery::new(&g);
    let similar = q.find_similar(&high_security(), 3);
    assert!(similar.len() <= 3);
}

#[test]
fn query_find_similar_sorted_by_similarity() {
    let g = build_graph_with_iterations("fn", 5);
    let q = MemoryQuery::new(&g);
    let similar = q.find_similar(&high_security(), 5);
    for i in 1..similar.len() {
        assert!(
            similar[i - 1].similarity >= similar[i].similarity,
            "not sorted: {} >= {}",
            similar[i - 1].similarity,
            similar[i].similarity
        );
    }
}

#[test]
fn query_find_similar_empty_graph_returns_empty() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    assert!(q.find_similar(&high_security(), 5).is_empty());
}

#[test]
fn find_similar_k_zero_returns_empty() {
    let g = build_graph_with_iterations("fn", 5);
    let q = MemoryQuery::new(&g);
    let similar = q.find_similar(&high_security(), 0);
    assert!(similar.is_empty());
}

#[test]
fn find_similar_k_larger_than_graph_returns_all() {
    let g = build_graph_with_iterations("fn", 3);
    let q = MemoryQuery::new(&g);
    let similar = q.find_similar(&high_security(), 100);
    assert_eq!(similar.len(), 3);
}

#[test]
fn find_similar_has_valid_fields() {
    let g = build_graph_with_iterations("fn", 3);
    let q = MemoryQuery::new(&g);
    let similar = q.find_similar(&high_security(), 2);
    for s in &similar {
        assert!(!s.node_id.to_string().is_empty());
        assert!(s.similarity >= -1.0 && s.similarity <= 1.0);
    }
}

// ─── Gate 3: cosine_similarity ───────────────────────────────────────

#[test]
fn cosine_similarity_identical_is_one() {
    let v = high_security();
    let sim = MemoryQuery::cosine_similarity(&v, &v);
    assert!(
        (sim - 1.0).abs() < 1e-6,
        "expected 1.0, got {}",
        sim
    );
}

#[test]
fn cosine_similarity_zero_vector_is_zero() {
    let zero = zero_fitness();
    let v = high_security();
    let sim = MemoryQuery::cosine_similarity(&zero, &v);
    assert_eq!(sim, 0.0);
}

#[test]
fn cosine_similarity_symmetric() {
    let a = high_security();
    let b = low_security();
    let ab = MemoryQuery::cosine_similarity(&a, &b);
    let ba = MemoryQuery::cosine_similarity(&b, &a);
    assert!((ab - ba).abs() < 1e-10, "{} != {}", ab, ba);
}

#[test]
fn cosine_similarity_self_is_highest() {
    let a = high_security();
    let b = low_security();
    let aa = MemoryQuery::cosine_similarity(&a, &a);
    let ab = MemoryQuery::cosine_similarity(&a, &b);
    assert!(aa >= ab, "self-similarity ({}) < cross ({})", aa, ab);
}

// ─── Gate 4: counterfactual_gain ─────────────────────────────────────

#[test]
fn counterfactual_gain_alternative_better() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    let chosen = low_security();
    let alt = high_security();
    let gain = q.counterfactual_gain(&chosen, &alt);
    assert!(matches!(
        gain,
        CounterfactualGain::AlternativeWasBetter { .. }
    ));
    if let CounterfactualGain::AlternativeWasBetter { dimensions_better } = gain
    {
        assert_eq!(dimensions_better, 6, "all 6 dimensions should be better");
    }
}

#[test]
fn counterfactual_gain_choice_correct() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    let chosen = high_security();
    let alt = low_security();
    let gain = q.counterfactual_gain(&chosen, &alt);
    assert!(matches!(gain, CounterfactualGain::ChoiceWasCorrect));
}

#[test]
fn counterfactual_gain_equal_vectors() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    let v = high_security();
    let gain = q.counterfactual_gain(&v, &v);
    assert!(matches!(gain, CounterfactualGain::NoMeaningfulDifference));
}

#[test]
fn counterfactual_gain_tradeoff_dependent() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    let chosen = FitnessVector {
        security: 0.9,
        test_coverage: 0.3,
        ..Default::default()
    };
    let alt = FitnessVector {
        security: 0.3,
        test_coverage: 0.9,
        ..Default::default()
    };
    let gain = q.counterfactual_gain(&chosen, &alt);
    assert!(matches!(gain, CounterfactualGain::TradeoffDependent));
}

// ─── Gate 5: best_rejected_alternative ───────────────────────────────

#[test]
fn best_rejected_alternative_returns_best() {
    let mut g = CausalMemoryGraph::new();

    let builder = make_builder("test", high_security(), None)
        .add_alternative(ec_memory::RejectedAlternative::new(
            ArtifactSnapshot::new("alt1"),
            low_security(),
            ec_memory::RejectionReason::CatastrophicFailure {
                dimension: "security".into(),
            },
        ))
        .add_alternative(ec_memory::RejectedAlternative::new(
            ArtifactSnapshot::new("alt2"),
            high_security(),
            ec_memory::RejectionReason::SandboxFailure { correctness: 0.5 },
        ));

    let id = g.record_from_builder(builder).unwrap();
    let q = MemoryQuery::new(&g);
    let best = q.best_rejected_alternative(id);
    assert!(best.is_some());
    assert!(best.unwrap().fitness.security > 0.5);
}

#[test]
fn best_rejected_alternative_none_when_no_alternatives() {
    let mut g = CausalMemoryGraph::new();
    let builder = make_builder("test", high_security(), None);
    let id = g.record_from_builder(builder).unwrap();

    let q = MemoryQuery::new(&g);
    assert!(q.best_rejected_alternative(id).is_none());
}

#[test]
fn best_rejected_alternative_none_when_node_missing() {
    let g = CausalMemoryGraph::new();
    let q = MemoryQuery::new(&g);
    assert!(q.best_rejected_alternative(NodeId::new()).is_none());
}

// ─── Gate 6: artifact filtering ──────────────────────────────────────

#[test]
fn query_only_returns_matching_artifact() {
    let mut g = CausalMemoryGraph::new();
    g.record_from_builder(make_builder("alpha", high_security(), None)).unwrap();
    g.record_from_builder(make_builder("beta", low_security(), None)).unwrap();

    let q = MemoryQuery::new(&g);
    let evo = q.fitness_evolution("alpha");
    assert_eq!(evo.len(), 1);
    assert!(evo[0].fitness.security > 0.5);
}

// ─── Week 23 Final Gate ──────────────────────────────────────────────

#[test]
fn week23_gate_complete() {
    let g = build_graph_with_iterations("gate_test", 5);
    let q = MemoryQuery::new(&g);

    // Evolution
    let evo = q.fitness_evolution("gate_test");
    assert_eq!(evo.len(), 5);

    // Similar
    let similar = q.find_similar(&high_security(), 3);
    assert!(similar.len() <= 3);

    // Counterfactual
    let gain = q.counterfactual_gain(&low_security(), &high_security());
    assert!(matches!(
        gain,
        CounterfactualGain::AlternativeWasBetter { .. }
    ));

    // Cosine
    let sim = MemoryQuery::cosine_similarity(&high_security(), &high_security());
    assert!((sim - 1.0).abs() < 1e-6);

    println!("✅ Week 23 Gate: PASSED");
    println!("   - fitness_evolution: tracks iterations");
    println!("   - find_similar: cosine similarity sorting");
    println!("   - counterfactual_gain: 4 cases covered");
    println!("   - best_rejected_alternative: selects best");
}
