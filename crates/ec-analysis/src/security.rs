#![forbid(unsafe_code)]

//! مقاييس الأمان من التحليل الثابت.

use crate::metrics::{clamp01, count_pattern};

/// مقاييس الأمان المُشتقة من الكود.
#[derive(Debug, Clone)]
pub struct SecurityMetrics {
    /// عدد `unsafe` blocks.
    pub unsafe_blocks: usize,
    /// عدد `.unwrap()` calls.
    pub unwrap_calls: usize,
    /// عدد `.expect()` calls.
    pub expect_calls: usize,
    /// عدد `panic!` macros.
    pub panic_calls: usize,
}

impl SecurityMetrics {
    /// تحليل الكود واستخراج مقاييس الأمان.
    pub fn from_code(code: &str) -> Self {
        Self {
            unsafe_blocks: count_pattern(code, "unsafe"),
            unwrap_calls: count_pattern(code, ".unwrap()"),
            expect_calls: count_pattern(code, ".expect("),
            panic_calls: count_pattern(code, "panic!"),
        }
    }

    /// درجة الأمان (0.0 - 1.0).
    ///
    /// - 1.0 = لا مشاكل أمنية
    /// - كل `unsafe`: -0.3
    /// - كل `.unwrap()`: -0.05
    /// - كل `.expect()`: -0.05
    /// - كل `panic!`: -0.1
    pub fn score(&self) -> f64 {
        let mut s = 1.0;
        s -= self.unsafe_blocks as f64 * 0.3;
        s -= self.unwrap_calls as f64 * 0.05;
        s -= self.expect_calls as f64 * 0.05;
        s -= self.panic_calls as f64 * 0.1;
        clamp01(s)
    }

    /// هل الكود خالٍ من `unsafe`؟
    pub fn is_safe(&self) -> bool {
        self.unsafe_blocks == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_issues_scores_1() {
        let m = SecurityMetrics::from_code("fn main() { let x = 1; }");
        assert_eq!(m.score(), 1.0);
        assert!(m.is_safe());
    }

    #[test]
    fn one_unsafe_scores_07() {
        let m = SecurityMetrics::from_code("unsafe { }");
        assert!((m.score() - 0.7).abs() < 0.01);
        assert!(!m.is_safe());
    }

    #[test]
    fn unwrap_penalty() {
        let m = SecurityMetrics::from_code("x.unwrap()");
        assert!((m.score() - 0.95).abs() < 0.01);
    }

    #[test]
    fn panic_penalty() {
        let m = SecurityMetrics::from_code("panic!(\"no\")");
        assert!((m.score() - 0.9).abs() < 0.01);
    }

    #[test]
    fn combined_penalties() {
        let code = "unsafe { x.unwrap() } panic!(\"a\")";
        let m = SecurityMetrics::from_code(code);
        // 1.0 - 0.3 - 0.05 - 0.1 = 0.55
        assert!((m.score() - 0.55).abs() < 0.01);
    }

    #[test]
    fn many_unsafe_floors_at_zero() {
        let code = "unsafe {} unsafe {} unsafe {} unsafe {}";
        let m = SecurityMetrics::from_code(code);
        assert_eq!(m.score(), 0.0);
    }
}
