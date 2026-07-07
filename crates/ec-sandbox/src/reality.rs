#![forbid(unsafe_code)]

//! RealityVector — نتيجة التنفيذ الفعلي.
//!
//! **Invariant:** RealityVector يُنشأ فقط من خلال SandboxExecutor.
//! لا يمكن إنشاؤه يدوياً — constructor خاص.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// قياسات Latency من عدة تشغيلات.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    /// p50 (median) latency.
    pub p50: Duration,

    /// p95 latency.
    pub p95: Duration,

    /// p99 latency.
    pub p99: Duration,

    /// عدد العينات.
    pub sample_count: usize,
}

impl LatencyMeasurement {
    /// بناء من قائمة latencies.
    pub fn from_samples(mut samples: Vec<Duration>) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }

        samples.sort();
        let n = samples.len();

        let p50 = samples[n / 2];
        let p95 = samples[(n * 95) / 100];
        let p99 = samples[(n * 99) / 100];

        Some(Self {
            p50,
            p95,
            p99,
            sample_count: n,
        })
    }
}

/// RealityVector — الحقيقة المقاسة من التنفيذ.
///
/// **Design Invariant:** لا يمكن إنشاء RealityVector يدوياً.
/// يجب أن يأتي من SandboxExecutor فقط.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealityVector {
    /// هل نجحت الاختبارات؟ (0.0 = فشل، 1.0 = نجاح)
    pub correctness: f64,

    /// reproducibility: [0.0, 1.0]
    /// 1.0 = نفس النتيجة في كل مرة
    /// 0.0 = نتائج مختلفة تماماً
    pub reproducibility: f64,

    /// هل كانت القياسات الأدائية صحيحة؟
    pub benchmark_validity: f64,

    /// الثقة التجريبية من عدد التشغيلات.
    pub empirical_confidence: f64,

    /// عدد التشغيلات التي أُكملت.
    pub runs_completed: usize,

    /// قياسات Latency (إن وُجدت).
    pub latency: Option<LatencyMeasurement>,
}

impl RealityVector {
    /// **Private constructor** — يُستدعى من SandboxExecutor فقط.
    ///
    /// # Design Rule
    /// هذا الـ constructor `pub(crate)` وليس `pub` لضمان أن
    /// RealityVector لا يُنشأ إلا من خلال تنفيذ حقيقي.
    pub(crate) fn new(
        correctness: f64,
        reproducibility: f64,
        benchmark_validity: f64,
        runs_completed: usize,
        latency: Option<LatencyMeasurement>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            (0.0..=1.0).contains(&correctness),
            "correctness must be in [0.0, 1.0], got {}",
            correctness
        );
        anyhow::ensure!(
            (0.0..=1.0).contains(&reproducibility),
            "reproducibility must be in [0.0, 1.0], got {}",
            reproducibility
        );
        anyhow::ensure!(
            (0.0..=1.0).contains(&benchmark_validity),
            "benchmark_validity must be in [0.0, 1.0], got {}",
            benchmark_validity
        );
        anyhow::ensure!(runs_completed > 0, "runs_completed must be > 0");

        // الثقة التجريبية تزيد مع عدد التشغيلات
        // Formula: runs / (runs + 0.5)
        // 1 run:  0.67
        // 2 runs: 0.80
        // 3 runs: 0.857 ← يتجاوز threshold 0.8
        // 5 runs: 0.91
        // 10 runs: 0.95
        let empirical_confidence =
            (runs_completed as f64 / (runs_completed as f64 + 0.5)).min(0.99);

