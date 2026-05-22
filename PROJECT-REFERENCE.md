# Engineering Civilization — الوثيقة المرجعية
## Phase 4 Complete · Week 34 · 517 tests

---

## 1. نظرة عامة
8 crates · ~80 ملف .rs · ~16,000 سطر
517 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
8 design invariants · 11 ADRs
Phase 4 مكتمل: syn AST + ConfidenceVector

text


---

## 2. الـ Crates

### `ec-fitness` — اللياقة الدستورية
المسؤولية: تعريف FitnessVector + عتبات الكارثة + Pareto
يعتمد على: (لا شيء — الأساس)
يُستخدم من: كل crate آخر

text


**الأنواع:**
- `FitnessVector` — 6 أبعاد: security, reversibility, test_coverage, maintainability, performance, architectural_stability
- `CatastropheThresholds` — عتبات لكل بُعد (defaults: security=0.70, reversibility=0.30, test_coverage=0.60, maintainability=0.40, performance=0.20, architectural_stability=0.50)
- `CatastrophicDimension` — enum للبُعد المتسبب بالكارثة

**الـ methods على FitnessVector:**
- `validate()` — كل بُعد finite وفي [0,1]
- `cosine_similarity(&other)` — التشابه (1.0=متطابق)
- `cosine_angle_degrees(&other)` — الزاوية بالدرجات (0°=متطابق)
- `magnitude()` — طول المتجه (private)

### `ec-epistemic` — الحالة المعرفية
المسؤولية: تتبع الثقة والدليل والمعايرة
يعتمد على: (لا شيء)
يُستخدم من: ec-constitutional, ec-app

text


**الأنواع:**
- `EpistemicState` — confidence + evidence + uncertainty + calibration
- `Evidence` — successes, failures, mean_score, variance_estimate
- `UncertaintyDecomposition` — aleatoric, epistemic, model_error
- `CalibrationState` — overconfident/underconfident/history
- `EpistemicResult<T>` — Result<T, EpistemicError>

### `ec-constitutional` — المحرك الدستوري
المسؤولية: تقييم الكود ضد ثوابت دستورية
يعتمد على: ec-fitness, ec-epistemic
يُستخدم من: ec-app

text


**الأنواع:**
- `ConstitutionalEngine` — يحمل Constitution ويُقيّم
- `Constitution` — مجموعة من Invariant
- `Invariant` trait — `evaluate(&FitnessVector) -> InvariantResult`
- `InvariantResult` — {is_valid, dimension, actual_value, threshold, message}
- `ConstitutionalEvaluation` — {is_valid, results, fitness, verdict}
- `ConstitutionalVerdict` — Compliant | Violated {violations}

**الثوابت المُدمجة:**
1. `SecurityInvariant` — security ≥ 0.70
2. `ReversibilityInvariant` — reversibility ≥ 0.30
3. `TestCoverageInvariant` — test_coverage ≥ 0.60
4. `MaintainabilityInvariant` — maintainability ≥ 0.40
5. `PerformanceInvariant` — performance ≥ 0.20
6. `ArchitecturalStabilityInvariant` — architectural_stability ≥ 0.50
7. `CatastrophePreventionInvariant` — لا بُعد تحت العتبة الكارثية
8. `EpistemicConfidenceInvariant` — confidence ≥ 0.30

### `ec-sandbox` — بيئة التنفيذ المعزولة
المسؤولية: تنفيذ الكود في Docker + قياس الواقع
يعتمد على: ec-fitness
يُستخدم من: ec-app

text


**الأنواع:**
- `SandboxExecutor` — ينشئ/يُشغّل Docker containers
- `SandboxConfig` — timeout, memory_limit, network_disabled, etc.
- `ExecutionResult` — status, stdout, stderr, metrics, duration
- `RealityVector` — correctness, reproducibility, empirical_confidence
- `SandboxOutcome` — نتيجة التنفيذ المُهيكلة
- `PredictionRecord` — predicted_validity + predicted_confidence
- `PredictionError` — validity_error + confidence_error
- `RealityFeedback` — يتعلم من التنبؤات ويقيس التحسن

