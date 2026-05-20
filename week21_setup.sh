#!/usr/bin/env bash
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "  Week 21 — ec-codegen: Template-Based Code Generation"
echo "═══════════════════════════════════════════════════════════════"

# إنشاء crate
cd crates
cargo new --lib ec-codegen 2>/dev/null || echo "ec-codegen exists, continuing..."
cd ..

# Cargo.toml
cat > crates/ec-codegen/Cargo.toml << 'TOML'
[package]
name = "ec-codegen"
version = "0.1.0"
edition = "2021"

[dependencies]
ec-fitness   = { path = "../ec-fitness" }
ec-analysis  = { path = "../ec-analysis" }
ec-memory    = { path = "../ec-memory" }
uuid         = { workspace = true }
anyhow       = { workspace = true }
serde        = { workspace = true }

[dev-dependencies]
TOML

# تحديث workspace
cat > Cargo.toml << 'TOML'
[workspace]
members = [
    "crates/ec-fitness",
    "crates/ec-epistemic",
    "crates/ec-constitutional",
    "crates/ec-sandbox",
    "crates/ec-app",
    "crates/ec-analysis",
    "crates/ec-memory",
    "crates/ec-codegen",
]
resolver = "2"

[workspace.dependencies]
serde        = { version = "1.0", features = ["derive"] }
proptest     = "1.6"
thiserror    = "2.0"
chrono       = { version = "0.4", features = ["serde"] }
tokio        = { version = "1.35", features = ["full"] }
async-trait  = "0.1"
dashmap      = "5.5"
parking_lot  = "0.12"
tracing      = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
anyhow       = "1.0"
uuid         = { version = "1.6", features = ["v4", "serde"] }
criterion    = { version = "0.5", features = ["html_reports"] }
toml         = "0.8"
thiserror    = "2.0"
TOML

echo "✅ Setup complete"
