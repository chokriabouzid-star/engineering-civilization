#![forbid(unsafe_code)]

//! ec-api — REST API (thin transport layer)
//!
//! Phase 6 (Weeks 47-50):
//! - 11 endpoints: analyze, memory, governance, health
//! - No business logic in handlers
//! - DTOs live here only — never in kernels

pub mod handlers;
pub mod routes;
pub mod state;

pub use routes::build_router;
pub use state::AppState;
