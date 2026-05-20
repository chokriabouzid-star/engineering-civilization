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

section "WORKSPACE Cargo.toml"
cat Cargo.toml

section "ALL CRATE Cargo.toml FILES"
find ./crates -name "Cargo.toml" | sort | while read f; do
    show_file "$f"
done

section "ec-fitness SOURCE"
find ./crates/ec-fitness/src -name "*.rs" | sort | while read f; do
    show_file "$f"
done

section "ec-epistemic SOURCE"
find ./crates/ec-epistemic/src -name "*.rs" | sort | while read f; do
    show_file "$f"
done
find ./crates/ec-epistemic/tests -name "*.rs" 2>/dev/null | sort | while read f; do
    show_file "$f"
done

section "ec-constitutional SOURCE"
find ./crates/ec-constitutional/src -name "*.rs" | sort | while read f; do
    show_file "$f"
done

section "ec-constitutional TESTS"
find ./crates/ec-constitutional/tests -name "*.rs" | sort | while read f; do
    show_file "$f"
done

section "ec-analysis SOURCE"
find ./crates/ec-analysis/src -name "*.rs" | sort | while read f; do
    show_file "$f"
done
find ./crates/ec-analysis/tests -name "*.rs" 2>/dev/null | sort | while read f; do
    show_file "$f"
done

section "ec-app SOURCE"
find ./crates/ec-app/src -name "*.rs" | sort | while read f; do
    show_file "$f"
done
find ./crates/ec-app/tests -name "*.rs" 2>/dev/null | sort | while read f; do
    show_file "$f"
done

section "CARGO TEST --workspace (summary)"
cargo test --workspace 2>&1 | grep -E "^(test |running |FAILED|ok|error|warning)" | tail -60 || true

section "CARGO TEST --workspace (full last 40 lines)"
cargo test --workspace 2>&1 | tail -40 || true

section "CARGO CLIPPY"
cargo clippy --workspace 2>&1 | tail -20 || true

section "FILE TREE (no target)"
find ./crates -not -path "*/target/*" -not -path "*/.git/*" | sort

