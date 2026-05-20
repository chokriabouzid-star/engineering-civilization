#!/usr/bin/env bash

set -e

echo "========================================="
echo "WEEK 3 VALIDATION"
echo "========================================="

echo
echo "[1/5] cargo test"
cargo test -p ec-constitutional

echo
echo "[2/5] cargo clippy"
cargo clippy -p ec-constitutional -- -D warnings

echo
echo "[3/5] checking invariant files"

test -f crates/ec-constitutional/src/invariant.rs
test -f crates/ec-constitutional/src/security.rs
test -f crates/ec-constitutional/src/coverage.rs
test -f crates/ec-constitutional/src/reversibility.rs
test -f crates/ec-constitutional/src/type_safety.rs
test -f crates/ec-constitutional/src/verdict.rs

echo "✓ invariant files exist"

echo
echo "[4/5] checking ADR"

test -f docs/adr/003-constitutional-architecture.md

echo "✓ ADR exists"

echo
echo "[5/5] final"

echo
echo "========================================="
echo "✓ WEEK 3 GATE PASSED"
echo "Constitutional layer validated"
echo "Ready for Week 4"
echo "========================================="