### `ec-analysis` — التحليل الثابت
المسؤولية: تحليل كود Rust → FitnessVector + ConfidenceVector
يعتمد على: ec-fitness, syn
يُستخدم من: ec-app, ec-codegen

text


**الواجهات:**
- `analyze_code(code: &str) -> FitnessVector` — الواجهة القديمة (keyword heuristic) — لا تتغير
- `analyze_code_full(code: &str) -> AnalysisReport` — الواجهة الجديدة (syn AST)

**الأنواع (الجديدة — Phase 4):**
- `AnalysisReport` — {fitness, confidence, warnings, parse_successful}
- `ConfidenceVector` — 6 أبعاد: security, test_coverage, maintainability, performance, architectural_stability, reversibility
  - `overall()` — أدنى بُعد (min)
- `AnalysisWarning` — ParseFailed | LowConfidence | UnsafeWithoutComment | HighComplexity

**الـ AST Visitors (6):**
1. `UnsafeVisitor` — يكشف unsafe blocks/fns/impls/traits
   - doc comments تُخفّض العقوبة بنسبة 40%
   - (security_score, confidence) — confidence=0.95 إذا لا unsafe, 0.90 إذا وجد
2. `ComplexityVisitor` — cyclomatic complexity حقيقي
   - يعدّ: if, while, for, match arms (non-wildcard), &&, ||, ?
   - يفصل test fns عن production fns
   - (maintainability_score, confidence) — confidence=0.80 إذا لا دوال, 0.88 إذا وجد
3. `TestVisitor` — يعدّ #[test] fns مقابل production fns + assert macros
   - (coverage_ratio, confidence) — confidence=0.25 بدون tests, 0.50 مع tests
4. `CouplingVisitor` — يعدّ use statements (std vs external)
   - std/core/alloc: 0.03 penalty each
   - external: 0.12 penalty each
   - (stability_score, confidence=0.75)
5. `SideEffectVisitor` — يكشف println/eprintln, static mut
   - stdout writes: 0.12 penalty each
   - static mut: 0.24 penalty each (×2)
   - (reversibility_score, confidence=0.70)
6. `PerformanceVisitor` — يكشف allocations + clones
   - Vec::new, String::new, Box::new, HashMap::new, etc.: allocation
   - clone(), to_string(), to_owned(): clone
   - format!, vec! macros: allocation
   - 0.04 penalty per item, floor at 0.3
   - (performance_score, confidence=0.75)

**`AstAnalyzer`:**
- يُشغّل الـ 6 visitors على AST
- يُجمّع النتائج في AnalysisReport
- يُنتج warnings (UnsafeWithoutComment, HighComplexity>20, LowConfidence<0.50)

**الملفات القديمة (لا تزال تعمل):**
- `analyzer.rs` — `analyze_code()` القديمة (keyword counting)
- `security.rs`, `complexity.rs`, `coverage.rs`, `reversibility.rs`, `metrics.rs`

**Calibration Dataset:**
- `fixtures/tier1_excellent/pure_math.rs` — pure functions + 3 tests
- `fixtures/tier4_bad/unsafe_mess.rs` — unsafe + static mut + no tests + high complexity

### `ec-memory` — الذاكرة السببية
المسؤولية: تخزين قرارات التصميم في DAG + استعلامات + انجراف
يعتمد على: ec-fitness
يُستخدم من: ec-app, ec-codegen

text


