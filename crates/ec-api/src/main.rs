#![forbid(unsafe_code)]

//! ec-server — HTTP server entry point

use ec_api::build_router;
use ec_api::state::AppState;

#[tokio::main]
async fn main() {
    let db = std::env::var("EC_DB").unwrap_or_else(|_| "ec.db".into());
    let state = AppState::open(std::path::Path::new(&db))
        .expect("Failed to build app state");

    let app = build_router(state);

    println!("🚀 Engineering Civilization API: http://0.0.0.0:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind port 8080");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
