#![forbid(unsafe_code)]

//! AppState — shared state for the API server

use std::sync::Arc;
use tokio::sync::Mutex;
use ec_governance::proposal::ProposalStore;
use ec_governance::audit::AuditLog;
use ec_governance::storage::GovernanceStorage;
use ec_memory::CausalMemoryGraph;

/// Shared application state — thin wrapper, no business logic
#[derive(Clone)]
pub struct AppState {
    pub proposals: Arc<Mutex<ProposalStore>>,
    pub audit: Arc<Mutex<AuditLog>>,
    pub memory: Arc<Mutex<CausalMemoryGraph>>,
    pub gov_storage: Arc<GovernanceStorage>,
}

impl AppState {
    /// Build state with in-memory storage (for tests)
    pub fn in_memory() -> Result<Self, ec_governance::GovernanceError> {
        let gov_storage = GovernanceStorage::in_memory()?;
        Ok(Self {
            proposals: Arc::new(Mutex::new(ProposalStore::new())),
            audit: Arc::new(Mutex::new(AuditLog::new())),
            memory: Arc::new(Mutex::new(CausalMemoryGraph::new())),
            gov_storage: Arc::new(gov_storage),
        })
    }

    /// Build state with file storage
    pub fn open(db_path: &std::path::Path) -> Result<Self, ec_governance::GovernanceError> {
        let gov_storage = GovernanceStorage::open(db_path)?;

        // Restore proposals from disk
        let proposals_vec = gov_storage.load_proposals().unwrap_or_default();
        let mut proposal_store = ProposalStore::new();
        for p in proposals_vec {
            proposal_store.submit(p);
        }

        Ok(Self {
            proposals: Arc::new(Mutex::new(proposal_store)),
            audit: Arc::new(Mutex::new(AuditLog::new())),
            memory: Arc::new(Mutex::new(CausalMemoryGraph::new())),
            gov_storage: Arc::new(gov_storage),
        })
    }
}
