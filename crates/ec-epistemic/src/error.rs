#![forbid(unsafe_code)]
use thiserror::Error;

/// أنواع الأخطاء الخاصة بالنموذج المعرفي.
#[derive(Debug, Error)]
pub enum EpistemicError {
    /// القيمة ليست رقماً محدوداً.
    #[error("non-finite value for {field}: {value}")]
    NonFiniteValue {
        /// اسم الحقل.
        field: &'static str,
        /// القيمة.
        value: f64,
    },
    /// القيمة خارج النطاق.
    #[error("out of range for {field}: {value} (expected {min}..={max})")]
    OutOfRange {
        /// اسم الحقل.
        field: &'static str,
        /// القيمة.
        value: f64,
        /// الأدنى.
        min: f64,
        /// الأقصى.
        max: f64,
    },
}

/// نتيجة العمليات المعرفية.
pub type EpistemicResult<T> = Result<T, EpistemicError>;

pub(crate) fn ensure_finite(field: &'static str, v: f64) -> EpistemicResult<()> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(EpistemicError::NonFiniteValue { field, value: v })
    }
}

pub(crate) fn ensure_in_range(
    field: &'static str,
    v: f64,
    min: f64,
    max: f64,
) -> EpistemicResult<()> {
    ensure_finite(field, v)?;
    if (min..=max).contains(&v) {
        Ok(())
    } else {
        Err(EpistemicError::OutOfRange {
            field,
            value: v,
            min,
            max,
        })
    }
}
