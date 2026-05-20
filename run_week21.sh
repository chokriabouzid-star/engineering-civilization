#!/usr/bin/env bash
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "  Week 21 — ec-codegen: Template-Based Code Generation"
echo "═══════════════════════════════════════════════════════════════"

cargo build -p ec-codegen
echo "✅ Build passed"

cargo test -p ec-codegen
echo "✅ All ec-codegen tests passed"

cargo test -p ec-codegen week21_gate_complete -- --nocapture

cargo test --workspace 2>&1 | grep -E "^(test result|FAILED|error)" | tail -20
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  ✅ Week 21: COMPLETE"
echo "═══════════════════════════════════════════════════════════════"
