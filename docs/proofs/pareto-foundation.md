Pareto Foundation for Constitutional Governance Kernels
1. Why Weighted Sum Fails
The weighted sum collapses a multi-dimensional evaluation into a single scalar,
destroying structural information in the process.

The critical failure: compensability.
A weighted sum allows catastrophic failure in one dimension to be
arithmetically cancelled by excellence in another.

Concrete Numerical Example
Consider two proposals evaluated on [security, performance, maintainability]:

Proposal	Security	Performance	Maintainability	Weighted Sum (0.3/0.4/0.3)
A	0.10	1.00	1.00	0.73
B	0.80	0.75	0.80	0.78
The weighted sum selects B over A — reasonable.
But it also ranks A as acceptable (0.73 > threshold 0.6).

A has a security score of 0.10. This is a constitutionally broken artifact.
No amount of performance or maintainability compensates for a system that is
fundamentally insecure. The weighted sum has no concept of a floor, a veto,
or a non-negotiable constitutional constraint. It treats all dimensions as
interchangeable currency.

This is not an implementation flaw. It is a mathematical flaw in the model.

2. Pareto Dominance — Formal Definition
Let a proposal be a vector in metric space:


p ∈ ℝⁿ  where each dimension i ∈ {1...n} is a quality criterion
Formal Definition:

Proposal A Pareto-dominates proposal B (written A ≻ B) if and only if:


∀i ∈ {1...n}: A[i] ≥ B[i]     (A is no worse in every dimension)
∃j ∈ {1...n}: A[j] > B[j]     (A is strictly better in at least one dimension)
Plain English:
A dominates B if A is at least as good as B on every criterion,
and strictly better on at least one. There is no trade-off that saves B.

Concrete Example:


A = [0.9, 0.8, 0.7]
B = [0.8, 0.7, 0.6]
A ≻ B because A ≥ B in all three dimensions and A > B in all three.


A = [0.9, 0.5, 0.8]
C = [0.7, 0.9, 0.6]
A ⊁ C and C ⊁ A — neither dominates. They are incomparable. This is valid.

3. Mathematical Properties of Pareto Dominance
Property 1: Irreflexivity
Formal: ¬(A ≻ A) for all A

A proposal cannot dominate itself, because the strict inequality
∃j: A[j] > A[j] is impossible.
Why it matters: Prevents circular reasoning in elimination logic.

Property 2: Asymmetry
Formal: A ≻ B → ¬(B ≻ A)

If A dominates B, then B cannot dominate A.
Why it matters: Guarantees consistent, non-contradictory ordering.
Elimination decisions are unambiguous and irreversible.

Property 3: Transitivity
Formal: A ≻ B ∧ B ≻ C → A ≻ C

Dominance chains are consistent.
Why it matters: Enables sound transitive elimination across large proposal
sets without re-evaluating every pair.

Together, these three properties make ≻ a strict partial order —
the correct mathematical structure for constitutional evaluation.

4. The Pareto Frontier
Definition:
The Pareto Frontier (or Pareto Front) is the set of all proposals that are
not dominated by any other proposal:


F = { A ∈ P | ¬∃B ∈ P : B ≻ A }
Why it matters for this system:

The frontier is the constitutionally admissible set — the proposals where
no improvement is possible in any dimension without sacrificing another.
Proposals inside the frontier are provably inferior and can be rejected
by mathematical proof, not by opinion.

The weighted sum gives you one point.
The Pareto Frontier gives you the complete set of defensible choices,
preserving legitimate trade-offs while eliminating the indefensible ones.

This is the correct foundation for a governance kernel.

