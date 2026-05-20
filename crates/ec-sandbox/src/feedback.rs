#![forbid(unsafe_code)]

//! Reality Feedback Loop — التعلم من مقارنة التوقع بالواقع.
//!
//! Week 15: الدستور يتوقع → Sandbox ينفذ → نحسب الفرق → نتعلم

use crate::reality::RealityVector;
use ec_constitutional::constitution::ObservedOutcome;
use ec_constitutional::evaluation::ConstitutionalEvaluation;

/// ما توقعه الدستور قبل التنفيذ.
#[derive(Debug, Clone)]
pub struct PredictionRecord {
    /// معرف الـ artifact.
    pub artifact_id: String,
    /// هل توقعنا أن يكون valid؟
    pub predicted_validity: bool,
    /// مستوى الثقة المتوقع.
    pub predicted_confidence: f64,
}

impl PredictionRecord {
    /// بناء من ConstitutionalEvaluation.
    pub fn from_evaluation(eval: &ConstitutionalEvaluation) -> Self {
        Self {
            artifact_id: eval.artifact_id.clone(),
            predicted_validity: eval.is_valid,
            predicted_confidence: eval.epistemic.confidence,
        }
    }
}

/// الخطأ بين التوقع والواقع.
#[derive(Debug, Clone)]
pub struct PredictionError {
    /// معرف الـ artifact.
    pub artifact_id: String,
    /// 0.0 = توقع صحيح، 1.0 = توقع خاطئ تماماً
    pub validity_error: f64,
    /// |predicted_confidence - actual_correctness|
    pub confidence_gap: f64,
    /// توقعنا نجاح (confidence > 0.8) لكن الواقع فشل (correctness < 0.5)
    pub overconfident: bool,
    /// توقعنا فشل (confidence < 0.4) لكن الواقع نجح (correctness > 0.8)
    pub underconfident: bool,
}

impl PredictionError {
    /// حساب الخطأ من التوقع والواقع.
    pub fn compute(prediction: &PredictionRecord, reality: &RealityVector) -> Self {
        let actual_valid = reality.is_correct() && reality.is_reproducible();

        let validity_error = if prediction.predicted_validity == actual_valid {
            0.0
        } else {
            1.0
        };

        // نستخدم correctness كمقياس للواقع الفعلي
        let confidence_gap = (prediction.predicted_confidence - reality.correctness).abs();

        Self {
            artifact_id: prediction.artifact_id.clone(),
            validity_error,
            confidence_gap,
            overconfident: prediction.predicted_confidence > 0.8 && reality.correctness < 0.5,
            underconfident: prediction.predicted_confidence < 0.4 && reality.correctness > 0.8,
        }
    }

    /// هل الخطأ مقبول؟
    pub fn is_acceptable(&self) -> bool {
        self.validity_error < 0.5 && self.confidence_gap < 0.3
    }
}

/// يجمع الأخطاء ويكتشف الأنماط.
pub struct RealityFeedback {
    history: Vec<PredictionError>,
    window: usize, // 50 قرار
}

impl RealityFeedback {
    /// إنشاء feedback tracker جديد.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            window: 50,
        }
    }

    /// تعلم من prediction جديد.
    pub fn learn(
        &mut self,
        prediction: &PredictionRecord,
        reality: &RealityVector,
    ) -> PredictionError {
        let error = PredictionError::compute(prediction, reality);
        self.history.push(error.clone());

        // نحتفظ بآخر `window` قرار فقط
        if self.history.len() > self.window {
            self.history.remove(0);
        }

        error
    }

    /// متوسط validity_error في الـ window.
    pub fn mean_validity_error(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        self.history.iter().map(|e| e.validity_error).sum::<f64>() / self.history.len() as f64
    }

    /// هل الخطأ ينخفض؟
    pub fn is_improving(&self) -> bool {
        if self.history.len() < 20 {
            return true; // لا نستطيع الحكم بعد
        }

        let mean_error = self.mean_validity_error();

        // إذا كان الخطأ صفر (كل التوقعات صحيحة) → تحسن مثالي
        if mean_error < 0.001 {
            return true;
        }

        let n = self.history.len();
        let first: f64 = self.history[..10]
            .iter()
            .map(|e| e.validity_error)
            .sum::<f64>()
            / 10.0;
        let last: f64 = self.history[n - 10..]
            .iter()
            .map(|e| e.validity_error)
            .sum::<f64>()
            / 10.0;

        // يتحسن فقط إذا انخفض بشكل ملموس
        last < first - 0.05
    }

    /// يقترح constitutional review بعد 50 قرار سيء.
    pub fn needs_constitutional_review(&self) -> bool {
        if self.history.len() < self.window {
            return false;
        }

        let mean_error = self.mean_validity_error();
        let improving = self.is_improving();

        // نحتاج review إذا:
        // 1. mean_error عالي (> 0.3)
        // 2. وإما: لا يتحسن OR mean_error كارثي (> 0.8)
        mean_error > 0.3 && (!improving || mean_error > 0.8)
    }

    /// تحويل RealityVector إلى ObservedOutcome (للـ Constitution::learn).
    pub fn to_observed_outcome(reality: &RealityVector) -> ObservedOutcome {
        ObservedOutcome {
            correctness: reality.correctness,
            reproducibility: reality.reproducibility,
        }
    }
}

