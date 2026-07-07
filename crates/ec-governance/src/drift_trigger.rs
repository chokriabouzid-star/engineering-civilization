#![forbid(unsafe_code)]

//! DriftTrigger — ربط DriftReport بـ Proposal تلقائي
//! D2 محفوظ: لا يعدّل Fitness

use crate::proposal::{ConstitutionalProposal, ProposalOrigin, ProposedChange, SystemTrigger};
use ec_memory::DriftReport;

/// يُولّد اقتراح من تقرير انجراف إذا استدعى الأمر ذلك
pub fn propose_from_drift(report: &DriftReport) -> Option<ConstitutionalProposal> {
    match &report.recommended_action {
        ec_memory::DriftAction::HumanIntervention { reason } => Some(ConstitutionalProposal::new(
            ProposalOrigin::System {
                trigger: SystemTrigger::DriftDetected {
                    angle_degrees: report.drift_angle_degrees,
                    classification: format!("{:?}", report.classification),
                },
            },
            ProposedChange::UpdatePolicy {
                policy_key: "drift_response".into(),
                description: format!(
                    "Drift {:.1}° detected — human review required: {}",
                    report.drift_angle_degrees, reason
                ),
            },
            format!("Auto-generated from drift analysis: {}", reason),
        )),
        ec_memory::DriftAction::ReviewConstitution { reason } => Some(ConstitutionalProposal::new(
            ProposalOrigin::System {
                trigger: SystemTrigger::DriftDetected {
                    angle_degrees: report.drift_angle_degrees,
                    classification: format!("{:?}", report.classification),
                },
            },
            ProposedChange::UpdatePolicy {
                policy_key: "constitutional_review".into(),
                description: format!("Constitutional review suggested: {}", reason),
            },
            reason.clone(),
        )),
        _ => None,
    }
}
