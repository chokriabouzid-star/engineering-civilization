# Engineering Civilization — الوثيقة المرجعية النهائية
## Week 56 · 11 crates · 662 tests · Phase 6 Complete

---

## 1. نظرة عامة
11 crates · ~110 ملف .rs · ~22,000 سطر
662 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
10 design invariants · 12 ADRs
جميع المراحل مكتملة: Phase 1 → Phase 6

text

**المبادئ الثابتة:**

| # | المبدأ | المعنى |
|---|--------|--------|
| 1 | الدستور أولاً | كل كود يُقيَّم دستورياً قبل القبول |
| 2 | الحقيقة ≠ اللياقة | Fitness تنبؤ، Reality حقيقة |
| 3 | الماضي لا يتغيّر | الذاكرة append-only |
| 4 | التعلم من الخطأ | Prediction error → calibration |
| 5 | باريتو للأفضليات | لا بُعد واحد يسيطر |
| 6 | الثبات الدائم | القرارات تبقى بعد إغلاق البرنامج |

---

## 2. هيكل المشروع

### الـ Workspace
[workspace]
members = [
"crates/ec-fitness", # الأساس — اللياقة
"crates/ec-epistemic", # الأساس — المعرفة
"crates/ec-constitutional", # المحرك الدستوري
"crates/ec-sandbox", # التنفيذ المعزول
"crates/ec-analysis", # التحليل الثابت
"crates/ec-memory", # الذاكرة السببية
"crates/ec-codegen", # توليد الكود
"crates/ec-app", # التطبيق الرئيسي
"crates/ec-governance", # الحوكمة (Phase 6)
"crates/ec-api", # REST API (Phase 6)
"crates/ec-cli", # CLI (Phase 6)
]

text

### Dependency Graph
ec-fitness ← (لا شيء)
ec-epistemic ← (لا شيء)
ec-constitutional ← ec-fitness, ec-epistemic
ec-sandbox ← ec-fitness, ec-constitutional, ec-epistemic
ec-analysis ← ec-fitness, syn
ec-memory ← ec-fitness, ec-epistemic, rusqlite
ec-codegen ← ec-fitness, ec-memory
ec-governance ← ec-fitness, ec-constitutional, ec-memory, ec-epistemic
ec-app ← الجميع
ec-api ← ec-app, ec-governance, ec-analysis, ec-memory
ec-cli ← ec-analysis, ec-governance, ec-memory

text

### مبدأ الحدود الدلالية (ADR-020)
Kernel (لا يتغير — لا async، لا HTTP، لا serde DTOs):
ec-fitness, ec-epistemic, ec-constitutional
ec-sandbox, ec-analysis, ec-memory, ec-codegen

Interface layer (محوّلات فقط):
ec-governance ← منطق حوكمة (sync)
ec-api ← HTTP adapter (axum, async)
ec-cli ← CLI adapter (clap)

text

---

## 3. الـ Crates — مرجع كامل

### 3.1 `ec-fitness` — اللياقة الدستورية

**الدور:** تعريف FitnessVector + عتبات الكارثة + Pareto + Cosine similarity

**الأنواع:**
- `FitnessVector` — 6 أبعاد [0,1]: security, reversibility, test_coverage, maintainability, performance, architectural_stability
- `CatastropheThresholds` — عتبات لكل بُعد (defaults: security=0.70, reversibility=0.30, etc.)
- `CatastrophicDimension` — enum للبُعد المتسبب بالكارثة
- `ParetoOrdering` — Dominates | Dominated | Equal | NonDominated

**الـ methods:**
- `validate()` — كل بُعد finite وفي [0,1]
- `cosine_similarity(&other) -> f64` — التشابه (1.0=متطابق)
- `cosine_angle_degrees(&other) -> f64` — الزاوية بالدرجات
- `pareto_compare(&other) -> ParetoOrdering` — مقارنة باريتو

---

### 3.2 `ec-epistemic` — الحالة المعرفية

**الدور:** تتبع الثقة والدليل والمعايرة + Bayesian inference

