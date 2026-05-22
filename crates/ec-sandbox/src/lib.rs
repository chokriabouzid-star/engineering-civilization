#![forbid(unsafe_code)]

//! ec-sandbox — Sandbox execution (Phase 2)
//!
//! Safe execution environment with 5-layer security,
//! reproducibility measurement, and reality feedback.

pub mod bayesian;
pub mod compiler;
pub mod config;
pub mod docker;
pub mod executor;
pub mod feedback;
pub mod hardened;
pub mod metrics;
pub mod reality;
pub mod security;

// ─── Re-exports (العقد — الاختبارات تعتمد عليها) ────────────────────

pub use config::{
    NetworkPolicy, ResourceLimits, SandboxConfig, SandboxMode, SyscallPolicy,
};
pub use executor::{ExecutionResult, SandboxExecutor};
pub use feedback::{PredictionError, PredictionRecord, RealityFeedback};
pub use reality::{LatencyMeasurement, RealityVector};
pub use security::SecurityViolation;
pub use bayesian::BayesianTracker;