**الأنواع:**
- `CausalMemoryGraph` — DAG من DecisionNode
- `DecisionNode` — id, artifact_id, artifact_snapshot, fitness, constitutional_valid, sandbox_outcome, causal_parents, created_at, retrospective_assessments
- `NodeId` — Uuid wrapper
- `ArtifactSnapshot` — code_hash + code (optional)
- `DecisionNodeBuilder` —Builder pattern (D4)
- `RetrospectiveAssessment` — actual_validity + assessment_reason + assessed_at
- `SandboxOutcome` — correctness + reproducibility + empirical_confidence
- `MemoryQuery` — استعلامات على الذاكرة
- `SimilarDecision` — node_id + similarity + fitness + was_accepted
- `HistoricalDriftAnalyzer` — تحليل الانجراف
- `DriftReport` — classification + angle + details
- `DriftClassification` — Stable | LearningProgress | ValueShift | HumanIntervention
- `MemoryStorage` trait — save/load
- `SqliteStorage` — تنفيذ SQLite
- `StorageError` — أخطاء التخزين

**CausalMemoryGraph methods:**
- `new()` → Self
- `record_from_builder(builder)` → Result<NodeId, MemoryError>
- `update_retrospective(node_id, assessment)` → Result<(), MemoryError>
- `get(&id)` → Option<&DecisionNode>
- `all()` → &Vec<DecisionNode>
- `len()`, `is_empty()`
- `validate_acyclic(&id, parents)` → Result<(), MemoryError>
- ❌ لا delete(), update_fitness(), clear() — D1

**MemoryQuery methods:**
- `counterfactual(id, fitness)` → CounterfactualResult
- `pareto_frontier()` → Vec<&DecisionNode>
- `find_successful_analogies(target, min_similarity, max_results)` → Vec<SimilarDecision>

**HistoricalDriftAnalyzer methods:**
- `analyze(graph, baseline, window)` → DriftReport
- `classify(angle, rejection_increase, pareto_improved)` → DriftClassification

**SqliteStorage:**
- `new(path)` → Result<Self, StorageError>
- `in_memory()` → Result<Self, StorageError>
- `save(&graph)` → Result<(), StorageError>
- `load()` → Result<CausalMemoryGraph, StorageError>
- Schema: `decisions` table + `retrospective_assessments` table
- `from_nodes(Vec<DecisionNode>)` — pub(crate) للتحميل

### `ec-codegen` — توليد الكود
المسؤولية: توليد كود Rust من مواصفات
يعتمد على: ec-fitness, ec-memory
يُستخدم من: ec-app

text


**الأنواع:**
- `CodeGenerator` — يُنشئ كود من GenerationSpec
- `GenerationSpec` — artifact_id + requirements + constraints + templates
- `GenerationResult` — Success(code) | Failure(reason)
- `CodeTemplate` trait — name + priority + generate(spec)
- `GenerationSuccess` — code + template_used + generation_time
- `FailureContext` — previous_attempts + failure_reasons

### `ec-app` — التطبيق الرئيسي
المسؤولية: integration pipeline + iterative pipeline
يعتمد على: كل crates الأخرى

text


**الأنواع:**
- `IntegrationPipeline` — pipeline كامل (analyze → evaluate → sandbox → verdict)
- `IterativePipeline` — تكرار حتى النجاح
- `PipelineVerdict` — Accepted | Rejected {reason}
- `PipelineResult` — verdict + fitness + eval + execution_result + node_id
- `IterativePipelineResult` — attempts + final_result + converged
- `AttemptRecord` — iteration + code + verdict + fitness

**Functions:**
- `build_epistemic_from_fitness(&FitnessVector)` → EpistemicResult<EpistemicState>
- `determine_verdict(&eval, &execution_result)` → PipelineVerdict

---

## 3. الـ Dependencies بين Crates
ec-fitness ← (لا شيء)
ec-epistemic ← (لا شيء)
ec-constitutional ← ec-fitness, ec-epistemic
ec-sandbox ← ec-fitness
ec-analysis ← ec-fitness, syn
ec-memory ← ec-fitness, rusqlite
ec-codegen ← ec-fitness, ec-memory
ec-app ← ec-fitness, ec-epistemic, ec-constitutional,
ec-sandbox, ec-analysis, ec-memory, ec-codegen

text


---

## 4. ثوابت التصميم (Design Invariants)

### D1: Append-Only Memory
CausalMemoryGraph لا تحتوي على delete(), update_fitness(), clear().
فقط retrospective assessments تُضاف.

