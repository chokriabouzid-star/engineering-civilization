#![forbid(unsafe_code)]

//! Bayesian Evidence — تحديث Bayesian للأدلة
//! إضافة فقط — لا يُعدّل Evidence القديم

use crate::error::{ensure_in_range, EpistemicResult};

/// أدلة Bayesian — تتبع النجاحات والفشل بشكل صريح.
///
///不同于 Evidence القديم (sample_size/age/reproducibility),
/// هذا الـ struct يُتبع outcomes فعليّة.
#[derive(Debug, Clone, PartialEq)]
pub struct BayesianEvidence {
    /// عدد النجاحات
    pub successes: u32,
    /// عدد الفشل
    pub failures: u32,
    /// متوسط الدرجات
    pub mean_score: f64,
    /// تقدير التباين
    pub variance_estimate: f64,
}

impl BayesianEvidence {
    /// Prior غير متحيز — لا نفترض نجاحاً لم يحدث
    pub fn initial_prior() -> EpistemicResult<Self> {
        Ok(Self {
            successes: 0,
            failures: 0,
            mean_score: 0.5,
            variance_estimate: 0.8,
        })
    }

    /// إنشاء من تاريخ فعلي
    pub fn from_history(successes: u32, failures: u32, mean_score: f64) -> EpistemicResult<Self> {
        ensure_in_range("mean_score", mean_score, 0.0, 1.0)?;
        Ok(Self {
            successes,
            failures,
            mean_score,
            variance_estimate: 0.9,
        })
    }

    /// تحديث Bayesian بعد تشغيل فعلي
    pub fn update_with_outcome(&self, was_correct: bool, score: f64) -> EpistemicResult<Self> {
        ensure_in_range("score", score, 0.0, 1.0)?;
        let n = (self.successes + self.failures) as f64;
        let new_mean = if n > 0.0 {
            (self.mean_score * n + score) / (n + 1.0)
        } else {
            score
        };

        Ok(if was_correct {
            Self {
                successes: self.successes + 1,
                failures: self.failures,
                mean_score: new_mean,
                variance_estimate: 0.9,
            }
        } else {
            Self {
                successes: self.successes,
                failures: self.failures + 1,
                mean_score: new_mean,
                variance_estimate: 0.9,
            }
        })
    }

    /// Wilson confidence interval — أدق من mean_score مباشرة
    pub fn credible_confidence(&self) -> f64 {
        let n = (self.successes + self.failures) as f64;
        if n < 5.0 {
            return 0.45;
        }

        let p = self.successes as f64 / n;
        let z = 1.645_f64;
        let denom = 1.0 + z * z / n;
        let center = p + z * z / (2.0 * n);
        let spread = (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();

        ((center - z * spread) / denom).clamp(0.30, 0.95)
    }

    /// إجمالي المشاهدات
    pub fn total_observations(&self) -> u32 {
        self.successes + self.failures
    }
}
