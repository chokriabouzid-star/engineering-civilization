#![forbid(unsafe_code)]

//! HTTP handlers — thin adapters, no business logic

use axum::extract::{Path as AxumPath, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use ec_analysis::analyze_code_full;
use ec_governance::audit::GovernanceEvent;
use ec_governance::proposal::{
    ConstitutionalProposal, ProposalOrigin, ProposedChange, ThresholdDirection,
};

// ─── DTOs (ec-api only — never in kernels) ──────────────────────────

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub security: f64,
    pub test_coverage: f64,
    pub maintainability: f64,
    pub performance: f64,
    pub architectural_stability: f64,
    pub reversibility: f64,
    pub confidence_overall: f64,
    pub parse_successful: bool,
    pub warnings_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ProposalRequest {
    pub dimension: String,
    pub current_value: f64,
    pub proposed_value: f64,
    pub justification: String,
    pub proposed_by: String,
}

#[derive(Debug, Serialize)]
pub struct ProposalResponse {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub by: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub memory_nodes: usize,
    pub pending_proposals: usize,
    pub version: String,
}

// ─── Handlers ───────────────────────────────────────────────────────

/// POST /api/v1/analyze — analyze code
pub async fn analyze(Json(req): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let report = analyze_code_full(&req.code);
    Json(AnalyzeResponse {
        security: report.fitness.security,
        test_coverage: report.fitness.test_coverage,
        maintainability: report.fitness.maintainability,
        performance: report.fitness.performance,
        architectural_stability: report.fitness.architectural_stability,
        reversibility: report.fitness.reversibility,
        confidence_overall: report.confidence.overall(),
        parse_successful: report.parse_successful,
        warnings_count: report.warnings.len(),
    })
}

/// POST /api/v1/governance/proposals — submit proposal
pub async fn submit_proposal(
    State(state): State<AppState>,
    Json(req): Json<ProposalRequest>,
) -> Json<ProposalResponse> {
    let direction = if req.proposed_value > req.current_value {
        ThresholdDirection::Tighten
    } else {
        ThresholdDirection::Loosen
    };

    let proposal = ConstitutionalProposal::new(
        ProposalOrigin::Human {
            name: req.proposed_by.clone(),
        },
        ProposedChange::AdjustThreshold {
            dimension: req.dimension.clone(),
            current: req.current_value,
            proposed: req.proposed_value,
            direction,
        },
        req.justification.clone(),
    );

    let id = proposal.id;
    let mut proposals = state.proposals.lock().await;
    proposals.submit(proposal.clone());
    let _ = state.gov_storage.save_proposal(&proposal);

    let mut audit = state.audit.lock().await;
    audit.record(
        GovernanceEvent::ProposalCreated {
            id,
            change_type: "AdjustThreshold".into(),
        },
        &req.proposed_by,
        "",
    );

    Json(ProposalResponse {
        id: id.to_string(),
        status: "Pending".into(),
    })
}

/// GET /api/v1/governance/proposals — list proposals
pub async fn list_proposals(State(state): State<AppState>) -> Json<serde_json::Value> {
    let proposals = state.proposals.lock().await;
    let summaries: Vec<serde_json::Value> = proposals
        .all()
        .iter()
        .map(|p| {
            let status_str = format!("{:?}", p.status)
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            serde_json::json!({
                "id": p.id.to_string(),
                "status": status_str,
                "justification": p.justification,
                "created_at": p.created_at.to_rfc3339(),
            })
        })
        .collect();
    Json(serde_json::json!(summaries))
}

/// PATCH /api/v1/governance/proposals/:id/approve
pub async fn approve_proposal(
    State(state): State<AppState>,
    AxumPath(id_str): AxumPath<String>,
    Json(req): Json<ApproveRequest>,
) -> Json<serde_json::Value> {
    let id = match Uuid::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({ "error": "invalid id" })),
    };

    let mut proposals = state.proposals.lock().await;
    match proposals.approve(id, &req.by, &req.note) {
        Ok(()) => {
            if let Some(p) = proposals.find(id) {
                let _ = state.gov_storage.save_proposal(p);
            }
            drop(proposals);

            let mut audit = state.audit.lock().await;
            audit.record(
                GovernanceEvent::ProposalApproved {
                    id,
                    by: req.by.clone(),
                },
                &req.by,
                "",
            );

            Json(serde_json::json!({ "status": "approved" }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// PATCH /api/v1/governance/proposals/:id/reject
pub async fn reject_proposal(
    State(state): State<AppState>,
    AxumPath(id_str): AxumPath<String>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let id = match Uuid::parse_str(&id_str) {
        Ok(id) => id,
        Err(_) => return Json(serde_json::json!({ "error": "invalid id" })),
    };
    let reason = body["reason"].as_str().unwrap_or("no reason given");
    let mut proposals = state.proposals.lock().await;
    match proposals.reject(id, reason) {
        Ok(()) => {
            if let Some(p) = proposals.find(id) {
                let _ = state.gov_storage.save_proposal(p);
            }
            drop(proposals);

            let mut audit = state.audit.lock().await;
            audit.record(
                GovernanceEvent::ProposalRejected {
                    id,
                    reason: reason.into(),
                },
                "api",
                "",
            );

            Json(serde_json::json!({ "status": "rejected" }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

/// GET /api/v1/governance/audit — audit log
pub async fn get_audit(State(state): State<AppState>) -> Json<serde_json::Value> {
    let audit = state.audit.lock().await;
    let entries: Vec<serde_json::Value> = audit
        .last_n(50)
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id.to_string(),
                "timestamp": e.timestamp.to_rfc3339(),
                "event": format!("{:?}", e.event),
                "actor": e.actor,
            })
        })
        .collect();
    Json(serde_json::json!(entries))
}

/// GET /api/v1/health — health check
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let memory_nodes = state.memory.lock().await.len();
    let pending = state.proposals.lock().await.pending().len();
    Json(HealthResponse {
        status: "ok".into(),
        memory_nodes,
        pending_proposals: pending,
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

/// GET /api/v1/memory/nodes — list memory nodes
pub async fn list_nodes(State(state): State<AppState>) -> Json<serde_json::Value> {
    let memory = state.memory.lock().await;
    let nodes: Vec<serde_json::Value> = memory
        .all()
        .iter()
        .take(100)
        .map(|n| {
            serde_json::json!({
                "id": n.id.to_string(),
                "artifact_id": n.artifact_id,
                "constitutional_valid": n.constitutional_valid,
                "security": n.fitness.security,
            })
        })
        .collect();
    Json(serde_json::json!(nodes))
}

/// GET /api/v1/memory/drift — drift report
pub async fn get_drift(State(state): State<AppState>) -> Json<serde_json::Value> {
    let memory = state.memory.lock().await;
    if memory.len() < 20 {
        return Json(serde_json::json!({
            "status": "insufficient_data",
            "nodes": memory.len(),
            "required": 20
        }));
    }
    let analyzer = ec_memory::drift::HistoricalDriftAnalyzer::new(&memory, 10, 10);
    let report = analyzer.analyze();
    Json(serde_json::json!({
        "angle": report.drift_angle_degrees,
        "classification": format!("{:?}", report.classification),
        "action": format!("{:?}", report.recommended_action),
        "requires_action": report.requires_action(),
    }))
}

/// GET /api/v1/memory/similar — find similar decisions
pub async fn find_similar(State(state): State<AppState>) -> Json<serde_json::Value> {
    let memory = state.memory.lock().await;
    let q = ec_memory::MemoryQuery::new(&memory);
    let target = ec_fitness::fitness::FitnessVector::default();
    let similar = q.find_similar(&target, 5);
    let results: Vec<serde_json::Value> = similar
        .iter()
        .map(|s| {
            serde_json::json!({
                "node_id": s.node_id.to_string(),
                "similarity": s.similarity,
            })
        })
        .collect();
    Json(serde_json::json!(results))
}
