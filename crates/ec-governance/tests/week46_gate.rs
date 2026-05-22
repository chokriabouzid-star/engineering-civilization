#![forbid(unsafe_code)]

//! Week 46 Gate — GovernanceStorage persistence

use ec_governance::*;
use uuid::Uuid;

fn make_proposal(dim: &str, from: f64, to: f64) -> ConstitutionalProposal {
    ConstitutionalProposal::new(
        ProposalOrigin::Human { name: "engineer".into() },
        ProposedChange::AdjustThreshold {
            dimension: dim.into(),
            current: from,
            proposed: to,
            direction: ThresholdDirection::Tighten,
        },
        format!("Adjust {} from {} to {}", dim, from, to),
    )
}

// ─── Gate 1: proposals survive restart ──────────────────────────────

#[test]
fn w46_proposals_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");

    let id = {
        let storage = GovernanceStorage::open(&path).unwrap();
        let p = make_proposal("security", 0.70, 0.75);
        let id = p.id;
        storage.save_proposal(&p).unwrap();
        id
    };

    let storage = GovernanceStorage::open(&path).unwrap();
    let loaded = storage.load_proposals().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, id);
    assert_eq!(
        loaded[0].change,
        ProposedChange::AdjustThreshold {
            dimension: "security".into(),
            current: 0.70,
            proposed: 0.75,
            direction: ThresholdDirection::Tighten
        }
    );
}

// ─── Gate 2: audit survives restart ─────────────────────────────────

#[test]
fn w46_audit_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");

    let entry_id = {
        let storage = GovernanceStorage::open(&path).unwrap();
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
        let eid = entry.id;
        storage.save_audit(&entry).unwrap();
        eid
    };

    let storage = GovernanceStorage::open(&path).unwrap();
    let loaded = storage.load_audit().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, entry_id);
    assert_eq!(loaded[0].actor, "admin");
}

// ─── Gate 3: multiple proposals ─────────────────────────────────────

#[test]
fn w46_multiple_proposals_persist() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");
    let storage = GovernanceStorage::open(&path).unwrap();

    for i in 0..5 {
        let p = make_proposal(&format!("dim-{}", i), 0.5, 0.6);
        storage.save_proposal(&p).unwrap();
    }

    let loaded = storage.load_proposals().unwrap();
    assert_eq!(loaded.len(), 5);
}

// ─── Gate 4: proposal state update ──────────────────────────────────

#[test]
fn w46_proposal_state_update_persists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");

    let id = {
        let storage = GovernanceStorage::open(&path).unwrap();
        let mut p = make_proposal("security", 0.70, 0.75);
        let id = p.id;
        p.status = ProposalStatus::Approved {
            by: "lead".into(),
            at: chrono::Utc::now(),
            note: "ok".into(),
        };
        storage.save_proposal(&p).unwrap();
        id
    };

    let storage = GovernanceStorage::open(&path).unwrap();
    let loaded = storage.load_proposals().unwrap();
    assert_eq!(loaded.len(), 1);
    assert!(matches!(loaded[0].status, ProposalStatus::Approved { .. }));
    assert_eq!(loaded[0].id, id);
}

// ─── Gate 5: mixed audit events ─────────────────────────────────────

#[test]
fn w46_mixed_audit_events() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");
    let storage = GovernanceStorage::open(&path).unwrap();

    let pid = Uuid::new_v4();
    storage.save_audit(&AuditEntry {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        event: GovernanceEvent::ProposalCreated { id: pid, change_type: "AdjustThreshold".into() },
        actor: "system".into(),
        context: "".into(),
    }).unwrap();
    storage.save_audit(&AuditEntry {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        event: GovernanceEvent::ProposalApproved { id: pid, by: "lead".into() },
        actor: "lead".into(),
        context: "".into(),
    }).unwrap();
    storage.save_audit(&AuditEntry {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        event: GovernanceEvent::SystemAlertFired { kind: "drift".into(), details: "50°".into() },
        actor: "system".into(),
        context: "".into(),
    }).unwrap();

    let loaded = storage.load_audit().unwrap();
    assert_eq!(loaded.len(), 3);
}

// ─── Gate 6: empty storage loads nothing ────────────────────────────

#[test]
fn w46_empty_storage_loads_nothing() {
    let storage = GovernanceStorage::in_memory().unwrap();
    assert!(storage.load_proposals().unwrap().is_empty());
    assert!(storage.load_audit().unwrap().is_empty());
}

// ─── Gate 7: full governance cycle with persistence ─────────────────

#[test]
fn w46_full_cycle_with_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");

    // Session 1: إنشاء + موافقة
    let id = {
        let storage = GovernanceStorage::open(&path).unwrap();
        let mut store = ProposalStore::new();
        let p = make_proposal("security", 0.70, 0.80);
        let id = p.id;
        store.submit(p);
        store.approve(id, "lead", "good").unwrap();

        // حفظ الحالة المُحدثة
        if let Some(updated) = store.find(id) {
            storage.save_proposal(updated).unwrap();
        }
        id
    };

    // Session 2: تحميل + تطبيق
    {
        let storage = GovernanceStorage::open(&path).unwrap();
        let mut store = ProposalStore::new();
        for p in storage.load_proposals().unwrap() {
            store.submit(p);
        }

        assert_eq!(store.pending().len(), 0);
        assert_eq!(store.approved_pending_application().len(), 1);

        store.mark_applied(id, "security=0.80").unwrap();
        if let Some(updated) = store.find(id) {
            storage.save_proposal(updated).unwrap();
        }
    }

    // Session 3: تحقق
    {
        let storage = GovernanceStorage::open(&path).unwrap();
        let loaded = storage.load_proposals().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(matches!(loaded[0].status, ProposalStatus::Applied { .. }));
    }
}

// ─── Final Gate ─────────────────────────────────────────────────────

#[test]
fn w46_gate_complete() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gov.db");
    let storage = GovernanceStorage::open(&path).unwrap();

    let mut store = ProposalStore::new();
    let mut audit = AuditLog::new();

    let mut ids_to_approve: Vec<Uuid> = vec![];
    for i in 0..3 {
        let p = make_proposal(&format!("dim-{}", i), 0.5, 0.6 + i as f64 * 0.05);
        let id = p.id;
        ids_to_approve.push(id);
        store.submit(p.clone());
        storage.save_proposal(&p).unwrap();
        audit.record(
            GovernanceEvent::ProposalCreated { id, change_type: "AdjustThreshold".into() },
            "system", "",
        );
    }

    for id in ids_to_approve {
        store.approve(id, "lead", "ok").unwrap();
        if let Some(updated) = store.find(id) {
            storage.save_proposal(updated).unwrap();
        }
        audit.record(
            GovernanceEvent::ProposalApproved { id, by: "lead".into() },
            "lead", "",
        );
    }

    // حفظ audit
    for e in audit.all() {
        storage.save_audit(e).unwrap();
    }

    println!("═══════════════════════════════════════════════");
    println!("  Week 46 Gate — GovernanceStorage Persistence");
    println!("═══════════════════════════════════════════════");
    println!("  Proposals on disk:  {}", storage.load_proposals().unwrap().len());
    println!("  Audit on disk:      {}", storage.load_audit().unwrap().len());
    println!("  Approved:           {}", store.approved_pending_application().len());
    println!("═══════════════════════════════════════════════");

    assert_eq!(storage.load_proposals().unwrap().len(), 3);
    assert_eq!(storage.load_audit().unwrap().len(), 6);

    println!("  ✅ Week 46 Gate: PASSED");
}
