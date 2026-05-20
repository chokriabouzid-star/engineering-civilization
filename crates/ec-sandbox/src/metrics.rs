#![forbid(unsafe_code)]

//! قياس reproducibility وlatency من نتائج التنفيذ الفعلية — Week 14

use crate::compiler::RunOutput;
use crate::reality::LatencyMeasurement;
use std::time::Duration;

/// قياسات مستخرجة من عدة runs.
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// reproducibility: [0.0, 1.0]
    pub reproducibility: f64,
    /// هل كانت كل الـ runs ناجحة؟
    pub all_succeeded: bool,
    /// قياسات latency.
    pub latency: Option<LatencyMeasurement>,
    /// عدد الـ runs.
    pub run_count: usize,
}

/// حساب reproducibility من نتائج التنفيذ.
///
/// المنهج: SHA256 للـ stdout + exit_code لكل run،
/// ثم نسبة الـ runs التي تطابق أول run.
pub fn measure_reproducibility(runs: &[RunOutput]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }
    if runs.len() == 1 {
        // run واحدة: لا يمكن قياس التكرار
        return if runs[0].success() { 1.0 } else { 0.0 };
    }

    let first_hash = runs[0].content_hash();
    let identical = runs
        .iter()
        .filter(|r| r.content_hash() == first_hash)
        .count();

    identical as f64 / runs.len() as f64
}

/// استخراج latency measurements من الـ runs.
pub fn extract_latency(runs: &[RunOutput]) -> Option<LatencyMeasurement> {
    if runs.is_empty() {
        return None;
    }

    let samples: Vec<Duration> = runs.iter().map(|r| r.elapsed).collect();
    LatencyMeasurement::from_samples(samples)
}

/// حساب كل الـ metrics من runs.
pub fn compute_metrics(runs: &[RunOutput]) -> ExecutionMetrics {
    let reproducibility = measure_reproducibility(runs);
    let all_succeeded = !runs.is_empty() && runs.iter().all(|r| r.success());
    let latency = extract_latency(runs);

    ExecutionMetrics {
        reproducibility,
        all_succeeded,
        latency,
        run_count: runs.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_run(stdout: &str, exit_code: i32, elapsed_ms: u64) -> RunOutput {
        RunOutput {
            stdout: stdout.to_string(),
            stderr: String::new(),
            exit_code,
            elapsed: Duration::from_millis(elapsed_ms),
        }
    }

    #[test]
    fn empty_runs_gives_zero_reproducibility() {
        assert_eq!(measure_reproducibility(&[]), 0.0);
    }

    #[test]
    fn single_successful_run_gives_full_reproducibility() {
        let runs = vec![make_run("ok", 0, 100)];
        assert_eq!(measure_reproducibility(&runs), 1.0);
    }

    #[test]
    fn single_failed_run_gives_zero_reproducibility() {
        let runs = vec![make_run("", 1, 100)];
        assert_eq!(measure_reproducibility(&runs), 0.0);
    }

    #[test]
    fn identical_runs_give_full_reproducibility() {
        let runs = vec![
            make_run("hello\n", 0, 100),
            make_run("hello\n", 0, 105),
            make_run("hello\n", 0, 98),
        ];
        assert_eq!(measure_reproducibility(&runs), 1.0);
    }

    #[test]
    fn different_outputs_reduce_reproducibility() {
        let runs = vec![
            make_run("hello\n", 0, 100),
            make_run("world\n", 0, 100), // مختلف
            make_run("hello\n", 0, 100),
        ];
        // 2 من 3 متطابقة = 0.667
        let r = measure_reproducibility(&runs);
        assert!((r - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn latency_extracted_correctly() {
        let runs = vec![
            make_run("ok", 0, 100),
            make_run("ok", 0, 200),
            make_run("ok", 0, 150),
        ];
        let latency = extract_latency(&runs).unwrap();
        assert_eq!(latency.sample_count, 3);
        assert!(latency.p50 >= Duration::from_millis(100));
        assert!(latency.p99 <= Duration::from_millis(200));
    }

    #[test]
    fn compute_metrics_all_succeeded() {
        let runs = vec![
            make_run("ok\n", 0, 100),
            make_run("ok\n", 0, 110),
            make_run("ok\n", 0, 90),
        ];
        let m = compute_metrics(&runs);
        assert!(m.all_succeeded);
        assert_eq!(m.reproducibility, 1.0);
        assert_eq!(m.run_count, 3);
        assert!(m.latency.is_some());
    }

    #[test]
    fn compute_metrics_partial_failure() {
        let runs = vec![
            make_run("ok\n", 0, 100),
            make_run("ok\n", 1, 100), // فشل
        ];
        let m = compute_metrics(&runs);
        assert!(!m.all_succeeded);
    }
}
