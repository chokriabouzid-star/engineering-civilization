#![forbid(unsafe_code)]

//! Depth and structural nesting guard to prevent stack overflow during parsing.

/// أقصى عمق مسموح به للأقواس والكتل المتداخلة
pub const MAX_DELIMITER_DEPTH: usize = 128;

/// أقصى تكرار مسموح به للعمليات الأحادية المتتالية (مثل &&&& أو ****)
pub const MAX_UNARY_CHAIN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepthError {
    DelimiterDepthExceeded { depth: usize, max: usize },
    UnaryChainExceeded { count: usize, max: usize },
}

impl std::fmt::Display for DepthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DelimiterDepthExceeded { depth, max } => {
                write!(f, "Nesting depth {} exceeds safety limit of {}", depth, max)
            }
            Self::UnaryChainExceeded { count, max } => {
                write!(
                    f,
                    "Unary operator chain {} exceeds safety limit of {}",
                    count, max
                )
            }
        }
    }
}

/// فحص سريع وخفيف للبايتات للتأكد من عدم وجود تعشيق تعاودي يكسر الـ stack
pub fn check_nesting_safety(code: &str) -> Result<(), DepthError> {
    let bytes = code.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    let mut delimiter_depth: usize = 0;
    let mut max_observed_depth: usize = 0;
    let mut unary_chain: usize = 0;
    let mut max_unary_observed: usize = 0;

    while i < len {
        let b = bytes[i];

        // 1. تخطي التعليقات الخطية //
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // 2. تخطي التعليقات الكتلية /* ... */
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            let mut block_depth = 1;
            while i + 1 < len && block_depth > 0 {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    block_depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    block_depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // 3. تخطي السلاسل النصية العادية "..."
        if b == b'"' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2; // تخطي escape
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // 4. تخطي الحروف '...'
        if b == b'\'' {
            i += 1;
            while i < len {
                if bytes[i] == b'\\' {
                    i += 2;
                } else if bytes[i] == b'\'' {
                    i += 1;
                    break;
                } else if bytes[i] == b'\n' || bytes[i] == b';' {
                    break; // تجنب التعليق عند استخدام ' في lifetimes
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // 5. حساب عمق الأقواس
        match b {
            b'{' | b'(' | b'[' => {
                delimiter_depth += 1;
                if delimiter_depth > max_observed_depth {
                    max_observed_depth = delimiter_depth;
                }
                if delimiter_depth > MAX_DELIMITER_DEPTH {
                    return Err(DepthError::DelimiterDepthExceeded {
                        depth: delimiter_depth,
                        max: MAX_DELIMITER_DEPTH,
                    });
                }
                unary_chain = 0;
            }
            b'}' | b')' | b']' => {
                delimiter_depth = delimiter_depth.saturating_sub(1);
                unary_chain = 0;
            }
            b'&' | b'*' | b'!' => {
                unary_chain += 1;
                if unary_chain > max_unary_observed {
                    max_unary_observed = unary_chain;
                }
                if unary_chain > MAX_UNARY_CHAIN {
                    return Err(DepthError::UnaryChainExceeded {
                        count: unary_chain,
                        max: MAX_UNARY_CHAIN,
                    });
                }
            }
            b' ' | b'\t' | b'\r' | b'\n' => {
                // الفراغات لا تصفر تكرار العمليات الأحادية مثل & & &
            }
            _ => {
                unary_chain = 0;
            }
        }

        i += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shallow_code_passes() {
        let code = "fn main() { let x = (1 + 2); [1, 2, 3]; }";
        assert!(check_nesting_safety(code).is_ok());
    }

    #[test]
    fn deep_braces_rejected() {
        let mut code = String::from("fn f() ");
        for _ in 0..150 {
            code.push('{');
        }
        code.push_str("let x = 1;");
        for _ in 0..150 {
            code.push('}');
        }
        assert!(matches!(
            check_nesting_safety(&code),
            Err(DepthError::DelimiterDepthExceeded { .. })
        ));
    }

    #[test]
    fn deep_unary_rejected() {
        let code = format!("fn f() {{ let x: {}u8 = todo!(); }}", "&".repeat(100));
        assert!(matches!(
            check_nesting_safety(&code),
            Err(DepthError::UnaryChainExceeded { .. })
        ));
    }
}
