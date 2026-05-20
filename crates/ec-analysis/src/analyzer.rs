#![forbid(unsafe_code)]

//! المُحلِّل الرئيسي — يُنتج FitnessVector من الكود مباشرة.

use ec_fitness::fitness::FitnessVector;

use crate::complexity::ComplexityMetrics;
use crate::coverage::CoverageMetrics;
use crate::metrics::count_pattern;
use crate::reversibility::ReversibilityMetrics;
use crate::security::SecurityMetrics;

/// تحليل الكود وإنتاج FitnessVector.
///
/// هذا هو الـ API الرئيسي لـ ec-analysis.
/// يقيس خصائص الكود مباشرة — لا يعتمد على نتيجة التنفيذ.
///
/// # Design Rule: Truth ≠ Fitness
///
/// - `RealityVector` = نتيجة التنفيذ (صح/خطأ، تكرار، latency)
/// - `FitnessVector` = خصائص الكود (أمان، تغطية، تعقيد، ...)
/// - لا أحدهما يُشتق من الآخر
pub fn analyze_code(code: &str) -> FitnessVector {
    let security = SecurityMetrics::from_code(code);
    let coverage = CoverageMetrics::from_code(code);
    let complexity = ComplexityMetrics::from_code(code);
    let reversibility = ReversibilityMetrics::from_code(code);

    FitnessVector {
        security: security.score(),
        test_coverage: coverage.score(),
        reversibility: reversibility.score(),
        maintainability: complexity.maintainability_score(),
        performance: performance_score(code),
        architectural_stability: architectural_score(code),
    }
}

/// درجة الأداء — تُقدِّر كفاءة التخصيص (allocations).
///
/// - 0 تخصيصات → 1.0
/// - كل تخصيص: -0.03
/// - الحد الأدنى: 0.3
fn performance_score(code: &str) -> f64 {
    let allocations = count_pattern(code, "String::")
        + count_pattern(code, "Vec::")
        + count_pattern(code, "Box::")
        + count_pattern(code, "HashMap::")
        + count_pattern(code, "clone()")
        + count_pattern(code, "to_string()")
        + count_pattern(code, "to_owned()")
        + count_pattern(code, "format!");

    (1.0 - allocations as f64 * 0.03).clamp(0.3, 1.0)
}

/// درجة الاستقرار المعماري — تقيس الاقتران (coupling).
///
/// - 0 `use` statements → 1.0
/// - كل `use`: -0.02
/// - الحد الأدنى: 0.4
fn architectural_score(code: &str) -> f64 {
    let uses = count_pattern(code, "use ");
    (1.0 - uses as f64 * 0.02).clamp(0.4, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_code_all_in_range() {
        let f = analyze_code("");
        assert!(f.security >= 0.0 && f.security <= 1.0);
        assert!(f.test_coverage >= 0.0 && f.test_coverage <= 1.0);
        assert!(f.reversibility >= 0.0 && f.reversibility <= 1.0);
        assert!(f.maintainability >= 0.0 && f.maintainability <= 1.0);
        assert!(f.performance >= 0.0 && f.performance <= 1.0);
        assert!(f.architectural_stability >= 0.0 && f.architectural_stability <= 1.0);
    }

    #[test]
    fn simple_main_high_security() {
        let f = analyze_code("fn main() { let x = 42; }");
        assert_eq!(f.security, 1.0);
        assert!(f.reversibility > 0.9);
    }

    #[test]
    fn unsafe_code_low_security() {
        let f = analyze_code("unsafe { *ptr }");
        assert!(f.security < 0.8);
    }

    #[test]
    fn validates_successfully() {
        let f = analyze_code("fn main() {}");
        assert!(f.validate().is_ok());
    }

    #[test]
    fn consistent_results() {
        let code = "fn main() { let x = 1; }";
        let f1 = analyze_code(code);
        let f2 = analyze_code(code);
        assert_eq!(f1.security, f2.security);
        assert_eq!(f1.test_coverage, f2.test_coverage);
        assert_eq!(f1.maintainability, f2.maintainability);
    }

    #[test]
    fn many_allocations_lower_performance() {
        let code = "let a = String::new(); let b = Vec::new(); let c = Box::new(1); \
                    let d = a.clone(); let e = b.clone();";
        let f = analyze_code(code);
        assert!(f.performance < 0.9);
    }

    #[test]
    fn many_uses_lower_architecture() {
        let code = "use a; use b; use c; use d; use e; use f; use g; use h; \
                    use i; use j; use k; use l; use m; use n; use o; use p;";
        let f = analyze_code(code);
        assert!(f.architectural_stability < 0.8);
    }
}
