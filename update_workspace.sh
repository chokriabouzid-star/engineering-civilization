#!/usr/bin/env bash
set -euo pipefail

# إضافة ec-memory للـ workspace
cd ~/projects/engineering-civilization

# backup
cp Cargo.toml Cargo.toml.backup

# إضافة ec-memory
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
TOML

echo "Workspace updated successfully"
