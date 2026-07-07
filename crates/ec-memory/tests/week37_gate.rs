#![forbid(unsafe_code)]

//! Week 37 Gate — OutcomeStorage (SQLite)

use ec_memory::{MemoryStorage, OutcomeStorage, SqliteStorage};

fn make_storage() -> SqliteStorage {
    SqliteStorage::in_memory_with_outcomes().unwrap()
}

// ─── Gate 1: empty storage ──────────────────────────────────────────

#[test]
fn w37_empty_storage_zero_count() {
    let s = make_storage();
    assert_eq!(s.outcome_count().unwrap(), 0);
}

#[test]
fn w37_empty_evidence_unbiased() {
    let s = make_storage();
    let e = s.load_all_evidence().unwrap();
    assert_eq!(e.successes, 0);
    assert_eq!(e.failures, 0);
}

// ─── Gate 2: record + load ──────────────────────────────────────────

#[test]
fn w37_record_one_outcome() {
    let s = make_storage();
    s.record_outcome("test_artifact", true, 0.9).unwrap();
    assert_eq!(s.outcome_count().unwrap(), 1);
}

#[test]
fn w37_record_and_load_evidence() {
    let s = make_storage();
    s.record_outcome("a1", true, 0.9).unwrap();
    s.record_outcome("a1", true, 0.8).unwrap();
    s.record_outcome("a1", false, 0.2).unwrap();

    let e = s.load_evidence("a1").unwrap();
    assert_eq!(e.successes, 2);
    assert_eq!(e.failures, 1);
}

// ─── Gate 3: per-artifact isolation ────────────────────────────────

#[test]
fn w37_artifact_isolation() {
    let s = make_storage();
    s.record_outcome("a1", true, 0.9).unwrap();
    s.record_outcome("a1", true, 0.8).unwrap();
    s.record_outcome("a2", false, 0.1).unwrap();

    let e1 = s.load_evidence("a1").unwrap();
    let e2 = s.load_evidence("a2").unwrap();

    assert_eq!(e1.successes, 2);
    assert_eq!(e1.failures, 0);
    assert_eq!(e2.successes, 0);
    assert_eq!(e2.failures, 1);
}

#[test]
fn w37_missing_artifact_empty_evidence() {
    let s = make_storage();
    s.record_outcome("a1", true, 0.9).unwrap();

    let e = s.load_evidence("nonexistent").unwrap();
    assert_eq!(e.successes, 0);
    assert_eq!(e.failures, 0);
}

// ─── Gate 4: load_all ──────────────────────────────────────────────

#[test]
fn w37_load_all_aggregates() {
    let s = make_storage();
    s.record_outcome("a1", true, 0.9).unwrap();
    s.record_outcome("a2", false, 0.2).unwrap();
    s.record_outcome("a3", true, 0.85).unwrap();

    let e = s.load_all_evidence().unwrap();
    assert_eq!(e.successes, 2);
    assert_eq!(e.failures, 1);
    assert_eq!(e.total_observations(), 3);
}

// ─── Gate 5: confidence grows ──────────────────────────────────────

#[test]
fn w37_confidence_grows_with_successes() {
    let s = make_storage();
    for i in 0..15 {
        s.record_outcome("a1", true, 0.9 + i as f64 * 0.005)
            .unwrap();
    }

    let e = s.load_evidence("a1").unwrap();
    assert!(
        e.credible_confidence() > 0.45,
        "after 15 successes: {}",
        e.credible_confidence()
    );
}

// ─── Gate 6: old storage still works ────────────────────────────────

#[test]
fn w37_old_sqlite_storage_still_works() {
    let s = SqliteStorage::in_memory().unwrap();
    let graph = ec_memory::CausalMemoryGraph::new();
    s.save(&graph).unwrap();
    let loaded = s.load().unwrap();
    assert!(loaded.is_empty());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w37_gate_complete() {
    let s = make_storage();

    for i in 0..20 {
        let ok = i < 15;
        s.record_outcome("main", ok, if ok { 0.9 } else { 0.1 })
            .unwrap();
    }

    let e = s.load_evidence("main").unwrap();

    println!("═══════════════════════════════════════════════");
    println!("  Week 37 Gate — OutcomeStorage (SQLite)");
    println!("═══════════════════════════════════════════════");
    println!("  Outcomes:     {}", s.outcome_count().unwrap());
    println!("  Successes:    {}", e.successes);
    println!("  Failures:     {}", e.failures);
    println!("  Mean score:   {:.2}", e.mean_score);
    println!("  Confidence:   {:.3}", e.credible_confidence());
    println!("═══════════════════════════════════════════════");

    assert_eq!(s.outcome_count().unwrap(), 20);
    assert_eq!(e.successes, 15);
    assert_eq!(e.failures, 5);

    println!("  ✅ Week 37 Gate: PASSED");
}
