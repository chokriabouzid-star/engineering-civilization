# Engineering Civilization — الوثيقة المرجعية
## Phase 5 Complete · Week 42 · 606 tests

---

## 1. نظرة عامة
8 crates · ~90 ملف .rs · ~19,000 سطر
606 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
8 design invariants · 11 ADRs
Phase 5 مكتمل: Bayesian Intelligence

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
- `CatastropheThresholds` — عتبات لكل بُعد
- `CatastrophicDimension` — enum
- `ParetoOrdering` — Dominates/Dominated/Equal/NonDominated

**Methods على FitnessVector:**
- `validate()` — كل بُعد finite وفي [0,1]
- `cosine_similarity(&other)` — التشابه (1.0=متطابق)
- `cosine_angle_degrees(&other)` — الزاوية بالدرجات
- `pareto_compare(&other)` → ParetoOrdering

### `ec-epistemic` — الحالة المعرفية
المسؤولية: تتبع الثقة والدليل والمعايرة + Bayesian
يعتمد على: (لا شيء)
يُستخدم من: ec-constitutional, ec-sandbox, ec-memory, ec-app

text


**الأنواع (القديمة):**
- `EpistemicState` — confidence + evidence + uncertainty + calibration
- `Evidence` — sample_size + age_seconds + reproducibility + source_reliability
- `UncertaintyDecomposition` — aleatoric + epistemic + model
- `CalibrationState` — 10 bins + ECE
- `CalibrationBin` — lower/upper/count/sum_predicted/sum_actual

**الأنواع (الجديدة — Phase 5):**
- `BayesianEvidence` — successes + failures + mean_score + variance_estimate
  - `initial_prior()` → unbiased prior (0, 0, 0.5, 0.8)
  - `from_history(s, f, mean)` → من تاريخ فعلي
  - `update_with_outcome(correct, score)` → Bayesian update
  - `credible_confidence()` → Wilson interval
  - `total_observations()` → successes + failures
- `BayesianCalibration` — تشخيص + تعديل
  - `diagnose(&CalibrationState)` → CalibrationDiagnosis
  - `adjust_confidence(conf, diagnosis)` → f64
  - `adjusted_credible_confidence(&BayesianEvidence, &CalibrationState)` → f64
- `CalibrationDiagnosis` — WellCalibrated | Overconfident | Underconfident | InsufficientData

### `ec-constitutional` — المحرك الدستوري
المسؤولية: تقييم الكود ضد ثوابت دستورية
يعتمد على: ec-fitness, ec-epistemic
يُستخدم من: ec-app

text


**الأنواع:**
- `ConstitutionalEngine` — يحمل Constitution ويُقيّم
- `Constitution` — مجموعة من Invariant
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

### `ec-sandbox` — بيئة التنفيذ المعزولة
المسؤولية: تنفيذ الكود + قياس الواقع + Bayesian tracking
يعتمد على: ec-fitness, ec-constitutional, ec-epistemic
يُستخدم من: ec-app

text


**الأنواع (القديمة):**
- `SandboxExecutor`, `SandboxConfig`, `ExecutionResult`
- `RealityVector`, `RealityFeedback`, `PredictionRecord`, `PredictionError`

**الأنواع (الجديدة — Phase 5):**
- `BayesianTracker` — يتتبع outcomes بـ BayesianEvidence
  - `new()` → prior غير متحيز
  - `record(correct, score)` → تسجيل نتيجة
  - `evidence()` → &BayesianEvidence
  - `credible_confidence()` → Wilson
  - `has_sufficient_data()` → ≥5 مشاهدات
  - `total_observations()` → u32

### `ec-analysis` — التحليل الثابت
المسؤولية: تحليل كود Rust → FitnessVector + ConfidenceVector
يعتمد على: ec-fitness, syn
يُستخدم من: ec-app, ec-codegen

text


**الواجهات:**
- `analyze_code(code: &str) -> FitnessVector` — keyword heuristic (لا يتغير)
- `analyze_code_full(code: &str) -> AnalysisReport` — syn AST

**الأنواع (Phase 4):**
- `AnalysisReport` — {fitness, confidence, warnings, parse_successful}
- `ConfidenceVector` — 6 أبعاد + `overall()` (min)
- `AnalysisWarning` — ParseFailed | LowConfidence | UnsafeWithoutComment | HighComplexity

**6 AST Visitors:**
1. `UnsafeVisitor` — unsafe blocks/fns + doc comment discount
2. `ComplexityVisitor` — cyclomatic complexity حقيقي
3. `TestVisitor` — #[test] fns + assert macros
4. `CouplingVisitor` — use statements (std vs external)
5. `SideEffectVisitor` — println/eprintln, static mut
6. `PerformanceVisitor` — allocations + clones

