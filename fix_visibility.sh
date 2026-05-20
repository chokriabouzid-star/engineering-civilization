#!/usr/bin/env bash
set -euo pipefail

# تعديل node.rs: جعل الـ constructors عامة
sed -i 's/pub(crate) fn new(/pub fn new(/' crates/ec-memory/src/node.rs
sed -i 's/pub(crate) fn from_evaluation(/pub fn from_evaluation(/' crates/ec-memory/src/node.rs
sed -i 's/pub(crate) fn add_alternative(/pub fn add_alternative(/' crates/ec-memory/src/node.rs

# إزالة import غير مستخدم من tests
sed -i '/use crate::node::SandboxOutcome;/d' crates/ec-memory/tests/week20_gate.rs

echo "✅ Visibility fixed"
