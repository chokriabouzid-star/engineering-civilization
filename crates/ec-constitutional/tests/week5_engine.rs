#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_constitutional::constitution::Constitution;
use ec_constitutional::coverage::TestCoverageInvariant;
use ec_constitutional::engine::{ConstitutionalEngine, EvaluationContext};
use ec_constitutional::invariant::Invariant;
use ec_constitutional::reversibility::ReversibilityInvariant;
use ec_constitutional::security::SecurityInvariant;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{EpistemicState, Evidence, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use std::sync::Arc;
use std::time::{Duration, Instant};

// ─── Helpers ────────────────────────────────────────────────────────

fn build_test_constitution() -> Constitution {
    let invariants: Vec<Arc<dyn Invariant>> = vec![
        Arc::new(SecurityInvariant::default()),
        Arc::new(TestCoverageInvariant::default()),
        Arc::new(ReversibilityInvariant::default()),
    ];
    let thresholds = CatastropheThresholds {
        min_security: 0.70,
        min_test_coverage: 0.60,
        min_reversibility: 0.30,
        min_maintainability: 0.40,
        min_performance: 0.20,
        min_architectural_stability: 0.50,
    };
    Constitution::new(invariants, thresholds)
}

fn good_fitness() -> FitnessVector {
    FitnessVector {
        security: 0.90,
        reversibility: 0.80,
        test_coverage: 0.85,
        maintainability: 0.75,
        performance: 0.60,
        architectural_stability: 0.70,
    }
}

fn good_epistemic() -> EpistemicState {
    EpistemicState::new(
        0.95,
        Evidence {
            sample_size: 100,
            age_seconds: 3600,
            reproducibility: 0.98,
            source_reliability: 0.99,
        },
        UncertaintyDecomposition {
            aleatoric: 0.05,
            epistemic: 0.03,
            model: 0.02,
        },
        CalibrationState::default(),
    )
    .expect("good_epistemic: valid inputs")
}

fn bad_fitness() -> FitnessVector {
    FitnessVector {
        security: 0.50, // below catastrophic threshold 0.70
        reversibility: 0.80,
        test_coverage: 0.85,
        maintainability: 0.75,
        performance: 0.60,
        architectural_stability: 0.70,
    }
}

// ─── Engine Creation ────────────────────────────────────────────────

#[test]
fn engine_creates_with_default_cache() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    assert!(!engine.constitution_version().is_empty());
    assert_eq!(engine.cache_len(), 0);
}

#[test]
fn engine_version_is_unique() {
    let e1 = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let e2 = ConstitutionalEngine::with_default_cache(build_test_constitution());
    assert_ne!(e1.constitution_version(), e2.constitution_version());
}

// ─── Evaluation Correctness ─────────────────────────────────────────

#[test]
fn engine_evaluate_valid_artifact() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::new("Test valid artifact");

    let eval = engine.evaluate(
        "artifact-valid",
        1,
        &good_fitness(),
        &good_epistemic(),
        &context,
    );

    assert!(eval.is_valid, "Expected valid evaluation");
    assert_eq!(eval.artifact_id, "artifact-valid");
    assert!(eval.catastrophic.is_none());
    assert!(eval.violations.is_empty());
}

#[test]
fn engine_evaluate_catastrophic_security() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::new("Test catastrophic security");

    let eval = engine.evaluate(
        "artifact-bad",
        2,
        &bad_fitness(),
        &good_epistemic(),
        &context,
    );

    assert!(!eval.is_valid, "Expected invalid evaluation");
    assert!(
        eval.catastrophic.is_some(),
        "Expected catastrophic dimension"
    );
}

#[test]
fn engine_evaluate_stores_artifact_id() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();

    let eval = engine.evaluate(
        "my-artifact-42",
        99,
        &good_fitness(),
        &good_epistemic(),
        &context,
    );

    assert_eq!(eval.artifact_id, "my-artifact-42");
}

// ─── Cache Correctness ──────────────────────────────────────────────

