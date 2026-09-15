#![forbid(unsafe_code)]

//! بوابة الدوام — الاقتراحات وسجل التدقيق يجب أن تنجو من إعادة التشغيل.
//! هذا الاختبار يفشل قبل إصلاح C2 (كان سجل التدقيق in-memory فقط).

use axum::extract::State;
use axum::Json;
use ec_api::handlers::{submit_proposal, ProposalRequest};
use ec_api::state::AppState;

fn sample_req() -> ProposalRequest {
    ProposalRequest {
        dimension: "Security".into(),
        current_value: 0.70,
        proposed_value: 0.80,
        justification: "persistence gate".into(),
        proposed_by: "gate".into(),
    }
}

#[tokio::test]
async fn restart_preserves_proposals_and_audit() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("persistence_gate.db");

    // الجولة 1: submit عبر الـhandler الحقيقي (يكتب في التخزين الدائم)
    let s1 = AppState::open(&db, "gate-key").unwrap();
    match submit_proposal(State(s1.clone()), Json(sample_req())).await {
        Ok(_) => {}
        Err((status, body)) => panic!("submit_proposal failed: {} {:?}", status, body),
    }

    // الجولة 2: حالة جديدة من نفس الـDB (محاكاة إعادة تشغيل)
    let s2 = AppState::open(&db, "gate-key").unwrap();
    {
        let proposals = s2.proposals.lock().await;
        assert!(
            !proposals.all().is_empty(),
            "proposals must survive restart"
        );
    }
    {
        let audit = s2.audit.lock().await;
        assert!(
            !audit.all().is_empty(),
            "audit entries must survive restart"
        );
    }
}
