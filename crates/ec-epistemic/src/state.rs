#![forbid(unsafe_code)]
use crate::calibration::CalibrationState;
use crate::error::{ensure_finite, ensure_in_range, EpistemicResult};
use serde::{Deserialize, Serialize};

/// الأدلة الداعمة للتقييم.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    /// حجم العينة.
    pub sample_size: u64,
    /// عمر الدليل بالثواني.
    pub age_seconds: u64,
    /// قابلية إعادة الإنتاج.
    pub reproducibility: f64,
    /// موثوقية المصدر.
    pub source_reliability: f64,
}

impl Evidence {
    /// إنشاء دليل جديد.
    pub fn new(
        sample_size: u64,
        age_seconds: u64,
        reproducibility: f64,
        source_reliability: f64,
    ) -> EpistemicResult<Self> {
        ensure_in_range("reproducibility", reproducibility, 0.0, 1.0)?;
        ensure_in_range("source_reliability", source_reliability, 0.0, 1.0)?;
        Ok(Self {
            sample_size,
            age_seconds,
            reproducibility,
            source_reliability,
        })
    }

    /// التحقق من صحة الدليل.
    pub fn validate(&self) -> EpistemicResult<()> {
        ensure_in_range("reproducibility", self.reproducibility, 0.0, 1.0)?;
        ensure_in_range("source_reliability", self.source_reliability, 0.0, 1.0)?;
        Ok(())
    }

    /// اختيار الدليل الأضعف.
    pub fn weakest(all: &[Evidence]) -> Option<Evidence> {
        let first = all.first()?.clone();
        Some(all.iter().skip(1).fold(first, |acc, e| Evidence {
            sample_size: acc.sample_size.min(e.sample_size),
            age_seconds: acc.age_seconds.max(e.age_seconds),
            reproducibility: acc.reproducibility.min(e.reproducibility),
            source_reliability: acc.source_reliability.min(e.source_reliability),
        }))
    }
}

/// تحليل مكونات عدم اليقين.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UncertaintyDecomposition {
    /// عشوائي (Aleatoric).
    pub aleatoric: f64,
    /// معرفي (Epistemic).
    pub epistemic: f64,
    /// نموذجي (Model).
    pub model: f64,
}

impl UncertaintyDecomposition {
    /// إنشاء تحليل جديد.
    pub fn new(aleatoric: f64, epistemic: f64, model: f64) -> EpistemicResult<Self> {
        ensure_finite("aleatoric", aleatoric)?;
        ensure_finite("epistemic", epistemic)?;
        ensure_finite("model", model)?;
        if aleatoric < 0.0 || epistemic < 0.0 || model < 0.0 {
            return Err(crate::error::EpistemicError::OutOfRange {
                field: "uncertainty",
                value: 0.0,
                min: 0.0,
                max: f64::INFINITY,
            });
        }
        Ok(Self {
            aleatoric,
            epistemic,
            model,
        })
    }

    /// إجمالي عدم اليقين (RMS).
    pub fn total(&self) -> f64 {
        (self.aleatoric.powi(2) + self.epistemic.powi(2) + self.model.powi(2)).sqrt()
    }

    /// التحقق من صحة التحليل.
    pub fn validate(&self) -> EpistemicResult<()> {
        ensure_finite("aleatoric", self.aleatoric)?;
        ensure_finite("epistemic", self.epistemic)?;
        ensure_finite("model", self.model)?;
        Ok(())
    }
}

/// الحالة المعرفية الكاملة.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpistemicState {
    /// الثقة [0,1].
    pub confidence: f64,
    /// الأدلة.
    pub evidence: Evidence,
    /// عدم اليقين.
    pub uncertainty: UncertaintyDecomposition,
    /// المعايرة.
    pub calibration: CalibrationState,
}

impl EpistemicState {
    /// إنشاء حالة جديدة.
    pub fn new(
        confidence: f64,
        evidence: Evidence,
        uncertainty: UncertaintyDecomposition,
        calibration: CalibrationState,
    ) -> EpistemicResult<Self> {
        ensure_in_range("confidence", confidence, 0.0, 1.0)?;
        evidence.validate()?;
        uncertainty.validate()?;
        Ok(Self {
            confidence,
            evidence,
            uncertainty,
            calibration,
        })
    }

    /// إجمالي عدم اليقين للحالة.
    pub fn total_uncertainty(&self) -> f64 {
        self.uncertainty.total()
    }

    /// التحقق من صحة الحالة.
    pub fn validate(&self) -> EpistemicResult<()> {
        ensure_in_range("confidence", self.confidence, 0.0, 1.0)?;
        self.evidence.validate()?;
        self.uncertainty.validate()?;
        Ok(())
    }
}
