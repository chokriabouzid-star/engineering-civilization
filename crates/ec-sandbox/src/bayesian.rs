#![forbid(unsafe_code)]

//! BayesianTracker — يتتبع outcomes بـ BayesianEvidence
//! إضافة فقط — لا يُعدّل RealityFeedback

use ec_epistemic::BayesianEvidence;

/// يتتبع نتائج التنفيذ الفعلي باستخدام Bayesian updating.
pub struct BayesianTracker {
    evidence: BayesianEvidence,
}

impl Default for BayesianTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl BayesianTracker {
    /// إنشاء tracker جديد بـ prior غير متحيز
    pub fn new() -> Self {
        Self {
            evidence: BayesianEvidence::initial_prior().expect("initial_prior should never fail"),
        }
    }

    /// تسجيل نتيجة تنفيذ
    pub fn record(&mut self, was_correct: bool, score: f64) {
        if let Ok(updated) = self.evidence.update_with_outcome(was_correct, score) {
            self.evidence = updated;
        }
    }

    /// الوصول للأدلة الحالية
    pub fn evidence(&self) -> &BayesianEvidence {
        &self.evidence
    }

    /// مستوى الثقة المُحسَّب (Wilson interval)
    pub fn credible_confidence(&self) -> f64 {
        self.evidence.credible_confidence()
    }

    /// إجمالي المشاهدات
    pub fn total_observations(&self) -> u32 {
        self.evidence.total_observations()
    }

    /// هل نملك بيانات كافية للحكم؟ (≥5 مشاهدات)
    pub fn has_sufficient_data(&self) -> bool {
        self.evidence.total_observations() >= 5
    }
}
