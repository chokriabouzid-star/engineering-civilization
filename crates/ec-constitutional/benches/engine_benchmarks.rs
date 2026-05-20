#![forbid(unsafe_code)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ec_constitutional::constitution::Constitution;
use ec_constitutional::coverage::TestCoverageInvariant;
use ec_constitutional::engine::{ConstitutionalEngine, EvaluationContext};
use ec_constitutional::invariant::Invariant;
use ec_constitutional::reversibility::ReversibilityInvariant;
use ec_constitutional::security::SecurityInvariant;
use ec_epistemic::calibration::CalibrationState;
use ec_epistemic::state::{Evidence, EpistemicState, UncertaintyDecomposition};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use std::sync::Arc;
use std::time::Duration;

// ─── Helpers ────────────────────────────────────────────────────────

fn build_constitution() -> Constitution {
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
    .expect("valid epistemic state")
}

// ─── Benchmarks ─────────────────────────────────────────────────────

fn bench_single_evaluation(c: &mut Criterion) {
    let engine = ConstitutionalEngine::with_default_cache(build_constitution());
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::new("benchmark");

    // Warmup
    engine.evaluate("warmup", 0, &fitness, &epistemic, &context);

    c.bench_function("single_evaluation_cache_miss", |b| {
        let mut hash: u64 = 1;
        b.iter(|| {
            hash = hash.wrapping_add(1);
            black_box(engine.evaluate(
                black_box("bench-artifact"),
                black_box(hash),
                black_box(&fitness),
                black_box(&epistemic),
                black_box(&context),
            ))
        })
    });
}

fn bench_cache_hit(c: &mut Criterion) {
    let engine = ConstitutionalEngine::with_default_cache(build_constitution());
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::new("benchmark-cache");

    // Populate cache
    engine.evaluate("cached-artifact", 42, &fitness, &epistemic, &context);

    c.bench_function("single_evaluation_cache_hit", |b| {
        b.iter(|| {
            black_box(engine.evaluate(
                black_box("cached-artifact"),
                black_box(42u64),
                black_box(&fitness),
                black_box(&epistemic),
                black_box(&context),
            ))
        })
    });
}

fn bench_frontier_100(c: &mut Criterion) {
    let engine = ConstitutionalEngine::with_default_cache(build_constitution());
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::default();

    let evaluations: Vec<_> = (0..100)
        .map(|i| engine.evaluate(&format!("art-{}", i), i as u64, &fitness, &epistemic, &context))
        .collect();

    c.bench_function("frontier_100_artifacts", |b| {
        b.iter(|| {
            black_box(engine.build_frontier(black_box(&evaluations)))
        })
    });
}

fn bench_frontier_1000(c: &mut Criterion) {
    let engine = ConstitutionalEngine::with_default_cache(build_constitution());
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::default();

    let evaluations: Vec<_> = (0..1000)
        .map(|i| engine.evaluate(&format!("art-{}", i), i as u64, &fitness, &epistemic, &context))
        .collect();

    let mut group = c.benchmark_group("frontier_large");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("frontier_1000_artifacts", |b| {
        b.iter(|| {
            black_box(engine.build_frontier(black_box(&evaluations)))
        })
    });

    group.finish();
}

fn bench_compare(c: &mut Criterion) {
    let engine = ConstitutionalEngine::with_default_cache(build_constitution());
    let fitness = good_fitness();
    let epistemic = good_epistemic();
    let context = EvaluationContext::default();

    let left = engine.evaluate("left", 1, &fitness, &epistemic, &context);
    let right = engine.evaluate("right", 2, &fitness, &epistemic, &context);

    c.bench_function("compare_two_evaluations", |b| {
        b.iter(|| {
            black_box(engine.compare(black_box(&left), black_box(&right)))
        })
    });
}

// ─── Registration ────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_single_evaluation,
    bench_cache_hit,
    bench_frontier_100,
    bench_frontier_1000,
    bench_compare,
);
criterion_main!(benches);
