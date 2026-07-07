//! Week 24 Gate — Value Drift from Memory
//!
//! Tests: 10
//!
//! Invariants:
//!   - Stable when vectors identical
//!   - ValueShift when priorities rotate > 10°
//!   - Corruption when rejections increase > 20%
//!   - HumanIntervention when angle > 45°
//!   - InsufficientData when memory too small

use ec_fitness::fitness::FitnessVector;
use ec_memory::{
    ArtifactSnapshot, CausalMemoryGraph, DecisionNodeBuilder, DriftAction, DriftClassification,
    HistoricalDriftAnalyzer,
};

// ─── helpers ────────────────────────────────────

fn make_builder(artifact: &str, fitness: FitnessVector, valid: bool) -> DecisionNodeBuilder {
    DecisionNodeBuilder::new(artifact, ArtifactSnapshot::new("fn main() {}"), fitness)
        .constitutional_valid(valid)
        .causal_parents(vec![])
}

fn high_security() -> FitnessVector {
    FitnessVector {
        security: 0.9,
        reversibility: 0.8,
        test_coverage: 0.8,
        maintainability: 0.8,
        performance: 0.7,
        architectural_stability: 0.8,
    }
}

fn low_security() -> FitnessVector {
    FitnessVector {
        security: 0.2,
        reversibility: 0.3,
        test_coverage: 0.3,
        maintainability: 0.4,
        performance: 0.5,
        architectural_stability: 0.4,
    }
}

fn build_stable_memory(n: usize) -> CausalMemoryGraph {
    let mut g = CausalMemoryGraph::new();
    for i in 0..n {
        g.record_from_builder(make_builder(&format!("a{i}"), high_security(), true))
            .expect("record failed");
    }
    g
}

fn build_drifted_memory(baseline: usize, current: usize) -> CausalMemoryGraph {
    let mut g = CausalMemoryGraph::new();
    for i in 0..baseline {
        g.record_from_builder(make_builder(&format!("b{i}"), high_security(), true))
            .expect("record failed");
    }
    for i in 0..current {
        g.record_from_builder(make_builder(
            &format!("c{i}"),
            FitnessVector {
                security: 0.2,
                reversibility: 0.3,
                test_coverage: 0.3,
                maintainability: 0.4,
                performance: 0.95,
                architectural_stability: 0.3,
            },
            true,
        ))
        .expect("record failed");
    }
    g
}

// ─── tests ──────────────────────────────────────

#[test]
fn t01_insufficient_data_when_memory_too_small() {
    let g = build_stable_memory(3);
    let a = HistoricalDriftAnalyzer::new(&g, 5, 5);
    let r = a.analyze();
    assert!(
        matches!(
            r.classification,
            DriftClassification::InsufficientData { .. }
        ),
        "expected InsufficientData, got {:?}",
        r.classification
    );
    assert!(matches!(r.recommended_action, DriftAction::None));
}

#[test]
fn t02_stable_when_identical_vectors() {
    let g = build_stable_memory(20);
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert!(
        matches!(r.classification, DriftClassification::Stable),
        "expected Stable, got {:?}",
        r.classification
    );
    assert!(
        r.drift_angle_degrees < 1.0,
        "angle should be ~0°, got {:.2}°",
        r.drift_angle_degrees
    );
}

#[test]
fn t03_value_shift_detected_when_priorities_change() {
    let g = build_drifted_memory(10, 10);
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert!(
        r.drift_angle_degrees > 10.0,
        "expected drift > 10°, got {:.1}°",
        r.drift_angle_degrees
    );
    assert!(
        !matches!(r.classification, DriftClassification::Stable),
        "expected non-Stable, got Stable"
    );
}

#[test]
fn t04_corruption_detected_when_rejections_increase() {
    let mut g = CausalMemoryGraph::new();
    for i in 0..10 {
        g.record_from_builder(make_builder(&format!("b{i}"), high_security(), true))
            .expect("record failed");
    }
    for i in 0..10 {
        g.record_from_builder(make_builder(&format!("c{i}"), low_security(), false))
            .expect("record failed");
    }
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert!(
        matches!(r.classification, DriftClassification::Corruption { .. }),
        "expected Corruption, got {:?}",
        r.classification
    );
    assert!(r.requires_action(), "corruption must require action");
}

