#![forbid(unsafe_code)]

//! Week 44 Gate — ec-governance Foundation

use ec_governance::*;
use uuid::Uuid;

fn make_proposal(dim: &str, from: f64, to: f64) -> ConstitutionalProposal {
    ConstitutionalProposal::new(
        ProposalOrigin::Human {
            name: "engineer".into(),
        },
        ProposedChange::AdjustThreshold {
            dimension: dim.into(),
            current: from,
            proposed: to,
            direction: ThresholdDirection::Tighten,
        },
        format!("Adjust {} from {} to {}", dim, from, to),
    )
}

// ─── Gate 1: proposal lifecycle ─────────────────────────────────────

#[test]
fn w44_proposal_full_lifecycle() {
    let mut store = ProposalStore::new();
    let mut audit = AuditLog::new();

    // إنشاء
    let p = make_proposal("security", 0.70, 0.75);
    let id = p.id;
    store.submit(p);
    audit.record(
        GovernanceEvent::ProposalCreated {
            id,
            change_type: "AdjustThreshold".into(),
        },
        "system",
        "",
    );
    assert_eq!(store.pending().len(), 1);

    // موافقة
    store.approve(id, "lead-eng", "مبرر كافٍ").unwrap();
    audit.record(
        GovernanceEvent::ProposalApproved {
            id,
            by: "lead-eng".into(),
        },
        "lead-eng",
        "",
    );
    assert_eq!(store.pending().len(), 0);
    assert_eq!(store.approved_pending_application().len(), 1);

    // تطبيق
    store
        .mark_applied(id, "security threshold raised to 0.75")
        .unwrap();
    audit.record(
        GovernanceEvent::ProposalApplied {
            id,
            effect: "security_threshold=0.75".into(),
        },
        "system",
        "",
    );
    assert_eq!(store.approved_pending_application().len(), 0);

    // Audit trail
    assert_eq!(audit.for_proposal(id).len(), 3);
    assert_eq!(audit.all().len(), 3);
}

// ─── Gate 2: invalid transitions ────────────────────────────────────

#[test]
fn w44_reject_after_approve_fails() {
    let mut store = ProposalStore::new();
    let id = store.submit(make_proposal("performance", 0.20, 0.30));
    store.approve(id, "a", "ok").unwrap();
    assert!(store.reject(id, "changed mind").is_err());
}

#[test]
fn w44_double_approve_fails() {
    let mut store = ProposalStore::new();
    let id = store.submit(make_proposal("coverage", 0.60, 0.70));
    store.approve(id, "a", "ok").unwrap();
    assert!(store.approve(id, "b", "again").is_err());
}

#[test]
fn w44_apply_without_approve_fails() {
    let mut store = ProposalStore::new();
    let id = store.submit(make_proposal("security", 0.70, 0.80));
    assert!(store.mark_applied(id, "effect").is_err());
}

#[test]
fn w44_not_found_errors() {
    let mut store = ProposalStore::new();
    let fake = Uuid::new_v4();
    assert!(store.approve(fake, "a", "ok").is_err());
    assert!(store.reject(fake, "reason").is_err());
    assert!(store.mark_applied(fake, "effect").is_err());
}

// ─── Gate 3: audit is append-only ───────────────────────────────────

#[test]
fn w44_audit_is_append_only() {
    let mut audit = AuditLog::new();
    audit.record(
        GovernanceEvent::HumanOverride {
            target: "test".into(),
            reason: "test".into(),
        },
        "admin",
        "",
    );
    assert_eq!(audit.all().len(), 1);
    // audit.clear(); // compile error — D1 محفوظ ✅
}

#[test]
fn w44_audit_last_n() {
    let mut audit = AuditLog::new();
    for i in 0..10 {
        audit.record(
            GovernanceEvent::SystemAlertFired {
                kind: format!("alert-{}", i),
                details: "test".into(),
            },
            "system",
            "",
        );
    }
    assert_eq!(audit.last_n(3).len(), 3);
    assert_eq!(audit.all().len(), 10);
}

// ─── Gate 4: proposal with evidence ────────────────────────────────

#[test]
fn w44_proposal_with_evidence() {
    let p =
        make_proposal("security", 0.70, 0.75).with_evidence(vec!["node-1".into(), "node-2".into()]);
    assert_eq!(p.evidence_refs.len(), 2);
}

#[test]
fn w44_proposal_system_origin() {
    let p = ConstitutionalProposal::new(
        ProposalOrigin::System {
            trigger: SystemTrigger::DriftDetected {
                angle_degrees: 50.0,
                classification: "ValueShift".into(),
            },
        },
        ProposedChange::UpdatePolicy {
            policy_key: "drift".into(),
            description: "test".into(),
        },
        "auto",
    );
    assert!(matches!(p.proposed_by, ProposalOrigin::System { .. }));
    assert!(matches!(p.status, ProposalStatus::Pending));
}

