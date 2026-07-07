#![forbid(unsafe_code)]

//! Week 33 Gate — PerformanceVisitor (AST-based)
//!
//! Gate Criteria:
//! ✓ كود بلا allocations → performance = 1.0
//! ✓ Vec::new / String::new / Box::new → عقوبة
//! ✓ clone() / to_string() → عقوبة
//! ✓ format! / vec! macro → عقوبة
//! ✓ أكثر allocations → score أقل
//! ✓ confidence معقولة
//! ✓ يمر عبر analyze_code_full()

use ec_analysis::analyze_code_full;
use ec_analysis::visitors::PerformanceVisitor;
use syn::visit::Visit;

fn perf_visitor(code: &str) -> PerformanceVisitor {
    let ast = syn::parse_file(code).unwrap();
    let mut v = PerformanceVisitor::new();
    v.visit_file(&ast);
    v
}

fn perf_score(code: &str) -> f64 {
    analyze_code_full(code).fitness.performance
}

// ─── Gate 1: لا allocations → 1.0 ──────────────────────────────────

#[test]
fn w33_no_allocations_score_1() {
    let v = perf_visitor("fn f() -> i32 { 1 }");
    let (score, _) = v.score();
    assert_eq!(score, 1.0);
}

#[test]
fn w33_pure_math_score_1() {
    assert_eq!(perf_score("fn add(a: i32, b: i32) -> i32 { a + b }"), 1.0);
}

// ─── Gate 2: allocations → عقوبة ──────────────────────────────────

#[test]
fn w33_vec_new_penalized() {
    let v = perf_visitor("fn f() { let v = Vec::new(); }");
    assert_eq!(v.allocations, 1);
    let (score, _) = v.score();
    assert!(score < 1.0, "got: {}", score);
}

#[test]
fn w33_string_new_penalized() {
    let v = perf_visitor("fn f() { let s = String::new(); }");
    assert_eq!(v.allocations, 1);
}

#[test]
fn w33_box_new_penalized() {
    let v = perf_visitor("fn f() { let b = Box::new(42); }");
    assert_eq!(v.allocations, 1);
}

#[test]
fn w33_hashmap_new_penalized() {
    let v = perf_visitor("fn f() { let m = HashMap::new(); }");
    assert_eq!(v.allocations, 1);
}

// ─── Gate 3: clone / to_string → عقوبة ────────────────────────────

#[test]
fn w33_clone_penalized() {
    let v = perf_visitor("fn f(x: Vec<i32>) { let y = x.clone(); }");
    assert_eq!(v.clones, 1);
    let (score, _) = v.score();
    assert!(score < 1.0, "got: {}", score);
}

#[test]
fn w33_to_string_penalized() {
    let v = perf_visitor("fn f(n: i32) { let s = n.to_string(); }");
    assert_eq!(v.clones, 1);
}

// ─── Gate 4: format! / vec! macros ─────────────────────────────────

#[test]
fn w33_format_macro_penalized() {
    let v = perf_visitor("fn f() { let s = format!(\"{}\"); }");
    assert_eq!(v.allocations, 1);
}

#[test]
fn w33_vec_macro_penalized() {
    let v = perf_visitor("fn f() { let v = vec![1, 2, 3]; }");
    assert_eq!(v.allocations, 1);
}

// ─── Gate 5: أكثر → أقل ────────────────────────────────────────────

#[test]
fn w33_more_allocs_lower_score() {
    let s1 = perf_score("fn f() { let v = Vec::new(); }");
    let s3 =
        perf_score("fn f() { let v = Vec::new(); let s = String::new(); let b = Box::new(1); }");
    assert!(s3 < s1, "s1={:.2}, s3={:.2}", s1, s3);
}

// ─── Gate 6: confidence ─────────────────────────────────────────────

#[test]
fn w33_perf_confidence() {
    let v = perf_visitor("fn f() {}");
    let (_, conf) = v.score();
    assert_eq!(conf, 0.75);
}

// ─── Gate 7: integration ────────────────────────────────────────────

#[test]
fn w33_full_report_perf() {
    let r = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    assert_eq!(r.fitness.performance, 1.0);
    assert!(r.confidence.performance >= 0.70);
}

#[test]
fn w33_full_report_alloc_code() {
    let r = analyze_code_full("fn f() { let v = Vec::new(); let s = String::from(\"hi\"); }");
    assert!(
        r.fitness.performance < 1.0,
        "got: {}",
        r.fitness.performance
    );
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn week33_gate_complete() {
    let pure = analyze_code_full("fn add(a: i32, b: i32) -> i32 { a + b }");
    let alloc = analyze_code_full(
        "fn f() { let v = Vec::new(); let s = String::new(); let b = Box::new(1); let m = HashMap::new(); }"
    );

    println!("═══════════════════════════════════════════════");
    println!("  Week 33 Gate — PerformanceVisitor");
    println!("═══════════════════════════════════════════════");
    println!(
        "  Pure code:   perf={:.2} conf={:.2}",
        pure.fitness.performance, pure.confidence.performance
    );
    println!(
        "  Alloc code:  perf={:.2} conf={:.2}",
        alloc.fitness.performance, alloc.confidence.performance
    );
    println!("═══════════════════════════════════════════════");

    assert_eq!(pure.fitness.performance, 1.0);
    assert!(alloc.fitness.performance < pure.fitness.performance);

    println!("  ✅ Week 33 Gate: PASSED");
}