#[test]
fn cache_miss_then_hit() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::new("Cache test");

    assert_eq!(engine.cache_len(), 0);

    // First call: cache miss
    let eval1 = engine.evaluate(
        "artifact-cache",
        10,
        &good_fitness(),
        &good_epistemic(),
        &context,
    );
    assert_eq!(engine.cache_len(), 1);

    // Second call: cache hit — result must be identical
    let eval2 = engine.evaluate(
        "artifact-cache",
        10,
        &good_fitness(),
        &good_epistemic(),
        &context,
    );
    assert_eq!(engine.cache_len(), 1); // لم تُضف مدخلة جديدة

    assert_eq!(eval1.is_valid, eval2.is_valid);
    assert_eq!(eval1.artifact_id, eval2.artifact_id);
    assert_eq!(eval1.catastrophic, eval2.catastrophic);
}

#[test]
fn cache_different_hashes_are_independent() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();

    engine.evaluate("a", 100, &good_fitness(), &good_epistemic(), &context);
    engine.evaluate("b", 200, &good_fitness(), &good_epistemic(), &context);
    engine.evaluate("c", 300, &good_fitness(), &good_epistemic(), &context);

    assert_eq!(engine.cache_len(), 3);
}

#[test]
fn cache_hit_is_fast() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::new("Speed test");

    // Populate cache
    engine.evaluate("fast", 777, &good_fitness(), &good_epistemic(), &context);

    // Cache hit must be < 1ms
    let start = Instant::now();
    engine.evaluate("fast", 777, &good_fitness(), &good_epistemic(), &context);
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(1),
        "Cache hit took {:?}, expected < 1ms",
        elapsed
    );
}

#[test]
fn cache_ttl_expires_and_reevaluates() {
    // TTL بالغ القصر: 20ms
    let engine = ConstitutionalEngine::new(build_test_constitution(), Duration::from_millis(20));
    let context = EvaluationContext::default();

    engine.evaluate("ttl", 555, &good_fitness(), &good_epistemic(), &context);
    assert_eq!(engine.cache_len(), 1);

    std::thread::sleep(Duration::from_millis(30));

    // بعد انتهاء TTL: تقييم جديد يُضاف
    engine.evaluate("ttl", 555, &good_fitness(), &good_epistemic(), &context);
    assert_eq!(engine.cache_len(), 1); // القديم حُذف والجديد أُضيف
}

#[test]
fn cache_purge_removes_expired() {
    let engine = ConstitutionalEngine::new(build_test_constitution(), Duration::from_millis(20));
    let context = EvaluationContext::default();

    for i in 0..5u64 {
        engine.evaluate(
            &format!("art-{}", i),
            i,
            &good_fitness(),
            &good_epistemic(),
            &context,
        );
    }
    assert_eq!(engine.cache_len(), 5);

    std::thread::sleep(Duration::from_millis(30));

    let purged = engine.purge_cache();
    assert_eq!(purged, 5);
    assert_eq!(engine.cache_len(), 0);
}

// ─── Compare & Frontier ─────────────────────────────────────────────

#[test]
fn engine_compare_equal_fitness_vectors() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();

    let left = engine.evaluate("left", 1, &good_fitness(), &good_epistemic(), &context);
    let right = engine.evaluate("right", 2, &good_fitness(), &good_epistemic(), &context);

    let ordering = engine.compare(&left, &right);
    assert!(
        matches!(ordering, ec_fitness::ParetoOrdering::Equal),
        "Identical fitness vectors should be Equal, got {:?}",
        ordering
    );
}

#[test]
fn engine_frontier_single_valid() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();

    let eval = engine.evaluate("only", 1, &good_fitness(), &good_epistemic(), &context);
    let frontier = engine.build_frontier(&[eval]);

    assert_eq!(frontier.len(), 1);
}

#[test]
fn engine_frontier_excludes_invalid() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::default();

    let valid = engine.evaluate("valid", 1, &good_fitness(), &good_epistemic(), &context);
    let invalid = engine.evaluate("invalid", 2, &bad_fitness(), &good_epistemic(), &context);

    assert!(valid.is_valid);
    assert!(!invalid.is_valid);

    let frontier = engine.build_frontier(&[valid, invalid]);
    assert_eq!(
        frontier.len(),
        1,
        "Frontier should exclude invalid artifacts"
    );
    assert_eq!(frontier[0].artifact_id, "valid");
}

// ─── Performance ────────────────────────────────────────────────────

