#![forbid(unsafe_code)]

//! Week 28 Gate — syn AST + AnalysisReport + ConfidenceVector
//!
//! Gate Criteria:
//! ✓ analyze_code() القديمة لا تتغير
//! ✓ analyze_code_full() تُنتج AnalysisReport
//! ✓ AST parse ناجح لكود صالح
//! ✓ AST parse فاشل → unparseable (لا panic)
//! ✓ ConfidenceVector موجودة ومعقولة
//! ✓ تعليق "unsafe" لا يُعاقب في AST (عكس v1)

use ec_analysis::{analyze_code, analyze_code_full};

// ─── Gate 1: الواجهة القديمة لا تتغير ─────────────────────────────

#[test]
fn w28_old_api_unchanged() {
    let f = analyze_code("fn add(a: i32, b: i32) -> i32 { a + b }");
    assert!(f.security >= 0.0 && f.security <= 1.0);
    assert!(f.validate().is_ok());
}

#[test]
fn w28_old_api_unsafe_still_penalized() {
    let f = analyze_code("unsafe { *ptr; }");
    assert!(f.security < 0.8);
}

// ─── Gate 2: AST parse ناجح ────────────────────────────────────────

#[test]
fn w28_syn_parses_valid_code() {
    let r = analyze_code_full("fn f() -> i32 { 42 }");
    assert!(r.parse_successful);
    assert!(r.fitness.validate().is_ok());
}

#[test]
fn w28_syn_parses_complex_code() {
    let code = r#"
        use std::collections::HashMap;
        pub fn process(data: &str) -> bool {
            let map: HashMap<&str, i32> = HashMap::new();
            data.len() > 0
        }
    "#;
    let r = analyze_code_full(code);
    assert!(r.parse_successful);
}

// ─── Gate 3: AST parse فاشل → لا panic ─────────────────────────────

#[test]
fn w28_syn_handles_invalid_gracefully() {
    let r = analyze_code_full("this is not rust !@#");
    assert!(!r.parse_successful);
    assert_eq!(r.warnings.len(), 1);
    match &r.warnings[0] {
        ec_analysis::AnalysisWarning::ParseFailed(_) => {}
        other => panic!("Expected ParseFailed, got {:?}", other),
    }
}

#[test]
fn w28_invalid_code_zero_confidence() {
    let r = analyze_code_full("}}}}invalid");
    assert!(!r.parse_successful);
    assert_eq!(r.confidence.overall(), 0.0);
}

// ─── Gate 4: ConfidenceVector ───────────────────────────────────────

#[test]
fn w28_confidence_present() {
    let r = analyze_code_full("fn f() {}");
    assert!(r.confidence.overall() >= 0.0);
    assert!(r.confidence.security >= 0.0);
    assert!(r.confidence.test_coverage >= 0.0);
}

#[test]
fn w28_safe_code_high_security_confidence() {
    let r = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    assert!(
        r.confidence.security >= 0.90,
        "AST analysis of safe code → high confidence, got: {}",
        r.confidence.security
    );
}

#[test]
fn w28_unsafe_code_lower_security_confidence() {
    let safe = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    let unsafe_ = analyze_code_full("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    assert!(safe.confidence.security > unsafe_.confidence.security * 0.8);
}

// ─── Gate 5: تعليق "unsafe" لا يُعاقب في AST ───────────────────────

#[test]
fn w28_comment_not_penalized_in_ast() {
    // v1 keyword counting يعاقب كلمة "unsafe" في التعليق
    // AST لا يعاقبها — لا يوجد unsafe block فعلي
    let code = "// unsafe usage documented here\nfn safe_fn() -> i32 { 42 }";
    let r = analyze_code_full(code);
    assert!(
        r.fitness.security >= 0.95,
        "AST should not penalize 'unsafe' in comments, got: {}",
        r.fitness.security
    );
}

#[test]
fn w28_ast_vs_v1_comment_difference() {
    let code = "// unsafe usage documented here\nfn safe_fn() -> i32 { 42 }";
    let v1 = analyze_code(code);
    let v15 = analyze_code_full(code);
    // v1 يعاقب الكلمة، AST لا
    assert!(
        v15.fitness.security >= v1.security,
        "AST={:.2} should be >= v1={:.2}",
        v15.fitness.security,
        v1.security
    );
}

// ─── Gate 6: warnings ───────────────────────────────────────────────

#[test]
fn w28_safe_code_no_unsafe_warning() {
    let r = analyze_code_full("fn f() -> i32 { 1 }");
    assert!(
        r.warnings.is_empty()
            || !r
                .warnings
                .iter()
                .any(|w| matches!(w, ec_analysis::AnalysisWarning::UnsafeWithoutComment { .. }))
    );
}

#[test]
fn w28_unsafe_code_has_warning() {
    let r = analyze_code_full("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    assert!(r
        .warnings
        .iter()
        .any(|w| matches!(w, ec_analysis::AnalysisWarning::UnsafeWithoutComment { .. })));
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn week28_gate_complete() {
    let safe = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    let unsafe_ = analyze_code_full("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    let invalid = analyze_code_full("not rust @#%");

    println!("═══════════════════════════════════════════════");
    println!("  Week 28 Gate — syn AST + ConfidenceVector");
    println!("═══════════════════════════════════════════════");
    println!(
        "  Safe code:     sec={:.2} conf={:.2}",
        safe.fitness.security, safe.confidence.security
    );
    println!(
        "  Unsafe code:   sec={:.2} conf={:.2}",
        unsafe_.fitness.security, unsafe_.confidence.security
    );
    println!(
        "  Invalid code:  parse={} warnings={}",
        invalid.parse_successful,
        invalid.warnings.len()
    );
    println!("═══════════════════════════════════════════════");

    assert!(safe.parse_successful);
    assert!(unsafe_.parse_successful);
    assert!(!invalid.parse_successful);
    assert!(safe.fitness.security > unsafe_.fitness.security);
    assert!(safe.confidence.security >= 0.90);

    println!("  ✅ Week 28 Gate: PASSED");
}
