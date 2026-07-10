#![allow(clippy::field_reassign_with_default)]
#![forbid(unsafe_code)]

//! Sandbox execution engine.
//! Week 13: Simulated mode
//! Week 14: Docker mode (real execution)

use crate::compiler::RustSandboxCompiler;
use crate::config::{SandboxConfig, SandboxMode};
use crate::docker::DockerRunner;
use crate::metrics::compute_metrics;
use crate::reality::{LatencyMeasurement, RealityVector};
use crate::security::SecurityViolation;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// نتيجة التنفيذ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// معرف فريد للتنفيذ.
    pub execution_id: Uuid,
    /// RealityVector (إن نجح التنفيذ).
    pub reality: Option<RealityVector>,
    /// الانتهاكات الأمنية المكتشفة.
    pub violations: Vec<SecurityViolation>,
    /// هل نجح التنفيذ؟
    pub success: bool,
    /// رسالة الخطأ (إن وُجدت).
    pub error_message: Option<String>,
    /// وقت التنفيذ الفعلي.
    pub elapsed: Duration,
}

impl ExecutionResult {
    /// هل كان التنفيذ آمناً (بدون انتهاكات catastrophic)؟
    pub fn is_secure(&self) -> bool {
        !self.violations.iter().any(|v| v.is_catastrophic())
    }
}

/// Sandbox Executor.
pub struct SandboxExecutor {
    config: SandboxConfig,
}

impl SandboxExecutor {
    /// إنشاء executor جديد.
    pub fn new(config: SandboxConfig) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// تنفيذ artifact وقياس RealityVector.
    pub fn execute(&self, artifact_id: &str, code: &str) -> ExecutionResult {
        let execution_id = Uuid::new_v4();
        let start = Instant::now();

        match self.config.mode {
            SandboxMode::Simulated => self.execute_simulated(execution_id, artifact_id, start),
            SandboxMode::Local => self.execute_local(execution_id, start),
            SandboxMode::Docker => self.execute_docker(execution_id, code, start),
        }
    }

    // ─── Simulated (Week 13 — لم يتغير) ────────────────────────────

    fn execute_simulated(
        &self,
        execution_id: Uuid,
        artifact_id: &str,
        start: Instant,
    ) -> ExecutionResult {
        let success = !artifact_id.contains("fail");
        let correctness = if success { 1.0 } else { 0.0 };
        let reproducibility = if artifact_id.contains("flaky") {
            0.6
        } else {
            0.98
        };

        let mut latencies = Vec::new();
        for i in 0..self.config.runs_for_reproducibility {
            let jitter = (i as u64 * 5) % 10;
            latencies.push(Duration::from_millis(100 + jitter));
        }
        let latency = LatencyMeasurement::from_samples(latencies);

        let reality = RealityVector::new(
            correctness,
            reproducibility,
            0.95,
            self.config.runs_for_reproducibility,
            latency,
        )
        .ok();

        let mut violations = Vec::new();
        if artifact_id.contains("unsafe") {
            violations.push(SecurityViolation::ForbiddenSyscall {
                syscall: "execve".to_string(),
            });
        }
        if artifact_id.contains("escape") {
            violations.push(SecurityViolation::SandboxEscape {
                method: "ptrace".to_string(),
            });
        }

        ExecutionResult {
            execution_id,
            reality,
            violations,
            success,
            error_message: if success {
                None
            } else {
                Some("Simulated failure".to_string())
            },
            elapsed: start.elapsed(),
        }
    }

    // ─── Local (TODO) ────────────────────────────────────────────────

    fn execute_local(&self, execution_id: Uuid, start: Instant) -> ExecutionResult {
        ExecutionResult {
            execution_id,
            reality: None,
            violations: vec![],
            success: false,
            error_message: Some("Local execution not implemented yet".to_string()),
            elapsed: start.elapsed(),
        }
    }

    // ─── Docker (Week 14) ────────────────────────────────────────────

