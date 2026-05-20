# ADR-013: Integration Pipeline — Week 17-18

## Status
✅ Accepted — 2024

## Context
بعد إتمام Phase 2 (Weeks 13-16)، لدينا 4 crates مستقلة:
- ec-fitness: FitnessVector + Pareto
- ec-epistemic: Uncertainty + Calibration
- ec-constitutional: Constitution + Engine
- ec-sandbox: Sandbox + Reality + Security

لكن لا يوجد crate يربطها في pipeline واحد.
كل crate يعمل بمفرده لكن لا يوجد "glue code".

## Decision

### إنشاء ec-app كـ Integration Layer

```text
ec-app (integration crate)
  ├── depends on: ec-fitness, ec-epistemic, ec-constitutional, ec-sandbox
  ├── pipeline.rs: IntegrationPipeline
  └── لا يحتوي على منطق أعمال — فقط orchestration
Pipeline Architecture
text

Intent (string)
    │
    ▼
SandboxExecutor.execute(artifact_id, code)
    │
    ▼
ExecutionResult { reality: RealityVector }
    │
    ├──▶ estimate_fitness_from_reality() → FitnessVector
    ├──▶ build_epistemic_from_reality()  → EpistemicState
    │
    ▼
ConstitutionalEngine.evaluate(fitness, epistemic)
    │
    ▼
ConstitutionalEvaluation { is_valid, violations }
    │
    ├──▶ PredictionRecord::from_evaluation()
    ├──▶ RealityFeedback.learn(prediction, reality)
    ├──▶ Constitution.learn(evaluation, observed_outcome)
    │
    ▼
PipelineVerdict: Accepted | RejectedByConstitution | RejectedByReality | ExecutionFailed
Key Design Decisions
1. estimate_fitness_from_reality() — تقدير لا تحويل
Rust

/// Truth ≠ Fitness: هذا تقدير إرشادي فقط
pub fn estimate_fitness_from_reality(reality: &RealityVector) -> FitnessVector
RealityVector لا يُحوَّل مباشرة إلى FitnessVector
التقدير إرشادي — Phase 3 سيُستبدل بـ static analysis
2. RealityVector::dummy() — للـ error handling
Rust

pub fn dummy() -> Self  // correctness=0, reprod=0, confidence=0
يُستخدم عندما يفشل التنفيذ كلياً
pub (وليس pub(crate)) لأن ec-app يحتاجه
3. RealityVector::test_fixture() — للاختبارات
Rust

pub fn test_fixture(correctness, reproducibility, benchmark_validity, runs) -> Self
يُستخدم في tests خارج ec-sandbox
يحسب empirical_confidence تلقائياً
pub(crate) constructor يبقى محمياً — test_fixture هي البديل الآمن
4. PipelineVerdict — 4 حالات فقط
Rust

enum PipelineVerdict {
    Accepted,                          // كل شيء صحيح
    RejectedByConstitution { reason }, // دستورياً مرفوض
    RejectedByReality { reason },      // واقعياً مرفوض
    ExecutionFailed { error },         // فشل تنفيذي
}
Test Architecture
text

Week 17: 18 integration tests
  - Pipeline creation
  - Simulated mode (accept/reject)
  - Constitutional rejection
  - PipelineResult fields
  - Prediction error calculation
  - Feedback learning
  - Multiple artifacts
  - Fitness estimation
  - Epistemic building
  - Security handling

Week 18: 10 gate tests
  - Truth ≠ Fitness invariant
  - RealityVector from sandbox only
  - Pipeline end-to-end
  - Feedback improves
  - Constitutional review triggers
  - Escape vectors contained
  - Network isolation
  - ADR documentation
  - Epistemic from reality
  - Phase 2 final gate
Consequences
✅ Gains
One pipeline: كل الطبقات مربوطة
Testable: 28 tests تختبر الـ integration
Clean separation: ec-app لا يحتوي على منطق أعمال
Truth ≠ Fitness: محفوظ بالكود + بالاختبارات
Feedback loop: Constitution → Sandbox → Learn يعمل
⚠️ Limitations
Simulated mode only: Pipeline لا يستخدم Docker mode بشكل افتراضي
Fitness estimation: بدائي — يحتاج static analysis في Phase 3
No multi-iteration: Pipeline يعمل مرة واحدة — لا يعيد المحاولة
Constitution.learn(): placeholder في pipeline (يُحسَّن في Phase 3)
🔴 Phase 3 Improvements
Static analysis للـ fitness (بدلاً من estimation من reality)
Multi-iteration pipeline (حتى يُقبل أو ينفد المحاولات)
Code generation (حتى الآن: الكود يُمرَّر يدوياً)
Docker mode كـ default
References
ADR-009: Sandbox Foundation (Week 13)
ADR-010: Docker Execution Strategy (Week 14)
ADR-012: Security Hardening (Week 16)
Phase 2 Gate: ✅ PASSED (230 tests, 0 failures)
