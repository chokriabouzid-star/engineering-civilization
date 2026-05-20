#!/usr/bin/env bash
set -euo pipefail

# إصلاح Cargo.toml
cat > crates/ec-memory/Cargo.toml << 'TOML'
[package]
name = "ec-memory"
version = "0.1.0"
edition = "2021"

[dependencies]
ec-fitness = { path = "../ec-fitness" }
ec-epistemic = { path = "../ec-epistemic" }
ec-constitutional = { path = "../ec-constitutional" }
serde = { workspace = true }
chrono = { workspace = true }
uuid = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
TOML

echo "✅ Cargo.toml fixed"

# إصلاح graph.rs (إزالة import غير مستخدم)
sed -i 's/use crate::node::{DecisionNode, RejectedAlternative};/use crate::node::DecisionNode;/' crates/ec-memory/src/graph.rs

echo "✅ Unused import fixed"
