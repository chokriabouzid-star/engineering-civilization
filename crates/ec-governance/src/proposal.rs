#![forbid(unsafe_code)]

//! ConstitutionalProposal — اقتراح تعديل دستوري، immutable بعد الإنشاء

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// اقتراح تعديل دستوري
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionalProposal {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub proposed_by: ProposalOrigin,
    pub change: ProposedChange,
    pub justification: String,
    pub evidence_refs: Vec<String>,
    pub status: ProposalStatus,
}

impl ConstitutionalProposal {
    /// إنشاء اقتراح جديد (حالة: Pending)
    pub fn new(
        origin: ProposalOrigin,
        change: ProposedChange,
        justification: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            proposed_by: origin,
            change,
            justification: justification.into(),
            evidence_refs: vec![],
            status: ProposalStatus::Pending,
        }
    }

    /// إضافة مراجع أدلة
    pub fn with_evidence(mut self, refs: Vec<String>) -> Self {
        self.evidence_refs = refs;
        self
    }
}

/// من أين جاء الاقتراح
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalOrigin {
    /// من إنسان
    Human { name: String },
    /// من النظام تلقائياً
    System { trigger: SystemTrigger },
}

/// محفز النظام
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemTrigger {
    DriftDetected {
        angle_degrees: f64,
        classification: String,
    },
    OssificationDetected {
        rejection_rate: f64,
    },
    BayesianCalibrationDrift {
        diagnosis: String,
    },
}

/// نوع التغيير المقترح
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposedChange {
    AdjustThreshold {
        dimension: String,
        current: f64,
        proposed: f64,
        direction: ThresholdDirection,
    },
    AddInvariant {
        name: String,
        description: String,
        severity: String,
    },
    RemoveInvariant {
        name: String,
        reason: String,
    },
    UpdatePolicy {
        policy_key: String,
        description: String,
    },
}

/// اتجاه التعديل
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThresholdDirection {
    Tighten,
    Loosen,
}

/// حالة الاقتراح
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    UnderReview {
        reviewer: String,
        since: DateTime<Utc>,
    },
    Approved {
        by: String,
        at: DateTime<Utc>,
        note: String,
    },
    Rejected {
        reason: String,
        at: DateTime<Utc>,
    },
    Applied {
        at: DateTime<Utc>,
        effect: String,
    },
    Superseded {
        by: Uuid,
    },
}

/// Append-only store للاقتراحات — D1 محفوظ
pub struct ProposalStore {
    proposals: Vec<ConstitutionalProposal>,
}

impl ProposalStore {
    /// إنشاء store جديد
    pub fn new() -> Self {
        Self { proposals: vec![] }
    }

    /// تقديم اقتراح جديد
    pub fn submit(&mut self, p: ConstitutionalProposal) -> Uuid {
        let id = p.id;
        self.proposals.push(p);
        id
    }

    /// الموافقة على اقتراح
    pub fn approve(&mut self, id: Uuid, by: &str, note: &str) -> Result<(), GovernanceError> {
        self.transition(id, |p| match &p.status {
            ProposalStatus::Pending | ProposalStatus::UnderReview { .. } => {
                p.status = ProposalStatus::Approved {
                    by: by.into(),
                    at: Utc::now(),
                    note: note.into(),
                };
                Ok(())
            }
            _ => Err(GovernanceError::InvalidTransition {
                current: format!("{:?}", p.status),
                target: "Approved".into(),
            }),
        })
    }

    /// رفض اقتراح
    pub fn reject(&mut self, id: Uuid, reason: &str) -> Result<(), GovernanceError> {
        self.transition(id, |p| match &p.status {
            ProposalStatus::Pending | ProposalStatus::UnderReview { .. } => {
                p.status = ProposalStatus::Rejected {
                    reason: reason.into(),
                    at: Utc::now(),
                };
                Ok(())
            }
            _ => Err(GovernanceError::InvalidTransition {
                current: format!("{:?}", p.status),
                target: "Rejected".into(),
            }),
        })
    }

    /// تحديد اقتراح كـ مُطبَّق
    pub fn mark_applied(&mut self, id: Uuid, effect: &str) -> Result<(), GovernanceError> {
        self.transition(id, |p| match &p.status {
            ProposalStatus::Approved { .. } => {
                p.status = ProposalStatus::Applied {
                    at: Utc::now(),
                    effect: effect.into(),
                };
                Ok(())
            }
            _ => Err(GovernanceError::InvalidTransition {
                current: format!("{:?}", p.status),
                target: "Applied".into(),
            }),
        })
    }

    /// الاقتراحات المعلقة
    pub fn pending(&self) -> Vec<&ConstitutionalProposal> {
        self.proposals
            .iter()
            .filter(|p| matches!(p.status, ProposalStatus::Pending))
            .collect()
    }

    /// الاقتراحات الموافق عليها ولم تُطبَّق
    pub fn approved_pending_application(&self) -> Vec<&ConstitutionalProposal> {
        self.proposals
            .iter()
            .filter(|p| matches!(p.status, ProposalStatus::Approved { .. }))
            .collect()
    }

    /// كل الاقتراحات
    pub fn all(&self) -> &[ConstitutionalProposal] {
        &self.proposals
    }

    /// البحث عن اقتراح
    pub fn find(&self, id: Uuid) -> Option<&ConstitutionalProposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    /// البحث mutable
    pub fn find_mut(&mut self, id: Uuid) -> Option<&mut ConstitutionalProposal> {
        self.proposals.iter_mut().find(|p| p.id == id)
    }

    fn transition<F>(&mut self, id: Uuid, f: F) -> Result<(), GovernanceError>
    where
        F: FnOnce(&mut ConstitutionalProposal) -> Result<(), GovernanceError>,
    {
        self.proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(GovernanceError::NotFound(id))
            .and_then(f)
    }
}

impl Default for ProposalStore {
    fn default() -> Self {
        Self::new()
    }
}

/// أخطاء الحوكمة
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    /// اقتراح غير موجود
    #[error("Proposal not found: {0}")]
    NotFound(Uuid),
    /// انتقال غير صالح
    #[error("Invalid transition from {current} to {target}")]
    InvalidTransition { current: String, target: String },
    /// خطأ تخزين
    #[error("Storage error: {0}")]
    Storage(String),
}
