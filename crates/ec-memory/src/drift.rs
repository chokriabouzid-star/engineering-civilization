#![forbid(unsafe_code)]

//! Value drift detection from historical memory.
//!
//! Week 24 — Phase 3

use crate::CausalMemoryGraph;
use ec_fitness::fitness::FitnessVector;

// ────────────────────────────────────────────────
// الأنواع الأساسية
// ────────────────────────────────────────────────

/// تصنيف الانجراف القيمي.
#[derive(Debug, Clone, PartialEq)]
pub enum DriftClassification {
    /// أقل من 10° — النظام مستقر.
    Stable,
    /// Pareto front تحسّن + زاوية صغيرة — النظام يتعلم.
    LearningProgress {
        /// الزاوية بالدرجات.
        angle: f64,
    },
    /// زاوية كبيرة بدون تحسن — تحول في القيم.
    ValueShift {
        /// الزاوية بالدرجات.
        angle: f64,
    },
    /// زادت الانتهاكات — الدستور يتدهور.
    Corruption {
        /// الزيادة في معدل الرفض.
        rejection_increase: f64,
    },
    /// بيانات غير كافية.
    InsufficientData {
        /// عدد القرارات المتوفرة.
        available: usize,
        /// عدد القرارات المطلوبة.
        required: usize,
    },
}

/// الإجراء الموصى به.
#[derive(Debug, Clone, PartialEq)]
pub enum DriftAction {
    /// لا إجراء مطلوب.
    None,
    /// مراقبة مستمرة.
    Monitor,
    /// مراجعة دستورية.
    ReviewConstitution {
        /// السبب.
        reason: String,
    },
    /// تدخل بشري.
    HumanIntervention {
        /// السبب.
        reason: String,
    },
}

/// تقرير الانجراف.
#[derive(Debug, Clone)]
pub struct DriftReport {
    /// الزاوية بالدرجات.
    pub drift_angle_degrees: f64,
    /// حجم الـ baseline.
    pub baseline_count: usize,
    /// حجم الـ current window.
    pub current_count: usize,
    /// العدد الكلي للقرارات.
    pub total_decisions: usize,
    /// التصنيف.
    pub classification: DriftClassification,
    /// الإجراء الموصى به.
    pub recommended_action: DriftAction,
}

impl DriftReport {
    /// هل يتطلب إجراءً؟
    pub fn requires_action(&self) -> bool {
        !matches!(
            self.recommended_action,
            DriftAction::None | DriftAction::Monitor
        )
    }
}

// ────────────────────────────────────────────────
// المحلل
// ────────────────────────────────────────────────

/// محلل الانجراف التاريخي.
pub struct HistoricalDriftAnalyzer<'a> {
    memory: &'a CausalMemoryGraph,
    baseline_size: usize,
    current_window: usize,
}

impl<'a> HistoricalDriftAnalyzer<'a> {
    /// إنشاء محلل جديد.
    pub fn new(memory: &'a CausalMemoryGraph, baseline_size: usize, current_window: usize) -> Self {
        Self {
            memory,
            baseline_size,
            current_window,
        }
    }

    /// تحليل الانجراف.
    pub fn analyze(&self) -> DriftReport {
        let all = self.memory.all();
        let total = all.len();
        let required = self.baseline_size + self.current_window;

        if total < required {
            return DriftReport {
                drift_angle_degrees: 0.0,
                baseline_count: 0,
                current_count: total.min(self.baseline_size),
                total_decisions: total,
                classification: DriftClassification::InsufficientData {
                    available: total,
                    required,
                },
                recommended_action: DriftAction::None,
            };
        }

        let baseline_nodes = &all[..self.baseline_size];
        let current_nodes = &all[total - self.current_window..];

        let baseline_vecs: Vec<&FitnessVector> =
            baseline_nodes.iter().map(|n| &n.fitness).collect();
        let current_vecs: Vec<&FitnessVector> = current_nodes.iter().map(|n| &n.fitness).collect();

        let baseline_avg = average_fitness(&baseline_vecs);
        let current_avg = average_fitness(&current_vecs);
        let angle = baseline_avg.cosine_angle_degrees(&current_avg);

        // معدل الرفض
        let baseline_rejections = baseline_nodes
            .iter()
            .filter(|n| !n.constitutional_valid)
            .count();
        let current_rejections = current_nodes
            .iter()
            .filter(|n| !n.constitutional_valid)
            .count();

        let baseline_rate = baseline_rejections as f64 / self.baseline_size as f64;
        let current_rate = current_rejections as f64 / self.current_window as f64;
        let rejection_increase = current_rate - baseline_rate;

        let pareto_improved = self.pareto_front_improved(&baseline_vecs, &current_vecs);

        let classification = classify(angle, rejection_increase, pareto_improved);
        let recommended_action = recommend(&classification);

        DriftReport {
            drift_angle_degrees: angle,
            baseline_count: self.baseline_size,
            current_count: self.current_window,
            total_decisions: total,
            classification,
            recommended_action,
        }
    }

    fn pareto_front_improved(
        &self,
        baseline: &[&FitnessVector],
        current: &[&FitnessVector],
    ) -> bool {
        let b = average_fitness(baseline);
        let c = average_fitness(current);
        c.security > b.security || c.test_coverage > b.test_coverage
    }
}

// ────────────────────────────────────────────────
// الدوال المساعدة
// ────────────────────────────────────────────────

fn average_fitness(vectors: &[&FitnessVector]) -> FitnessVector {
    if vectors.is_empty() {
        return FitnessVector::default();
    }
    let n = vectors.len() as f64;
    FitnessVector {
        security: vectors.iter().map(|v| v.security).sum::<f64>() / n,
        reversibility: vectors.iter().map(|v| v.reversibility).sum::<f64>() / n,
        test_coverage: vectors.iter().map(|v| v.test_coverage).sum::<f64>() / n,
        maintainability: vectors.iter().map(|v| v.maintainability).sum::<f64>() / n,
        performance: vectors.iter().map(|v| v.performance).sum::<f64>() / n,
        architectural_stability: vectors
            .iter()
            .map(|v| v.architectural_stability)
            .sum::<f64>()
            / n,
    }
}

fn classify(angle: f64, rejection_increase: f64, pareto_improved: bool) -> DriftClassification {
    if rejection_increase > 0.2 {
        return DriftClassification::Corruption { rejection_increase };
    }
    if angle < 10.0 {
        return DriftClassification::Stable;
    }
    if pareto_improved {
        DriftClassification::LearningProgress { angle }
    } else {
        DriftClassification::ValueShift { angle }
    }
}

fn recommend(classification: &DriftClassification) -> DriftAction {
    match classification {
        DriftClassification::Stable => DriftAction::None,
        DriftClassification::LearningProgress { .. } => DriftAction::Monitor,
        DriftClassification::ValueShift { angle } if *angle > 45.0 => {
            DriftAction::HumanIntervention {
                reason: format!("Value rotation {:.1}° exceeds 45° threshold", angle),
            }
        }
        DriftClassification::ValueShift { angle } => DriftAction::ReviewConstitution {
            reason: format!("Value rotation {:.1}° exceeds 10° threshold", angle),
        },
        DriftClassification::Corruption { rejection_increase } => DriftAction::HumanIntervention {
            reason: format!(
                "Rejection rate increased by {:.1}% — possible corruption",
                rejection_increase * 100.0
            ),
        },
        DriftClassification::InsufficientData { .. } => DriftAction::None,
    }
}