// ─── Gate 5: find ───────────────────────────────────────────────────

#[test]
fn w44_find_existing() {
    let mut store = ProposalStore::new();
    let id = store.submit(make_proposal("security", 0.70, 0.75));
    assert!(store.find(id).is_some());
}

#[test]
fn w44_find_nonexistent() {
    let store = ProposalStore::new();
    assert!(store.find(Uuid::new_v4()).is_none());
}

// ─── Gate 6: drift trigger ──────────────────────────────────────────

#[test]
fn w44_drift_trigger_generates_proposal() {
    let report = ec_memory::DriftReport {
        drift_angle_degrees: 50.0,
        baseline_count: 10,
        current_count: 10,
        total_decisions: 20,
        classification: ec_memory::DriftClassification::ValueShift { angle: 50.0 },
        recommended_action: ec_memory::DriftAction::HumanIntervention {
            reason: "Value rotation exceeds 45°".into(),
        },
    };

    let proposal = propose_from_drift(&report);
    assert!(proposal.is_some());
    let p = proposal.unwrap();
    assert!(matches!(p.proposed_by, ProposalOrigin::System { .. }));
}

#[test]
fn w44_drift_stable_no_proposal() {
    let report = ec_memory::DriftReport {
        drift_angle_degrees: 3.0,
        baseline_count: 10,
        current_count: 10,
        total_decisions: 20,
        classification: ec_memory::DriftClassification::Stable,
        recommended_action: ec_memory::DriftAction::None,
    };

    assert!(propose_from_drift(&report).is_none());
}

#[test]
fn w44_drift_review_constitution_generates_proposal() {
    let report = ec_memory::DriftReport {
        drift_angle_degrees: 35.0,
        baseline_count: 10,
        current_count: 10,
        total_decisions: 20,
        classification: ec_memory::DriftClassification::ValueShift { angle: 35.0 },
        recommended_action: ec_memory::DriftAction::ReviewConstitution {
            reason: "Significant value shift".into(),
        },
    };

    let proposal = propose_from_drift(&report);
    assert!(proposal.is_some());
}

// ─── Gate 7: storage ────────────────────────────────────────────────

#[test]
fn w44_storage_in_memory() {
    let storage = GovernanceStorage::in_memory();
    assert!(storage.is_ok());
}

#[test]
fn w44_storage_save_load_proposal() {
    let storage = GovernanceStorage::in_memory().unwrap();
    let p = make_proposal("security", 0.70, 0.75);
    storage.save_proposal(&p).unwrap();

    let loaded = storage.load_proposals().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, p.id);
}

#[test]
fn w44_storage_save_load_audit() {
    let storage = GovernanceStorage::in_memory().unwrap();
    let entry = AuditEntry {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        event: GovernanceEvent::HumanOverride {
            target: "test".into(),
            reason: "test".into(),
        },
        actor: "admin".into(),
        context: "".into(),
    };
    storage.save_audit(&entry).unwrap();

    let loaded = storage.load_audit().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].actor, "admin");
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w44_gate_complete() {
    let mut store = ProposalStore::new();
    let mut audit = AuditLog::new();

    let p1 = make_proposal("security", 0.70, 0.75);
    let p2 = make_proposal("coverage", 0.60, 0.70);
    let id1 = p1.id;
    let id2 = p2.id;

    store.submit(p1);
    store.submit(p2);
    audit.record(
        GovernanceEvent::ProposalCreated {
            id: id1,
            change_type: "AdjustThreshold".into(),
        },
        "system",
        "",
    );
    audit.record(
        GovernanceEvent::ProposalCreated {
            id: id2,
            change_type: "AdjustThreshold".into(),
        },
        "system",
        "",
    );

    store.approve(id1, "lead", "ok").unwrap();
    store.reject(id2, "not needed").unwrap();

    println!("═══════════════════════════════════════════════");
    println!("  Week 44 Gate — ec-governance Foundation");
    println!("═══════════════════════════════════════════════");
    println!("  Proposals:     {}", store.all().len());
    println!("  Pending:       {}", store.pending().len());
    println!(
        "  Approved:      {}",
        store.approved_pending_application().len()
    );
    println!("  Audit entries: {}", audit.all().len());
    println!("═══════════════════════════════════════════════");

    assert_eq!(store.all().len(), 2);
    assert_eq!(store.pending().len(), 0);
    assert_eq!(audit.all().len(), 2);

    println!("  ✅ Week 44 Gate: PASSED");
}