### `ec-memory` — الذاكرة السببية
المسؤولية: تخزين قرارات التصميم + outcomes + Bayesian queries
يعتمد على: ec-fitness, ec-epistemic, rusqlite
يُستخدم من: ec-app, ec-codegen

text


**الأنواع (القديمة):**
- `CausalMemoryGraph`, `DecisionNode`, `DecisionNodeBuilder`
- `MemoryQuery`, `SimilarDecision`, `CounterfactualGain`, `FitnessSnapshot`
- `SqliteStorage`, `MemoryStorage` trait
- `HistoricalDriftAnalyzer`, `DriftReport`, `DriftClassification`

**الأنواع (الجديدة — Phase 5):**
- `OutcomeStorage` trait — record_outcome + load_evidence + load_all_evidence + outcome_count
- `BayesianQuery<'a, S: OutcomeStorage>` — استعلامات مع Bayesian confidence
  - `find_similar_with_confidence(target, k)` → Vec<BayesianSimilarDecision>
  - `best_by_confidence(k)` → Vec<BayesianSimilarDecision>
- `BayesianSimilarDecision` — node_id + artifact_id + similarity + bayesian_confidence + combined + fitness + was_accepted + total_observations

**SqliteStorage extensions:**
- `in_memory_with_outcomes()` — مع bayesian_outcomes table
- `new_with_outcomes(path)` — مع bayesian_outcomes table
- `init_outcome_schema()` — يُضيف bayesian_outcomes table

### `ec-codegen` — توليد الكود
المسؤولية: توليد كود Rust من مواصفات
يعتمد على: ec-fitness, ec-memory
يُستخدم من: ec-app

text


**الأنواع:**
- `CodeGenerator`, `GenerationSpec`, `GenerationResult`, `CodeTemplate`

### `ec-app` — التطبيق الرئيسي
المسؤولية: integration pipelines
يعتمد على: كل crates الأخرى

text


**الأنواع (القديمة):**
- `IntegrationPipeline` — pipeline كامل
- `IterativePipeline` — تكرار حتى النجاح
- `PipelineVerdict`, `PipelineResult`, `IterativePipelineResult`, `AttemptRecord`

**الأنواع (الجديدة — Phase 5):**
- `BayesianPipeline` — pipeline مع Bayesian tracking + calibration
  - `new(constitution)` → Self
  - `run(artifact_id, code)` → BayesianPipelineResult
  - `tracker()` → &BayesianTracker
  - `calibration()` → &CalibrationState
- `BayesianPipelineResult` — run_id + bayesian_confidence + raw_confidence + calibration_diagnosis + total_observations + verdict
- `build_epistemic_from_bayesian(evidence, calibration)` → EpistemicResult<EpistemicState>

---

## 3. الـ Dependencies بين Crates
ec-fitness ← (لا شيء)
ec-epistemic ← (لا شيء)
ec-constitutional ← ec-fitness, ec-epistemic
ec-sandbox ← ec-fitness, ec-constitutional, ec-epistemic
ec-analysis ← ec-fitness, syn
ec-memory ← ec-fitness, ec-epistemic, rusqlite
ec-codegen ← ec-fitness, ec-memory
ec-app ← ec-fitness, ec-epistemic, ec-constitutional,
ec-sandbox, ec-analysis, ec-memory, ec-codegen

text


---

## 4. ثوابت التصميم (Design Invariants)

| # | الاسم | الوصف |
|---|-------|-------|
| D1 | Append-Only Memory | لا delete/update_fitness/clear في CausalMemoryGraph |
| D2 | Truth ≠ Fitness | FitnessVector ≠ RealityVector — لا يُخلط |
| D3 | DAG Enforcement | validate_acyclic() قبل كل record() |
| D4 | Builder Pattern | DecisionNode::new() = pub(crate), Builder = pub |
| D5 | Single Similarity Source | cosine_similarity() وحيد |
| D6 | Constitutional Primacy | فشل دستوري = رفض نهائي |
| D7 | Persistent Memory | SQLite roundtrip guarantee |
| D8 | Confidence Separate | ConfidenceVector/BayesianEvidence ≠ FitnessVector |

---

## 5. تدفق البيانات — Phase 5

