# ADR-009: Sandbox Foundation

**Status:** ✅ Implemented (Week 13)  
**Date:** 2026-05-16  
**Replaces:** N/A

---

## Context

Phase 2 requires transforming the Constitutional system from "prediction" to "reality testing". We need a sandbox that:

1. **Executes code safely** (isolation, resource limits, syscall filtering)
2. **Measures RealityVector** (correctness, reproducibility, latency)
3. **Detects security violations** (escape attempts, forbidden operations)
4. **Enforces construction invariants** (RealityVector from execution only)

---

## Decision

### Core Architecture

```rust
SandboxConfig → SandboxExecutor → ExecutionResult → RealityVector
Execution Modes:

Simulated: Development/testing (Week 13)
Local: Direct process execution (Week 14)
Docker: Full isolation (Week 14)
RealityVector Invariant:

Rust

// ❌ Cannot construct manually:
// let rv = RealityVector::new(...); // compile error

// ✅ Must come from execution:
let result = executor.execute(artifact_id, code);
let rv = result.reality.unwrap();
Constructor is pub(crate) — only accessible within ec-sandbox.

Implementation
1. Configuration (config.rs)
Rust

pub struct ResourceLimits {
    max_cpu_percent: f64,      // [0.0, 1.0]
    max_memory_mb: u64,
    max_disk_mb: u64,
    max_execution_time: Duration,
}

pub enum NetworkPolicy {
    Isolated,              // Default — no network
    LoopbackOnly,          // localhost only
    Allowlist(Vec<String>), // Specific domains
}

pub enum SyscallPolicy {
    Allowlist(Vec<String>),
    Blocklist(Vec<String>),
    DefaultSafe,           // read, write, exit, etc.
}
Validation: All configs validate on construction.

2. RealityVector (reality.rs)
Rust

pub struct RealityVector {
    pub correctness: f64,          // [0.0, 1.0]
    pub reproducibility: f64,      // [0.0, 1.0]
    pub benchmark_validity: f64,   // [0.0, 1.0]
    pub empirical_confidence: f64, // computed from runs
    pub runs_completed: usize,
    pub latency: Option<LatencyMeasurement>,
}

impl RealityVector {
    pub(crate) fn new(...) -> anyhow::Result<Self> { ... }
    
    pub fn is_correct(&self) -> bool {
        self.correctness > 0.99
    }
    
    pub fn is_reproducible(&self) -> bool {
        self.reproducibility > 0.95
    }
    
    pub fn is_trustworthy(&self) -> bool {
        self.is_correct() && 
        self.is_reproducible() && 
        self.empirical_confidence > 0.8
    }
}
Empirical Confidence Formula:

text

confidence = runs / (runs + 0.5)

1 run:  0.67
2 runs: 0.80
3 runs: 0.857 ← exceeds 0.8 threshold
5 runs: 0.91
10 runs: 0.95
Design Rationale:

Asymptotic to 1.0 (never reaches perfect certainty)
3 runs sufficient for "trustworthy" threshold
Formula: f(n) = n/(n+0.5) chosen empirically
3. Security (security.rs)
Rust

pub enum SecurityViolation {
    SandboxEscape { method: String },          // Catastrophic
    ForbiddenSyscall { syscall: String },
    ResourceLimitExceeded { resource, value, limit },
    UnauthorizedFileAccess { path: String },   // Catastrophic
    UnauthorizedNetworkAccess { target: String },
}

impl SecurityViolation {
    pub fn is_catastrophic(&self) -> bool {
        // SandboxEscape + UnauthorizedFileAccess → immediate halt
    }
}
Catastrophic violations:

Sandbox escape attempts
Unauthorized file access (e.g., /etc/passwd)
Non-catastrophic violations:

Resource limit exceeded → terminates gracefully
Forbidden syscall → logged, execution continues (if safe)
4. Executor (executor.rs)
Rust

pub struct SandboxExecutor {
    config: SandboxConfig,
}

impl SandboxExecutor {
    pub fn execute(&self, artifact_id: &str, code: &str) 
        -> ExecutionResult 
    {
        match self.config.mode {
            Simulated => execute_simulated(...),
            Local     => execute_local(...),     // Week 14
            Docker    => execute_docker(...),    // Week 14
        }
    }
}

pub struct ExecutionResult {
    pub execution_id: Uuid,
    pub reality: Option<RealityVector>,
    pub violations: Vec<SecurityViolation>,
    pub success: bool,
    pub error_message: Option<String>,
    pub elapsed: Duration,
}
Simulated Mode (Week 13):

Artifact behavior inferred from artifact_id:
"fail" → failure
"flaky" → low reproducibility
"unsafe" → syscall violation
"escape" → sandbox escape attempt
Runs multiple times (default: 3) to measure reproducibility
Generates realistic latency distributions
Guarantees
Type-Level Enforcement
RealityVector from execution only:

Rust

// ❌ Compile error (pub(crate) constructor):
let rv = RealityVector::new(1.0, 0.9, 0.8, 3, None);

// ✅ Only way:
let executor = SandboxExecutor::new(config)?;
let result = executor.execute(artifact_id, code);
Config validation before execution:

Rust

let config = SandboxConfig { ... };
let executor = SandboxExecutor::new(config)?; // validates
Security violations always recorded:

Rust

assert!(result.violations.is_empty() || !result.is_secure());
Testing
Coverage
text

30 unit tests (config, reality, security, executor)
18 gate tests (integration scenarios)
───────────────────────────────────────────────
48 total tests — 0 failures
Gate Criteria (Week 13)
text

✅ Default config validates
✅ ResourceLimits enforces ranges
✅ RealityVector requires executor
✅ Simulated execution succeeds for normal artifacts
✅ Simulated execution fails for "fail" artifacts
✅ Security violations detected (forbidden syscall, escape)
✅ Latency measurement from samples
✅ Reproducibility computed from multiple runs
✅ Empirical confidence increases with runs
✅ Multiple executions have unique IDs
Performance
Simulated Mode:

Single execution: < 1ms
3 runs (reproducibility): ~3ms
No actual process spawning
Week 14 Targets (Docker):

Single execution: < 100ms
3 runs: < 300ms
Overhead: Docker container startup
Security Posture
Week 13 (Simulated)
✅ Type safety enforced
✅ Violation detection logic implemented
⚠️ No actual isolation (simulated only)
Week 14 (Docker)
✅ Docker container isolation
✅ seccomp syscall filtering
✅ cgroup resource limits
✅ Network namespace isolation
🎯 Target: Zero escapes in 100 executions
Deferred to Week 14
Docker execution:

Rust

fn execute_docker(...) -> ExecutionResult {
    // TODO: Build minimal Rust container
    // TODO: Run code in isolated namespace
    // TODO: Collect stdout/stderr
    // TODO: Measure actual latency
}
Real reproducibility measurement:

Run code 3+ times in Docker
Compare outputs byte-for-byte
Measure latency variance
Syscall filtering:

Apply seccomp-bpf profile
Allowlist: read, write, exit, mmap, etc.
Blocklist: reboot, mount, ptrace, etc.
Escape vector testing:

5+ documented attack vectors
Test each in Docker sandbox
Verify detection + prevention
Consequences
Positive
RealityVector trusted by design — cannot be forged
Security violations explicit — all attempts logged
Reproducibility quantified — variance measured empirically
Incremental implementation — Simulated → Local → Docker
Negative
Docker dependency — adds complexity (Week 14)
Performance overhead — container startup latency
False negatives possible — simulation doesn't catch all issues
Neutral
Empirical confidence formula — tuned to project needs (may adjust)
Simulated mode permanent — useful for testing higher layers
Related
ADR-004: Integration Architecture (Constitutional + Reality)
ADR-010: (Next) Docker Execution Strategy
Week 14: RealityVector Construction (real measurements)
Week 15: Reality Feedback Loop (Constitution.learn())
Validation
Bash

# Week 13 Gate Test
cargo test -p ec-sandbox week13_gate_full_pipeline -- --nocapture

# Expected Output:
# ✅ Week 13 Gate: PASSED
#    - Configuration validated
#    - RealityVector constructed from execution
#    - Security violations detected
#    - Latency measured
#    - Reproducibility computed
Status: ✅ Week 13 Complete — Gate PASSED
Next: Week 14 — Docker Execution + Real RealityVector Measurement