**الأنواع الأساسية:**
- `EpistemicState` — confidence + evidence + uncertainty + calibration
- `Evidence` — sample_size + age_seconds + reproducibility + source_reliability
- `UncertaintyDecomposition` — aleatoric + epistemic + model
- `CalibrationState` — 10 bins + ECE + is_calibrated()
- `CalibrationBin` — lower/upper/count/sum_predicted/sum_actual

**الأنواع Bayesian (Phase 5):**
- `BayesianEvidence` — successes + failures + mean_score + variance_estimate
  - `initial_prior() -> EpistemicResult<Self>` — prior غير متحيز (0,0,0.5,0.8)
  - `from_history(s, f, mean) -> EpistemicResult<Self>` — من تاريخ فعلي
  - `update_with_outcome(correct, score) -> EpistemicResult<Self>` — Bayesian update
  - `credible_confidence() -> f64` — Wilson score interval
  - `total_observations() -> u32` — successes + failures
- `BayesianCalibration` — تشخيص + تعديل الثقة
  - `diagnose(&CalibrationState) -> CalibrationDiagnosis`
  - `adjust_confidence(conf, diagnosis) -> f64`
- `CalibrationDiagnosis` — WellCalibrated | Overconfident | Underconfident | InsufficientData

---

### 3.3 `ec-constitutional` — المحرك الدستوري

**الدور:** تقييم الكود ضد 8 ثوابت دستورية

**الأنواع:**
- `ConstitutionalEngine` — يحمل Constitution ويُقيّم
- `Constitution` — مجموعة من `Arc<dyn Invariant>`
- `Invariant` trait — `check(&FitnessVector, &EpistemicState) -> Result<(), ViolationReport>`
- `ConstitutionalEvaluation` — {is_valid, results, fitness, verdict, catastrophic, epistemic, artifact_id, explanation}
- `ConstitutionalVerdict` — Compliant | Violated {violations}
- `ObservedOutcome` — correctness + reproducibility

**الثوابت المُدمجة (8):**
1. `SecurityInvariant` — security ≥ 0.70
2. `ReversibilityInvariant` — reversibility ≥ 0.30
3. `TestCoverageInvariant` — test_coverage ≥ 0.60
4. `MaintainabilityInvariant` — maintainability ≥ 0.40
5. `PerformanceInvariant` — performance ≥ 0.20
6. `ArchitecturalStabilityInvariant` — architectural_stability ≥ 0.50
7. `CatastrophePreventionInvariant` — لا بُعد تحت العتبة الكارثية
8. `EpistemicConfidenceInvariant` — confidence ≥ 0.30

---

### 3.4 `ec-sandbox` — بيئة التنفيذ المعزولة

**الدور:** تنفيذ الكود في Docker + قياس RealityVector + Bayesian tracking

**الأنواع الأساسية:**
- `SandboxExecutor` — ينشئ/يُشغّل Docker containers
- `SandboxConfig` — timeout, memory_limit, network_disabled, etc.
- `SandboxMode` — Simulated | Docker | Hardened | Local
- `ExecutionResult` — status, stdout, stderr, metrics, duration
- `RealityVector` — correctness, reproducibility, empirical_confidence
- `PredictionRecord` — predicted_validity + predicted_confidence
- `PredictionError` — validity_error + confidence_error
- `RealityFeedback` — يتعلم من التنبؤات ويقيس التحسن

**الأنواع Bayesian (Phase 5):**
- `BayesianTracker` — يتتبع outcomes بـ BayesianEvidence
  - `new()` → prior غير متحيز
  - `record(correct, score)` → تسجيل نتيجة
  - `evidence() -> &BayesianEvidence`
  - `credible_confidence() -> f64` — Wilson
  - `has_sufficient_data() -> bool` — ≥5 مشاهدات

---

### 3.5 `ec-analysis` — التحليل الثابت

**الدور:** تحليل كود Rust → FitnessVector + ConfidenceVector

**الواجهات:**
- `analyze_code(code: &str) -> FitnessVector` — keyword heuristic (قديم، لا يتغير)
- `analyze_code_full(code: &str) -> AnalysisReport` — syn AST (جديد)

