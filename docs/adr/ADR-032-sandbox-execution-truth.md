# ADR-032: Sandbox execution truth: program exit codes and the compile/run separator

**Status:** Accepted
**Date:** 2026-10-02
**Relates to:** ADR-024 section F1 (the hardened runner is the only execution path), ADR-028 (the parity gate relies on the `---OUTPUT---` separator), ADR-031

## Context

An independent audit reported that the Docker execution path could mark a failed program as successful. The claim was reproduced before any change, on `b8b9a1e` (parent `505d586`), with direct container probes and a two-layer gate (`crates/ec-sandbox/tests/sandbox_truth_gate.rs`):

- Raw layer (`HardenedDockerRunner::default_hardened()`, seccomp on): the runner was honest. `panic!` returned 101, `exit(3)` returned 3, an OOM kill returned a non-zero code (137 in the direct probe), and `rustc` diagnostics for source that contains the separator text echoed that text.
- Executor layer (`SandboxExecutor`, `SandboxMode::Docker`, one run, 512 MB): all four cases (panic, `exit(3)`, OOM, uncompilable source containing `---OUTPUT---`) were reported with `success=true`, `correctness=1.0` and `violations=[]`.

Cause, read from source and confirmed by the red gate:

1. SBX-01: `hardened.rs` ran `rustc ... 2>&1 && echo '---OUTPUT---' && /workspace/program`, so the separator is printed before the program runs. `compiler.rs` then forced `RunOutput.exit_code` to 0 whenever the separator was in stdout. `metrics.rs` derives `all_succeeded` from `exit_code == 0`, and `executor.rs` derives `success` and `correctness` from it.
2. SBX-02: `rustc` output went to stdout and a diagnostic echoes the offending source line, so source that fails to compile but contains the separator text passed the same check.

## Decision

1. `hardened.rs`: compile first, with `rustc` stdout and stderr redirected to files under `/tmp`. If `rustc` fails, its output goes to the container stderr, the separator is NOT printed, and the script exits with `rustc`'s status. If it succeeds, compiler warnings go to stderr, then the separator is printed, then the program runs and its status is the container status. The separator text is unchanged.
2. `compiler.rs`: a missing separator in stdout means `CompilationResult::Failed`; with the separator present, the program's real exit code is kept in `RunOutput.exit_code`. The same check applies to every repeated run.
3. `week14_gate.rs`: six executor-level escape-vector/20-run tests are ignored when `docker_tests` is enabled, with an explicit SBX-03 reason; their bodies remain in place. The `is_secure()` assertions are removed from the two active Week 14 Docker-functional checks.

Not changed: the seccomp profile and allowlist, the `ExecutionResult` fields (no `stdout` field is added), and the mapping to `SecurityViolation` (a non-zero exit is not turned into a violation).

## Evidence

Measured on the working tree based on `b8b9a1e`, before the commit of this change. The numbers are re-measured on the merge commit in the next PROJECT-REFERENCE refresh.

| Item | Before | After |
|---|---|---|
| `sandbox_truth_gate` raw layer (6 tests) | 6 pass | 6 pass |
| `sandbox_truth_gate` executor layer (4 tests) | 4 fail (`success=true`, `correctness=1.0`) | 4 pass |
| `seccomp_parity_gate` | pass (505d586 suite) | pass, standalone and in the suite |
| `ec-sandbox` Docker suite | 152 / 0 / 1 (505d586) | 156 / 0 / 7 (six Week 14 executor-level checks ignored while SBX-03 remains open) |
| `week14_gate` Docker target | 15 passed / 0 failed / 0 ignored before classification | 9 passed / 0 failed / 6 ignored (six explicit SBX-03 checks) |
| `ec-app` Docker suite | 108 / 0 / 1 (505d586) | 108 / 0 / 1 |
| workspace without Docker | 721 / 0 / 51 (505d586) | 721 / 0 / 61 |

The red run is the first row pair: 6 passed and 4 failed on `b8b9a1e`; the green run is 10 of 10.

## How to verify automatically

~~~bash
cargo test -p ec-sandbox --locked --features docker_tests \
  --test sandbox_truth_gate -- --test-threads=1
~~~

All 10 tests must pass. The four `executor_must_not_report_*` tests assert `success == false` and `correctness < 1.0` (or a `Compilation failed` message for the marker case) for programs that exit non-zero or fail to compile.

## Consequences

- Program exit codes are reported truthfully, and source text can no longer forge a compilation success.
- Docker-mode pipelines in `ec-app` now reject (`RejectedByReality`) programs that exit non-zero; earlier they could be accepted. The `ec-app` Docker suite is unchanged (108 / 0 / 1).
- `DockerOutput.stdout` now holds the separator and the program output only; `rustc` warnings and errors are on stderr.
- **SBX-03 stays open.** The Docker path still constructs an empty `ExecutionResult.violations` vector; this change does not implement violation detection. The six Week 14 executor-level escape tests are explicitly ignored under `docker_tests`, and the active functional checks no longer use `is_secure()` as evidence. The separately passing `seccomp_parity_gate` observes five raw-runner vectors and requires their `BLOCKED`/`CONTAINED` markers; that is bounded evidence, not a general proof of containment. The contents of similar slow gates in `week16_gate` and `week18_phase2_gate` were not reviewed and are not covered by this statement.
- Residual (from reading `executor.rs`, not asserted by the gate): a program that compiles and exits non-zero now gives `success=false` with `error_message=None`. Surfacing the exit code needs an `ExecutionResult` change, deferred with the SBX-03 design.