#[test]
fn t05_report_counts_are_correct() {
    let g = build_stable_memory(30);
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert_eq!(r.baseline_count, 10);
    assert_eq!(r.current_count, 10);
    assert_eq!(r.total_decisions, 30);
}

#[test]
fn t06_drift_angle_in_valid_range() {
    let g = build_drifted_memory(10, 10);
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert!(
        r.drift_angle_degrees >= 0.0,
        "angle must be >= 0, got {}",
        r.drift_angle_degrees
    );
    assert!(
        r.drift_angle_degrees <= 180.0,
        "angle must be <= 180, got {}",
        r.drift_angle_degrees
    );
}

#[test]
fn t07_human_intervention_for_large_shift() {
    let mut g = CausalMemoryGraph::new();
    for i in 0..10 {
        g.record_from_builder(make_builder(
            &format!("b{i}"),
            FitnessVector {
                security: 1.0,
                reversibility: 0.0,
                test_coverage: 0.0,
                maintainability: 0.0,
                performance: 0.0,
                architectural_stability: 0.0,
            },
            true,
        ))
        .expect("record failed");
    }
    for i in 0..10 {
        g.record_from_builder(make_builder(
            &format!("c{i}"),
            FitnessVector {
                security: 0.0,
                reversibility: 0.0,
                test_coverage: 0.0,
                maintainability: 0.0,
                performance: 1.0,
                architectural_stability: 0.0,
            },
            true,
        ))
        .expect("record failed");
    }
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert!(
        r.drift_angle_degrees > 45.0,
        "expected > 45°, got {:.1}°",
        r.drift_angle_degrees
    );
    assert!(
        matches!(r.recommended_action, DriftAction::HumanIntervention { .. }),
        "expected HumanIntervention, got {:?}",
        r.recommended_action
    );
}

#[test]
fn t08_stable_requires_no_action() {
    let g = build_stable_memory(20);
    let a = HistoricalDriftAnalyzer::new(&g, 10, 10);
    let r = a.analyze();
    assert!(
        !r.requires_action(),
        "stable system should not require action"
    );
}

#[test]
fn t09_empty_memory_returns_insufficient_data() {
    let g = CausalMemoryGraph::new();
    let a = HistoricalDriftAnalyzer::new(&g, 5, 5);
    let r = a.analyze();
    assert!(
        matches!(
            r.classification,
            DriftClassification::InsufficientData {
                available: 0,
                required: 10
            }
        ),
        "expected InsufficientData{{0, 10}}, got {:?}",
        r.classification
    );
}

#[test]
fn t10_week24_gate_complete() {
    // 1. Stable system
    let stable = build_stable_memory(20);
    let r = HistoricalDriftAnalyzer::new(&stable, 10, 10).analyze();
    assert!(matches!(r.classification, DriftClassification::Stable));

    // 2. Drifted system
    let drifted = build_drifted_memory(10, 10);
    let r2 = HistoricalDriftAnalyzer::new(&drifted, 10, 10).analyze();
    assert!(!matches!(r2.classification, DriftClassification::Stable));
    assert!(!matches!(
        r2.classification,
        DriftClassification::InsufficientData { .. }
    ));

    // 3. الزاوية في [0, 180]
    assert!(r2.drift_angle_degrees >= 0.0 && r2.drift_angle_degrees <= 180.0);

    // 4. Empty
    let empty = CausalMemoryGraph::new();
    let r3 = HistoricalDriftAnalyzer::new(&empty, 5, 5).analyze();
    assert!(matches!(
        r3.classification,
        DriftClassification::InsufficientData { .. }
    ));

    println!(
        "✅ Week 24 Gate: PASSED — drift={:.1}°",
        r2.drift_angle_degrees
    );
}
