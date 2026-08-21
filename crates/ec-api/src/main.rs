#![forbid(unsafe_code)]

//! ec-server — HTTP server entry point

use ec_api::build_router;
use ec_api::state::AppState;

#[tokio::main]
async fn main() {
    let db = std::env::var("EC_DB").unwrap_or_else(|_| "ec.db".into());

    let api_key = std::env::var("EC_API_KEY").unwrap_or_else(|_| {
        eprintln!("❌ EC_API_KEY غير مُهيَّأ. الخادم يرفض البدء بلا مفتاح مصادقة (ADR-024 F2)");
        std::process::exit(1);
    });

    let bind_addr = std::env::var("EC_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());

    let state =
        AppState::open(std::path::Path::new(&db), api_key).expect("Failed to build app state");

    let app = build_router(state);

    println!("🚀 Engineering Civilization API: http://{}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind {}: {}", bind_addr, e));
    axum::serve(listener, app).await.expect("Server error");
}
