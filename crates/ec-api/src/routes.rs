#![forbid(unsafe_code)]

//! Route definitions

use crate::auth::require_api_key;
use crate::handlers;
use crate::state::AppState;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, patch, post};
use axum::Router;

/// أقصى حجم مسموح به لجسم طلب التحليل: 2 ميجابايت
pub const MAX_ANALYZE_BODY_BYTES: usize = 2 * 1024 * 1024;

/// Build the API router
pub fn build_router(state: AppState) -> Router {
    let protected = Router::new()
        // Analysis — محمي بحد أقصى لحجم الجسم لمنع هجمات الاستنزاف
        .route(
            "/api/v1/analyze",
            post(handlers::analyze).layer(DefaultBodyLimit::max(MAX_ANALYZE_BODY_BYTES)),
        )
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ));

    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .merge(protected)
        .with_state(state)
}
