#![forbid(unsafe_code)]

//! تقدير تغطية الاختبارات من الكود.

use crate::metrics::{count_functions, count_test_functions};

/// مقاييس التغطية المُشتقة من الكود.
#[derive(Debug, Clone)]
pub struct CoverageMetrics {
    /// عدد الدوال الإجمالي.
    pub total_functions: usize,
    /// عدد دوال الاختبار.
    pub test_functions: usize,
}

impl CoverageMetrics {
    /// تحليل الكود واستخراج مقاييس التغطية.
    pub fn from_code(code: &str) -> Self {
        Self {
            total_functions: count_functions(code),
            test_functions: count_test_functions(code),
        }
    }

    /// درجة التغطية (0.0 - 1.0).
    ///
    /// - لا دوال → 0.5 (محايد)
    /// - دوال بدون اختبارات → 0.0
    /// - كل الدوال مُختبرة → 1.0
    pub fn score(&self) -> f64 {
        if self.total_functions == 0 {
            return 0.5;
        }
        (self.test_functions as f64 / self.total_functions as f64).clamp(0.0, 1.0)
    }

    /// هل كل الدوال مُختبرة؟
    pub fn is_fully_covered(&self) -> bool {
        self.total_functions > 0 && self.test_functions >= self.total_functions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_functions_neutral() {
        let m = CoverageMetrics::from_code("let x = 1;");
        assert_eq!(m.score(), 0.5);
    }

    #[test]
    fn one_fn_no_test_zero() {
        let m = CoverageMetrics::from_code("fn main() {}");
        assert_eq!(m.score(), 0.0);
    }

    #[test]
    fn one_fn_one_test_full() {
        let code = "#[test]\nfn test_it() {} fn main() {}";
        let m = CoverageMetrics::from_code(code);
        assert!((m.score() - 0.5).abs() < 0.01);
        assert!(!m.is_fully_covered());
    }

    #[test]
    fn all_tested() {
        let code = "#[test]\nfn test_a() {} #[test]\nfn test_b() {}";
        let m = CoverageMetrics::from_code(code);
        assert!(m.is_fully_covered());
        assert_eq!(m.score(), 1.0);
    }
}
