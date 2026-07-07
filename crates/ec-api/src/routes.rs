#![forbid(unsafe_code)]

//! Route definitions

use crate::handlers;
use crate::state::AppState;
use axum::routing::{get, patch, post};
use axum::Router;

/// Build the API router
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Analysis
        .route("/api/v1/analyze", post(handlers::analyze))
        // Memory
        .route("/api/v1/memory/nodes", get(handlers::list_nodes))
        .route("/api/v1/memory/drift", get(handlers::get_drift))
        .route("/api/v1/memory/similar", get(handlers::find_similar))
        // Governance
        .route(
            "/api/v1/governance/proposals",
            get(handlers::list_proposals).post(handlers::submit_proposal),
        )
        .route(
            "/api/v1/governance/proposals/:id/approve",
            patch(handlers::approve_proposal),
        )
        .route(
            "/api/v1/governance/proposals/:id/reject",
            patch(handlers::reject_proposal),
        )
        .route("/api/v1/governance/audit", get(handlers::get_audit))
        // Health
        .route("/api/v1/health", get(handlers::health))
        .with_state(state)
}
