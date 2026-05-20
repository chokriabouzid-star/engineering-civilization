#![deny(warnings)]
#![forbid(unsafe_code)]

use ec_sandbox::*;
use std::time::Duration;

// ─── Gate Test 1: Configuration ─────────────────────────────────────

#[test]
fn gate_default_config_is_valid() {
    let config = SandboxConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.mode, SandboxMode::Simulated);
    assert_eq!(config.network, NetworkPolicy::Isolated);
}

#[test]
fn gate_resource_limits_validation() {
    let mut limits = ResourceLimits::default();
    
    // Valid limits
    assert!(limits.validate().is_ok());
    
    // Invalid CPU
    limits.max_cpu_percent = 2.0;
    assert!(limits.validate().is_err());
    limits.max_cpu_percent = 0.5;
    
    // Invalid memory
    limits.max_memory_mb = 0;
    assert!(limits.validate().is_err());
    limits.max_memory_mb = 512;
    
    // Invalid execution time
    limits.max_execution_time = Duration::from_millis(50);
    assert!(limits.validate().is_err());
}

// ─── Gate Test 2: RealityVector Construction ────────────────────────

#[test]
fn gate_reality_vector_requires_executor() {
    // RealityVector::new is pub(crate) — cannot be called from tests
    // This is enforced by the compiler
    
    // Instead, we must use SandboxExecutor
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("test", "fn main() {}");
    
    assert!(result.reality.is_some());
}

#[test]
fn gate_reality_vector_validates_ranges() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("normal-artifact", "");
    
    assert!(result.success);
    let reality = result.reality.unwrap();
    
    // All values must be in [0.0, 1.0]
    assert!((0.0..=1.0).contains(&reality.correctness));
    assert!((0.0..=1.0).contains(&reality.reproducibility));
    assert!((0.0..=1.0).contains(&reality.benchmark_validity));
    assert!((0.0..=1.0).contains(&reality.empirical_confidence));
}

// ─── Gate Test 3: Simulated Execution ───────────────────────────────

#[test]
fn gate_simulated_execution_normal_artifact() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("normal", "fn main() { println!(\"Hello\"); }");
    
    assert!(result.success);
    assert!(result.reality.is_some());
    assert!(result.violations.is_empty());
    assert!(result.is_secure());
    
    let reality = result.reality.unwrap();
    assert!(reality.is_correct());
    assert!(reality.is_reproducible());
}

#[test]
fn gate_simulated_execution_failing_artifact() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("fail-test", "");
    
    assert!(!result.success);
    assert!(result.error_message.is_some());
}

#[test]
fn gate_simulated_execution_flaky_artifact() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("flaky-behavior", "");
    
    assert!(result.success);
    let reality = result.reality.unwrap();
    assert!(!reality.is_reproducible()); // reproducibility < 0.95
}

// ─── Gate Test 4: Security Violations ───────────────────────────────

#[test]
fn gate_detects_forbidden_syscall() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("unsafe-code", "");
    
    assert!(!result.violations.is_empty());
    assert!(matches!(
        result.violations[0],
        SecurityViolation::ForbiddenSyscall { .. }
    ));
}

#[test]
fn gate_detects_sandbox_escape_attempt() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("escape-attempt", "");
    
    assert!(!result.is_secure());
    assert!(result.violations.iter().any(|v| v.is_catastrophic()));
}

#[test]
fn gate_security_violation_descriptions() {
    let v1 = SecurityViolation::ForbiddenSyscall {
        syscall: "reboot".to_string(),
    };
    assert!(v1.description().contains("reboot"));
    
    let v2 = SecurityViolation::SandboxEscape {
        method: "ptrace".to_string(),
    };
    assert!(v2.is_catastrophic());
}

// ─── Gate Test 5: Latency Measurement ───────────────────────────────

#[test]
fn gate_latency_measurement_from_samples() {
    let samples = vec![
        Duration::from_millis(90),
        Duration::from_millis(100),
        Duration::from_millis(110),
        Duration::from_millis(200),
        Duration::from_millis(500),
    ];
    
    let latency = LatencyMeasurement::from_samples(samples).unwrap();
    
    assert_eq!(latency.sample_count, 5);
    assert!(latency.p50 <= latency.p95);
    assert!(latency.p95 <= latency.p99);
}

