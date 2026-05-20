#!/usr/bin/env bash
set -euo pipefail

# إنشاء ec-memory crate
cd crates
cargo new --lib ec-memory
cd ec-memory

# تحديث Cargo.toml
cat > Cargo.toml << 'TOML'
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

[dev-dependencies]
TOML

# إنشاء ملفات المصدر
mkdir -p src
mkdir -p tests

echo "ec-memory crate created successfully"
