# ADR-005: Constitutional Evaluation Pipeline Architecture

**الحالة:** مُنفَّذ ومُثبت
**التاريخ:** 2024-05-16
**السياق:** تنظيم تسلسل تقييم الجودة الدستورية عبر دمج فحوصات الثوابت، الانتهاكات الكارثية، ونشر عدم اليقين المعرفي.

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
