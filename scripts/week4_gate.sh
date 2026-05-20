#!/usr/bin/env bash

set -euo pipefail

echo
echo "=================================================="
echo " Engineering Civilization — Week 4 Gate"
echo "=================================================="
echo

ROOT_DIR="$(pwd)"

echo "[1/10] Workspace Root"
echo "--------------------------------------------------"
pwd
echo

echo "[2/10] Rust Version"
echo "--------------------------------------------------"
rustc --version
cargo --version
echo

echo "[3/10] Workspace Structure"
echo "--------------------------------------------------"
find crates -maxdepth 2 -type f \( -name "*.rs" -o -name "Cargo.toml" \) | sort
echo

echo "[4/10] Cargo Workspace Metadata"
echo "--------------------------------------------------"
cargo metadata --no-deps --format-version 1 > /tmp/ec_metadata.json
echo "Workspace metadata generated."
echo

echo "[5/10] Formatting Check"
echo "--------------------------------------------------"
cargo fmt --all --check
echo "Formatting OK"
echo

echo "[6/10] Clippy Validation"
echo "--------------------------------------------------"
cargo clippy --workspace -- -D warnings
echo "Clippy OK"
echo

echo "[7/10] Workspace Compilation"
echo "--------------------------------------------------"
cargo check --workspace
echo "Compilation OK"
echo

echo "[8/10] Workspace Tests"
echo "--------------------------------------------------"
cargo test --workspace
echo "Tests OK"
echo

echo "[9/10] Week 4 Integration Tests"
echo "--------------------------------------------------"

if [ -f tests/week4_integration.rs ]; then
    cargo test --test week4_integration -- --nocapture
else
    echo "WARNING: tests/week4_integration.rs missing"
fi

echo

if [ -f tests/phase_0_gate.rs ]; then
    cargo test --test phase_0_gate -- --nocapture
else
    echo "WARNING: tests/phase_0_gate.rs missing"
fi

echo
echo "Integration tests completed."
echo

echo "[10/10] Architectural Consistency Checks"
echo "--------------------------------------------------"

echo
echo "Searching for deprecated ec-core references..."
if grep -R "ec_core\|ec-core" crates tests >/dev/null 2>&1; then
    echo "ERROR: deprecated ec-core references detected"
    grep -R "ec_core\|ec-core" crates tests || true
    exit 1
else
    echo "OK: no ec-core references"
fi

echo
echo "Checking Arc/Box invariant consistency..."

ARC_COUNT=$(grep -R "Arc<dyn Invariant>" crates tests 2>/dev/null | wc -l || true)
BOX_COUNT=$(grep -R "Box<dyn Invariant>" crates tests 2>/dev/null | wc -l || true)

echo "Arc invariant usages : $ARC_COUNT"
echo "Box invariant usages : $BOX_COUNT"

if [ "$BOX_COUNT" -gt 0 ]; then
    echo
    echo "WARNING:"
    echo "Box<dyn Invariant> still exists."
    echo "Architectural convergence incomplete."
else
    echo
    echo "OK: invariant ownership unified"
fi

echo
echo "Checking Pareto ownership..."

if grep -R "enum ParetoOrdering" crates/ec-constitutional >/dev/null 2>&1; then
    echo "WARNING: ParetoOrdering still inside ec-constitutional"
    echo "Recommendation: move Pareto semantics into ec-fitness"
else
    echo "OK: Pareto semantics placement acceptable"
fi

echo
echo "=================================================="
echo " WEEK 4 GATE STATUS"
echo "=================================================="
echo
echo "If all checks passed:"
echo
echo "  Phase 0 constitutional kernel is stable."
echo
echo "Ready for:"
echo
echo "  - ADR-004"
echo "  - ADR-005"
echo "  - Final Week 4 integration"
echo "  - Phase 0 Gate validation"
echo
echo "==================================================="