### Bayesian Pipeline
code: &str
│
├── analyze_code() → FitnessVector
│
├── build_epistemic_from_bayesian(tracker.evidence, calibration)
│ ├── evidence.credible_confidence() → raw
│ ├── BayesianCalibration::adjusted_credible_confidence() → adjusted
│ └── EpistemicState { confidence: adjusted, ... }
│
├── ConstitutionalEngine.evaluate()
│
├── SandboxExecutor.execute()
│
├── determine_verdict()
│
├── tracker.record(was_correct, score)
│
├── calibration.record(predicted, actual)
│
├── BayesianCalibration::diagnose(calibration)
│
└── BayesianPipelineResult
{ bayesian_confidence, raw_confidence,
calibration_diagnosis, total_observations, verdict }

text


### Outcome Storage Flow
SqliteStorage.in_memory_with_outcomes()
│
├── record_outcome(artifact_id, was_correct, score)
│ └── INSERT INTO bayesian_outcomes
│
├── load_evidence(artifact_id)
│ ├── SELECT was_correct, score WHERE artifact_id = ?
│ └── BayesianEvidence::initial_prior() → update_with_outcome() × N
│
├── load_all_evidence()
│ └── SELECT was_correct, score → BayesianEvidence
│
└── outcome_count()
└── SELECT COUNT(*)

text


### Bayesian Query Flow
BayesianQuery::new(&graph, &storage)
│
├── find_similar_with_confidence(target, k)
│ ├── cosine_similarity(target, node.fitness) × N
│ ├── sort by similarity desc
│ ├── for each top-k:
│ │ ├── storage.load_evidence(artifact_id)
│ │ ├── evidence.credible_confidence() → bayesian_conf
│ │ └── combined = min(similarity, bayesian_conf)
│ └── Vec<BayesianSimilarDecision>
│
└── best_by_confidence(k)
├── for each node: storage.load_evidence(artifact_id)
├── sort by bayesian_confidence desc
└── Vec<BayesianSimilarDecision>

text


---

## 6. الاختبارات — خريطة كاملة

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
| Week 28-33 | ec-analysis/tests/week{28-33}_gate | 95 |
| Phase 4 | ec-analysis/tests/phase4_gate | 15 |
| Week 35 | ec-epistemic/tests/week35_gate | 11 |
| Week 36 | ec-sandbox/tests/week36_gate | 12 |
| Week 37 | ec-memory/tests/week37_gate | 10 |
| Week 38 | ec-memory/tests/week38_gate | 8 |
| Week 40 | ec-epistemic/tests/week40_gate | 14 |
| Week 41 | ec-app/tests/week41_gate | 9 |
| Phase 5 | ec-epistemic/tests/phase5_gate | 10 |
| Phase 5 | ec-app/tests/phase5_gate | 15 |
| **المجموع** | | **~606** |

---

## 7. سجل الإصلاحات — Phase 5

| # | الأسبوع | الإصلاح | الملفات |
|---|---------|---------|---------|
| 30 | 35 | BayesianEvidence — prior + update + Wilson | ec-epistemic/src/bayesian.rs |
| 31 | 36 | BayesianTracker — sandbox tracking | ec-sandbox/src/bayesian.rs |
| 32 | 37 | OutcomeStorage — SQLite per artifact | ec-memory/src/outcome_storage.rs |
| 33 | 38 | BayesianQuery — similar + confidence | ec-memory/src/bayesian_query.rs |
| 34 | 40 | BayesianCalibration — diagnose + adjust | ec-epistemic/src/bayesian_calibration.rs |
| 35 | 41 | BayesianPipeline — full integration | ec-app/src/pipeline.rs |
| 36 | 42 | Phase 5 Gate | ec-epistemic + ec-app tests |

---

## 8. خريطة الطريق

### المُنجز

| Phase | Weeks | المحتوى | Tests |
|-------|-------|---------|-------|
| 1 | 1-6 | الدستور + اللياقة + المعرفية | ~90 |
| 2 | 7-18 | Sandbox + Integration + Feedback | ~150 |
| 3 | 19-27 | Analysis + Memory + Codegen + Drift + SQLite | ~163 |
| 4 | 28-34 | syn AST + ConfidenceVector + 6 Visitors | +114 → 517 |
| **5** | **35-42** | **Bayesian Intelligence** | **+89 → 606** |

### التالي

| Phase | Weeks | المحتوى | الهدف |
|-------|-------|---------|-------|
| 6 | 43-56 | Governance + API + CLI | 660+ |

---

## ملخص سريع
8 crates · ~90 ملف .rs · ~19,000 سطر
606 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
8 design invariants · 11 ADRs
Phase 5: Bayesian Intelligence ✅

text


---

*نهاية الوثيقة المرجعية — Phase 5 Complete*
