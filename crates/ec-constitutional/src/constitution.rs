use crate::evaluation::ConstitutionalEvaluation;
use crate::invariant::Invariant;
use crate::verdict::ConstitutionalVerdict;
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::CatastropheThresholds;
use ec_fitness::fitness::CatastrophicDimension;
use ec_fitness::fitness::FitnessVector;
use std::sync::Arc;

#[derive(Clone)]
pub struct Constitution {
    invariants: Vec<Arc<dyn Invariant>>,
    thresholds: CatastropheThresholds,
}

impl Constitution {
    pub fn new(invariants: Vec<Arc<dyn Invariant>>, thresholds: CatastropheThresholds) -> Self {
        Self {
            invariants,
            thresholds,
        }
    }

    pub fn evaluate(
        &self,
        artifact_id: &str,
        fitness: &FitnessVector,
        epistemic: &EpistemicState,
    ) -> ConstitutionalEvaluation {
        let verdict = ConstitutionalVerdict::evaluate(&self.invariants, fitness, epistemic);

        let catastrophic = if fitness.security < self.thresholds.min_security {
            Some(CatastrophicDimension::Security)
        } else if fitness.reversibility < self.thresholds.min_reversibility {
            Some(CatastrophicDimension::Reversibility)
        } else if fitness.test_coverage < self.thresholds.min_test_coverage {
            Some(CatastrophicDimension::TestCoverage)
        } else if fitness.maintainability < self.thresholds.min_maintainability {
            Some(CatastrophicDimension::Maintainability)
        } else if fitness.performance < self.thresholds.min_performance {
            Some(CatastrophicDimension::Performance)
        } else if fitness.architectural_stability < self.thresholds.min_architectural_stability {
            Some(CatastrophicDimension::ArchitecturalStability)
        } else {
            None
        };

        let violations = match &verdict {
            ConstitutionalVerdict::Accepted => vec![],
            ConstitutionalVerdict::Rejected { violations, .. } => violations.clone(),
        };

        let is_valid = catastrophic.is_none() && matches!(verdict, ConstitutionalVerdict::Accepted);

        ConstitutionalEvaluation {
            artifact_id: artifact_id.to_string(),
            fitness: fitness.clone(),
            epistemic: epistemic.clone(),
            violations,
            catastrophic,
            is_valid,
            explanation: "Evaluation complete".to_string(),
        }
    }
}

/// ما لاحظه الدستور من التنفيذ (للتعلم فقط).
///
/// **Design Note:**
/// هذا **ليس** RealityVector الكامل من ec-sandbox.
/// هذا نموذج مبسط للتعلم داخل ec-constitutional.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservedOutcome {
    /// هل نجحت الاختبارات الوظيفية؟ (0.0 or 1.0)
    pub correctness: f64,
    /// هل النتائج قابلة للتكرار؟ (0.0 to 1.0)
    pub reproducibility: f64,
}

/// الفرق بين ما توقعه الدستور وما حدث فعلاً.
#[derive(Debug, Clone, PartialEq)]
pub struct PredictionError {
    /// هل توقعنا `is_valid` بشكل صحيح؟
    pub validity_error: f64, // 0.0 if correct, 1.0 if wrong
    /// الفجوة بين الثقة المتوقعة والواقع.
    pub confidence_gap: f64,
}

impl Constitution {
    /// تحديث الدستور (أو أدوات المراقبة المرتبطة به) بناءً على الواقع.
    ///
    /// # Design Rule
    /// هذه الـ API تقبل `ObservedOutcome` (نسخة مبسطة)
    /// لأن ec-constitutional لا يعرف تفاصيل ec-sandbox.
    pub fn learn(
        &self,
        prediction: &ConstitutionalEvaluation,
        reality: &ObservedOutcome,
    ) -> PredictionError {
        let predicted_valid = prediction.is_valid;
        let actual_valid = reality.correctness >= 1.0;

        let validity_error = if predicted_valid == actual_valid { 0.0 } else { 1.0 };

        let confidence_gap = (prediction.epistemic.confidence - reality.reproducibility).abs();

        PredictionError {
            validity_error,
            confidence_gap,
        }
    }
}