text


### D2: Truth ≠ Fitness
FitnessVector = تقييم دستوري (تنبؤ) — من ec-analysis
RealityVector = نتيجة تنفيذ (حقيقة) — من ec-sandbox
لا يُخلط بينهما.

text


### D3: DAG Enforcement
validate_acyclic() تُنفَّذ قبل كل record().
3 حالات مرفوضة: CycleDetected, NodeNotFound

text


### D4: Builder Pattern for External Construction
DecisionNode::new() → pub(crate)
DecisionNodeBuilder → pub
id و created_at يُنشآن داخل .build() فقط.

text


### D5: Single Source of Truth for Similarity
FitnessVector::cosine_similarity() — الطريقة الوحيدة.
FitnessVector::cosine_angle_degrees() — الطريقة الوحيدة.

text


### D6: Constitutional Primacy
كل كود يُقيَّم دستورياً قبل القبول.
الفشل الدستوري = رفض نهائي.

text


### D7: Persistent Memory
SqliteStorage يحفظ كل قرار.
load() يُعيد نفس البيانات (roundtrip guarantee).

text


### D8: ConfidenceVector منفصل عن FitnessVector
ConfidenceVector يقيس ثقتنا في التحليل نفسه.
لا تُعدَّل FitnessVector — منفصل تماماً.
analyze_code() القديمة لا تتغير — فقط analyze_code_full() تُضيف.

text


---

## 5. تدفق البيانات

### تحليل الكود (Phase 4 — جديد)
code: &str
│
├── analyze_code() → FitnessVector (keyword heuristic — لا يتغير)
│
└── analyze_code_full() → AnalysisReport
│
├── syn::parse_str()
│ ├── Ok(ast) → AstAnalyzer::analyze_file(&ast)
│ │ │
│ │ ┌───────────┼───────────┬──────────────┬───────────┬──────────┐
│ │ │ │ │ │ │ │
│ │ UnsafeV ComplexV TestV CouplingV SideEffV PerfV
│ │ │ │ │ │ │ │
│ │ (sec,0.95) (maint,0.88) (cov,0.50) (stab,0.75) (rev,0.70) (perf,0.75)
│ │ │ │ │ │ │ │
│ │ └───────────┴───────────┴──────────────┴───────────┴──────────┘
│ │ │
│ │ AnalysisReport
│ │ {fitness, confidence, warnings, parse_successful: true}
│ │
│ └── Err(e) → AnalysisReport::unparseable(e)
│ {fitness: default, confidence: zero, warnings: [ParseFailed], parse_successful: false}

text


### تدفق القرار الواحد (IntegrationPipeline)
code: &str
│
├── analyze_code() → FitnessVector
│
├── build_epistemic_from_fitness()
│
├── ConstitutionalEngine.evaluate()
│
├── SandboxExecutor.execute()
│
├── determine_verdict()
│
├── DecisionNodeBuilder → memory.record_from_builder()
│
└── RealityFeedback.learn()
└── PipelineResult

text


### تدفق التكرار (IterativePipeline)
for iteration in 0..max_iterations
1. CodeGenerator.generate(spec) → code
2. analyze_code(code) → FitnessVector
3. build_epistemic_from_fitness()
4. ConstitutionalEngine.evaluate()
5. SandboxExecutor.execute()
6. determine_verdict()
7. memory.record_from_builder(...)
8. RealityFeedback.learn()
9. if Accepted → return
10. previous_node_id = Some(node_id)
→ next iteration

text


### تدفق الانجراف
CausalMemoryGraph
├── baseline (أول N قرار)
├── current (آخر M قرار)
│
├── average_fitness(baseline) → baseline_avg
├── average_fitness(current) → current_avg
│
├── angle = baseline_avg.cosine_angle_degrees(current_avg)
├── rejection_increase = current_rate - baseline_rate
├── pareto_improved = security↑ OR coverage↑
│
└── classify(angle, rejection_increase, pareto_improved)
├── Stable → None
├── LearningProgress → Monitor
├── ValueShift → ReviewConstitution
└── angle > 45° → HumanIntervention

