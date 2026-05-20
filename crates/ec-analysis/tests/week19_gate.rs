#![deny(warnings)]
#![forbid(unsafe_code)]

//! Week 19 Gate — Static Code Analysis
//!
//! Gate Criteria:
//! ✓ analyze_code() يُنتج FitnessVector صالح
//! ✓ security: unsafe → penalty
//! ✓ coverage: #[test] counted
//! ✓ complexity: branches increase complexity
//! ✓ reversibility: side effects lower score
//! ✓ performance: allocations lower score
//! ✓ architectural: use statements lower score
//! ✓ FitnessVector::validate() passes

use ec_analysis::analyze_code;

// ─── Gate 1: Basic Analysis ──────────────────────────────────────────

#[test]
fn gate_analyze_code_returns_valid_fitness() {
    let f = analyze_code("fn main() { let x = 42; }");
    assert!(f.validate().is_ok(), "FitnessVector should be valid");
}

#[test]
fn gate_all_dimensions_in_range() {
    let f = analyze_code("fn main() {}");
    assert!((0.0..=1.0).contains(&f.security));
    assert!((0.0..=1.0).contains(&f.test_coverage));
    assert!((0.0..=1.0).contains(&f.reversibility));
    assert!((0.0..=1.0).contains(&f.maintainability));
    assert!((0.0..=1.0).contains(&f.performance));
    assert!((0.0..=1.0).contains(&f.architectural_stability));
}

// ─── Gate 2: Security ────────────────────────────────────────────────

#[test]
fn gate_safe_code_security_1() {
    let f = analyze_code("fn main() { let x = 1; }");
    assert_eq!(f.security, 1.0);
}

#[test]
fn gate_unsafe_code_penalty() {
    let f = analyze_code("fn main() { unsafe { *ptr; } }");
    assert!(f.security < 0.8, "unsafe should penalize: {}", f.security);
}

#[test]
fn gate_unwrap_penalty() {
    let f1 = analyze_code("fn main() { let x = opt; }");
    let f2 = analyze_code("fn main() { let x = opt.unwrap(); }");
    assert!(f2.security < f1.security);
}

// ─── Gate 3: Coverage ────────────────────────────────────────────────

#[test]
fn gate_no_functions_coverage_neutral() {
    let f = analyze_code("let x = 1;");
    assert_eq!(f.test_coverage, 0.5);
}

#[test]
fn gate_functions_without_tests_coverage_zero() {
    let f = analyze_code("fn main() {} fn foo() {}");
    assert_eq!(f.test_coverage, 0.0);
}

#[test]
fn gate_test_functions_boost_coverage() {
    let code = "#[test]\nfn test_a() {} #[test]\nfn test_b() {} fn main() {}";
    let f = analyze_code(code);
    assert!(f.test_coverage > 0.0, "coverage: {}", f.test_coverage);
}

// ─── Gate 4: Complexity ──────────────────────────────────────────────

#[test]
fn gate_simple_code_high_maintainability() {
    let f = analyze_code("fn main() { let x = 1; }");
    assert!(f.maintainability > 0.9);
}

#[test]
fn gate_complex_code_lower_maintainability() {
    let simple = analyze_code("fn main() { let x = 1; }");
    let complex = analyze_code(
        "fn foo() { if a { if b { if c { if d { } } } } }"
    );
    assert!(complex.maintainability < simple.maintainability);
}

// ─── Gate 5: Reversibility ───────────────────────────────────────────

#[test]
fn gate_pure_code_high_reversibility() {
    let f = analyze_code("fn main() { let x = 1 + 2; }");
    assert_eq!(f.reversibility, 1.0);
}

#[test]
fn gate_io_lowers_reversibility() {
    let f = analyze_code("fn main() { println!(\"hello\"); }");
    assert!(f.reversibility < 1.0);
}

// ─── Gate 6: Performance ─────────────────────────────────────────────

#[test]
fn gate_no_allocations_high_performance() {
    let f = analyze_code("fn main() { let x = 1; }");
    assert_eq!(f.performance, 1.0);
}

#[test]
fn gate_allocations_lower_performance() {
    let f = analyze_code("fn main() { let s = String::new(); let v = Vec::new(); }");
    assert!(f.performance < 1.0);
}

// ─── Gate 7: Architectural Stability ─────────────────────────────────

#[test]
fn gate_no_uses_high_stability() {
    let f = analyze_code("fn main() {}");
    assert_eq!(f.architectural_stability, 1.0);
}

#[test]
fn gate_many_uses_lower_stability() {
    let code = "use a; use b; use c; use d; use e; \
                use f; use g; use h; use i; use j;";
    let f = analyze_code(code);
    assert!(f.architectural_stability < 1.0);
}

// ─── Gate 8: Consistency ─────────────────────────────────────────────

#[test]
fn gate_same_code_same_result() {
    let code = "fn main() { let x = 42; }";
    let f1 = analyze_code(code);
    let f2 = analyze_code(code);
    assert_eq!(f1.security, f2.security);
    assert_eq!(f1.test_coverage, f2.test_coverage);
    assert_eq!(f1.reversibility, f2.reversibility);
    assert_eq!(f1.maintainability, f2.maintainability);
    assert_eq!(f1.performance, f2.performance);
    assert_eq!(f1.architectural_stability, f2.architectural_stability);
}

// ─── Gate 9: Edge Cases ──────────────────────────────────────────────

#[test]
fn gate_empty_code_valid() {
    let f = analyze_code("");
    assert!(f.validate().is_ok());
}

#[test]
fn gate_comment_only_valid() {
    let f = analyze_code("// this is a comment\n/* block */");
    assert!(f.validate().is_ok());
}

// ─── Final Gate ──────────────────────────────────────────────────────

#[test]
fn week19_gate_complete() {
    let safe = analyze_code("fn main() { let x = 42; }");
    let unsafe_code = analyze_code("unsafe { *ptr; }");
    let tested = analyze_code("#[test]\nfn test_it() {} fn main() {}");
    let complex = analyze_code("fn foo() { if a && b || c { if d { match e { 1 => {}, _ => {} } } } }");
    let io = analyze_code("fn main() { println!(\"hi\"); fs::write(\"a\", \"b\"); }");

    println!("═══════════════════════════════════════════════");
    println!("  Week 19 Gate — Static Code Analysis");
    println!("═══════════════════════════════════════════════");
    println!("  Safe code:      sec={:.2} cov={:.2} maint={:.2}",
        safe.security, safe.test_coverage, safe.maintainability);
    println!("  Unsafe code:    sec={:.2} (should be < 0.8)",
        unsafe_code.security);
    println!("  Tested code:    cov={:.2} (should be > 0.0)",
        tested.test_coverage);
    println!("  Complex code:   maint={:.2} (should be < 0.9)",
        complex.maintainability);
    println!("  IO code:        rev={:.2} (should be < 1.0)",
        io.reversibility);
    println!("═══════════════════════════════════════════════");

    assert!(safe.security > 0.9);
    assert!(unsafe_code.security < 0.8);
    assert!(tested.test_coverage > 0.0);
    assert!(complex.maintainability < 0.9);
    assert!(io.reversibility < 1.0);

    assert!(safe.validate().is_ok());
    assert!(unsafe_code.validate().is_ok());
    assert!(tested.validate().is_ok());
    assert!(complex.validate().is_ok());
    assert!(io.validate().is_ok());

    println!("  ✅ Week 19 Gate: PASSED");
}
