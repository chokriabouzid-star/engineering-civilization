#![forbid(unsafe_code)]

//! Week 31 Gate — TestVisitor + CouplingVisitor + SideEffectVisitor

use syn::visit::Visit;
use ec_analysis::analyze_code_full;
use ec_analysis::visitors::{TestVisitor, CouplingVisitor, SideEffectVisitor};

// ─── TestVisitor ────────────────────────────────────────────────────

fn test_visitor(code: &str) -> TestVisitor {
    let ast = syn::parse_file(code).unwrap();
    let mut v = TestVisitor::new();
    v.visit_file(&ast);
    v
}

#[test]
fn w31_no_fns_coverage_neutral() {
    let v = test_visitor("// empty file\n");
    let (score, _) = v.score();
    assert_eq!(score, 0.5);
}

#[test]
fn w31_prod_only_zero_coverage() {
    let v = test_visitor("fn foo() {} fn bar() {}");
    let (score, _) = v.score();
    assert_eq!(score, 0.0);
}

#[test]
fn w31_equal_test_prod_full_coverage() {
    let code = "#[test]\nfn test_a() {} fn prod_a() {}";
    let v = test_visitor(code);
    let (score, _) = v.score();
    assert_eq!(score, 1.0);
}

#[test]
fn w31_half_coverage() {
    let code = "#[test]\nfn test_a() {} fn prod_a() {} fn prod_b() {}";
    let v = test_visitor(code);
    let (score, _) = v.score();
    assert!((score - 0.5).abs() < 0.01, "got: {}", score);
}

#[test]
fn w31_assert_macros_counted() {
    let code = "#[test]\nfn test_it() { assert!(true); assert_eq!(1, 1); assert_ne!(1, 2); }";
    let v = test_visitor(code);
    assert_eq!(v.assert_count, 3);
}

#[test]
fn w31_test_coverage_confidence_low_without_tests() {
    let v = test_visitor("fn foo() {}");
    let (_, conf) = v.score();
    assert_eq!(conf, 0.25, "no tests → low confidence");
}

#[test]
fn w31_test_coverage_confidence_medium_with_tests() {
    let code = "#[test]\nfn test_a() {} fn prod() {}";
    let v = test_visitor(code);
    let (_, conf) = v.score();
    assert_eq!(conf, 0.50, "has tests → medium confidence");
}

// ─── CouplingVisitor ────────────────────────────────────────────────

fn coupling_visitor(code: &str) -> CouplingVisitor {
    let ast = syn::parse_file(code).unwrap();
    let mut v = CouplingVisitor::new();
    v.visit_file(&ast);
    v
}

#[test]
fn w31_no_uses_full_stability() {
    let v = coupling_visitor("fn f() {}");
    let (score, _) = v.score();
    assert_eq!(score, 1.0);
}

#[test]
fn w31_std_uses_less_penalty() {
    // use std::io → UsePath { ident: "std" } → std_uses = 1 → 0.03 penalty
    let v = coupling_visitor("use std::io;");
    let (score, _) = v.score();
    assert!(score > 0.90, "std use → small penalty, got: {}", score);
}

#[test]
fn w31_external_uses_more_penalty() {
    // use serde::Serialize → UsePath { ident: "serde" } → external_uses = 1 → 0.12 penalty
    let v = coupling_visitor("use serde::Serialize;");
    let (score, _) = v.score();
    assert!(score < 0.95, "external use → bigger penalty, got: {}", score);
}

#[test]
fn w31_many_uses_lower_stability() {
    let v1 = coupling_visitor("use std::io;");
    let v5 = coupling_visitor("use a::b; use c::d; use e::f; use g::h; use i::j;");
    let (s1, _) = v1.score();
    let (s5, _) = v5.score();
    assert!(s5 < s1, "s1={:.2}, s5={:.2}", s1, s5);
}

#[test]
fn w31_coupling_confidence() {
    let v = coupling_visitor("fn f() {}");
    let (_, conf) = v.score();
    assert_eq!(conf, 0.75);
}

// ─── SideEffectVisitor ─────────────────────────────────────────────