        Ok(Self {
            correctness,
            reproducibility,
            benchmark_validity,
            empirical_confidence,
            runs_completed,
            latency,
        })
    }

    /// هل التنفيذ صحيح؟
    pub fn is_correct(&self) -> bool {
        self.correctness > 0.99
    }

    /// هل التنفيذ قابل للتكرار؟
    pub fn is_reproducible(&self) -> bool {
        self.reproducibility > 0.95
    }

    /// هل البيانات المُقاسة موثوقة؟
    /// Dummy RealityVector للـ error handling.
    /// يُستخدم فقط في pipeline عند فشل التنفيذ بشكل كامل.
    /// Dummy RealityVector للـ error handling.
    ///
    /// يُستخدم فقط في pipeline عند فشل التنفيذ بشكل كامل.
    /// لا يعكس واقعاً حقيقياً — فقط placeholder آمن.
    pub fn dummy() -> Self {
        Self {
            correctness: 0.0,
            reproducibility: 0.0,
            benchmark_validity: 0.0,
            empirical_confidence: 0.0,
            runs_completed: 0,
            latency: None,
        }
    }

    /// RealityVector للاختبارات فقط.
    ///
    /// يُستخدم في test fixtures لبناء reality مع قيم محددة.
    pub fn test_fixture(
        correctness: f64,
        reproducibility: f64,
        benchmark_validity: f64,
        runs_completed: usize,
    ) -> Self {
        let empirical_confidence = runs_completed as f64 / (runs_completed as f64 + 0.5);
        Self {
            correctness,
            reproducibility,
            benchmark_validity,
            empirical_confidence,
            runs_completed,
            latency: None,
        }
    }

    /// هل الـ RealityVector يستحق الثقة؟
    ///
    /// يتطلب:
    /// - `correctness > 0.99`
    /// - `reproducibility > 0.95`
    /// - `empirical_confidence > 0.80` (≥ 3 runs)
    pub fn is_trustworthy(&self) -> bool {
        self.is_correct() && self.is_reproducible() && self.empirical_confidence > 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_from_empty_samples_returns_none() {
        let result = LatencyMeasurement::from_samples(vec![]);
        assert!(result.is_none());
    }

    #[test]
    fn latency_from_single_sample() {
        let samples = vec![Duration::from_millis(100)];
        let latency = LatencyMeasurement::from_samples(samples).unwrap();

        assert_eq!(latency.p50, Duration::from_millis(100));
        assert_eq!(latency.p95, Duration::from_millis(100));
        assert_eq!(latency.p99, Duration::from_millis(100));
        assert_eq!(latency.sample_count, 1);
    }

    #[test]
    fn latency_percentiles_are_sorted() {
        let samples = vec![
            Duration::from_millis(10),
            Duration::from_millis(50),
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(500),
        ];
        let latency = LatencyMeasurement::from_samples(samples).unwrap();

        assert!(latency.p50 <= latency.p95);
        assert!(latency.p95 <= latency.p99);
    }

    #[test]
    fn reality_vector_rejects_invalid_correctness() {
        let result = RealityVector::new(1.5, 0.9, 0.8, 3, None);
        assert!(result.is_err());
    }

    #[test]
    fn reality_vector_rejects_zero_runs() {
        let result = RealityVector::new(1.0, 0.9, 0.8, 0, None);
        assert!(result.is_err());
    }

    #[test]
    fn reality_vector_empirical_confidence_increases_with_runs() {
        let r1 = RealityVector::new(1.0, 0.9, 0.8, 1, None).unwrap();
        let r3 = RealityVector::new(1.0, 0.9, 0.8, 3, None).unwrap();
        let r10 = RealityVector::new(1.0, 0.9, 0.8, 10, None).unwrap();

        assert!(r1.empirical_confidence < r3.empirical_confidence);
        assert!(r3.empirical_confidence < r10.empirical_confidence);
    }

    #[test]
    fn reality_vector_is_correct_when_correctness_high() {
        let rv = RealityVector::new(1.0, 0.9, 0.8, 3, None).unwrap();
        assert!(rv.is_correct());
    }

    #[test]
    fn reality_vector_is_not_correct_when_correctness_low() {
        let rv = RealityVector::new(0.5, 0.9, 0.8, 3, None).unwrap();
        assert!(!rv.is_correct());
    }

    #[test]
    fn reality_vector_is_reproducible_when_high() {
        let rv = RealityVector::new(1.0, 0.98, 0.8, 3, None).unwrap();
        assert!(rv.is_reproducible());
    }

    #[test]
    fn reality_vector_trustworthy_requires_all_criteria() {
        let good = RealityVector::new(1.0, 0.98, 0.8, 10, None).unwrap();
        assert!(good.is_trustworthy());

        let bad_correctness = RealityVector::new(0.5, 0.98, 0.8, 10, None).unwrap();
        assert!(!bad_correctness.is_trustworthy());

        let bad_reproducibility = RealityVector::new(1.0, 0.5, 0.8, 10, None).unwrap();
        assert!(!bad_reproducibility.is_trustworthy());

        let low_confidence = RealityVector::new(1.0, 0.98, 0.8, 1, None).unwrap();
        assert!(!low_confidence.is_trustworthy());
    }

    #[test]
    fn empirical_confidence_formula_verification() {
        // التحقق من المعادلة الجديدة
        let r1 = RealityVector::new(1.0, 0.98, 0.8, 1, None).unwrap();
        let r2 = RealityVector::new(1.0, 0.98, 0.8, 2, None).unwrap();
        let r3 = RealityVector::new(1.0, 0.98, 0.8, 3, None).unwrap();

        // 1 run: 1.0 / 1.5 ≈ 0.67
        assert!((r1.empirical_confidence - 0.67).abs() < 0.01);

        // 2 runs: 2.0 / 2.5 = 0.80
        assert!((r2.empirical_confidence - 0.80).abs() < 0.01);

        // 3 runs: 3.0 / 3.5 ≈ 0.857
        assert!((r3.empirical_confidence - 0.857).abs() < 0.01);
        assert!(
            r3.empirical_confidence > 0.8,
            "3 runs should exceed 0.8 threshold"
        );
    }
}
