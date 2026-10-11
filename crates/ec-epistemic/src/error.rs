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
    /// تجزئة SHA-256 للأثر غير صالحة (ليست 64 حرفًا سداسيًا صغيرًا).
    #[error("invalid artifact sha256 hash: {hash}")]
    InvalidArtifactHash {
        /// الهاش المرفوض كما ورد.
        hash: String,
    },
    /// مرجع صورة الحاوية غير صالح: غير مثبَّت بالبصمة، أو مستودعه فارغ.
    #[error("invalid sandbox image reference (must be digest-pinned): {image}")]
    InvalidSandboxImage {
        /// مرجع الصورة المرفوض كما ورد.
        image: String,
    },
    /// قائمة رموز الخروج فارغة: لا وجود لتشغيل واحد.
    #[error("exit codes cannot be empty")]
    EmptyExitCodes,
    /// رمز خروج غير صالح — `-1` يعني أن البرنامج لم يعد نتيجة أصلًا.
    #[error("invalid exit code {code}: -1 indicates missing execution result")]
    InvalidExitCode {
        /// رمز الخروج المرفوض.
        code: i32,
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
