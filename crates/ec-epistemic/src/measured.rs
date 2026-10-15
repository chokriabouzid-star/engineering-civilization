#![forbid(unsafe_code)]

//! الدليل التجريبي المقاس من بيئة تنفيذ حقيقية.
//!
//! هذا الدليل هو الحامل الوحيد المسموح له بنقل نتيجة تنفيذ فعلي إلى الحالة المعرفية.

use crate::error::{ensure_in_range, EpistemicError, EpistemicResult};
use serde::{Deserialize, Serialize};

/// مصدر بيئة تنفيذ الأثر البرمجي.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionProvenance {
    /// تنفيذ حقيقي ومعزول داخل حاوية Docker — المصدر الوحيد الذي يمنح سلطة.
    Docker,
    /// تنفيذ محاكى: لا يمنح سلطة، ويُبلغ عنه صراحةً.
    Simulated,
    /// تنفيذ محلي مباشر بلا عزل حاوية: لا يمنح سلطة.
    Local,
}

/// معرّف أوراكل التحقق من صحة التنفيذ. لا يوجد افتراضي له: غياب الاختيار غياب السلطة.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OracleId {
    /// النجاح مشروط بعودة رمز الخروج صفرًا **من البرنامج فعلًا** (أوراكل ExitCodeZero).
    ///
    /// لا يعني تحقيق متطلبات وظيفية يقدّمها المالك — ذلك مؤجَّل
    /// (ADR-033 «Not decided here»).
    ExitCodeZero,
}

fn is_lowercase_hex_64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// صورة حاوية مثبَّتة بالبصمة. يرفض اسم المستودع الفارغ (`@sha256:<digest>` وحده).
fn is_pinned_image(image: &str) -> bool {
    image
        .split_once("@sha256:")
        .is_some_and(|(repo, digest)| !repo.is_empty() && is_lowercase_hex_64(digest))
}

/// دليل مقاس من تشغيل فعلي للأثر البرمجي.
///
/// لا يُبنى إلا عبر [`MeasuredEvidence::new`]، ولا يُستخرج من نصّ عبر
/// `#[serde(try_from)]` إلا بمروره بالباني نفسه: لا طريق إلى دليل غير مُتحقَّق منه.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "MeasuredEvidenceRaw")]
pub struct MeasuredEvidence {
    provenance: ExecutionProvenance,
    artifact_sha256: String,
    sandbox_image: String,
    exit_codes: Vec<i32>,
    reproducibility: f64,
    oracle: OracleId,
}

#[derive(Deserialize)]
struct MeasuredEvidenceRaw {
    provenance: ExecutionProvenance,
    artifact_sha256: String,
    sandbox_image: String,
    exit_codes: Vec<i32>,
    reproducibility: f64,
    oracle: OracleId,
}

impl TryFrom<MeasuredEvidenceRaw> for MeasuredEvidence {
    type Error = EpistemicError;
    fn try_from(raw: MeasuredEvidenceRaw) -> Result<Self, Self::Error> {
        Self::new(
            raw.provenance,
            raw.artifact_sha256,
            raw.sandbox_image,
            raw.exit_codes,
            raw.reproducibility,
            raw.oracle,
        )
    }
}

