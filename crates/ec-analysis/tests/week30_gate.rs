#![forbid(unsafe_code)]

//! Week 30 Gate — ComplexityVisitor (Cyclomatic Complexity)

use syn::visit::Visit;
use ec_analysis::analyze_code_full;
use ec_analysis::visitors::ComplexityVisitor;

fn cc_of(code: &str) -> u32 {
    let ast = syn::parse_file(code).unwrap();
    let mut v = ComplexityVisitor::new();
    v.visit_file(&ast);
    v.functions.first().map(|f| f.cc).unwrap_or(0)
}

// ─── Gate 1: دالة بسيطة → cc = 1 ───────────────────────────────────

#[test]
fn w30_simple_fn_cc_1() {
    assert_eq!(cc_of("fn f() -> i32 { 1 }"), 1);
}

#[test]
fn w30_empty_fn_cc_1() {
    assert_eq!(cc_of("fn f() {}"), 1);
}

// ─── Gate 2: if تضيف 1 ─────────────────────────────────────────────

#[test]
fn w30_if_adds_1() {
    assert_eq!(cc_of("fn f(x: bool) { if x {} }"), 2);
}

#[test]
fn w30_two_ifs_add_2() {
    assert_eq!(cc_of("fn f(a: bool, b: bool) { if a {} if b {} }"), 3);
}

// ─── Gate 3: match arms تُحسب ──────────────────────────────────────

#[test]
fn w30_match_arms_counted() {
    let code = "fn f(x: i32) { match x { 1 => {}, 2 => {}, _ => {} } }";
    assert_eq!(cc_of(code), 3);
}

#[test]
fn w30_single_arm_plus_wildcard() {
    let code = "fn f(x: i32) { match x { 1 => {}, _ => {} } }";
    assert_eq!(cc_of(code), 2);
}

// ─── Gate 4: && و || تُحسب ─────────────────────────────────────────

#[test]
fn w30_and_counts() {
    assert_eq!(cc_of("fn f(a: bool, b: bool) { if a && b {} }"), 3);
}

#[test]
fn w30_or_counts() {
    assert_eq!(cc_of("fn f(a: bool, b: bool) { if a || b {} }"), 3);
}

// ─── Gate 5: ? يُحسب ───────────────────────────────────────────────

#[test]
fn w30_question_mark_counts() {
    let code = "fn f() -> Result<i32, ()> { let _ = ok()?; Ok(1) }";
    assert!(cc_of(code) >= 2, "expected >= 2, got {}", cc_of(code));
}

#[test]
fn w30_multiple_question_marks() {
    let code = "fn f() -> Result<i32, ()> { let _ = ok()?; let _ = ok()?; let _ = ok()?; Ok(1) }";
    assert!(cc_of(code) >= 4, "expected >= 4, got {}", cc_of(code));
}

// ─── Gate 6: تعقيد أعلى → maintainability أقل ──────────────────────

#[test]
fn w30_complex_lower_maintainability() {
    let simple = analyze_code_full("fn f() -> i32 { 1 }");
    let complex = analyze_code_full(
        "fn f(a: bool, b: bool, c: bool) { if a { if b { if c {} } } }"
    );
    assert!(simple.fitness.maintainability > complex.fitness.maintainability,
        "simple={:.2}, complex={:.2}",
        simple.fitness.maintainability, complex.fitness.maintainability);
}

#[test]
fn w30_maintainability_in_range() {
    let r = analyze_code_full("fn f() { if a { if b { if c { if d {} } } } }");
    assert!(r.fitness.maintainability >= 0.0 && r.fitness.maintainability <= 1.0);
}

// ─── Gate 7: دوال الاختبار لا تؤثر ─────────────────────────────────

#[test]
fn w30_test_fn_excluded_from_production() {
    let code = "#[test]\nfn test_foo() { assert!(true); } fn prod() {}";
    let ast = syn::parse_file(code).unwrap();
    let mut v = ComplexityVisitor::new();
    v.visit_file(&ast);

    assert_eq!(v.functions.len(), 2);
    let prod = v.functions.iter().find(|f| f.name == "prod").unwrap();
    assert!(!prod.is_test);
    let test = v.functions.iter().find(|f| f.name == "test_foo").unwrap();
    assert!(test.is_test);
}

// ─── Gate 8: دوال متداخلة ──────────────────────────────────────────

#[test]
fn w30_multiple_functions_independent() {
    let code = "fn simple() {} fn complex(a: bool) { if a { if b {} } }";
    let ast = syn::parse_file(code).unwrap();
    let mut v = ComplexityVisitor::new();
    v.visit_file(&ast);

    assert_eq!(v.functions.len(), 2);
    let simple = v.functions.iter().find(|f| f.name == "simple").unwrap();
    let complex = v.functions.iter().find(|f| f.name == "complex").unwrap();
    assert!(simple.cc < complex.cc);
}

// ─── Gate 9: confidence ─────────────────────────────────────────────

#[test]
fn w30_confidence_reasonable() {
    let r = analyze_code_full("fn f() -> i32 { 1 }");
    assert!(r.confidence.maintainability >= 0.80,
        "got: {}", r.confidence.maintainability);
}

#[test]
fn w30_high_complexity_no_crash() {
    let code = "fn f() { if a {} if b {} if c {} }";
    let r = analyze_code_full(code);
    assert!(r.fitness.validate().is_ok());
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn week30_gate_complete() {
    let simple = analyze_code_full("fn f() -> i32 { 1 }");
    let complex = analyze_code_full(
        "fn f(a: bool, b: bool, c: bool) { if a && b || c { if d {} } }"
    );

    println!("═══════════════════════════════════════════════");
    println!("  Week 30 Gate — ComplexityVisitor (CC)");
    println!("═══════════════════════════════════════════════");
    println!("  Simple fn:   maint={:.2} conf={:.2}",
        simple.fitness.maintainability, simple.confidence.maintainability);
    println!("  Complex fn:  maint={:.2} conf={:.2}",
        complex.fitness.maintainability, complex.confidence.maintainability);
    println!("═══════════════════════════════════════════════");

    assert!(simple.fitness.maintainability > complex.fitness.maintainability);
    assert!(simple.confidence.maintainability >= 0.80);

    println!("  ✅ Week 30 Gate: PASSED");
}