text


### تدفق التخزين (SQLite)
CausalMemoryGraph
│
├── save(graph)
│ └── لكل node:
│ ├── INSERT OR IGNORE decisions
│ └── لكل retrospective:
│ └── INSERT retrospective_assessments
│
└── load()
├── Phase 1: SELECT decisions ORDER BY rowid → from_nodes()
└── Phase 2: SELECT retrospective_assessments → update_retrospective()

text


---

## 6. الاختبارات — خريطة كاملة

### حسب الـ Crate

| Crate | ملفات الاختبار | عدد التقريبي |
|-------|---------------|-------------|
| ec-fitness | (داخلي) | 0 |
| ec-epistemic | tests/ (8 ملفات) | ~15 |
| ec-analysis | lib.rs internal | ~10 |
| ec-analysis | week19_gate | 20 |
| ec-analysis | week28_gate | 14 |
| ec-analysis | week29_gate | 13 |
| ec-analysis | week30_gate | 17 |
| ec-analysis | week31_gate | 23 |
| ec-analysis | week32_gate | 13 |
| ec-analysis | week33_gate | 15 |
| ec-analysis | phase4_gate | 15 |
| ec-constitutional | tests/ (8 ملفات) | ~90 |
| ec-sandbox | tests/ (4 ملفات) | ~83 |
| ec-codegen | tests/ | ~14 |
| ec-memory | tests/ (4 ملفات) | ~75 |
| ec-memory | SQLite (week27) | 11 |
| ec-app | tests/ (4 ملفات) | ~37 |
| **المجموع** | | **~517** |

### حسب Phase Gate

| Gate | الملف | الاختبارات |
|------|-------|-----------|
| Week 3-7 | ec-constitutional/tests/ | ~48 |
| Week 9-11 | ec-constitutional/tests/ | ~13 |
| Week 13-16 | ec-sandbox/tests/ | ~53 |
| Week 17-18 | ec-app/tests/ | ~7 |
| Week 19 | ec-analysis/tests/week19_gate | 20 |
| Week 20-24 | ec-memory/tests/ | ~49 |
| Week 25 | ec-app/tests/week25_gate | 8 |
| Week 27 | ec-memory/tests/week27_gate | 11 |
| Phase 3 | ec-app/tests/phase3_gate | 18 |
| **Week 28** | **ec-analysis/tests/week28_gate** | **14** |
| **Week 29** | **ec-analysis/tests/week29_gate** | **13** |
| **Week 30** | **ec-analysis/tests/week30_gate** | **17** |
| **Week 31** | **ec-analysis/tests/week31_gate** | **23** |
| **Week 32** | **ec-analysis/tests/week32_gate** | **13** |
| **Week 33** | **ec-analysis/tests/week33_gate** | **15** |
| **Phase 4** | **ec-analysis/tests/phase4_gate** | **15** |

### Feature Flags

| Crate | Feature | الوصف |
|-------|---------|-------|
| ec-sandbox | `slow_tests` | يُفعّل اختبارات Docker (14 test) |

---

## 7. سجل الإصلاحات (Weeks 25-34)

### Week 25-27: Hardening + SQLite (18 إصلاح)
(محفوظ في Git سابقاً)

### Phase 4: Weeks 28-34 (7 خطوات)

