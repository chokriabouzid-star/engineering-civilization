#![forbid(unsafe_code)]

//! ec-governance — Constitutional governance
//!
//! Phase 6 (Weeks 43-46):
//! - ProposalStore: اقتراحات تعديل دستوري (append-only)
//! - AuditLog: سجل أحداث لا يُحذف
//! - GovernanceStorage: SQLite persistence
//! - drift_trigger: DriftReport → Proposal تلقائي

pub mod audit;
pub mod drift_trigger;
pub mod proposal;
pub mod storage;

pub use audit::{AuditEntry, AuditLog, GovernanceEvent};
pub use drift_trigger::propose_from_drift;
pub use proposal::{
    ConstitutionalProposal, GovernanceError, ProposalOrigin, ProposalStatus, ProposalStore,
    ProposedChange, SystemTrigger, ThresholdDirection,
};
pub use storage::GovernanceStorage;
