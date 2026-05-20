#![forbid(unsafe_code)]

//! مقاييس القابلية للعكس — كشف التأثيرات الجانبية.

use crate::metrics::count_pattern;

/// مقاييس القابلية للعكس المُشتقة من الكود.
#[derive(Debug, Clone)]
pub struct ReversibilityMetrics {
    /// عدد عمليات التأثير الجانبي.
    pub side_effects: usize,
}

impl ReversibilityMetrics {
    /// تحليل الكود واستخراج مقاييس القابلية للعكس.
    pub fn from_code(code: &str) -> Self {
        let side_effects = count_pattern(code, "println!")
            + count_pattern(code, "print!")
            + count_pattern(code, "eprintln!")
            + count_pattern(code, "fs::")
            + count_pattern(code, "File::")
            + count_pattern(code, "Command::")
            + count_pattern(code, "write!")
            + count_pattern(code, "std::net::")
            + count_pattern(code, "std::process::")
            + count_pattern(code, "TcpStream::")
            + count_pattern(code, "UdpSocket::")
            + count_pattern(code, ".write(")
            + count_pattern(code, ".flush()");

        Self { side_effects }
    }

    /// درجة القابلية للعكس (0.0 - 1.0).
    ///
    /// - 0 تأثيرات جانبية → 1.0
    /// - كل تأثير: -0.05
    /// - الحد الأدنى: 0.2
    pub fn score(&self) -> f64 {
        (1.0 - self.side_effects as f64 * 0.05).clamp(0.2, 1.0)
    }

    /// هل الكود خالٍ من التأثيرات الجانبية؟
    pub fn is_pure(&self) -> bool {
        self.side_effects == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_code_scores_1() {
        let m = ReversibilityMetrics::from_code("fn main() { let x = 1; }");
        assert_eq!(m.score(), 1.0);
        assert!(m.is_pure());
    }

    #[test]
    fn println_lowers_score() {
        let m = ReversibilityMetrics::from_code("println!(\"hi\")");
        assert!((m.score() - 0.95).abs() < 0.01);
        assert!(!m.is_pure());
    }

    #[test]
    fn many_side_effects_floor() {
        let code = "println!() println!() println!() println!() println!() \
                    println!() println!() println!() println!() println!() \
                    println!() println!() println!() println!() println!() \
                    println!() println!() println!() println!() println!()";
        let m = ReversibilityMetrics::from_code(code);
        assert_eq!(m.score(), 0.2);
    }

    #[test]
    fn fs_counted_as_side_effect() {
        let m = ReversibilityMetrics::from_code("fs::write(\"a\", \"b\")");
        assert!(m.side_effects >= 1);
    }
}