#[test]
fn gate_execution_measures_latency() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    let result = executor.execute("perf-test", "");
    
    assert!(result.success);
    let reality = result.reality.unwrap();
    assert!(reality.latency.is_some());
    
    let latency = reality.latency.unwrap();
    assert_eq!(latency.sample_count, 3); // default runs
}

// ─── Gate Test 6: Reproducibility ───────────────────────────────────

#[test]
fn gate_reproducibility_measurement() {
    let mut config = SandboxConfig::default();
    config.runs_for_reproducibility = 5;
    
    let executor = SandboxExecutor::new(config).unwrap();
    let result = executor.execute("stable-test", "");
    
    assert!(result.success);
    let reality = result.reality.unwrap();
    assert_eq!(reality.runs_completed, 5);
    assert!(reality.reproducibility > 0.95);
}

// ─── Gate Test 7: Multiple Runs ─────────────────────────────────────

#[test]
fn gate_multiple_executions_independent() {
    let executor = SandboxExecutor::new(SandboxConfig::default()).unwrap();
    
    let r1 = executor.execute("art-1", "");
    let r2 = executor.execute("art-2", "");
    let r3 = executor.execute("art-3", "");
    
    // كل تنفيذ له execution_id فريد
    assert_ne!(r1.execution_id, r2.execution_id);
    assert_ne!(r2.execution_id, r3.execution_id);
    assert_ne!(r1.execution_id, r3.execution_id);
}

// ─── Gate Test 8: Empirical Confidence ──────────────────────────────

#[test]
fn gate_empirical_confidence_increases_with_runs() {
    let executor1 = SandboxExecutor::new(SandboxConfig {
        runs_for_reproducibility: 1,
        ..Default::default()
    })
    .unwrap();
    
    let executor10 = SandboxExecutor::new(SandboxConfig {
        runs_for_reproducibility: 10,
        ..Default::default()
    })
    .unwrap();
    
    let r1 = executor1.execute("test", "").reality.unwrap();
    let r10 = executor10.execute("test", "").reality.unwrap();
    
    assert!(r1.empirical_confidence < r10.empirical_confidence);
}

// ─── Gate Test 9: Network Policy ────────────────────────────────────

#[test]
fn gate_network_policies() {
    let isolated = NetworkPolicy::Isolated;
    let loopback = NetworkPolicy::LoopbackOnly;
    let allowlist = NetworkPolicy::Allowlist(vec!["example.com".to_string()]);
    
    assert_ne!(isolated, loopback);
    assert_ne!(loopback, allowlist);
}

// ─── Gate Test 10: Syscall Policy ───────────────────────────────────

#[test]
fn gate_default_safe_syscalls_list() {
    let syscalls = SyscallPolicy::default_safe_syscalls();
    
    assert!(syscalls.contains(&"read".to_string()));
    assert!(syscalls.contains(&"write".to_string()));
    assert!(syscalls.contains(&"exit".to_string()));
    assert!(!syscalls.contains(&"reboot".to_string()));
}

// ─── Final Gate: Integration ────────────────────────────────────────

#[test]
fn week13_gate_full_pipeline() {
    // Setup
    let mut config = SandboxConfig::default();
    config.mode = SandboxMode::Simulated;
    config.runs_for_reproducibility = 3;
    config.limits.max_execution_time = Duration::from_secs(10);
    
    assert!(config.validate().is_ok());
    
    let executor = SandboxExecutor::new(config).unwrap();
    
    // Execute normal artifact
    let result = executor.execute("integration-test", "fn main() { /* good code */ }");
    
    // Assertions
    assert!(result.success, "Execution should succeed");
    assert!(result.is_secure(), "Execution should be secure");
    assert!(result.reality.is_some(), "Should have RealityVector");
    
    let reality = result.reality.unwrap();
    assert!(reality.is_correct(), "Should be correct");
    assert!(reality.is_reproducible(), "Should be reproducible");
    assert!(reality.is_trustworthy(), "Should be trustworthy");
    assert_eq!(reality.runs_completed, 3, "Should complete 3 runs");
    assert!(reality.latency.is_some(), "Should measure latency");
    
    println!("✅ Week 13 Gate: PASSED");
    println!("   - Configuration validated");
    println!("   - RealityVector constructed from execution");
    println!("   - Security violations detected");
    println!("   - Latency measured");
    println!("   - Reproducibility computed");
}