**الأنواع:**
- `AnalysisReport` — {fitness, confidence, warnings, parse_successful}
- `ConfidenceVector` — 6 أبعاد + `overall()` (min)
- `AnalysisWarning` — ParseFailed | LowConfidence | UnsafeWithoutComment | HighComplexity

**6 AST Visitors (Phase 4):**
1. `UnsafeVisitor` — unsafe blocks/fns/impls/traits + doc comment discount (40%)
2. `ComplexityVisitor` — cyclomatic complexity حقيقي (if, while, for, match, &&, ||, ?)
3. `TestVisitor` — #[test] fns vs production fns + assert macros
4. `CouplingVisitor` — use statements (std: 0.03 penalty, external: 0.12)
5. `SideEffectVisitor` — println/eprintln, static mut
6. `PerformanceVisitor` — allocations + clones

---

### 3.6 `ec-memory` — الذاكرة السببية

**الدور:** تخزين قرارات التصميم في DAG + استعلامات + انجراف + Bayesian queries

**الأنواع الأساسية:**
- `CausalMemoryGraph` — DAG من DecisionNode (append-only)
- `DecisionNode` — id, artifact_id, artifact, fitness, constitutional_valid, sandbox_outcome, causal_parents, created_at, retrospective_assessments
- `NodeId` — Uuid wrapper
- `ArtifactSnapshot` — code_hash + code (Arc<str>)
- `DecisionNodeBuilder` — Builder pattern (D4)
- `SandboxOutcome` — correctness + reproducibility + empirical_confidence
- `RetrospectiveAssessment` — actual_validity + assessment_reason + assessed_at
- `MemoryQuery` — استعلامات على الذاكرة
- `SimilarDecision` — node_id + similarity + fitness + was_accepted
- `HistoricalDriftAnalyzer` — تحليل الانجراف القيمي
- `DriftReport` — classification + angle + details
- `DriftClassification` — Stable | LearningProgress | ValueShift | Corruption | InsufficientData
- `DriftAction` — None | Monitor | ReviewConstitution | HumanIntervention
- `SqliteStorage` — تنفيذ SQLite (D7)
- `MemoryStorage` trait — save/load

**الأنواع Bayesian (Phase 5):**
- `OutcomeStorage` trait — record_outcome + load_evidence + load_all_evidence + outcome_count
- `BayesianQuery<'a, S: OutcomeStorage>` — استعلامات مع Bayesian confidence
  - `find_similar_with_confidence(target, k)` → Vec<BayesianSimilarDecision>
  - `best_by_confidence(k)` → Vec<BayesianSimilarDecision>
- `BayesianSimilarDecision` — node_id + artifact_id + similarity + bayesian_confidence + combined + fitness + was_accepted + total_observations

**SQLite Schema:**
- `decisions` table + `retrospective_assessments` table (Week 27)
- `bayesian_outcomes` table (Phase 5)

---

### 3.7 `ec-codegen` — توليد الكود

**الدور:** توليد كود Rust من مواصفات

**الأنواع:**
- `CodeGenerator` — يُنشئ كود من GenerationSpec
- `GenerationSpec` — artifact_id + requirements + constraints + templates
- `GenerationResult` — Success(code) | Failure(reason)
- `CodeTemplate` trait — name + priority + generate(spec)
- `GenerationSuccess` — code + template_used + generation_time
- `FailureContext` — previous_attempts + failure_reasons

---

### 3.8 `ec-governance` — الحوكمة الدستورية (Phase 6)

**الدور:** إدارة الاقتراحات الدستورية + سجل تدقيق + ربط DriftReport

**الأنواع:**
- `ConstitutionalProposal` — اقتراح تعديل دستوري (immutable بعد الإنشاء)
- `ProposalOrigin` — Human {name} | System {trigger}
- `SystemTrigger` — DriftDetected | OssificationDetected | BayesianCalibrationDrift
- `ProposedChange` — AdjustThreshold | AddInvariant | RemoveInvariant | UpdatePolicy
- `ThresholdDirection` — Tighten | Loosen
- `ProposalStatus` — Pending | UnderReview | Approved | Rejected | Applied | Superseded
- `ProposalStore` — append-only store للاقتراحات
  - `submit()`, `approve()`, `reject()`, `mark_applied()`
  - `pending()`, `approved_pending_application()`, `all()`, `find()`
