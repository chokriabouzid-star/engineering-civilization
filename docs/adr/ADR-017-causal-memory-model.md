# ADR-017: Causal Memory Model

**Status:** ✅ Accepted  
**Date:** 2026-05-18  
**Context:** Week 20 — Phase 3  

---

## Context

Phase 2 أثبت أن النظام يُقيّم ويُنفّذ الكود بأمان. لكن **لا ذاكرة**.

كل قرار يُنسى فور اتخاذه. لا تاريخ، لا تعلم، لا تحسن.

Phase 3 تبني **ذاكرة سببية** — سجل دائم لكل قرار اتُخذ، مع:
- **Why** was it chosen?
- **What** alternatives were rejected?
- **Where from** did this decision come (causal parents)?

---

## Decision

نبني `ec-memory` — Causal Memory Graph مع:

### 1. Append-only History

```rust
// ✅ المسموح
graph.record(node);

// ❌ الممنوع (لا توجد هذه الـ methods)
graph.delete(node_id);     // compile error
graph.update_fitness(...); // compile error
graph.clear();             // compile error
Guarantee: Type-level enforcement.
لا توجد طريقة لتعديل الماضي — فقط إضافة قرارات جديدة.

2. Mutable Interpretation
Rust

pub struct DecisionNode {
    // ─── Immutable ───────────────────────────────────────────
    pub id: NodeId,
    pub created_at: DateTime<Utc>,
    pub artifact: ArtifactSnapshot,
    pub fitness: FitnessVector,
    pub constitutional_valid: bool,
    pub sandbox_outcome: Option<SandboxOutcome>,
    pub causal_parents: Vec<NodeId>,
    pub alternatives: Vec<RejectedAlternative>,
    
    // ─── Mutable (append-only log) ──────────────────────────
    pub retrospective: Vec<RetrospectiveAssessment>,
}
Philosophy:

الماضي لا يتغير (immutable facts)
فهمنا للماضي يتغير (mutable interpretation)
عندما يتعلم Historian Agent بعد 6 أشهر أن البديل المرفوض كان أفضل:

Rust

let assessment = RetrospectiveAssessment::new(
    true,  // was_better_choice
    0.85,  // confidence
    "Lower complexity led to easier maintenance"
);

memory.update_retrospective(rejected_alt_id, assessment)?;
3. ArtifactSnapshot with Arc Sharing
المشكلة:
بعد 10,000 قرار، لو كل node يحتفظ بنسخة من الكود:

Rust

pub struct DecisionNode {
    pub code: String,  // 10k × 500 bytes = 5 MB تكرار
}
الحل:

Rust

pub struct ArtifactSnapshot {
    pub hash: ArtifactHash,
    pub code: Arc<str>,  // shared ownership
}
نفس الكود يُشارَك بين آلاف القرارات:

Rust

let snap1 = ArtifactSnapshot::new("fn main() {}");
let snap2 = snap1.clone();
assert!(Arc::ptr_eq(&snap1.code, &snap2.code));  // ✅ نفس المؤشر
4. DAG Structure (Directed Acyclic Graph)
Rust

pub struct DecisionNode {
    pub causal_parents: Vec<NodeId>,
}
كل قرار يُسجل من أين جاء:

text

iteration_3
    ↓
iteration_2
    ↓
iteration_1
    ↓
baseline
Validation:
عند record():

Rust

fn validate_acyclic(&self, node: &DecisionNode) -> Result<(), MemoryError>
يمنع الدورات (cycles) — DAG محفوظ.

5. RejectedAlternative Storage
Rust

pub struct RejectedAlternative {
    pub id: NodeId,
    pub artifact: ArtifactSnapshot,
    pub fitness: FitnessVector,
    pub reason: RejectionReason,
    pub rejected_at: DateTime<Utc>,
    pub retrospective: Vec<RetrospectiveAssessment>,
}
لماذا؟

معظم الأنظمة تخزن ما نجح فقط.

لكن الذكاء الحقيقي يحتاج:

What almost succeeded?
Why was it rejected?
Was it actually better? (retrospective)
Consequences
✅ Positive
Full Causal Trace: كل قرار مُوثق مع سببه
Counterfactual Queries: "ماذا كان سيحدث لو..."
Learning from History: النظام يتعلم من أخطائه
Auditability: كل قرار قابل للمراجعة
Memory Efficiency: Arc sharing يوفر الذاكرة
⚠️ Negative
Storage Growth: الذاكرة تنمو بلا حد (حل: PostgreSQL في Phase 4)
Query Performance: O(n) linear scan (حل: indexes لاحقاً)
No Delete: لا يمكن حذف قرارات قديمة (feature, not bug)
🔮 Risks Mitigated
Cycle Detection: DAG validation تمنع الدورات
Arc Leaks: Drop semantics محفوظة
Retrospective Abuse: Append-only log (لا overwrite)
Alternatives Considered
❌ HashMap instead of Vec
Rust

nodes: HashMap<NodeId, DecisionNode>
رُفض: الترتيب الزمني ضروري.
latest_n() و causal_chain() تحتاج ترتيب.

❌ Mutable DecisionNode
Rust

pub fn update_fitness(&mut self, new_fitness: FitnessVector)
رُفض: يكسر append-only invariant.
الماضي لا يتغير.

❌ String instead of Arc<str>
Rust

pub code: String
رُفض: تكرار ضخم في الذاكرة.

Implementation
Rust

// ✅ Week 20: Complete
crates/ec-memory/
├── src/
│   ├── types.rs        // NodeId, ArtifactSnapshot, RejectionReason
│   ├── node.rs         // DecisionNode, RejectedAlternative
│   └── graph.rs        // CausalMemoryGraph (append-only)
└── tests/
    └── week20_gate.rs  // 16 tests
Validation
Bash

cargo test -p ec-memory week20_gate_complete
Output:

text

✅ Gate 1: Memory creates
✅ Gate 2: Append-only (no delete)
✅ Gate 3: Retrospective mutable
✅ Gate 4: Causal chain works
✅ Gate 5: ArtifactSnapshot shares Arc
✅ Gate 6: Decisions for artifact
✅ Gate 7: Latest N
References
Phase 3 Week 19 Plan
Truth ≠ Fitness (preserved)
Append-only log patterns
DAG algorithms
