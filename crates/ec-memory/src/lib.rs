#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! ec-memory — Causal Memory for Engineering Civilization
//!
//! Week 20 — Phase 3
//!
//! **Design Invariants:**
//! - Append-only: الماضي لا يتغير
//! - Vec-based: الترتيب الزمني محفوظ
//! - DAG enforced: لا دورات
//! - Retrospective only: الشيء الوحيد المُتغيّر هو التفسير

/// الأنواع الأساسية.
pub mod types;

/// عقدة القرار.
pub mod node;

/// Causal graph.
pub mod graph;

/// استعلامات الذاكرة (Week 23).
pub mod query;

/// تخزين SQLite (Week 27).
pub mod storage;

pub use graph::{CausalMemoryGraph, MemoryError};
pub use node::{DecisionNode, DecisionNodeBuilder, RejectedAlternative, SandboxOutcome};
pub use query::{
    CounterfactualGain, FitnessSnapshot, MemoryQuery, SimilarDecision,
};
pub use types::{
    ArtifactHash, ArtifactSnapshot, NodeId, RejectionReason,
    RetrospectiveAssessment,
};

/// كشف الانجراف القيمي من الذاكرة التاريخية (Week 24).
pub mod drift;

pub use storage::{MemoryStorage, SqliteStorage, StorageError};

pub use drift::{
    DriftAction, DriftClassification, DriftReport,
    HistoricalDriftAnalyzer,
};
pub mod outcome_storage;
pub use outcome_storage::OutcomeStorage;
pub mod bayesian_query;
pub use bayesian_query::{BayesianQuery, BayesianSimilarDecision};
