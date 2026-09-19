
ADR-027: Kernel Purity Gate (Structural, Deterministic)
Date: 2026-09-18
Status: Accepted
Relates to: ADR-023 (Kernel Purity)

Context
ADR-023 mandates that the constitutional kernel (ec-constitutional) remains
free of async runtime dependencies (tokio, async-trait). This invariant is
architectural: it constrains the dependency graph, not a FitnessVector.

Invariant::check in EC receives only FitnessVector and EpistemicState;
it never sees the dependency tree. Therefore kernel purity cannot be expressed
as a constitutional Invariant. It must be a standalone test that inspects the
real link graph.

Decision
Add crates/ec-constitutional/tests/kernel_purity.rs, which:

Runs cargo tree -p ec-constitutional -e normal (normal edges only:
respects features/target, excludes dev/build dependencies — no false
positives from test-only async deps).
Fails if any of tokio, tokio-util, async-trait appears on the runtime
link tree.
Is fail-closed: if cargo is unavailable, the test fails rather than
silently skipping.
Why not an Invariant
Invariant inputs cannot express dependency-graph facts.
A structural check is deterministic and requires no calibration, unlike
semantic quality metrics. It is correct from day one and does not inherit
clippy-scale maintenance cost.
Verification (negative control)
Empirically validated: on a clean tree the test passes; after adding
tokio as a normal dependency of ec-constitutional, the test fails
(exit 101). This proves the gate can fail, not merely pass.

Risks
cargo absent in the environment → fail-closed (documented).
Reliance on cargo tree output format for {p} — stable and preferred over
parsing internal metadata ids, which changed recently.
Consequences
First EC gate proven by a negative control.
Establishes the project rule: no gate is trusted until observed failing in
the case where it must fail.
