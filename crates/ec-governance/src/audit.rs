#![forbid(unsafe_code)]

//! AuditLog — سجل لا يُحذف أبداً (D1)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// حدث حوكمي مُسجَّل
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event: GovernanceEvent,
    pub actor: String,
    pub context: String,
}

/// أنواع الأحداث
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceEvent {
    ProposalCreated { id: Uuid, change_type: String },
    ProposalApproved { id: Uuid, by: String },
    ProposalRejected { id: Uuid, reason: String },
    ProposalApplied { id: Uuid, effect: String },
    SystemAlertFired { kind: String, details: String },
    ConstitutionAmended { proposal_id: Uuid, summary: String },
    HumanOverride { target: String, reason: String },
}

/// Append-only audit log — D1 يُطبَّق هنا أيضاً
pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// إنشاء سجل جديد
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    /// تسجيل حدث
    pub fn record(
        &mut self,
        event: GovernanceEvent,
        actor: impl Into<String>,
        context: impl Into<String>,
    ) {
        self.entries.push(AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event,
            actor: actor.into(),
            context: context.into(),
        });
    }

    /// كل المدخلات
    pub fn all(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// آخر N مدخلات
    pub fn last_n(&self, n: usize) -> &[AuditEntry] {
        let len = self.entries.len();
        &self.entries[len.saturating_sub(n)..]
    }

    /// مدخلات اقتراح معين
    pub fn for_proposal(&self, id: Uuid) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| match &e.event {
                GovernanceEvent::ProposalCreated { id: pid, .. } => *pid == id,
                GovernanceEvent::ProposalApproved { id: pid, .. } => *pid == id,
                GovernanceEvent::ProposalRejected { id: pid, .. } => *pid == id,
                GovernanceEvent::ProposalApplied { id: pid, .. } => *pid == id,
                _ => false,
            })
            .collect()
    }
    // ❌ لا delete() لا clear() لا purge()
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}
