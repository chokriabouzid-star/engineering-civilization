# ADR-033: One adjudication pathway in which measured reality is a constitutional law

**Status:** Accepted (owner decision, option B, 2026-10-07)
**Date:** 2026-10-07
**Relates to:** ADR-005 (constitutional evaluation pipeline), ADR-013 and ADR-014 (fitness and reality are separate inputs), ADR-024 F1 (the hardened runner is the only execution path), ADR-026 (Simulated is a test double), ADR-032 (sandbox execution truth)

## Context

Read on `037c2c4` (tree `c6275e3c`), unless marked otherwise:

1. No production caller. `IntegrationPipeline`, `IterativePipeline` and `BayesianPipeline` are constructed only in tests. `ec-cli` does not depend on `ec-app`; `ec-api` depends on `ec-app` and `ec-constitutional` but its sources do not use them. `Constitution::new` appears in `crates/*/src` only inside `#[cfg(test)]` modules.
2. In `IntegrationPipeline::run` the epistemic state is built from static fitness (`build_epistemic_from_fitness`, line 188) and evaluated (line 192) before the sandbox runs (line 201). Reality reaches only the later verdict rule (`determine_verdict`).
3. Building the epistemic state from reality does not by itself close the gap. `build_epistemic_from_reality` sets `confidence = empirical_confidence`, and `RealityVector` computes `empirical_confidence = runs / (runs + 0.5)` from the run count only. `correctness` is written into `Evidence.reproducibility`, and `sample_size` is fixed at 1. `SecurityInvariant` reads only `epistemic.confidence`; no invariant reads `Evidence`.
   - Measured outside the repository with a probe crate (path dependencies on `037c2c4`, Rust 1.96.0, no Docker needed because the functions are pure), with a constitution of the four default invariants and identical fitness: a passing reality (`correctness=1`) and a failing reality (`correctness=0`) both give `confidence=0.857` at 3 runs (`0.667` at 1 run) and `is_valid=true` with 0 violations.
4. The sandbox executes one Rust source file with `fn main` (`rustc main.rs && ./program`, no cargo, no external crates, `--network none`). It cannot run a crate or `cargo test` today.
5. The pipeline's artifact hash is the first 8 bytes of SHA-256 as `u64`; `ec-memory` uses `DefaultHasher`, which its own comment marks as a development placeholder. `ConstitutionalEngine` caches evaluations keyed by (artifact hash, constitution version), without the evidence; `evaluate` writes the cache but does not read it.
6. `SandboxConfig::default()` selects `SandboxMode::Simulated` (a name oracle in debug builds, rejected in release builds). `SandboxExecutor::mode()` reports the configured mode.

## Decision

EC will have one real adjudication pathway, built in this order:

~~~text
artifact (one file) -> Docker execution (hardened runner) -> measured evidence bound to the artifact
-> epistemic state built from that evidence -> constitutional evaluation -> verdict
~~~

1. **Measured reality is a constitutional law.** A new non-compensable invariant (working name `MeasuredRealityInvariant`) rejects an artifact unless measured evidence shows that it ran successfully and reproducibly. The rejection reason names the measured fact (for example, the exit code and how many runs produced it). It never reuses the security confidence message.
2. **Evidence reaches the invariant through the epistemic state.** `Invariant::check` receives only `FitnessVector` and `EpistemicState`, so the measured facts (correctness, reproducibility, run count, provenance) must be carried by the epistemic state under fields named for what they hold. The exact representation is chosen in the first implementation step after reading every construction site (on `037c2c4`: no struct-literal construction of `EpistemicState`; 4 struct-literal constructions of `Evidence` outside `ec-epistemic`, all in `ec-constitutional` tests and benches). Existing constructors and existing test expectations must not change.
3. **Static fitness never produces confidence for this pathway.** Fitness still feeds the existing invariants. Absence of measurement means the verdict is `Unverified`, not `Accepted`.
4. **Docker is mandatory for authority.** If the executor mode is not `Docker`, the verdict is `Unverified` before any evaluation. Simulated results never contribute to authority.
5. **Evidence is bound to the artifact.** The evidence record carries the full SHA-256 of the artifact bytes, the sandbox image reference (`DEFAULT_IMAGE` digest), the mode, the run count and the per-run exit codes. A verdict is reported together with that hash. Any cache that could return a verdict must key on the evidence as well as the artifact.
6. **One entry point, shadow mode first.** The entry point is a new CLI command (working name `ec adjudge <file.rs>`). It prints its verdict and evidence and exits 0 regardless of the verdict until a later, separate decision introduces blocking. No API entry point in this decision.
7. **Gates are active.** The composition gate runs in the default workspace test suite without Docker, against the pure core: the same artifact and fitness with failing versus passing evidence must give different verdicts, and Simulated provenance must give `Unverified`. A mutation that makes the new invariant ignore its evidence must be killed. One end-to-end Docker test runs under `docker_tests`, which CI executes. No new `#[ignore]` gate.

## Delivery

Each step is its own PR, red before green, with negative controls:

1. Pure core: evidence representation, `MeasuredRealityInvariant`, the evidence-built epistemic state, active composition and mutation gates. No CLI, no Docker.
2. `ec adjudge` in shadow mode with Docker and `Unverified`.
3. End-to-end Docker test of the pathway.

## Not decided here

- Running tests contained in the artifact (`rustc --test`) or a whole crate. Both need a new runner method and the full Docker suites; they are separate decisions.
- An independent oracle. Running an agent's own tests shows self-consistency, not correctness: a test that asserts a wrong value (EC-ANA-01) still passes. Owner-supplied acceptance tests are a later decision.
- Blocking policy, API adjudication, reuse or removal of the three existing pipelines, `RealityFeedback` and `Constitution::learn`.
- The split between the CLI static analyzer and `analyze_code` (EC-ANA-02); this pathway does not change either.

## Consequences

- The constitution hears execution failure through a named law instead of a later rule, and the stated reason matches the measured fact.
- `ec-epistemic`, `ec-constitutional`, `ec-app` and `ec-cli` are touched across the three PRs; `ec-cli` gains a dependency on `ec-app` (added with `cargo add`).
- Until step 2 merges, no shipped entry point adjudicates; `ec check` and `ec analyze` remain static measurement.