impl MeasuredEvidence {
    /// الطريق الوحيد لبناء دليل مقاس. يرفض كل ما لا يمكن الدفاع عنه:
    ///
    /// - هاش الأثر: 64 حرفًا سداسيًا صغيرًا (صيغة SHA-256؛ الحساب في `ec-app`).
    /// - صورة الحاوية: مثبَّتة بالبصمة `@sha256:` إلزامًا (ADR-033)، وبمستودع غير فارغ.
    /// - قائمة رموز الخروج: غير فارغة.
    /// - **كل رمز خارج `0..=255` مرفوض**: نظام التشغيل يحجب حالة خروج البرنامج
    ///   في هذا المجال. خاصةً `-1`، وهو ما يكتبه `hardened.rs:230` حين لا تُرجع
    ///   **عملية `docker` المحلية** رمز خروج (`ExitStatus::code() == None`) —
    ///   أي فشل مُشغِّل، **لا** «قتل البرنامج بإشارة». قبول أيٍّ من ذلك كان
    ///   سيجعل ادّعاء `ProgramRuns` كذبة.
    /// - معامل التكرار في `[0, 1]` (ويرفض NaN/∞ مجانًا عبر `ensure_in_range`).
    ///
    /// **EC-ADJ-01:** تحويل `-1` وأعطال الحاوية إلى خطأ تنفيذ صريح (لا إلى دليل)
    /// مؤجَّل إلى PR2 عند ربط مخرجات Docker الفعلية.
    pub fn new(
        provenance: ExecutionProvenance,
        artifact_sha256: String,
        sandbox_image: String,
        exit_codes: Vec<i32>,
        reproducibility: f64,
        oracle: OracleId,
    ) -> EpistemicResult<Self> {
        if !is_lowercase_hex_64(&artifact_sha256) {
            return Err(EpistemicError::InvalidArtifactHash {
                hash: artifact_sha256,
            });
        }
        if !is_pinned_image(&sandbox_image) {
            return Err(EpistemicError::InvalidSandboxImage {
                image: sandbox_image,
            });
        }
        if exit_codes.is_empty() {
            return Err(EpistemicError::EmptyExitCodes);
        }
        if let Some(&code) = exit_codes.iter().find(|&&c| !(0..=255).contains(&c)) {
            return Err(EpistemicError::InvalidExitCode { code });
        }
        ensure_in_range("reproducibility", reproducibility, 0.0, 1.0)?;
        Ok(Self {
            provenance,
            artifact_sha256,
            sandbox_image,
            exit_codes,
            reproducibility,
            oracle,
        })
    }

    /// إعادة التحقق — يُستدعى من `EpistemicState::validate`.
    pub fn validate(&self) -> EpistemicResult<()> {
        if !is_lowercase_hex_64(&self.artifact_sha256) {
            return Err(EpistemicError::InvalidArtifactHash {
                hash: self.artifact_sha256.clone(),
            });
        }
        if !is_pinned_image(&self.sandbox_image) {
            return Err(EpistemicError::InvalidSandboxImage {
                image: self.sandbox_image.clone(),
            });
        }
        if self.exit_codes.is_empty() {
            return Err(EpistemicError::EmptyExitCodes);
        }
        if let Some(&code) = self.exit_codes.iter().find(|&&c| !(0..=255).contains(&c)) {
            return Err(EpistemicError::InvalidExitCode { code });
        }
        ensure_in_range("reproducibility", self.reproducibility, 0.0, 1.0)?;
        Ok(())
    }

    /// مصدر البيئة الذي أُنتج منه الدليل.
    pub fn provenance(&self) -> ExecutionProvenance {
        self.provenance
    }
    /// بصمة SHA-256 للأثر الذي قِيس فعليًا.
    pub fn artifact_sha256(&self) -> &str {
        &self.artifact_sha256
    }
    /// صورة الحاوية المثبَّتة بالبصمة.
    pub fn sandbox_image(&self) -> &str {
        &self.sandbox_image
    }
    /// رموز الخروج المسجّلة، بترتيب التشغيلات.
    pub fn exit_codes(&self) -> &[i32] {
        &self.exit_codes
    }
    /// معامل استقرار النتيجة عبر التشغيلات، في `[0, 1]`.
    pub fn reproducibility(&self) -> f64 {
        self.reproducibility
    }
    /// أوراكل التحكيم المعتمد في هذا الدليل.
    pub fn oracle(&self) -> OracleId {
        self.oracle
    }
    /// عدد التشغيلات المكتملة — **مشتق** من `exit_codes.len()`، لا حقل مخزَّن.
    pub fn runs_completed(&self) -> usize {
        self.exit_codes.len()
    }
    /// هل نجحت كل التشغيلات (كل رموز الخروج صفرًا)؟
    pub fn all_succeeded(&self) -> bool {
        self.exit_codes.iter().all(|&c| c == 0)
    }
    /// عدد التشغيلات الفاشلة (رمز خروج غير صفري) — **مشتق** كذلك.
    pub fn failed_runs(&self) -> usize {
        self.exit_codes.iter().filter(|&&c| c != 0).count()
    }
}
