# ADR-002: Epistemic Uncertainty Model

## Status
Accepted (Week 2)

## Context
FitnessVector scores alone are insufficient: identical scores may be supported by radically different evidence.
We need explicit modeling of confidence, evidence, uncertainty sources, and calibration.

## Decision
We introduce `ec-epistemic`:

### EpistemicState
- confidence ∈ [0,1]
- evidence:
  - sample_size: u64
  - age_seconds: u64
  - reproducibility ∈ [0,1]
  - source_reliability ∈ [0,1]
- uncertainty decomposition (finite, non-negative):
  - aleatoric, epistemic, model
- calibration: fixed 10-bin CalibrationState tracking ECE

### Conservative propagation
For a set of states:
- confidence = min(confidences)
- uncertainty components compound via RMS:
  component = sqrt(sum(component_i^2))
- evidence = weakest:
  sample_size=min, reproducibility=min, source_reliability=min, age=max
- calibration merge: sum bin counts and sums

Properties are enforced via proptest (1000 cases):
- combined confidence is conservative
- combined uncertainty is monotone

### Temporal decay
Exponential half-life:
decay_factor = 0.5^(elapsed / half_life), half_life=30 days.
- new_confidence = old_confidence * decay_factor
- epistemic uncertainty increases conservatively (clamped)

### Calibration (ECE)
- 10 bins [0.0-0.1), ... [0.9-1.0]
- record(predicted ∈ [0,1], actual ∈ {0,1})
- ECE = Σ weight(bin) * |avg_pred - avg_actual|
- calibrated iff ECE < 0.1

## Consequences
- Fitness and confidence are separate and cannot be implicitly converted.
- The system can track uncertainty, decay stale beliefs, and quantify calibration.

## Future work
- Continuous actual outcomes in [0,1].
- Alternative combine modes if min-confidence becomes too pessimistic for long chains.
