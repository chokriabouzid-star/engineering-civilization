#![forbid(unsafe_code)]

//! Bayesian Calibration — يكشف overconfident/underconfident ويُعدّل
//! Week 40 — إضافة فقط

use crate::bayesian::BayesianEvidence;
use crate::calibration::CalibrationState;

/// نتيجة تشخيص المعايرة
#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationDiagnosis {
    /// معايرة جيدة (ECE < 0.10)
    WellCalibrated {
        /// قيمة ECE
        ece: f64,
    },
    /// ثقة مفرطة — التوقعات أعلى من الواقع
    Overconfident {
        /// قيمة ECE
        ece: f64,
        /// متوسط الفجوة (predicted - actual)
        avg_gap: f64,
    },
    /// ثقة ناقصة — التوقعات أقل من الواقع
    Underconfident {
        /// قيمة ECE
        ece: f64,
        /// متوسط الفجوة (actual - predicted)
        avg_gap: f64,
    },
    /// بيانات غير كافية (< 10 عينات)
    InsufficientData {
        /// عدد العينات
        samples: u64,
    },
}

/// مُعدِّل المعايرة — يُعدّل الثقة بناءً على التشخيص
#[derive(Debug, Clone)]
pub struct BayesianCalibration;

impl BayesianCalibration {
    /// تشخيص حالة المعايرة
    pub fn diagnose(calibration: &CalibrationState) -> CalibrationDiagnosis {
        if calibration.total < 10 {
            return CalibrationDiagnosis::InsufficientData {
                samples: calibration.total,
            };
        }

        let ece = calibration.ece();

        // نحسب متوسط الفجوة عبر bins مع بيانات
        let mut total_gap = 0.0;
        let mut bins_with_data = 0u64;
        for b in &calibration.bins {
            if b.count > 0 {
                total_gap += b.avg_predicted() - b.avg_actual();
                bins_with_data += 1;
            }
        }

        let avg_gap = if bins_with_data > 0 {
            total_gap / bins_with_data as f64
        } else {
            0.0
        };

        if ece < 0.10 {
            CalibrationDiagnosis::WellCalibrated { ece }
        } else if avg_gap > 0.05 {
            CalibrationDiagnosis::Overconfident { ece, avg_gap }
        } else if avg_gap < -0.05 {
            CalibrationDiagnosis::Underconfident {
                ece,
                avg_gap: -avg_gap,
            }
        } else {
            CalibrationDiagnosis::WellCalibrated { ece }
        }
    }

    /// تعديل الثقة بناءً على التشخيص
    pub fn adjust_confidence(confidence: f64, diagnosis: &CalibrationDiagnosis) -> f64 {
        match diagnosis {
            CalibrationDiagnosis::WellCalibrated { .. } => confidence,
            CalibrationDiagnosis::Overconfident { avg_gap, .. } => {
                (confidence - avg_gap).clamp(0.10, 1.0)
            }
            CalibrationDiagnosis::Underconfident { avg_gap, .. } => {
                (confidence + avg_gap * 0.5).clamp(0.10, 1.0)
            }
            CalibrationDiagnosis::InsufficientData { .. } => (confidence - 0.10).clamp(0.10, 1.0),
        }
    }

    /// تعديل BayesianEvidence.confidence بناءً على المعايرة
    pub fn adjusted_credible_confidence(
        evidence: &BayesianEvidence,
        calibration: &CalibrationState,
    ) -> f64 {
        let raw = evidence.credible_confidence();
        let diagnosis = Self::diagnose(calibration);
        Self::adjust_confidence(raw, &diagnosis)
    }
}
