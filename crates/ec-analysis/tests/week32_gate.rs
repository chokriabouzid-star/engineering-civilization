#![forbid(unsafe_code)]

//! Week 32 Gate — Calibration Dataset
//!
//! Gate Criteria:
//! ✓ tier1 (excellent) أعلى من tier4 (bad) في security + coverage
//! ✓ AST أفضل من v1 heuristic للتعليقات
//! ✓ confidence أعلى لـ AST من heuristic
//! ✓ tier1 يُنتج warnings أقل من tier4
//! ✓ كلتا الملفتين تُparsed بنجاح

use ec_analysis::{analyze_code, analyze_code_full};

const TIER1: &str = include_str!("fixtures/tier1_excellent/pure_math.rs");
const TIER4: &str = include_str!("fixtures/tier4_bad/unsafe_mess.rs");

// ─── Gate 1: tier1 أعلى من tier4 ───────────────────────────────────

#[test]
fn w32_tier1_higher_security_than_tier4() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.fitness.security >= t4.fitness.security + 0.20,
        "security: t1={:.2}, t4={:.2}", t1.fitness.security, t4.fitness.security);
}

#[test]
fn w32_tier1_higher_coverage_than_tier4() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.fitness.test_coverage > t4.fitness.test_coverage,
        "coverage: t1={:.2}, t4={:.2}", t1.fitness.test_coverage, t4.fitness.test_coverage);
}

#[test]
fn w32_tier1_higher_maintainability_than_tier4() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.fitness.maintainability >= t4.fitness.maintainability,
        "maint: t1={:.2}, t4={:.2}", t1.fitness.maintainability, t4.fitness.maintainability);
}

#[test]
fn w32_tier1_higher_reversibility_than_tier4() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.fitness.reversibility > t4.fitness.reversibility,
        "rev: t1={:.2}, t4={:.2}", t1.fitness.reversibility, t4.fitness.reversibility);
}

// ─── Gate 2: AST أفضل من v1 للتعليقات ─────────────────────────────

#[test]
fn w32_ast_better_than_v1_heuristic_for_comments() {
    let code = "// unsafe usage documented here\nfn safe_fn() -> i32 { 42 }";
    let v1 = analyze_code(code);
    let v15 = analyze_code_full(code);
    assert!(v15.fitness.security > v1.security,
        "AST={:.2} > v1={:.2}", v15.fitness.security, v1.security);
}

#[test]
fn w32_ast_not_fooled_by_string_literal() {
    let code = r#"fn msg() -> &'static str { "unsafe operation" }"#;
    let v1 = analyze_code(code);
    let v15 = analyze_code_full(code);
    // v1 يعاقب الكلمة في string، AST لا
    assert!(v15.fitness.security >= v1.security,
        "AST={:.2} >= v1={:.2}", v15.fitness.security, v1.security);
}

// ─── Gate 3: confidence ─────────────────────────────────────────────

#[test]
fn w32_confidence_higher_for_ast_than_estimated() {
    let report = analyze_code_full("fn safe() -> i32 { 42 }");
    assert!(report.confidence.security >= 0.85,
        "AST analysis → confidence عالية، got: {}", report.confidence.security);
}

#[test]
fn w32_confidence_lower_for_heuristic_dimensions() {
    let report = analyze_code_full("fn safe() -> i32 { 42 }");
    // test_coverage heuristic → confidence منخفضة (لا tests)
    assert!(report.confidence.test_coverage <= 0.60,
        "Coverage heuristic → confidence منخفضة، got: {}", report.confidence.test_coverage);
}

// ─── Gate 4: warnings ───────────────────────────────────────────────

#[test]
fn w32_tier1_fewer_warnings_than_tier4() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.warnings.len() <= t4.warnings.len(),
        "t1 warnings={}, t4 warnings={}", t1.warnings.len(), t4.warnings.len());
}

#[test]
fn w32_tier4_has_unsafe_warning() {
    let t4 = analyze_code_full(TIER4);
    assert!(t4.warnings.iter().any(|w| matches!(
        w, ec_analysis::AnalysisWarning::UnsafeWithoutComment { .. }
    )));
}

// ─── Gate 5: parse ناجح ────────────────────────────────────────────

#[test]
fn w32_both_files_parse_successfully() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.parse_successful);
    assert!(t4.parse_successful);
}

#[test]
fn w32_both_fitness_valid() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);
    assert!(t1.fitness.validate().is_ok());
    assert!(t4.fitness.validate().is_ok());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn week32_gate_complete() {
    let t1 = analyze_code_full(TIER1);
    let t4 = analyze_code_full(TIER4);

    println!("═══════════════════════════════════════════════");
    println!("  Week 32 Gate — Calibration Dataset");
    println!("═══════════════════════════════════════════════");
    println!("  Tier1 (excellent):");
    println!("    sec={:.2} cov={:.2} maint={:.2} rev={:.2}",
        t1.fitness.security, t1.fitness.test_coverage,
        t1.fitness.maintainability, t1.fitness.reversibility);
    println!("    warnings={}", t1.warnings.len());
    println!("  Tier4 (bad):");
    println!("    sec={:.2} cov={:.2} maint={:.2} rev={:.2}",
        t4.fitness.security, t4.fitness.test_coverage,
        t4.fitness.maintainability, t4.fitness.reversibility);
    println!("    warnings={}", t4.warnings.len());
    println!("═══════════════════════════════════════════════");

    assert!(t1.fitness.security > t4.fitness.security);
    assert!(t1.fitness.test_coverage > t4.fitness.test_coverage);

    println!("  ✅ Week 32 Gate: PASSED");
}