| # | الأسبوع | الإصلاح | الملفات |
|---|---------|---------|---------|
| 19 | 28 | إضافة syn dependency | ec-analysis/Cargo.toml |
| 20 | 28 | AnalysisReport + ConfidenceVector | ec-analysis/src/report.rs |
| 21 | 28 | 6 AST visitors | ec-analysis/src/visitors/*.rs |
| 22 | 28 | AstAnalyzer يُجمّع الـ 6 visitors | ec-analysis/src/ast_analyzer.rs |
| 23 | 28 | analyze_code_full() — إضافة فقط | ec-analysis/src/lib.rs |
| 24 | 29 | UnsafeVisitor: doc comment يُخفّض العقوبة | visitors/unsafe_visitor.rs |
| 25 | 30 | ComplexityVisitor: cyclomatic complexity حقيقي | visitors/complexity_visitor.rs |
| 26 | 31 | TestVisitor + CouplingVisitor + SideEffectVisitor | visitors/*.rs |
| 27 | 32 | Calibration dataset: tier1 vs tier4 | tests/fixtures/ |
| 28 | 33 | PerformanceVisitor بدلاً من hardcoded 0.80 | visitors/performance_visitor.rs |
| 29 | 34 | Phase 4 Gate | tests/phase4_gate.rs |

---

## 8. قرارات التصميم (ADRs)

| ADR | العنوان | المسار |
|-----|---------|--------|
| 004 | Integration Architecture | docs/adr/ADR-004 |
| 009 | Sandbox Foundation | docs/adr/ADR-009 |
| 010 | Docker Execution Strategy | docs/adr/ADR-010 |
| 012 | Security Hardening | docs/adr/ADR-012 |
| 013 | Integration Pipeline | docs/adr/ADR-013 |
| 014 | Static Code Analysis | docs/adr/ADR-014 |
| 015 | Causal Memory | docs/adr/ADR-015 |
| 016 | Code Generation | docs/adr/ADR-016 |
| 017 | Iterative Pipeline | docs/adr/ADR-017 |
| 018 | Counterfactual Query | docs/adr/ADR-018 |
| 019 | Value Drift Enhanced | docs/adr/ADR-019 |

---

## 9. دليل الصيانة

### إضافة بُعد جديد لـ Fitness
1. أضف الحقل إلى `FitnessVector` في ec-fitness/src/fitness.rs
2. حدّث `validate()`, `magnitude()`, `cosine_similarity()`
3. حدّث `CatastropheThresholds` + `CatastrophicDimension`
4. حدّث `pareto_compare()` في ec-fitness/src/pareto.rs
5. أضف visitor جديد في ec-analysis/src/visitors/
6. حدّث `AstAnalyzer::analyze_file()` ليُشغّل الـ visitor
7. أضف بُعد إلى `ConfidenceVector` في ec-analysis/src/report.rs
8. أضف ثابت دستوري في ec-constitutional
9. حدّث SQLite schema في ec-memory/src/storage.rs
10. شغّل `cargo test --workspace`

### إضافة AST Visitor جديد
1. أنشئ `visitors/new_visitor.rs` — يُنفّذ `syn::visit::Visit`
2. أضفه إلى `visitors/mod.rs`
3. يُقدّم `score() -> (f64, f64)` — (fitness_score, confidence)
4. أضفه إلى `AstAnalyzer::analyze_file()`
5. أضف warning مناسب إن لزم
6. أضف اختبارات gate

### إضافة ثابت دستوري جديد
1. أنشئ struct يُنفّذ `Invariant` trait
2. أضفه إلى `Constitution::new()`
3. أضف اختبارات

### أوامر التحقق
```bash
cargo check --workspace
cargo clippy --workspace --tests -- -D warnings
cargo test --workspace
cargo test --workspace --features ec-sandbox/slow_tests
10. خريطة الطريق
المُنجز
Phase	Weeks	المحتوى	Tests
1	1-6	الدستور + اللياقة + المعرفية	~90
2	7-18	Sandbox + Integration + Feedback	~150
3	19-27	Analysis + Memory + Codegen + Drift + SQLite	~163
4	28-34	syn AST + ConfidenceVector + 6 Visitors	+114 → 517
التالي
Phase	Weeks	المحتوى	الهدف
5	35-42	Bayesian + RealityFeedback SQLite + Analogies	560+
6	43-56	Governance + API + CLI	620+
ملخص سريع
text

8 crates · ~80 ملف .rs · ~16,000 سطر
517 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
8 design invariants · 11 ADRs
Phase 4: syn AST + ConfidenceVector ✅
نهاية الوثيقة المرجعية — Phase 4 Complete