- `AuditLog` — سجل أحداث append-only
  - `record(event, actor, context)`
  - `all()`, `last_n()`, `for_proposal(id)`
- `GovernanceEvent` — ProposalCreated | ProposalApproved | ProposalRejected | ProposalApplied | SystemAlertFired | ConstitutionAmended | HumanOverride
- `GovernanceStorage` — SQLite persistence (proposals + audit_log tables)
- `drift_trigger::propose_from_drift()` — DriftReport → Proposal تلقائي

---

### 3.9 `ec-api` — REST API (Phase 6)

**الدور:** طبقة نقل HTTP رفيعة — لا منطق عمل

**الـ endpoints (11):**
POST /api/v1/analyze ← AnalysisReport
POST /api/v1/governance/proposals ← submit proposal
GET /api/v1/governance/proposals ← list proposals
PATCH /api/v1/governance/proposals/:id/approve ← approve
PATCH /api/v1/governance/proposals/:id/reject ← reject
GET /api/v1/governance/audit ← audit log
GET /api/v1/memory/nodes ← memory nodes
GET /api/v1/memory/drift ← drift report
GET /api/v1/memory/similar ← similar decisions
GET /api/v1/health ← health check

text

**الأنواع:**
- `AppState` — shared state (proposals + audit + memory + gov_storage)
- `handlers` — دوال رفيعة: استقبال HTTP → استدعاء kernels → إرجاع JSON
- `routes` — Router builder

---

### 3.10 `ec-cli` — Command Line Interface (Phase 6)

**الدور:** طبقة أوامر طرفية — read args → call → print

**الأوامر:**
ec analyze <path> ← تحليل ملف (جدول منسق أو JSON)
ec drift ← تقرير الانجراف
ec propose submit/list/approve ← إدارة الاقتراحات
ec audit ← سجل التدقيق
ec health ← فحص صحة النظام

text

**الأنواع:**
- `Cli` — هيكل الأوامر (clap derive)
- دوال `cmd_*` — قراءة المُدخلات، استدعاء الدوال، طباعة النتائج

---

### 3.11 `ec-app` — التطبيق الرئيسي

**الدور:** ربط كل الـ pipelines معاً

**الأنواع:**
- `IntegrationPipeline` — pipeline كامل (analyze → evaluate → sandbox → verdict)
- `IterativePipeline` — تكرار حتى النجاح
- `BayesianPipeline` — pipeline مع Bayesian tracking + calibration
- `PipelineVerdict`, `PipelineResult`, `IterativePipelineResult`, `AttemptRecord`
- `BayesianPipelineResult` — bayesian_confidence + raw_confidence + calibration_diagnosis
- `build_epistemic_from_fitness()` — الدالة القديمة
- `build_epistemic_from_bayesian()` — الدالة Bayesian
- `determine_verdict()` — قرار القبول/الرفض

---

## 4. ثوابت التصميم (Design Invariants)

| # | الاسم | الوصف | المكان |
|---|-------|-------|--------|
| D1 | Append-Only Memory | لا delete/update_fitness/clear | ec-memory |
| D2 | Truth ≠ Fitness | FitnessVector ≠ RealityVector | ec-sandbox + ec-analysis |
| D3 | DAG Enforcement | validate_acyclic() قبل كل record() | ec-memory |
| D4 | Builder Pattern | DecisionNode::new() = pub(crate) | ec-memory |
| D5 | Single Similarity Source | cosine_similarity() وحيد | ec-fitness |
| D6 | Constitutional Primacy | فشل دستوري = رفض نهائي | ec-constitutional |
| D7 | Persistent Memory | SQLite roundtrip guarantee | ec-memory |
| D8 | Confidence Separate | ConfidenceVector/BayesianEvidence ≠ FitnessVector | ec-analysis + ec-epistemic |
| D9 | Bayesian Primacy | إذا N≥10 outcomes → credible_confidence() بدلاً من raw | ec-epistemic + ec-constitutional |
| D10 | Outcome Transparency | كل outcome يُخزَّن بالـ raw score قبل أي تحويل | ec-memory (OutcomeStorage) |