#[test]
fn single_evaluation_under_1ms() {
    let engine = ConstitutionalEngine::with_default_cache(build_test_constitution());
    let context = EvaluationContext::new("Performance test");
    let fitness = good_fitness();
    let epistemic = good_epistemic();

    // Warmup
    engine.evaluate("warmup", 0, &fitness, &epistemic, &context);

    // Fresh hash — no cache
    let start = Instant::now();
    engine.evaluate("perf-test", 999999, &fitness, &epistemic, &context);
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_millis(1),
        "Single evaluation took {:?}, expected < 1ms",
        elapsed
    );
}

// ─── Concurrency ────────────────────────────────────────────────────

#[test]
fn concurrent_access_no_deadlocks() {
    use std::sync::Arc;
    use std::thread;

    let engine = Arc::new(ConstitutionalEngine::with_default_cache(
        build_test_constitution(),
    ));
    let mut handles = vec![];

    for thread_id in 0u64..8 {
        let engine = engine.clone();
        let handle = thread::spawn(move || {
            let fitness = FitnessVector {
                security: 0.90,
                reversibility: 0.80,
                test_coverage: 0.85,
                maintainability: 0.75,
                performance: 0.60,
                architectural_stability: 0.70,
            };
            let epistemic = EpistemicState::new(
                0.95,
                Evidence {
                    sample_size: 100,
                    age_seconds: 3600,
                    reproducibility: 0.98,
                    source_reliability: 0.99,
                },
                UncertaintyDecomposition {
                    aleatoric: 0.05,
                    epistemic: 0.03,
                    model: 0.02,
                },
                CalibrationState::default(),
            )
            .expect("valid epistemic");

            let context = EvaluationContext::new(format!("thread-{}", thread_id));

            for i in 0u64..50 {
                let hash = thread_id * 1000 + i;
                let eval = engine.evaluate(
                    &format!("art-{}-{}", thread_id, i),
                    hash,
                    &fitness,
                    &epistemic,
                    &context,
                );
                assert!(
                    eval.is_valid,
                    "Thread {} eval {} should be valid",
                    thread_id, i
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // 8 threads × 50 unique hashes = 400 cache entries
    assert_eq!(engine.cache_len(), 400);
}

// ─── Week 8: Load Test ───────────────────────────────────────────────

#[tokio::test]
async fn load_test_1000_evaluations_per_second() {
    use std::sync::Arc;
    use tokio::time::Instant;

    let engine = Arc::new(ConstitutionalEngine::with_default_cache(
        build_test_constitution(),
    ));
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::default();

    let start = Instant::now();
    let mut handles = vec![];

    // 1000 evaluations متزامنة
    for i in 0..1000 {
        let engine = engine.clone();
        let fitness = fitness.clone();
        let epistemic = epistemic.clone();
        let context = context.clone();
        let handle = tokio::spawn(async move {
            engine
                .evaluate_async(format!("load-{}", i), i as u64, fitness, epistemic, context)
                .await
        });
        handles.push(handle);
    }

    // انتظار جميع المهام
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    let elapsed = start.elapsed();
    let rate = 1000.0 / elapsed.as_secs_f64();

    println!("Load test: 1000 evaluations in {:?}", elapsed);
    println!("Rate: {:.2} evaluations/sec", rate);

    assert!(
        rate >= 1000.0,
        "Expected >= 1000 evals/sec, got {:.2}",
        rate
    );
}

// ─── Week 8: 100 Simultaneous Evaluations ───────────────────────────

#[tokio::test]
async fn test_100_simultaneous_evaluations() {
    use std::sync::Arc;
    use tokio::time::Instant;

    let engine = Arc::new(ConstitutionalEngine::with_default_cache(
        build_test_constitution(),
    ));
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::default();

    let start = Instant::now();
    let mut handles = vec![];

    // 100 evaluations متزامنة
    for i in 0..100 {
        let engine = engine.clone();
        let fitness = fitness.clone();
        let epistemic = epistemic.clone();
        let context = context.clone();
        let handle = tokio::spawn(async move {
            engine
                .evaluate_async(format!("sim-{}", i), i as u64, fitness, epistemic, context)
                .await
        });
        handles.push(handle);
    }

    // انتظار جميع المهام
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    let elapsed = start.elapsed();
    println!("100 simultaneous evaluations in {:?}", elapsed);
    assert!(
        elapsed < Duration::from_secs(1),
        "Should complete within 1 second"
    );
}
