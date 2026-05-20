Formal Proofs: Pareto Dominance Properties
Preliminaries
Definition 1 — FitnessVector:
Let a fitness vector be a tuple in metric space:

A = (a₁, a₂, ..., aₙ)  where  aᵢ ∈ [0.0, 1.0]  for all i ∈ {1, ..., n}
Definition 2 — Pareto Dominance (≻):
A ≻ B if and only if:

(∀i ∈ {1,...,n}: aᵢ ≥ bᵢ)  ∧  (∃j ∈ {1,...,n}: aⱼ > bⱼ)
We call the first clause universal weak superiority
and the second clause strict superiority in at least one dimension.

Both clauses must hold simultaneously for dominance to hold.

Proof 1: Irreflexivity
Theorem: ¬(A ≻ A) for all A ∈ [0,1]ⁿ

What we are proving:
No fitness vector dominates itself.
The dominance relation is irreflexive.

Proof:

Assume for contradiction that A ≻ A.

By Definition 2, this requires:

(1)  ∀i ∈ {1,...,n}: aᵢ ≥ aᵢ        [universal weak superiority]
(2)  ∃j ∈ {1,...,n}: aⱼ > aⱼ        [strict superiority in some dimension]
Clause (1) is trivially satisfied: aᵢ ≥ aᵢ holds by reflexivity of ≥
over ℝ for all i.

Clause (2) requires aⱼ > aⱼ for some j.
But aⱼ > aⱼ asserts a real number is strictly greater than itself.
This contradicts the irreflexivity of > over ℝ:

∀x ∈ ℝ: ¬(x > x)
Clause (2) cannot be satisfied for any j.
Therefore the conjunction of (1) and (2) is false.
Therefore A ≻ A is false.
Therefore ¬(A ≻ A). ∎

Proof 2: Asymmetry
Theorem: (A ≻ B) → ¬(B ≻ A) for all A, B ∈ [0,1]ⁿ

What we are proving:
If A dominates B, then B cannot dominate A.
Dominance is a one-way relation — it cannot hold in both directions
between any pair of distinct vectors.

Proof:

Assume A ≻ B.

By Definition 2, this gives us:

(1)  ∀i: aᵢ ≥ bᵢ
(2)  ∃j: aⱼ > bⱼ
Now assume for contradiction that B ≻ A also holds.

By Definition 2 applied to B ≻ A:

(3)  ∀i: bᵢ ≥ aᵢ
(4)  ∃k: bₖ > aₖ
From (1) and (3), for all i:

aᵢ ≥ bᵢ  and  bᵢ ≥ aᵢ
By antisymmetry of ≥ over ℝ:

∀i: aᵢ = bᵢ
But if aᵢ = bᵢ for all i, then in particular aⱼ = bⱼ for the j
guaranteed by clause (2).

This contradicts clause (2), which requires aⱼ > bⱼ.

aⱼ > bⱼ  contradicts  aⱼ = bⱼ
Our assumption that B ≻ A leads to contradiction.
Therefore ¬(B ≻ A).
Therefore (A ≻ B) → ¬(B ≻ A). ∎

Proof 3: Transitivity
Theorem: (A ≻ B) ∧ (B ≻ C) → (A ≻ C) for all A, B, C ∈ [0,1]ⁿ

What we are proving:
If A dominates B and B dominates C, then A dominates C.
Dominance chains are consistent and sound.
This property enables transitive elimination in the governance kernel.

Proof:

Assume A ≻ B and B ≻ C.

From A ≻ B by Definition 2:


(1)  ∀i: aᵢ ≥ bᵢ
(2)  ∃j: aⱼ > bⱼ
From B ≻ C by Definition 2:

(3)  ∀i: bᵢ ≥ cᵢ
(4)  ∃k: bₖ > cₖ
Step 1 — Prove universal weak superiority of A over C:

From (1) and (3), for all i:


aᵢ ≥ bᵢ  and  bᵢ ≥ cᵢ
By transitivity of ≥ over ℝ:

∀i: aᵢ ≥ cᵢ
Step 2 — Prove strict superiority of A over C in at least one dimension:

From (2), there exists j such that aⱼ > bⱼ.
From (3) applied to index j: bⱼ ≥ cⱼ.

Combining:


aⱼ > bⱼ  and  bⱼ ≥ cⱼ
By the mixed transitivity law over ℝ:


(x > y) ∧ (y ≥ z)  →  x > z
Therefore: aⱼ > cⱼ

This satisfies the existential clause with witness j.

Conclusion:

We have shown:

∀i: aᵢ ≥ cᵢ    [universal weak superiority]
∃j: aⱼ > cⱼ    [strict superiority at witness j]
By Definition 2: A ≻ C.
Therefore (A ≻ B) ∧ (B ≻ C) → (A ≻ C). ∎

Incomparability
Definition 3 — Incomparable Vectors (∥):

Two vectors A and B are incomparable, written A ∥ B, if and only if:


A ∥ B  ⟺  ¬(A ≻ B)  ∧  ¬(B ≻ A)
Plain English:
Neither vector dominates the other. Each vector is strictly better than
the other in at least one dimension — they represent genuinely different
trade-offs. Incomparable vectors are both candidates for the Pareto frontier.
Incomparability is not a failure state — it is the precise condition
that makes the frontier non-trivial and trade-off reasoning necessary.

Note on exhaustiveness:
For any two vectors A and B, exactly one of these holds:

A ≻ B    (A dominates B)
B ≻ A    (B dominates A)
A ∥ B    (incomparable — both may be on the frontier)
By Proof 2 (Asymmetry), the first two cases are mutually exclusive.

Incomparability Example
Let n = 2, dimensions = [security, efficiency].


A = (0.9, 0.4)    high security, low efficiency
B = (0.3, 0.95)   low security, high efficiency
Test A ≻ B:


Clause 1: a₁ ≥ b₁ → 0.9 ≥ 0.3  ✓
           a₂ ≥ b₂ → 0.4 ≥ 0.95 ✗
Clause 1 fails at dimension 2. Therefore A ⊁ B.

Test B ≻ A:



Clause 1: b₁ ≥ a₁ → 0.3 ≥ 0.9  ✗
Clause 1 fails at dimension 1. Therefore B ⊁ A.

Conclusion: A ∥ B

Both vectors are on the Pareto frontier. The governance kernel cannot
eliminate either without imposing a value judgment — specifically, deciding
whether security or efficiency matters more. This is precisely the decision
the kernel should surface to human reviewers rather than resolve silently.

Note: In a real evaluation, A would additionally trigger the security
catastrophe threshold at 0.3 < 0.40, resulting in immediate rejection
regardless of its frontier membership. Catastrophe thresholds are
pre-dominance constitutional filters, not dominance comparisons.

Summary
Property	Statement	Consequence
Irreflexivity	¬(A ≻ A)	Self-comparison is always false
Asymmetry	A ≻ B → ¬(B ≻ A)	Dominance is strictly directional
Transitivity	A ≻ B ∧ B ≻ C → A ≻ C	Chains are sound; transitive elimination holds
Incomparability	¬(A ≻ B) ∧ ¬(B ≻ A)	Both are frontier candidates
These three properties together confirm that ≻ is a strict partial order
on the space of fitness vectors — the correct mathematical foundation
for a constitutional governance kernel.
