# ADR-004: Integration Architecture

## Status
Accepted (2024-05-16)

## Context
Week 4 required integrating ec-fitness, ec-epistemic, and ec-constitutional
into a unified constitutional evaluation pipeline.

## Decision

### Canonical Architecture
The system uses three crates in strict dependency order:
ec-fitness → ec-epistemic → ec-constitutional

### Constitution as Orchestrator
`Constitution` struct serves as the orchestration layer:
- Owns Vec<Arc<dyn Invariant>>
- Owns CatastropheThresholds
- Produces ConstitutionalEvaluation via evaluate()
- Delegates Pareto comparison to ec-fitness

### Arc Unification
All invariants use Arc<dyn Invariant> exclusively.
Box drift eliminated to maintain semantic clarity:
invariants are immutable, shared constitutional laws.

### Pareto Ownership
ParetoOrdering and pareto_compare() live in ec-fitness,
not ec-constitutional. Mathematics stays in the fitness kernel.

## Consequences
- Single direction dependency flow
- No circular dependencies
- Clear separation: mathematics vs governance
- ConstitutionalEvaluation is the canonical evaluation boundary


# ADR-005: Constitutional Evaluation Pipeline
cat > docs/adr/ADR-005-constitutional-evaluation-pipeline.md << 'EOF'
# ADR-005: Constitutional Evaluation Pipeline

## Status
Accepted (2024-05-16)

## Context
Need a unified pipeline that combines invariant checking,
catastrophic detection, and epistemic propagation.

## Decision

### Pipeline Order
Artifact → FitnessVector → EpistemicState → Invariant Evaluation
→ Violation Aggregation → Catastrophic Detection → ConstitutionalEvaluation

### ConstitutionalEvaluation as Boundary
ConstitutionalEvaluation is immutable, traceable, serializable.
Contains:
- artifact_id
- fitness
- epistemic
- violations
- catastrophic flag
- is_valid
- explanation

### Non-Compensability Preserved
Catastrophic thresholds cause immediate rejection.
Pareto compensation cannot override constitutional violations.

### Epistemic Propagation
EpistemicState is preserved through the pipeline without modification.
Uncertainty is never dropped or averaged away.

### Separation of Concerns
- evaluate(): constitutional validity
- compare(): Pareto dominance (delegated to ec-fitness)
- build_frontier(): non-dominated filtering

## Consequences
- Clean evaluation boundary for Phase 1+
- Traceable decisions
- No mixing of truth/fitness/confidence