---

## 5. قرارات التصميم (ADRs)

| ADR | العنوان | المرحلة |
|-----|---------|---------|
| 004 | Integration Architecture | Phase 2 |
| 009 | Sandbox Foundation | Phase 2 |
| 010 | Docker Execution Strategy | Phase 2 |
| 012 | Security Hardening | Phase 2 |
| 013 | Integration Pipeline | Phase 3 |
| 014 | Static Code Analysis | Phase 3 |
| 015 | Causal Memory | Phase 3 |
| 016 | Code Generation | Phase 3 |
| 017 | Iterative Pipeline | Phase 3 |
| 018 | Counterfactual Query | Phase 3 |
| 019 | Value Drift Enhanced | Phase 3 |
| 020 | Semantic Boundary Protection | Phase 6 |

---

## 6. تدفقات البيانات الرئيسية

### تحليل الكود (Phase 4)
code: &str
├── analyze_code() → FitnessVector (keyword heuristic — قديم)
└── analyze_code_full() → AnalysisReport (syn AST — جديد)
├── syn::parse_file()
├── AstAnalyzer::analyze_file()
│ ├── UnsafeVisitor → (security, 0.95)
│ ├── ComplexityVisitor → (maintainability, 0.88)
│ ├── TestVisitor → (coverage, 0.50)
│ ├── CouplingVisitor → (stability, 0.75)
│ ├── SideEffectVisitor → (reversibility, 0.70)
│ └── PerformanceVisitor → (performance, 0.75)
└── AnalysisReport {fitness, confidence, warnings, parse_successful}

text

### Bayesian Pipeline (Phase 5)
code: &str → analyze_code() → FitnessVector
→ BayesianTracker.evidence.credible_confidence()
→ BayesianCalibration.adjusted_credible_confidence()
→ EpistemicState {confidence: adjusted}
→ ConstitutionalEngine.evaluate()
→ SandboxExecutor.execute() → RealityVector
→ tracker.record(was_correct, score)
→ calibration.record(predicted, actual)
→ BayesianPipelineResult {bayesian_confidence, raw_confidence, diagnosis, verdict}

text

### Governance Flow (Phase 6)
DriftReport (من ec-memory)
→ drift_trigger::propose_from_drift()
→ ConstitutionalProposal (تلقائي)
→ ProposalStore.submit()
→ AuditLog.record(ProposalCreated)
→ Human reviews
→ ProposalStore.approve()
→ AuditLog.record(ProposalApproved)
→ ProposalStore.mark_applied()
→ AuditLog.record(ProposalApplied)
→ كل شيء يُخزَّن في GovernanceStorage (SQLite)

text

---

## 7. الاختبارات — الإحصائيات النهائية

### حسب الـ Crate

| Crate | عدد الاختبارات التقريبي |
|-------|------------------------|
| ec-fitness | ~10 |
| ec-epistemic | ~30 |
| ec-constitutional | ~90 |
| ec-sandbox | ~83 |
| ec-analysis | ~130 |
| ec-memory | ~86 |
| ec-codegen | ~14 |
| ec-governance | ~26 |
| ec-api | ~14 |
| ec-cli | ~16 |
| ec-app | ~50 |
| **المجموع** | **~662** |

### حسب الـ Phase

| Phase | Weeks | المحتوى الرئيسي | Tests |
|-------|-------|----------------|-------|
| 1 | 1-6 | Fitness + Epistemic + Constitutional | ~90 |
| 2 | 7-18 | Sandbox + Integration + Feedback | ~150 |
| 3 | 19-27 | Analysis + Memory + Codegen + Drift + SQLite | ~163 |
| 4 | 28-34 | syn AST + ConfidenceVector + 6 Visitors | +114 |
| 5 | 35-42 | Bayesian Intelligence (Evidence, Tracker, Calibration, Pipeline) | +89 |
| 6 | 43-56 | Governance + API + CLI + Final Gate | +56 |
| **المجموع** | **56** | | **662** |

