#![forbid(unsafe_code)]

//! أدوات مشتركة لتحليل الكود.

/// عد non-overlapping occurrences لنمط في الكود.
pub fn count_pattern(code: &str, pattern: &str) -> usize {
    code.matches(pattern).count()
}

/// عد السطور غير الفارغة.
pub fn count_lines(code: &str) -> usize {
    code.lines().filter(|l| !l.trim().is_empty()).count()
}

/// عد تعريفات الدوال.
pub fn count_functions(code: &str) -> usize {
    code.matches("fn ").count()
}

/// عد دوال الاختبار.
pub fn count_test_functions(code: &str) -> usize {
    code.matches("#[test]").count()
        + code.matches("#[tokio::test]").count()
}

/// تأكد من أن القيمة في النطاق [0.0, 1.0].
pub fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_pattern_works() {
        assert_eq!(count_pattern("unsafe { unsafe {", "unsafe"), 2);
        assert_eq!(count_pattern("no match here", "unsafe"), 0);
    }

    #[test]
    fn count_functions_works() {
        assert_eq!(count_functions("fn main() {} fn foo() {}"), 2);
        assert_eq!(count_functions("no functions"), 0);
    }

    #[test]
    fn count_test_functions_works() {
        let code = "#[test]\nfn test_a() {} #[tokio::test]\nfn test_b() {}";
        assert_eq!(count_test_functions(code), 2);
    }

    #[test]
    fn clamp01_works() {
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(1.5), 1.0);
    }
}
