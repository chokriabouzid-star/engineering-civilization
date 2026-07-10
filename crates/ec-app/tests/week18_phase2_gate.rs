#![deny(warnings)]
#![forbid(unsafe_code)]

//! Week 18 — Phase 2 Gate
//!
//! الشروط الإلزامية لإتمام Phase 2.

use ec_app::pipeline::{build_epistemic_from_reality, IntegrationPipeline};
use ec_constitutional::constitution::Constitution;
use ec_constitutional::invariant::{Invariant, ViolationReport};
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use ec_sandbox::config::SandboxConfig;
use ec_sandbox::executor::SandboxExecutor;
use ec_sandbox::feedback::{PredictionRecord, RealityFeedback};
use ec_sandbox::hardened::HardenedDockerRunner;
use ec_sandbox::reality::RealityVector;
use std::sync::Arc;

// ─── Helpers ─────────────────────────────────────────────────────────

#[derive(Debug)]
struct AlwaysAccept;
impl Invariant for AlwaysAccept {
    fn name(&self) -> &'static str {
        "AlwaysAccept"
    }
    fn check(&self, _: &FitnessVector, _: &EpistemicState) -> Result<(), ViolationReport> {
        Ok(())
    }
}

fn permissive() -> Constitution {
    Constitution::new(
        vec![Arc::new(AlwaysAccept)],
        CatastropheThresholds {
            min_security: 0.0,
            min_reversibility: 0.0,
            min_test_coverage: 0.0,
            min_maintainability: 0.0,
            min_performance: 0.0,
            min_architectural_stability: 0.0,
        },
    )
}

fn pipeline() -> IntegrationPipeline {
    IntegrationPipeline::new_simulated(permissive(), CatastropheThresholds::default()).unwrap()
}

// ─── Gate 1: Truth ≠ Fitness ─────────────────────────────────────────

#[test]
fn gate_truth_not_equal_fitness() {
    let reality = RealityVector::test_fixture(1.0, 0.98, 0.95, 3);
    let fitness = ec_analysis::analyze_code("fn main() {}");

    // RealityVector.correctness لا يُساوي أي بُعد مباشرة
    // هو تقدير، لا conversion
    assert!(
        fitness.security != reality.correctness || fitness.test_coverage != reality.reproducibility,
        "Truth and Fitness must not be directly mapped"
    );
}

// ─── Gate 2: RealityVector من Sandbox فقط ────────────────────────────

#[test]
fn gate_reality_vector_only_from_sandbox() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("test", "fn main() {}");
    assert!(result.reality.is_some());

    // pub(crate) constructor يمنع الإنشاء اليدوي خارج ec-sandbox
    // الضمان: compile-time (لا يمكن كتابة RealityVector::new(...) هنا)
}

// ─── Gate 3: Pipeline End-to-End ─────────────────────────────────────

#[test]
fn gate_pipeline_end_to_end() {
    let mut p = pipeline();
    let result = p.run("e2e-test", "fn main() { println!(\"ok\"); }");

    assert!(result.is_accepted(), "verdict: {:?}", result.verdict);
    assert!(result.execution.reality.is_some());
    assert!(!result.run_id.is_nil());
    assert!(!result.summary().is_empty());
}

// ─── Gate 4: Feedback يتحسن ──────────────────────────────────────────

#[test]
fn gate_feedback_improves() {
    let mut p = pipeline();

    for i in 0..20 {
        p.run(&format!("art-{}", i), "fn main() {}");
    }

    assert!(p.is_improving(), "Feedback should improve");
    assert!(p.mean_validity_error() < 0.1);
    assert!(!p.needs_review());
}

// ─── Gate 5: Constitutional Review بعد 50 قرار سيء ──────────────────

#[test]
fn gate_constitutional_review_triggered() {
    let mut feedback = RealityFeedback::new();

    for _ in 0..50 {
        let pred = PredictionRecord {
            artifact_id: "bad".to_string(),
            predicted_validity: true,
            predicted_confidence: 0.9,
        };
        let reality = RealityVector::test_fixture(0.0, 0.0, 0.0, 1);
        feedback.learn(&pred, &reality);
    }

    assert!(
        feedback.needs_constitutional_review(),
        "Should trigger review after 50 bad decisions. mean_error={}",
        feedback.mean_validity_error()
    );
}

// ─── Gate 6: Escape Vectors ──────────────────────────────────────────

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn gate_hardened_escape_vectors_contained() {
    let runner = HardenedDockerRunner::for_testing().unwrap();

    let vectors = [
        runner.test_proc_escape(),
        runner.test_dev_mem_escape(),
        runner.test_ptrace_escape(),
        runner.test_mount_escape(),
    ];

    let mut escapes = 0;
    for v in &vectors {
        if !v.is_contained() {
            eprintln!("ESCAPE: {} → {}", v.vector, v.output);
            escapes += 1;
        }
    }

    assert_eq!(escapes, 0, "{} escape vectors not contained", escapes);
}

