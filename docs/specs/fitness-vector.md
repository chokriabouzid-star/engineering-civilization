# FitnessVector Specification

## Six Dimensions

| Dimension                | Range      | Threshold | Compensable? |
|--------------------------|------------|-----------|--------------|
| `security`               | [0.0, 1.0] | 0.40      | No           |
| `efficiency`             | [0.0, 1.0] | 0.20      | Yes          |
| `maintainability`        | [0.0, 1.0] | 0.25      | Yes          |
| `test_coverage`          | [0.0, 1.0] | 0.30      | Yes          |
| `architectural_stability`| [0.0, 1.0] | 0.35      | No           |
| `changeability`          | [0.0, 1.0] | 0.30      | Yes          |

## Dimension Definitions

### `security`
- **Measures:** Resistance to exploitation and integrity violations
- **0.0:** Known critical CVEs, hardcoded secrets, no input validation
- **1.0:** Zero known vulnerabilities, least privilege enforced
- **Threshold 0.40:** CVSS ≥ 7.0 or structural security failure
- **Non-compensable:** No other dimension offsets a security failure

### `efficiency`
- **Measures:** Resource utilization relative to optimal solution
- **0.0:** Exponential/unbounded resource consumption
- **1.0:** Asymptotically optimal, within 5% of theoretical minimum
- **Threshold 0.20:** Pathological inefficiency causing system degradation

### `maintainability`
- **Measures:** Ease of understanding and modifying by unfamiliar engineer
- **0.0:** No docs, cyclomatic complexity > 50, global mutable state
- **1.0:** Self-documenting, complexity ≤ 5, clear separation of concerns
- **Threshold 0.25:** Any modification has unpredictable consequences

### `test_coverage`
- **Measures:** Breadth and meaningfulness of automated verification
- **0.0:** Zero tests exist
- **1.0:** 100% branch coverage, mutation score > 90%
- **Threshold 0.30:** Effectively unverified artifact

### `architectural_stability`
- **Measures:** Respect for system boundaries and interface invariants
- **0.0:** Circular dependencies, broken interfaces, implicit coupling
- **1.0:** Zero circular deps, all interfaces respected, acyclic graph
- **Threshold 0.35:** Structural violations that radiate outward
- **Non-compensable:** Destabilizes the entire system's evaluability

### `changeability`
- **Measures:** How safely the artifact can be modified or rolled back
- **0.0:** No rollback path, hard-wired into 10+ downstream consumers
- **1.0:** Full rollback in one atomic operation, interface-only coupling
- **Threshold 0.30:** No recovery path under failure conditions

## Design Decisions

### Why not `performance`?
Renamed to `efficiency` — "performance" is ambiguous (speed? throughput?).
Efficiency captures resource utilization across time and space complexity.

### Why not `reversibility`?
Renamed to `changeability` — reversibility implies binary (can/cannot).
Changeability captures a continuous spectrum of modification safety.

### Why these thresholds?
See `docs/adr/001-fitness-vector.md` for full justification.
Non-compensable thresholds are higher because their failures radiate outward.
