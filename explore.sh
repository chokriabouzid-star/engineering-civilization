#!/usr/bin/env bash
set -euo pipefail

SEP="════════════════════════════════════════════════════════════════"

section() {
    echo ""
    echo "$SEP"
    echo "  $1"
    echo "$SEP"
}

show_file() {
    echo ""
    echo "--- $1 ---"
    cat "$1"
}

# 1. WORKSPACE
section "WORKSPACE — Cargo.toml"
cat Cargo.toml

# 2. ALL TOML FILES
section "ALL CRATE Cargo.toml FILES"
find ./crates -name "Cargo.toml" | sort | while read f; do
    show_file "$f"
done

# 3. ALL SOURCE FILES
section "ALL SOURCE FILES (*.rs) — excluding target"
find ./crates -type f -name "*.rs" | grep -v target | sort | while read f; do
    show_file "$f"
done

# 4. CARGO TEST
section "CARGO TEST --workspace"
cargo test --workspace 2>&1 || true

# 5. CARGO CLIPPY
section "CARGO CLIPPY --workspace"
cargo clippy --workspace 2>&1 || true

# 6. FILE TREE
section "FILE TREE"
find ./crates -not -path "*/target/*" | sort

echo ""
echo "$SEP"
echo "  DONE"
echo "$SEP"
