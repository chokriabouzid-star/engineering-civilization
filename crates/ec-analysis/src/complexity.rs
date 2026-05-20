#![forbid(unsafe_code)]

//! تعقيد الكود ومقاييس القابلية للصيانة.

use crate::metrics::{count_functions, count_pattern};

/// مقاييس التعقيد المُشتقة من الكود.
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    /// التعقيد الحلقي المُقدَّر (cyclomatic complexity).
    pub cyclomatic: usize,
    /// عدد الدوال.
    pub function_count: usize,
}

impl ComplexityMetrics {
    /// تحليل الكود واستخراج مقاييس التعقيد.
    pub fn from_code(code: &str) -> Self {
        let cyclomatic = count_pattern(code, "if ")
            + count_pattern(code, "else if")
            + count_pattern(code, "match ")
            + count_pattern(code, "&&")
            + count_pattern(code, "||")
            + count_pattern(code, "while ")
            + count_pattern(code, "for ")
            + count_pattern(code, "loop ")
            + count_pattern(code, "?");

        Self {
            cyclomatic,
            function_count: count_functions(code),
        }
    }

    /// متوسط التعقيد لكل دالة.
    pub fn avg_complexity(&self) -> f64 {
        if self.function_count == 0 {
            return 1.0;
        }
        self.cyclomatic as f64 / self.function_count as f64
    }

    /// درجة القابلية للصيانة (0.0 - 1.0).
    ///
    /// - تعقيد 1 → 1.0
    /// - تعقيد 5 → 0.71
    /// - تعقيد 10 → 0.53
    /// - تعقيد 20 → 0.33
    pub fn maintainability_score(&self) -> f64 {
        let avg = self.avg_complexity();
        1.0 / (1.0 + (avg - 1.0).max(0.0) * 0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_function_low_complexity() {
        let m = ComplexityMetrics::from_code("fn main() { let x = 1; }");
        assert_eq!(m.cyclomatic, 0);
        assert!((m.maintainability_score() - 1.0).abs() < 0.01);
    }

    #[test]
    fn branching_increases_complexity() {
        let code = "fn foo() { if x { } if y { } }";
        let m = ComplexityMetrics::from_code(code);
        assert_eq!(m.cyclomatic, 2);
    }

    #[test]
    fn match_counts_as_branch() {
        let code = "fn foo() { match x { 1 => {}, 2 => {} } }";
        let m = ComplexityMetrics::from_code(code);
        assert!(m.cyclomatic >= 1);
    }

    #[test]
    fn complex_code_lower_score() {
        let code = "fn foo() { if a && b || c { if d { } } }";
        let m = ComplexityMetrics::from_code(code);
        assert!(m.maintainability_score() < 0.8);
    }

    #[test]
    fn no_functions_defaults_to_1() {
        let m = ComplexityMetrics::from_code("let x = 1;");
        assert_eq!(m.avg_complexity(), 1.0);
    }
}