fn side_visitor(code: &str) -> SideEffectVisitor {
    let ast = syn::parse_file(code).unwrap();
    let mut v = SideEffectVisitor::new();
    v.visit_file(&ast);
    v
}

#[test]
fn w31_pure_code_full_reversibility() {
    let v = side_visitor("fn add(a: i32, b: i32) -> i32 { a + b }");
    let (score, _) = v.score();
    assert_eq!(score, 1.0);
}

#[test]
fn w31_println_penalized() {
    let v = side_visitor("fn f() { println!(\"hi\"); }");
    let (score, _) = v.score();
    assert!(score < 1.0, "got: {}", score);
}

#[test]
fn w31_eprintln_penalized() {
    let v = side_visitor("fn f() { eprintln!(\"err\"); }");
    let (score, _) = v.score();
    assert!(score < 1.0, "got: {}", score);
}

#[test]
fn w31_static_mut_double_penalty() {
    let v = side_visitor("static mut COUNT: i32 = 0;");
    assert_eq!(v.static_muts, 1);
    let (score, _) = v.score();
    assert!(score < 0.80, "static mut → double penalty, got: {}", score);
}

#[test]
fn w31_static_const_no_penalty() {
    let v = side_visitor("const MAX: i32 = 100;");
    assert_eq!(v.static_muts, 0);
    let (score, _) = v.score();
    assert_eq!(score, 1.0);
}

#[test]
fn w31_multiple_side_effects_cumulative() {
    let v = side_visitor("fn f() { println!(\"a\"); println!(\"b\"); eprintln!(\"c\"); }");
    let (score, _) = v.score();
    assert!(score < 0.70, "3 writes → cumulative, got: {}", score);
}

#[test]
fn w31_side_effect_confidence() {
    let v = side_visitor("fn f() {}");
    let (_, conf) = v.score();
    assert_eq!(conf, 0.70);
}

// ─── Integration: analyze_code_full ─────────────────────────────────

#[test]
fn w31_full_report_safe_code() {
    let r = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    assert!(r.fitness.security >= 0.95);
    assert!(r.fitness.reversibility >= 0.95);
    assert!(r.fitness.architectural_stability >= 0.95);
    assert!(r.confidence.security >= 0.90);
}

#[test]
fn w31_full_report_unsafe_io_code() {
    let code = "use serde::Serialize;\nfn f(p: *const i32) { unsafe { println!(\"{:?}\", *p); } }";
    let r = analyze_code_full(code);
    assert!(r.fitness.security < 0.90);
    assert!(r.fitness.reversibility < 1.0);
    assert!(r.fitness.architectural_stability < 1.0);
}

#[test]
fn w31_full_report_with_tests() {
    let code = "#[test]\nfn test_add() { assert_eq!(1 + 1, 2); }\nfn add(a: i32, b: i32) -> i32 { a + b }";
    let r = analyze_code_full(code);
    assert!(r.fitness.test_coverage > 0.0);
    assert!(r.confidence.test_coverage >= 0.40);
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn week31_gate_complete() {
    let safe = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    let coupled = analyze_code_full("use serde::Serialize; use tokio::io;");
    let side = analyze_code_full("fn f() { println!(\"hi\"); eprintln!(\"err\"); }");
    let tested = analyze_code_full("#[test]\nfn test_it() { assert!(true); }\nfn prod() {}");

    println!("═══════════════════════════════════════════════");
    println!("  Week 31 Gate — 3 Visitors");
    println!("═══════════════════════════════════════════════");
    println!("  Safe:      rev={:.2} stab={:.2} cov={:.2}",
        safe.fitness.reversibility, safe.fitness.architectural_stability,
        safe.fitness.test_coverage);
    println!("  Coupled:   stab={:.2}", coupled.fitness.architectural_stability);
    println!("  SideFx:    rev={:.2}", side.fitness.reversibility);
    println!("  Tested:    cov={:.2}", tested.fitness.test_coverage);
    println!("═══════════════════════════════════════════════");

    assert!(safe.fitness.reversibility > side.fitness.reversibility);
    assert!(safe.fitness.architectural_stability > coupled.fitness.architectural_stability);
    assert!(tested.fitness.test_coverage > safe.fitness.test_coverage);

    println!("  ✅ Week 31 Gate: PASSED");
}