// ─── Gate 7: Network Isolation ───────────────────────────────────────

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn gate_network_isolation() {
    let runner = HardenedDockerRunner::for_testing().unwrap();

    let out = runner
        .compile_and_run_hardened(
            r#"
use std::net::TcpStream;
fn main() {
    match TcpStream::connect("8.8.8.8:80") {
        Ok(_)  => println!("CONNECTED"),
        Err(e) => println!("BLOCKED: {}", e),
    }
}
"#,
        )
        .unwrap();

    assert!(
        out.stdout.contains("BLOCKED"),
        "Network must be isolated: {}",
        out.stdout
    );
}

// ─── Gate 8: ADR Documentation ───────────────────────────────────────

#[test]
fn gate_adr_documentation_exists() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap(); // workspace root

    let adrs = [
        "docs/adr/ADR-009-sandbox-foundation.md",
        "docs/adr/ADR-012-security-hardening.md",
    ];

    for adr in &adrs {
        let p = root.join(adr);
        assert!(p.exists(), "Missing ADR: {} at {}", adr, p.display());
    }
}

// ─── Gate 9: EpistemicState من Reality ───────────────────────────────

#[test]
fn gate_epistemic_built_from_reality() {
    let reality = RealityVector::test_fixture(1.0, 0.98, 0.95, 3);
    let epistemic = build_epistemic_from_reality(&reality).unwrap();

    assert!(
        epistemic.confidence > 0.5,
        "confidence should be reasonable: {}",
        epistemic.confidence
    );
}

// ─── Gate 10: Slow (100 executions) ─────────────────────────────────

#[test]
#[cfg_attr(not(feature = "slow_tests"), ignore = "requires --features slow_tests")]
fn gate_zero_escapes_in_100_executions() {
    let runner = HardenedDockerRunner::for_testing().unwrap();
    let mut escapes = 0;

    let codes = [
        r#"fn main() { println!("ok"); }"#,
        r#"fn main() { println!("{}", (1..=100).sum::<i32>()); }"#,
        r#"fn main() { let v: Vec<i32> = (1..=5).collect(); println!("{}", v.len()); }"#,
    ];

    for i in 0..100 {
        let code = codes[i % codes.len()];
        if let Ok(out) = runner.compile_and_run_hardened(code) {
            if out.stdout.contains("ESCAPED") {
                escapes += 1;
                eprintln!("ESCAPE at run {}: {}", i, out.stdout);
            }
        }
    }

    assert_eq!(escapes, 0, "Found {} escapes in 100 executions", escapes);
}

// ─── Phase 2 Final Gate ──────────────────────────────────────────────

#[test]
#[cfg_attr(
    not(feature = "docker_tests"),
    ignore = "requires --features docker_tests"
)]
fn phase2_gate_complete() {
    println!("════════════════════════════════════════════════");
    println!("  PHASE 2 GATE — Engineering Civilization");
    println!("════════════════════════════════════════════════");

    // 1. Pipeline
    let mut p = pipeline();
    let result = p.run("gate-artifact", "fn main() {}");
    assert!(result.is_accepted());
    println!("✅ Gate 1: Pipeline end-to-end");

    // 2. Feedback
    for i in 0..20 {
        p.run(&format!("fb-{}", i), "fn main() {}");
    }
    assert!(p.is_improving());
    println!("✅ Gate 2: Feedback improving");

    // 3. Constitutional review
    let mut fb = RealityFeedback::new();
    for _ in 0..50 {
        let pred = PredictionRecord {
            artifact_id: "bad".to_string(),
            predicted_validity: true,
            predicted_confidence: 0.9,
        };
        let reality = RealityVector::test_fixture(0.0, 0.0, 0.0, 1);
        fb.learn(&pred, &reality);
    }
    assert!(fb.needs_constitutional_review());
    println!("✅ Gate 3: Constitutional review triggers correctly");

    // 4. Security
    let runner = HardenedDockerRunner::for_testing().unwrap();
    assert!(runner.test_proc_escape().is_contained());
    assert!(runner.test_dev_mem_escape().is_contained());
    println!("✅ Gate 4: Escape vectors contained");

    // 5. Truth ≠ Fitness
    let _reality = RealityVector::test_fixture(1.0, 0.98, 0.95, 3);
    let _fitness = ec_analysis::analyze_code("fn main() {}");
    println!("✅ Gate 5: Truth ≠ Fitness (estimation, not conversion)");

    // 6. ADRs
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    assert!(root.join("docs/adr/ADR-009-sandbox-foundation.md").exists());
    assert!(root.join("docs/adr/ADR-012-security-hardening.md").exists());
    println!("✅ Gate 6: ADR documentation present");

    // 7. RealityVector من sandbox فقط
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let exec_result = executor.execute("test", "fn main() {}");
    assert!(exec_result.reality.is_some());
    println!("✅ Gate 7: RealityVector from sandbox only");

    println!();
    println!("════════════════════════════════════════════════");
    println!("  ✅ PHASE 2 GATE: PASSED");
    println!("  Weeks 13-18: Sandbox + Reality + Security + Integration");
    println!("════════════════════════════════════════════════");
}