---

## 8. سجل الإصلاحات الكامل

### Weeks 25-27: Hardening + SQLite (18 إصلاحاً)
- إزالة cosine_angle_degrees المكرر من drift.rs
- unwrap() → expect() في pipeline
- Docker tests feature flag
- validate_acyclic() إصلاح كشف الدورات
- Builder Pattern: DecisionNode::new() → pub(crate)
- cosine_similarity → FitnessVector (D5)
- فصل ec-constitutional عن ec-memory
- SQLite persistence: SqliteStorage + MemoryStorage trait
- Mutex<Connection> إصلاح :memory:
- ArtifactSnapshot: Serde يدوي لـ Arc<str>

### Phase 4 (Weeks 28-34): التحليل الحقيقي (7 خطوات)
- إضافة syn dependency لـ ec-analysis
- AnalysisReport + ConfidenceVector (D8)
- 6 AST Visitors: Unsafe, Complexity, Test, Coupling, SideEffect, Performance
- AstAnalyzer يجمع الـ 6 visitors
- analyze_code_full() — إضافة فقط، لا كسر
- Calibration dataset: tier1 vs tier4 fixtures
- Phase 4 Gate: 15 اختباراً

### Phase 5 (Weeks 35-42): Bayesian Intelligence (6 خطوات)
- BayesianEvidence: initial_prior + update_with_outcome + Wilson interval
- BayesianTracker: sandbox-level Bayesian tracking
- OutcomeStorage: SQLite persistence per artifact
- BayesianQuery: find_similar_with_confidence + best_by_confidence
- BayesianCalibration: overconfident/underconfident detection + adjustment
- BayesianPipeline: full integration with calibration feedback

### Phase 6 (Weeks 43-56): Governance + API + CLI (6 خطوات)
- D9 (Bayesian Primacy) + D10 (Outcome Transparency) + ADR-020
- ec-governance: ProposalStore + AuditLog + GovernanceStorage
- drift_trigger: DriftReport → Proposal تلقائي
- ec-api: 11 endpoint مع axum (handlers بدون business logic)
- ec-cli: 5 أوامر مع clap (ec analyze/drift/propose/audit/health)
- Final Gate: D1-D10 verification + backward compatibility

---

## 9. خريطة الطريق — المُنجز
Week 27: ✅ SQLite persistence (387 test)

Phase 4: التحليل الحقيقي (Weeks 28-34) +114 test → 517
Phase 5: Bayesian Intelligence (Weeks 35-42) +89 test → 606
Phase 6: Governance + API + CLI (Weeks 43-56) +56 test → 662

النهاية: Week 56 · 11 crate · 662 test · 0 heuristic زائفة

text

---

## 10. أوامر الصيانة

```bash
# فحص البنية
cargo check --workspace

# فحص clippy (يشمل الاختبارات)
cargo clippy --workspace --tests -- -D warnings

# تشغيل كل الاختبارات
cargo test --workspace

# تشغيل مع اختبارات Docker
cargo test --workspace --features ec-sandbox/slow_tests

# تشغيل الخادم
cargo run -p ec-api

# تحليل ملف
cargo run -p ec-cli -- analyze path/to/file.rs
11. ملخص سريع
text
11 crates · ~110 ملف .rs · ~22,000 سطر
662 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
10 design invariants (D1-D10)
12 ADRs (004-020)
6 Phases — 56 Weeks — مكتملة

Kernel purity: محفوظ (لا async، لا HTTP، لا serde DTOs)
Semantic boundaries: صارمة (ADR-020)
Constitutional primacy: مضمون (D6)
Append-only memory: مضمون (D1)
Bayesian intelligence: مدمج (D9-D10)
نهاية الوثيقة المرجعية — Engineering Civilization v1.5 Complete
