#!/usr/bin/env bash
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "  Week 20 — ec-memory: Causal Graph Foundation"
echo "═══════════════════════════════════════════════════════════════"

# 1. Build
echo "Building ec-memory..."
cargo build -p ec-memory

# 2. Test
echo "Running tests..."
cargo test -p ec-memory

# 3. Week 20 Gate
echo "Running Week 20 Gate..."
cargo test -p ec-memory week20_gate_complete -- --nocapture

# 4. All workspace tests
echo "Running all workspace tests..."
cargo test --workspace

# 5. Clippy
echo "Running clippy..."
cargo clippy -p ec-memory -- -D warnings

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  ✅ Week 20: COMPLETE"
echo "═══════════════════════════════════════════════════════════════"
