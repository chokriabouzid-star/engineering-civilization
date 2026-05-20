#![forbid(unsafe_code)]
use crate::error::{ensure_in_range, EpistemicResult};
use serde::{Deserialize, Serialize};

/// خلية معايرة واحدة (نطاق ثقة محدد).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalibrationBin {
    /// الحد الأدنى (شامل).
    pub lower_inclusive: f64,
    /// الحد الأعلى (غير شامل).
    pub upper_exclusive: f64,
    /// عدد العينات المسجلة.
    pub count: u64,
    /// مجموع قيم الثقة المتوقعة.
    pub sum_predicted: f64,
    /// مجموع النتائج الواقعية.
    pub sum_actual: f64,
}

impl CalibrationBin {
    /// إنشاء خلية جديدة.
    pub fn new(lower_inclusive: f64, upper_exclusive: f64) -> Self {
        Self {
            lower_inclusive,
            upper_exclusive,
            count: 0,
            sum_predicted: 0.0,
            sum_actual: 0.0,
        }
    }

    /// تسجيل عينة جديدة.
    pub fn record(&mut self, predicted: f64, actual: f64) {
        self.count += 1;
        self.sum_predicted += predicted;
        self.sum_actual += actual;
    }

    /// متوسط الثقة المتوقعة.
    pub fn avg_predicted(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_predicted / self.count as f64
        }
    }

    /// متوسط النتائج الواقعية.
    pub fn avg_actual(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_actual / self.count as f64
        }
    }
}

/// حالة المعايرة الكلية (10 خلايا).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalibrationState {
    /// الخلايا العشر.
    pub bins: Vec<CalibrationBin>,
    /// إجمالي العينات.
    pub total: u64,
}

impl Default for CalibrationState {
    fn default() -> Self {
        let mut bins = Vec::with_capacity(10);
        for i in 0..10 {
            let lo = i as f64 / 10.0;
            let hi = (i + 1) as f64 / 10.0;
            bins.push(CalibrationBin::new(lo, hi));
        }
        Self { bins, total: 0 }
    }
}

impl CalibrationState {
    /// تسجيل عينة معايرة.
    pub fn record(&mut self, predicted: f64, actual: f64) -> EpistemicResult<()> {
        ensure_in_range("predicted", predicted, 0.0, 1.0)?;
        ensure_in_range("actual", actual, 0.0, 1.0)?;
        let idx = if predicted >= 1.0 {
            9
        } else {
            ((predicted * 10.0).floor() as usize).min(9)
        };
        self.bins[idx].record(predicted, actual);
        self.total += 1;
        Ok(())
    }

    /// حساب خطأ المعايرة المتوقع (ECE).
    pub fn ece(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        let total = self.total as f64;
        let mut ece = 0.0;
        for b in &self.bins {
            if b.count == 0 {
                continue;
            }
            let w = b.count as f64 / total;
            let gap = (b.avg_predicted() - b.avg_actual()).abs();
            ece += w * gap;
        }
        ece.clamp(0.0, 1.0)
    }

    /// هل النظام معاير (ECE < 0.1)؟
    pub fn is_calibrated(&self) -> bool {
        self.ece() < 0.1
    }

    /// دمج حالات معايرة متعددة.
    pub fn merge_all(states: &[CalibrationState]) -> CalibrationState {
        let mut merged = CalibrationState::default();
        for s in states {
            merged.total += s.total;
            for (i, b) in s.bins.iter().enumerate().take(10) {
                merged.bins[i].count += b.count;
                merged.bins[i].sum_predicted += b.sum_predicted;
                merged.bins[i].sum_actual += b.sum_actual;
            }
        }
        merged
    }
}