impl Default for RealityFeedback {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reality::RealityVector;

    fn make_reality(correctness: f64, reproducibility: f64) -> RealityVector {
        RealityVector::new(correctness, reproducibility, 0.95, 3, None).unwrap()
    }

    fn make_prediction(valid: bool, confidence: f64) -> PredictionRecord {
        PredictionRecord {
            artifact_id: "test".to_string(),
            predicted_validity: valid,
            predicted_confidence: confidence,
        }
    }

    #[test]
    fn detects_false_positive() {
        let pred = make_prediction(true, 0.9);
        let reality = make_reality(0.0, 0.0);

        let error = PredictionError::compute(&pred, &reality);

        assert_eq!(error.validity_error, 1.0);
        assert!(error.overconfident);
    }

    #[test]
    fn detects_false_negative() {
        let pred = make_prediction(false, 0.3);
        let reality = make_reality(1.0, 0.98);

        let error = PredictionError::compute(&pred, &reality);

        assert_eq!(error.validity_error, 1.0);
        assert!(error.underconfident);
    }

    #[test]
    fn correct_prediction_zero_error() {
        let pred = make_prediction(true, 0.85);
        let reality = make_reality(1.0, 0.98);
        let error = PredictionError::compute(&pred, &reality);

        assert_eq!(error.validity_error, 0.0);
    }

    #[test]
    fn improving_after_correct_predictions() {
        let mut fb = RealityFeedback::new();

        for _ in 0..20 {
            let pred = make_prediction(true, 0.9);
            let reality = make_reality(1.0, 0.98);
            fb.learn(&pred, &reality);
        }

        assert!(fb.is_improving());
        assert!(!fb.needs_constitutional_review());
    }

    #[test]
    fn triggers_review_after_sustained_errors() {
        let mut fb = RealityFeedback::new();

        for _ in 0..50 {
            let pred = make_prediction(true, 0.9);
            let reality = make_reality(0.0, 0.0);
            fb.learn(&pred, &reality);
        }

        // mean_error = 1.0 > 0.8 → catastrophic
        assert_eq!(fb.mean_validity_error(), 1.0);
        assert!(fb.needs_constitutional_review());
    }

    #[test]
    fn mean_error_zero_for_perfect_predictions() {
        let mut fb = RealityFeedback::new();

        for _ in 0..10 {
            let pred = make_prediction(true, 0.9);
            let reality = make_reality(1.0, 0.98);
            fb.learn(&pred, &reality);
        }

        assert_eq!(fb.mean_validity_error(), 0.0);
    }

    #[test]
    fn acceptable_error_threshold() {
        let pred = make_prediction(true, 0.85);
        let reality = make_reality(1.0, 0.98);
        let error = PredictionError::compute(&pred, &reality);

        assert!(error.is_acceptable());
    }

    #[test]
    fn to_observed_outcome_conversion() {
        let reality = make_reality(1.0, 0.98);
        let outcome = RealityFeedback::to_observed_outcome(&reality);

        assert_eq!(outcome.correctness, 1.0);
        assert_eq!(outcome.reproducibility, 0.98);
    }
}
