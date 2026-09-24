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

---

## Addendum (2026-09-23): the lexical guard alone does NOT close F1

### Measured correction
The claim above — *"Zero exit 134 across all deep-nesting test cases"* — was
scoped to the delimiter and unary families only. Three reproduction gates added
after that commit failed on the **unmodified** binaries:

| Entry point | Observed |
|---|---|
| `POST /api/v1/analyze` (real `ec-server` child) | `exit=unix_wait_status(134)`, then `Connection refused (os error 111)` on the next `/health` |
| `ec analyze --json` | `exit=unix_wait_status(134)`, empty stdout |
| `ec check --json` | `exit=unix_wait_status(134)`, empty stdout |

### Root cause
`check_nesting_safety` counts `{ ( [` and the unary run `& * !`. The payload
`fn f() { let x: Vec<Vec<...>> = todo!(); }` with 600 nested generic arguments
touches **none** of those counters: brace depth is 1, paren depth is 2, unary
run is 1. The guard passes, and recursive descent over angle-bracketed generic
types overflows the stack inside the parse/visit phase.

### Why the guard was not simply extended to `<` and `>`
`<` and `>` are also comparison and shift operators. A byte-level counter cannot
distinguish `Vec<T>` from `a < b` or `x >> 2` without real tokenisation, so
extending it would reject valid code — the same false-positive defect already
recorded against `architectural_stability`.

### Why in-process mitigation is impossible
- `catch_unwind` cannot catch a stack overflow; the runtime raises `SIGABRT`.
- A guard thread does not help: overflow in any thread aborts the whole process.
- `fork(2)` requires `unsafe`, forbidden crate-wide.

### Decision (supersedes the scope of Decision 1 above, retains its text)
4. Analysis for **all three entry points** runs in a separate process
   (`ec_analysis::isolation`). The parent writes source to the child's stdin and
   reads `EC-ANALYZE-WORKER-V1` + one JSON line from stdout.
5. A dead worker is an **infrastructure failure, not an evaluation result**:
   - HTTP returns `502` (crash / protocol violation), `504` (timeout) or `500`
     (worker missing / spawn failure) with an `error` body; the server stays up.
   - `ec analyze` exits `2` with a diagnostic on stderr.
   - `ec check` records an `analysis_failed` violation and counts the file as
     failed (fail-closed), then exits non-zero.
6. A wall-clock budget (`WORKER_TIMEOUT`) kills and reaps the child.
7. `check_nesting_safety` is **retained** as a cheap first filter; it is no
   longer claimed to be sufficient on its own.
8. The worker starts with a cleared environment (`env_clear()`): it does not
   inherit `EC_API_KEY`, `EC_DB`, or any other parent secret.

### Verification
- `crates/ec-api/tests/f1_server_survival.rs` — real `ec-server` child process;
  asserts `502` for the hostile payload **and** a live `200 /health` afterwards.
- `crates/ec-cli/tests/f1_cli_survival.rs` — asserts a clean non-zero exit
  (`status.code().is_some()`), explicitly rejecting signal termination.
- `crates/ec-analysis/src/isolation.rs` — unit tests: foreign stdout rejected,
  hung worker killed at its deadline, non-zero exit and signal death reported
  as `Crashed`, spawn failure, cleared environment.
- `crates/ec-api/src/handlers.rs` — `f1_status_tests` pins the HTTP mapping.

### Known limits (not claimed closed)
- Process isolation protects the **parent's survival**. It does not by itself
  bound memory use or the number of concurrent workers.
- No per-request concurrency cap on `/analyze` yet.
- `locate_worker` has an in-process test fallback that searches `target/*` for
  a sibling `ec-server`/`ec` binary; production binaries use the self-hook.
- `ec check` spawns one worker per file, sequentially. Files that cannot be
  read as UTF-8 are still skipped silently (pre-existing, outside F1 scope).
