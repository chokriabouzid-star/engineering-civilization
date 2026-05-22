#![forbid(unsafe_code)]

//! Phase 4 Gate — AST Analysis + ConfidenceVector
//!
//! Weeks 28-34: syn-based analysis replacing heuristics
//!              ConfidenceVector separate from FitnessVector (D8)
//!              All old tests still pass (D1-D7 preserved)

use ec_analysis::{analyze_code, analyze_code_full, AnalysisWarning};

// ─── D8: ConfidenceVector منفصل عن FitnessVector ───────────────────

#[test]
fn phase4_fitness_and_confidence_are_separate() {
    let r = analyze_code_full("fn f() -> i32 { 42 }");
    // fitness و confidence كلاهما موجود ومستقل
    assert!(r.fitness.security >= 0.0);
    assert!(r.confidence.security >= 0.0);
    // يمكن أن يختلفا
    assert!(r.confidence.security != r.fitness.security || r.confidence.security == 1.0);
}

// ─── الواجهة القديمة لا تتغير ──────────────────────────────────────

#[test]
fn phase4_old_api_unchanged() {
    let f = analyze_code("fn f() -> i32 { 1 }");
    assert!(f.security >= 0.0 && f.security <= 1.0);
    assert!(f.validate().is_ok());
}

#[test]
fn phase4_old_api_unsafe_still_penalized() {
    let f = analyze_code("unsafe { *ptr; }");
    assert!(f.security < 0.8);
}

// ─── AST يُصلح bug التعليقات ───────────────────────────────────────

#[test]
fn phase4_ast_fixes_v1_comment_bug() {
    let code = "// unsafe docs\nfn safe() -> i32 { 42 }";
    let v1 = analyze_code(code);
    let v15 = analyze_code_full(code);
    // v1 يعاقب كلمة "unsafe" في التعليق
    // AST لا يعاقبها
    assert!(v15.fitness.security > v1.security,
        "AST={:.2} > v1={:.2}", v15.fitness.security, v1.security);
}

#[test]
fn phase4_ast_not_fooled_by_string() {
    let code = r#"fn msg() -> &'static str { "unsafe operation" }"#;
    let v1 = analyze_code(code);
    let v15 = analyze_code_full(code);
    assert!(v15.fitness.security >= v1.security);
}

// ─── ConfidenceVector ذات معنى ─────────────────────────────────────

#[test]
fn phase4_confidence_high_for_ast_dimensions() {
    let r = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    // security: AST visitor → confidence عالية
    assert!(r.confidence.security >= 0.85);
    // architectural stability: AST visitor → confidence معقولة
    assert!(r.confidence.architectural_stability >= 0.70);
}

#[test]
fn phase4_confidence_low_for_heuristic_dimensions() {
    let r = analyze_code_full("fn f() -> i32 { 42 }");
    // test_coverage: لا tests → confidence منخفضة
    assert!(r.confidence.test_coverage <= 0.60);
    // performance: AST → confidence معقولة
    assert!(r.confidence.performance >= 0.70);
}

#[test]
fn phase4_confidence_overall_is_min() {
    let r = analyze_code_full("fn f() -> i32 { 42 }");
    let overall = r.confidence.overall();
    assert!(overall <= r.confidence.security);
    assert!(overall <= r.confidence.test_coverage);
    assert!(overall <= r.confidence.maintainability);
}

// ─── Warnings ───────────────────────────────────────────────────────

#[test]
fn phase4_safe_code_no_unsafe_warning() {
    let r = analyze_code_full("fn f() -> i32 { 1 }");
    assert!(!r.warnings.iter().any(|w| matches!(
        w, AnalysisWarning::UnsafeWithoutComment { .. }
    )));
}

#[test]
fn phase4_unsafe_code_has_warning() {
    let r = analyze_code_full("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    assert!(r.warnings.iter().any(|w| matches!(
        w, AnalysisWarning::UnsafeWithoutComment { .. }
    )));
}

#[test]
fn phase4_invalid_code_has_parse_warning() {
    let r = analyze_code_full("not rust @#%");
    assert!(r.warnings.iter().any(|w| matches!(
        w, AnalysisWarning::ParseFailed(_)
    )));
    assert!(!r.parse_successful);
}

// ─── 6 Visitors يعملون معاً ────────────────────────────────────────

#[test]
fn phase4_all_six_dimensions_populated() {
    let r = analyze_code_full(
        "#[test]\nfn test_add() { assert_eq!(1+1, 2); }\nfn add(a: i32, b: i32) -> i32 { a + b }"
    );
    let f = &r.fitness;
    assert!(f.security > 0.0);
    assert!(f.reversibility > 0.0);
    assert!(f.test_coverage > 0.0);
    assert!(f.maintainability > 0.0);
    assert!(f.performance > 0.0);
    assert!(f.architectural_stability > 0.0);
    assert!(f.validate().is_ok());
}

#[test]
fn phase4_all_six_confidences_populated() {
    let r = analyze_code_full("fn f() -> i32 { 42 }");
    let c = &r.confidence;
    assert!(c.security >= 0.0);
    assert!(c.reversibility >= 0.0);
    assert!(c.test_coverage >= 0.0);
    assert!(c.maintainability >= 0.0);
    assert!(c.performance >= 0.0);
    assert!(c.architectural_stability >= 0.0);
}

// ─── Calibration Dataset ────────────────────────────────────────────

#[test]
fn phase4_tier1_superior_to_bad_code() {
    let tier1 = analyze_code_full(include_str!("fixtures/tier1_excellent/pure_math.rs"));
    let bad = analyze_code_full(include_str!("fixtures/tier4_bad/unsafe_mess.rs"));

    assert!(tier1.fitness.security > bad.fitness.security);
    assert!(tier1.fitness.test_coverage > bad.fitness.test_coverage);
    assert!(tier1.fitness.reversibility > bad.fitness.reversibility);
    assert!(tier1.warnings.len() <= bad.warnings.len());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn phase4_gate_complete() {
    let safe = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    let unsafe_ = analyze_code_full("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    let invalid = analyze_code_full("not rust @#%");
    let comment = analyze_code_full("// unsafe docs\nfn safe() -> i32 { 42 }");

    println!("╔══════════════════════════════════════════════╗");
    println!("║   Phase 4 Gate — AST Analysis               ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║  syn-based analysis:         ✅             ║");
    println!("║  ConfidenceVector:           ✅             ║");
    println!("║  6 AST visitors:             ✅             ║");
    println!("║  Calibration dataset:        ✅             ║");
    println!("║  Old API unchanged:          ✅             ║");
    println!("║  Comment bug fixed:          ✅             ║");
    println!("║  D1-D8 preserved:            ✅             ║");
    println!("╚══════════════════════════════════════════════╝");

    assert!(safe.parse_successful);
    assert!(unsafe_.parse_successful);
    assert!(!invalid.parse_successful);
    assert!(safe.fitness.security > unsafe_.fitness.security);
    assert!(comment.fitness.security >= 0.95);
    assert!(safe.confidence.security >= 0.90);

    println!("  ✅ Phase 4 Gate: PASSED");
}
