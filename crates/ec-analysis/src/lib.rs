#![forbid(unsafe_code)]

//! ec-analysis — Static Code Analysis (Week 19)
//! Produce FitnessVector from source code. Truth ≠ Fitness preserved.

use ec_fitness::FitnessVector;

/// Analyze source code and produce a FitnessVector.
/// Uses struct literal — FitnessVector has no ::new() constructor.
pub fn analyze_code(code: &str) -> FitnessVector {
    FitnessVector {
        security:               security_score(code),
        reversibility:          reversibility_score(code),
        test_coverage:          coverage_score(code),
        maintainability:        complexity_to_maintainability(complexity_score(code)),
        performance:            performance_score(code),
        architectural_stability: architectural_score(code),
    }
}

fn security_score(code: &str) -> f64 {
    let unsafe_count = code.matches("unsafe").count();
    let unwrap_count = code.matches("unwrap()").count();
    let expect_count = code.matches("expect(").count();
    let panic_count  = code.matches("panic!").count();

    let penalty = (unsafe_count as f64 * 0.5
        + unwrap_count as f64 * 0.2
        + expect_count as f64 * 0.1
        + panic_count  as f64 * 0.3)
        .min(1.0);
    1.0 - penalty
}

fn coverage_score(code: &str) -> f64 {
    let test_count = code.matches("#[test]").count();
    let fn_count   = code.matches("fn ").count().saturating_sub(test_count);
    if fn_count == 0 {
        return 0.5;
    }
    (test_count as f64 / fn_count as f64).min(1.0)
}

fn complexity_score(code: &str) -> f64 {
    let branches = code.matches("if ").count()
        + code.matches("match ").count()
        + code.matches("for ").count()
        + code.matches("while ").count()
        + code.matches("&&").count()
        + code.matches("||").count();
    (branches as f64 * 0.1).min(1.0)
}

fn complexity_to_maintainability(complexity: f64) -> f64 {
    1.0 - complexity
}

fn reversibility_score(code: &str) -> f64 {
    let side_effects = code.matches("println!").count()
        + code.matches("eprintln!").count()
        + code.matches("static mut").count()
        + code.matches("unsafe ").count()
        + code.matches("Mutex::new").count();
    1.0 - (side_effects as f64 * 0.15).min(1.0)
}

fn performance_score(code: &str) -> f64 {
    let allocations = code.matches("alloc").count()
        + code.matches("Vec::new").count()
        + code.matches("String::from").count()
        + code.matches("Box::new").count()
        + code.matches("HashMap::new").count();
    1.0 - (allocations as f64 * 0.2).min(1.0)
}

fn architectural_score(code: &str) -> f64 {
    let uses = code.matches("use ").count();
    1.0 - (uses as f64 * 0.1).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_simple_function() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let fitness = analyze_code(code);
        assert!(fitness.security > 0.8);
        assert!(fitness.reversibility > 0.8);
    }

    #[test]
    fn test_analyze_with_tests() {
        let code = "#[test]\nfn test_add() {}\nfn add(a: i32, b: i32) -> i32 { a + b }";
        let fitness = analyze_code(code);
        assert!(fitness.test_coverage > 0.0);
    }

    #[test]
    fn test_security_detects_unsafe() {
        let code = "unsafe fn bad() { let p: *const u8 = std::ptr::null(); }";
        let fitness = analyze_code(code);
        assert!(fitness.security < 0.8);
    }

    #[test]
    fn test_security_detects_unwrap() {
        let code = "fn foo() { let x = Some(1).unwrap(); }";
        let fitness = analyze_code(code);
        assert!(fitness.security < 1.0);
    }

    #[test]
    fn test_reversibility_detects_side_effects() {
        let code = "fn foo() { println!(\"hello\"); }";
        let fitness = analyze_code(code);
        assert!(fitness.reversibility < 1.0);
    }

    #[test]
    fn test_fitness_vector_all_dimensions_present() {
        let code = "fn foo() {}";
        let fitness = analyze_code(code);
        assert!(fitness.security >= 0.0 && fitness.security <= 1.0);
        assert!(fitness.reversibility >= 0.0 && fitness.reversibility <= 1.0);
        assert!(fitness.test_coverage >= 0.0 && fitness.test_coverage <= 1.0);
        assert!(fitness.maintainability >= 0.0 && fitness.maintainability <= 1.0);
        assert!(fitness.performance >= 0.0 && fitness.performance <= 1.0);
        assert!(fitness.architectural_stability >= 0.0 && fitness.architectural_stability <= 1.0);
    }

    #[test]
    fn test_pure_function_high_reversibility() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let fitness = analyze_code(code);
        assert_eq!(fitness.reversibility, 1.0);
    }

    #[test]
    fn test_complex_code_lower_maintainability() {
        let code = "fn f() { if true { if true { if true { if true { if true {} } } } } }";
        let fitness = analyze_code(code);
        assert!(fitness.maintainability < 1.0);
    }

    #[test]
    fn test_no_allocations_high_performance() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let fitness = analyze_code(code);
        assert_eq!(fitness.performance, 1.0);
    }

    #[test]
    fn test_no_uses_high_stability() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let fitness = analyze_code(code);
        assert_eq!(fitness.architectural_stability, 1.0);
    }
}
