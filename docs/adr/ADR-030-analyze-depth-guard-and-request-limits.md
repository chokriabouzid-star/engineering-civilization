# ADR-030: Analyze Entry Point Hardening (F1 Stack-Overflow & Request Body Limits)

## Status
Accepted

## Context
Sending deeply nested source code payloads (e.g. 500+ nested braces, 1000+ parentheses, or repetitive unary operators) to `analyze_code_full` caused an uncatchable runtime crash (`fatal runtime error: stack overflow`, exit code 134) in `syn::parse_str`. This directly impacted `ec analyze`, `ec check`, and `POST /api/v1/analyze`.

## Decision
1. Implement a non-allocating, $O(N)$ lexical pre-parse depth scanner (`check_nesting_safety`) in `ec-analysis`.
2. Enforce `MAX_DELIMITER_DEPTH = 128` and `MAX_UNARY_CHAIN = 64`. Any payload exceeding these thresholds returns `AnalysisReport::unparseable(...)` with a deterministic safety warning.
3. Explicitly pin `DefaultBodyLimit::max(2 * 1024 * 1024)` on `POST /api/v1/analyze` in `ec-api`.

## Verification & Automated Gate
- `crates/ec-analysis/tests/f1_depth_guard_gate.rs` tests nested braces, parentheses, references, and valid code.
- Zero exit 134 across all deep-nesting test cases.
