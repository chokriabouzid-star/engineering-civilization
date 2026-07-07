#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::evaluation::ConstitutionalEvaluation;
use ec_fitness::fitness::FitnessVector;
use serde::{Deserialize, Serialize};

// ─── 1. OssificationDetector ────────────────────────────────────────

/// سبب محتمل لمراجعة الدستور.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReviewReason {
    /// معدل الرفض مرتفع جداً.
    HighRejectionRate(f64),
    /// تنوع القرارات منخفض جداً.
    LowDecisionDiversity(f64),
    /// الأولويات المطبقة انجرفت عن خط الأساس.
    ValueDriftDetected { degrees: f64, threshold: f64 },
}

/// كاشف التحجر الدستوري.
/// يراقب ما إذا كان الدستور يرفض كل شيء.
#[derive(Debug, Default, Clone)]
pub struct OssificationDetector {
    total_evaluations: u64,
    rejected_evaluations: u64,
}

impl OssificationDetector {
    pub const REJECTION_RATE_THRESHOLD: f64 = 0.90;

    pub fn new() -> Self {
        Default::default()
    }

    /// تسجيل تقييم جديد.
    pub fn record(&mut self, evaluation: &ConstitutionalEvaluation) {
        self.total_evaluations += 1;
        if !evaluation.is_valid {
            self.rejected_evaluations += 1;
        }
    }

    /// حساب معدل الرفض الحالي.
    pub fn rejection_rate(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            self.rejected_evaluations as f64 / self.total_evaluations as f64
        }
    }

    /// التحقق مما إذا كانت المراجعة مطلوبة.
    pub fn needs_review(&self) -> Option<ReviewReason> {
        let rate = self.rejection_rate();
        if self.total_evaluations > 50 && rate > Self::REJECTION_RATE_THRESHOLD {
            Some(ReviewReason::HighRejectionRate(rate))
        } else {
            None
        }
    }
}

// ─── 2. ValueDriftDetector ──────────────────────────────────────────

fn dot_product(a: &FitnessVector, b: &FitnessVector) -> f64 {
    a.security * b.security
        + a.reversibility * b.reversibility
        + a.test_coverage * b.test_coverage
        + a.maintainability * b.maintainability
        + a.performance * b.performance
        + a.architectural_stability * b.architectural_stability
}

fn magnitude(v: &FitnessVector) -> f64 {
    dot_product(v, v).sqrt()
}

/// كاشف الانجراف القيمي.
/// يراقب ما إذا كانت أولويات النظام تتغير بمرور الوقت.
#[derive(Debug, Clone)]
pub struct ValueDriftDetector {
    baseline: FitnessVector,
    current_accepted: Vec<FitnessVector>,
    sample_window: usize,
}

impl ValueDriftDetector {
    pub const DRIFT_DEGREES_THRESHOLD: f64 = 30.0;

    /// إنشاء كاشف جديد مع خط أساس.
    pub fn new(baseline: FitnessVector, sample_window: usize) -> Self {
        Self {
            baseline,
            current_accepted: Vec::with_capacity(sample_window),
            sample_window,
        }
    }

    /// تسجيل `FitnessVector` لقرار مقبول.
    pub fn record(&mut self, fitness: &FitnessVector) {
        if self.current_accepted.len() >= self.sample_window {
            self.current_accepted.remove(0);
        }
        self.current_accepted.push(fitness.clone());
    }

    /// حساب متوسط `FitnessVector` الحالي.
    fn average_current(&self) -> Option<FitnessVector> {
        if self.current_accepted.is_empty() {
            return None;
        }
        let n = self.current_accepted.len() as f64;
        let mut sum = FitnessVector {
            security: 0.0,
            reversibility: 0.0,
            test_coverage: 0.0,
            maintainability: 0.0,
            performance: 0.0,
            architectural_stability: 0.0,
        };

        for fv in &self.current_accepted {
            sum.security += fv.security;
            sum.reversibility += fv.reversibility;
            sum.test_coverage += fv.test_coverage;
            sum.maintainability += fv.maintainability;
            sum.performance += fv.performance;
            sum.architectural_stability += fv.architectural_stability;
        }

        sum.security /= n;
        sum.reversibility /= n;
        sum.test_coverage /= n;
        sum.maintainability /= n;
        sum.performance /= n;
        sum.architectural_stability /= n;

        Some(sum)
    }

    /// حساب زاوية الانجراف بالدرجات.
    pub fn drift_degrees(&self) -> Option<f64> {
        let avg_current = self.average_current()?;
        let mag_baseline = magnitude(&self.baseline);
        let mag_current = magnitude(&avg_current);

        if mag_baseline == 0.0 || mag_current == 0.0 {
            return Some(0.0);
        }

        let dot = dot_product(&self.baseline, &avg_current);
        let cos_theta = (dot / (mag_baseline * mag_current)).clamp(-1.0, 1.0);
        Some(cos_theta.acos().to_degrees())
    }

    /// التحقق مما إذا كانت المراجعة مطلوبة.
    pub fn needs_review(&self) -> Option<ReviewReason> {
        if self.current_accepted.len() < self.sample_window {
            return None; // لا توجد عينات كافية بعد
        }

        if let Some(degrees) = self.drift_degrees() {
            if degrees > Self::DRIFT_DEGREES_THRESHOLD {
                return Some(ReviewReason::ValueDriftDetected {
                    degrees,
                    threshold: Self::DRIFT_DEGREES_THRESHOLD,
                });
            }
        }
        None
    }
}

// ─── 3. ConstitutionalAmendment ─────────────────────────────────────

/// حالة تعديل دستوري.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendmentStatus {
    Proposed,
    Ratified,
    Rejected,
    Superseded,
}

/// التغيير المقترح.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmendmentChange {
    /// تحديث سياسة معينة
    UpdatePolicy {
        policy_name: String,
        old_value: String,
        new_value: String,
    },
    /// إضافة ثابت جديد (invariant)
    AddInvariant { name: String, code_hash: String },
    /// إزالة ثابت
    RemoveInvariant { name: String },
}

/// سجل أدلة يدعم التعديل.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub reason: ReviewReason,
    pub supporting_data: String,
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

/// تعديل دستوري رسمي.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstitutionalAmendment {
    pub id: uuid::Uuid,
    pub change: AmendmentChange,
    pub evidence: Vec<EvidenceRecord>,
    pub status: AmendmentStatus,
    /// التقدير الأولي للأثر (0.0 - 1.0)
    pub estimated_impact: f64,
}

impl ConstitutionalAmendment {
    /// هل يتطلب هذا التعديل تصديقاً بشرياً؟
    pub fn requires_ratification(&self) -> bool {
        self.estimated_impact > 0.3
    }
}
