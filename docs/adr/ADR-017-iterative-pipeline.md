# ADR-017: Iterative Pipeline

## Status
Accepted

## Context
Phase 3 requires a pipeline that iteratively generates, evaluates, and refines code. Each iteration must be recorded in causal memory.

## Decision
- IterativePipeline wraps CodeGenerator → Analysis → Constitution → Sandbox → Memory
- Max iterations configurable (default 3)
- Each iteration recorded as a DecisionNode with causal parent
- Rejected alternatives tracked with RejectionReason
- Feedback loop via RealityFeedback

## Consequences
- Complete audit trail of every attempt
- Causal chains enable counterfactual analysis
- Pipeline stops on first acceptance or max iterations