    fn execute_docker(&self, execution_id: Uuid, code: &str, start: Instant) -> ExecutionResult {
        let runner = DockerRunner::new(
            "rust:1.75-slim",
            self.config.limits.max_memory_mb,
            self.config.limits.max_cpu_percent,
            self.config.limits.max_execution_time,
        );

        let compiler = RustSandboxCompiler::new(runner, self.config.runs_for_reproducibility);

        match compiler.compile_and_run(code) {
            Err(e) => ExecutionResult {
                execution_id,
                reality: None,
                violations: vec![],
                success: false,
                error_message: Some(format!("Docker error: {}", e)),
                elapsed: start.elapsed(),
            },

            Ok(result) => {
                if !result.succeeded() {
                    // فشل compilation
                    return ExecutionResult {
                        execution_id,
                        reality: None,
                        violations: vec![],
                        success: false,
                        error_message: Some(match &result {
                            crate::compiler::CompilationResult::Failed { stderr } => {
                                format!("Compilation failed: {}", stderr)
                            }
                            _ => "Unknown compilation error".to_string(),
                        }),
                        elapsed: start.elapsed(),
                    };
                }

                let runs = result.runs();
                let metrics = compute_metrics(runs);

                let correctness = if metrics.all_succeeded { 1.0 } else { 0.0 };

                let reality = RealityVector::new(
                    correctness,
                    metrics.reproducibility,
                    if metrics.reproducibility > 0.9 {
                        0.95
                    } else {
                        0.5
                    },
                    metrics.run_count,
                    metrics.latency,
                )
                .ok();

                ExecutionResult {
                    execution_id,
                    reality,
                    violations: vec![],
                    success: metrics.all_succeeded,
                    error_message: None,
                    elapsed: start.elapsed(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Simulated tests (Week 13 — لم تتغير) ───────────────────────

    #[test]
    fn executor_creation_validates_config() {
        let mut config = SandboxConfig::default();
        config.runs_for_reproducibility = 0;
        assert!(SandboxExecutor::new(config).is_err());
    }

    #[test]
    fn simulated_execution_succeeds_for_normal_artifact() {
        let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
        let result = executor.execute("test-artifact", "fn main() {}");
        assert!(result.success);
        assert!(result.reality.is_some());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn simulated_execution_fails_for_fail_artifact() {
        let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
        let result = executor.execute("fail-artifact", "");
        assert!(!result.success);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn simulated_execution_detects_unsafe_syscall() {
        let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
        let result = executor.execute("unsafe-artifact", "");
        assert!(!result.violations.is_empty());
        assert!(matches!(
            result.violations[0],
            SecurityViolation::ForbiddenSyscall { .. }
        ));
    }

    #[test]
    fn simulated_execution_detects_escape_attempt() {
        let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
        let result = executor.execute("escape-artifact", "");
        assert!(!result.is_secure());
    }

    #[test]
    fn simulated_execution_measures_latency() {
        let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
        let result = executor.execute("perf-test", "");
        assert!(result.reality.is_some());
        let latency = result.reality.unwrap().latency;
        assert!(latency.is_some());
        assert_eq!(latency.unwrap().sample_count, 3);
    }

    #[test]
    fn simulated_execution_reports_flaky_reproducibility() {
        let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
        let result = executor.execute("flaky-test", "");
        let reality = result.reality.unwrap();
        assert!(!reality.is_reproducible());
    }

    #[test]
    fn local_execution_not_implemented_yet() {
        let mut config = SandboxConfig::default();
        config.mode = SandboxMode::Local;
        let executor = SandboxExecutor::new(config).unwrap();
        let result = executor.execute("test", "");
        assert!(!result.success);
        assert!(result.error_message.unwrap().contains("not implemented"));
    }

    // ─── Docker tests (Week 14) ──────────────────────────────────────

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_compiles_and_runs_hello_world() {
        let config = SandboxConfig::new(SandboxMode::Docker);
        let executor = SandboxExecutor::new(config).unwrap();

        let result = executor.execute(
            "hello-world",
            r#"fn main() { println!("hello from docker"); }"#,
        );

        assert!(result.success, "error: {:?}", result.error_message);
        assert!(result.reality.is_some());

        let reality = result.reality.unwrap();
        assert!(reality.is_correct());
        assert!(reality.is_reproducible());
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_fails_on_invalid_code() {
        let config = SandboxConfig::new(SandboxMode::Docker);
        let executor = SandboxExecutor::new(config).unwrap();

        let result = executor.execute("bad-code", "this is not rust");

        assert!(!result.success);
        assert!(result.error_message.is_some());
        assert!(result.error_message.unwrap().contains("Compilation failed"));
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_measures_real_latency() {
        let config = SandboxConfig::new(SandboxMode::Docker);
        let executor = SandboxExecutor::new(config).unwrap();

        let result = executor.execute("latency-test", r#"fn main() { println!("done"); }"#);

        assert!(result.success);
        let reality = result.reality.unwrap();
        assert!(reality.latency.is_some());
        assert_eq!(
            reality.latency.unwrap().sample_count,
            3 // default runs
        );
    }

    #[test]
    #[cfg_attr(
        not(feature = "docker_tests"),
        ignore = "requires --features docker_tests"
    )]
    fn docker_reproducibility_from_real_runs() {
        let config = SandboxConfig::new(SandboxMode::Docker);
        let executor = SandboxExecutor::new(config).unwrap();

        let result = executor.execute("stable", r#"fn main() { println!("42"); }"#);

        assert!(result.success);
        let reality = result.reality.unwrap();
        // برنامج deterministic = reproducibility عالية
        assert!(
            reality.reproducibility > 0.95,
            "got: {}",
            reality.reproducibility
        );
    }
}
