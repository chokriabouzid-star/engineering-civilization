#![forbid(unsafe_code)]

//! Week 29 Gate — UnsafeVisitor (الأمن بالسياق)
//!
//! Gate Criteria:
//! ✓ تعليق "unsafe" في نص لا يُعاقب (AST vs v1)
//! ✓ كود آمن → score عالي
//! ✓ unsafe block → عقوبة
//! ✓ أكثر unsafe → score أقل
//! ✓ doc comment على unsafe fn → عقوبة مخفّضة
//! ✓ confidence عالية لتحليل AST

use ec_analysis::{analyze_code, analyze_code_full};

fn security_ast(code: &str) -> f64 {
    analyze_code_full(code).fitness.security
}

// ─── Gate 1: تعليق "unsafe" لا يُعاقب ──────────────────────────────

#[test]
fn w29_comment_not_penalized() {
    let code = "// unsafe usage here\nfn safe() -> i32 { 42 }";
    assert!(
        security_ast(code) >= 0.95,
        "تعليق 'unsafe' لا يُعاقب عليه، got: {}",
        security_ast(code)
    );
}

#[test]
fn w29_comment_vs_v1() {
    let code = "// unsafe usage documented here\nfn safe_fn() -> i32 { 42 }";
    let v1 = analyze_code(code).security;
    let v15 = security_ast(code);
    assert!(v15 >= v1, "AST={:.2} >= v1={:.2}", v15, v1);
}

#[test]
fn w29_string_literal_not_penalized() {
    let code = r#"fn msg() -> &str { "unsafe operation" }"#;
    assert!(
        security_ast(code) >= 0.95,
        "string containing 'unsafe' should not be penalized, got: {}",
        security_ast(code)
    );
}

// ─── Gate 2: كود آمن → score عالي ─────────────────────────────────

#[test]
fn w29_safe_code_scores_high() {
    assert!(security_ast("fn add(a: i32, b: i32) -> i32 { a + b }") >= 0.95);
}

#[test]
fn w29_safe_code_confidence_high() {
    let r = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    assert!(
        r.confidence.security >= 0.90,
        "AST analysis → confidence عالية، got: {}",
        r.confidence.security
    );
}

// ─── Gate 3: unsafe block → عقوبة ─────────────────────────────────

#[test]
fn w29_unsafe_block_penalized() {
    let score = security_ast("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    assert!(score < 0.85, "got: {}", score);
}

#[test]
fn w29_unsafe_fn_penalized() {
    let score = security_ast("unsafe fn raw(p: *const i32) -> i32 { *p }");
    assert!(score < 0.90, "got: {}", score);
}

// ─── Gate 4: أكثر unsafe → score أقل ──────────────────────────────

#[test]
fn w29_more_unsafe_lower_score() {
    let s1 = security_ast("fn f() { unsafe {} }");
    let s3 = security_ast("fn f() { unsafe {} unsafe {} unsafe {} }");
    assert!(s3 < s1, "s1={:.2}, s3={:.2}", s1, s3);
}

// ─── Gate 5: doc comment يُخفّض العقوبة ────────────────────────────

#[test]
fn w29_doc_comment_mitigates_unsafe_fn() {
    let without_doc = security_ast("unsafe fn raw(p: *const i32) -> i32 { *p }");
    let with_doc = security_ast(
        "/// # Safety\n/// Verified pointer\nunsafe fn raw(p: *const i32) -> i32 { *p }",
    );
    assert!(
        with_doc > without_doc,
        "with_doc={:.2} > without_doc={:.2}",
        with_doc,
        without_doc
    );
}

#[test]
fn w29_doc_comment_does_not_help_expr_block() {
    // ExprBlock لا يمكنه حمل doc comment مباشرة
    let without = security_ast("fn f() { unsafe { let _ = 1; } }");
    let with_doc = security_ast("/// safety\nfn f() { unsafe { let _ = 1; } }");
    // doc على الدالة لا تؤثر على unsafe block داخلها
    assert_eq!(
        with_doc, without,
        "doc on fn should not affect unsafe expr block inside"
    );
}

// ─── Gate 6: confidence ─────────────────────────────────────────────

#[test]
fn w29_confidence_high_for_ast() {
    let r = analyze_code_full("fn f() {}");
    assert!(
        r.confidence.security >= 0.90,
        "AST analysis → confidence عالية، got: {}",
        r.confidence.security
    );
}

#[test]
fn w29_unsafe_confidence_still_high() {
    let r = analyze_code_full("unsafe fn f() {}");
    // حتى مع unsafe، confidence في التحليل نفسه عالية (رأينا AST فعلاً)
    assert!(
        r.confidence.security >= 0.85,
        "confidence in detection should be high, got: {}",
        r.confidence.security
    );
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn week29_gate_complete() {
    let safe = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    let unsafe_ = analyze_code_full("fn f(p: *const i32) { unsafe { let _ = *p; } }");
    let comment = analyze_code_full("// unsafe docs\nfn safe() -> i32 { 42 }");
    let doc_unsafe = analyze_code_full("/// # Safety\nunsafe fn raw(p: *const i32) -> i32 { *p }");

    println!("═══════════════════════════════════════════════");
    println!("  Week 29 Gate — UnsafeVisitor (Context)");
    println!("═══════════════════════════════════════════════");
    println!(
        "  Safe code:       sec={:.2} conf={:.2}",
        safe.fitness.security, safe.confidence.security
    );
    println!(
        "  Unsafe block:    sec={:.2} conf={:.2}",
        unsafe_.fitness.security, unsafe_.confidence.security
    );
    println!(
        "  Comment only:    sec={:.2} (not penalized)",
        comment.fitness.security
    );
    println!(
        "  Doc+unsafe fn:   sec={:.2} (mitigated)",
        doc_unsafe.fitness.security
    );
    println!("═══════════════════════════════════════════════");

    assert!(safe.fitness.security > unsafe_.fitness.security);
    assert!(comment.fitness.security >= 0.95);
    assert!(safe.confidence.security >= 0.90);

    println!("  ✅ Week 29 Gate: PASSED");
}
