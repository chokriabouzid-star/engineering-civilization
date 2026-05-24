# Engineering Civilization — الوثيقة المرجعية
## v1.6 · Week 56+ · 662 tests · Production Ready

---

## 1. نظرة عامة
11 crates · ~110 ملف .rs · ~22,000 سطر
662 tests · 0 failed · 16 ignored
0 clippy warnings (مع --tests)
0 unwrap() في كود الإنتاج
10 design invariants · 12 ADRs
ec check: 71/129 files passing · score 0.874
pre-commit hook: نشط على sel-agent-v4

---

## 2. ما الجديد في v1.6

### الإصلاحات الثلاثة (بعد الاستخدام الفعلي)

| # | الإصلاح | المشكلة | الأثر |
|---|---------|---------|-------|
| 1 | `CouplingVisitor` depth tracking | يعدّ sub-paths كـ external → Stability=0.000 زائف | 4 → 71 ملفاً ✅ |
| 2 | `TestVisitor` no-fn case | ملفات بدون دوال → Coverage=0.500 زائف | false positives اختفت |
| 3 | `w31_no_fns_coverage_neutral` | توقع قيمة قديمة 0.5 | 0 failed |

### التحقق من الإصلاحات

```bash
# قبل: Stability=0.000 على كل ملفات الـ kernel
# بعد: Stability منطقي
ec analyze crates/ec-constitutional/src/constitution.rs
# Stability: 0.610 ✅

ec analyze crates/ec-memory/src/graph.rs
# Stability: 0.400 (منطقي — يعتمد على uuid+chrono+rusqlite)
```

### النشر الفعلي

```bash
# ec متاح من أي مكان
export PATH="$PATH:/home/chokribouzid/projects/engineering-civilization/target/release"
# في ~/.bashrc

# pre-commit hook نشط على sel-agent-v4
# يرفض: reversibility < 0.30 أو security < 0.70
```

---

## 3. الـ Crates — مرجع كامل

### 3.1 `ec-fitness` — اللياقة الدستورية

**الدور:** تعريف FitnessVector + عتبات الكارثة + Pareto + Cosine similarity

**الأنواع:**
- `FitnessVector` — 6 أبعاد [0,1]: security, reversibility, test_coverage, maintainability, performance, architectural_stability
- `CatastropheThresholds` — عتبات لكل بُعد
- `CatastrophicDimension` — enum للبُعد المتسبب بالكارثة
- `ParetoOrdering` — Dominates | Dominated | Equal | NonDominated

**الـ methods:**
- `validate()` — كل بُعد finite وفي [0,1]
- `cosine_similarity(&other) -> f64`
- `cosine_angle_degrees(&other) -> f64`
- `pareto_compare(&other) -> ParetoOrdering`

---

### 3.2 `ec-epistemic` — الحالة المعرفية

**الدور:** تتبع الثقة والدليل والمعايرة + Bayesian inference

**الأنواع الأساسية:**
- `EpistemicState` — confidence + evidence + uncertainty + calibration
- `Evidence` — sample_size + age_seconds + reproducibility + source_reliability
- `UncertaintyDecomposition` — aleatoric + epistemic + model
- `CalibrationState` — 10 bins + ECE

**الأنواع Bayesian:**
- `BayesianEvidence` — successes + failures + mean_score + variance_estimate
  - `initial_prior()` — prior غير متحيز (0,0,0.5,0.8)
  - `update_with_outcome(correct, score)` — Bayesian update
  - `credible_confidence()` — Wilson score interval
- `BayesianCalibration` — WellCalibrated | Overconfident | Underconfident | InsufficientData

---

### 3.3 `ec-constitutional` — المحرك الدستوري

**الدور:** تقييم الكود ضد 8 ثوابت دستورية

**الثوابت المُدمجة:**
1. SecurityInvariant — security ≥ 0.70
2. ReversibilityInvariant — reversibility ≥ 0.30
3. TestCoverageInvariant — test_coverage ≥ 0.60
4. MaintainabilityInvariant — maintainability ≥ 0.40
5. PerformanceInvariant — performance ≥ 0.20
6. ArchitecturalStabilityInvariant — architectural_stability ≥ 0.50
7. CatastrophePreventionInvariant
8. EpistemicConfidenceInvariant — confidence ≥ 0.30

---

### 3.4 `ec-sandbox` — بيئة التنفيذ

**الدور:** تنفيذ الكود في Docker + قياس RealityVector + Bayesian tracking

**الأنواع:**
- `SandboxExecutor`, `SandboxConfig`, `SandboxMode`
- `RealityVector` — correctness, reproducibility, empirical_confidence
- `RealityFeedback`, `PredictionError`, `PredictionRecord`
- `BayesianTracker` — outcomes per artifact

---

### 3.5 `ec-analysis` — التحليل الثابت ⭐ (محدّث في v1.6)

**الدور:** تحليل كود Rust → FitnessVector + ConfidenceVector

**الواجهات:**
- `analyze_code(code: &str) -> FitnessVector` — قديمة، لا تتغير
- `analyze_code_full(code: &str) -> AnalysisReport` — syn AST

**الأنواع:**
- `AnalysisReport` — {fitness, confidence, warnings, parse_successful}
- `ConfidenceVector` — 6 أبعاد + `overall()` (min)
- `AnalysisWarning` — ParseFailed | LowConfidence | UnsafeWithoutComment | HighComplexity

**6 AST Visitors:**

| Visitor | يقيس | Confidence |
|---------|------|-----------|
| `UnsafeVisitor` | unsafe blocks/fns + SAFETY comment discount | 0.95 (نظيف) / 0.90 (unsafe) |
| `ComplexityVisitor` | cyclomatic complexity | 0.80 (لا دوال) / 0.88 |
| `TestVisitor` ⭐ | test fns vs production fns | 0.30 (لا دوال إنتاج) / 0.25 (لا tests) / 0.50 |
| `CouplingVisitor` ⭐ | use statements — root level فقط | 0.75 |
| `SideEffectVisitor` | println, static mut | 0.70 |
| `PerformanceVisitor` | allocations + clones | 0.75 |

**تفاصيل الإصلاحات (v1.6):**

```rust
// CouplingVisitor — depth tracking (الإصلاح الرئيسي)
impl<'ast> Visit<'ast> for CouplingVisitor {
    fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
        if self.depth == 0 {  // ← root level فقط
            match node.ident.to_string().as_str() {
                "std" | "core" | "alloc" => self.std_uses += 1,
                s if s.starts_with("ec_") => {}  // workspace — معفى
                _ => self.external_uses += 1,
            }
        }
        self.depth += 1;
        syn::visit::visit_use_path(self, node);
        self.depth -= 1;
    }
}

// TestVisitor — لا دوال إنتاج = ليست مشكلة
pub fn score(&self) -> (f64, f64) {
    if self.production_fns == 0 {
        return (1.0, 0.30);  // ← tests في files منفصلة
    }
    // ...
}
```

**Calibration Dataset:**
tests/fixtures/tier1_excellent/pure_math.rs  → score ≥ 0.75
tests/fixtures/tier4_bad/unsafe_mess.rs      → score ≤ 0.50

---

### 3.6 `ec-memory` — الذاكرة السببية

**الدور:** DAG + استعلامات + انجراف + Bayesian queries + SQLite

**الأنواع الأساسية:**
- `CausalMemoryGraph` — append-only DAG
- `DecisionNode`, `DecisionNodeBuilder` (D4)
- `NodeId`, `ArtifactSnapshot` (Arc<str>)
- `SandboxOutcome`, `RetrospectiveAssessment`
- `MemoryQuery`, `SimilarDecision`
- `HistoricalDriftAnalyzer`, `DriftReport`
- `DriftClassification` — Stable | LearningProgress | ValueShift | Corruption | InsufficientData
- `SqliteStorage`, `MemoryStorage` trait (D7)

**الأنواع Bayesian:**
- `OutcomeStorage` trait — record_outcome + load_evidence
- `BayesianQuery<S>` — find_similar_with_confidence + best_by_confidence
- `BayesianSimilarDecision` — similarity + bayesian_confidence + combined

**SQLite Schema:**
```sql
decisions, retrospective_assessments, bayesian_outcomes
```

---

### 3.7 `ec-codegen` — توليد الكود

**الأنواع:**
- `CodeGenerator`, `GenerationSpec`, `GenerationResult`
- `CodeTemplate` trait, `GenerationSuccess`, `FailureContext`

---

### 3.8 `ec-governance` — الحوكمة الدستورية

**الأنواع:**
- `ConstitutionalProposal` — Pending→Approved→Applied
- `ProposalOrigin` — Human | System {trigger}
- `ProposedChange` — AdjustThreshold | AddInvariant | RemoveInvariant | UpdatePolicy
- `ProposalStore` — append-only
- `AuditLog` — append-only, لا delete()
- `GovernanceEvent` — كل أحداث الحوكمة
- `GovernanceStorage` — SQLite
- `drift_trigger::propose_from_drift()` — ربط تلقائي

---

### 3.9 `ec-api` — REST API

**11 endpoint:**
POST   /api/v1/analyze
POST   /api/v1/governance/proposals
GET    /api/v1/governance/proposals
PATCH  /api/v1/governance/proposals/:id/approve
PATCH  /api/v1/governance/proposals/:id/reject
GET    /api/v1/governance/audit
GET    /api/v1/memory/nodes
GET    /api/v1/memory/drift
GET    /api/v1/memory/similar
GET    /api/v1/health

**JSON Schema للـ analyze:**
```json
{
  "fitness": {
    "security": 1.0,
    "test_coverage": 0.5,
    "maintainability": 0.95,
    "performance": 1.0,
    "architectural_stability": 0.73,
    "reversibility": 1.0
  },
  "confidence": { "overall": 0.50 },
  "parse_successful": true,
  "warnings": 0
}
```

---

### 3.10 `ec-cli` — Command Line Interface

**الأوامر:**
```bash
ec analyze <path> [--json] [--verbose]
ec drift [--baseline N] [--window M]
ec propose submit|list|approve
ec audit [--limit N]
ec health
ec check <dir> [--json]        ← workspace-level analysis
```

**الـ PATH:**
```bash
# ~/.bashrc
export PATH="$PATH:/home/chokribouzid/projects/engineering-civilization/target/release"
```

---

### 3.11 `ec-app` — التطبيق الرئيسي

**الأنواع:**
- `IntegrationPipeline`, `IterativePipeline`, `BayesianPipeline`
- `PipelineVerdict`, `PipelineResult`, `BayesianPipelineResult`
- `build_epistemic_from_fitness()`, `build_epistemic_from_bayesian()`

---

## 4. Dependency Graph
ec-fitness          ← (لا شيء)
ec-epistemic        ← (لا شيء)
ec-constitutional   ← ec-fitness, ec-epistemic
ec-sandbox          ← ec-fitness, ec-constitutional, ec-epistemic
ec-analysis         ← ec-fitness, syn
ec-memory           ← ec-fitness, ec-epistemic, rusqlite
ec-codegen          ← ec-fitness, ec-memory
ec-governance       ← ec-fitness, ec-constitutional, ec-memory, ec-epistemic
ec-app              ← الجميع
ec-api              ← ec-app, ec-governance, ec-analysis, ec-memory
ec-cli              ← ec-analysis, ec-governance, ec-memory

---

## 5. ثوابت التصميم (D1-D10)

| # | الاسم | الوصف | المكان |
|---|-------|-------|--------|
| D1 | Append-Only Memory | لا delete/update_fitness/clear | ec-memory |
| D2 | Truth ≠ Fitness | FitnessVector ≠ RealityVector | ec-sandbox + ec-analysis |
| D3 | DAG Enforcement | validate_acyclic() قبل record() | ec-memory |
| D4 | Builder Pattern | DecisionNode::new() = pub(crate) | ec-memory |
| D5 | Single Similarity Source | cosine_similarity() وحيد | ec-fitness |
| D6 | Constitutional Primacy | فشل دستوري = رفض نهائي | ec-constitutional |
| D7 | Persistent Memory | SQLite roundtrip guarantee | ec-memory |
| D8 | Confidence Separate | ConfidenceVector ≠ FitnessVector | ec-analysis + ec-epistemic |
| D9 | Bayesian Primacy | N≥10 → credible_confidence() | ec-epistemic |
| D10 | Outcome Transparency | raw score يُخزَّن قبل أي تحويل | ec-memory |

---

## 6. مبدأ الحدود الدلالية (ADR-020)
Kernel (لا async، لا HTTP، لا serde DTOs):
ec-fitness, ec-epistemic, ec-constitutional
ec-sandbox, ec-analysis, ec-memory, ec-codegen
Interface layer:
ec-governance  ← منطق حوكمة sync
ec-api         ← HTTP adapter (axum)
ec-cli         ← CLI adapter (clap)
ممنوعات:
❌ لا async في kernels
❌ لا serde DTOs داخل kernels
❌ لا business logic في API handlers
❌ لا database types داخل fitness/epistemic

---

## 7. pre-commit Hook

```bash
# sel-agent-v4/.git/hooks/pre-commit
#!/usr/bin/env bash
EC=/home/chokribouzid/projects/engineering-civilization/target/release/ec
FAILED=0

for file in $(git diff --cached --name-only -- '*.rs'); do
    [ -f "$file" ] || continue
    result=$($EC analyze "$file" --json 2>/dev/null)
    [ -z "$result" ] && continue

    rev=$(echo "$result" | jq -r '.fitness.reversibility // 1')
    sec=$(echo "$result" | jq -r '.fitness.security // 1')

    if [ "$(echo "$rev < 0.30" | bc -l)" = "1" ]; then
        echo "❌ EC: $file — reversibility=$rev (min: 0.30)"
        FAILED=1
    fi
    if [ "$(echo "$sec < 0.70" | bc -l)" = "1" ]; then
        echo "❌ EC: $file — security=$sec (min: 0.70)"
        FAILED=1
    fi
done

[ "$FAILED" = "1" ] && echo "💡 Run: ec analyze <file> --verbose" && exit 1
exit 0
```

**مثال عملي:**
sel-agent-v4/src/llm/mod.rs
reversibility = 0.16  ← 7 println! (user-facing output — مقبول)
test_coverage  = 0.00 ← لا #[test] في الملف
الـ hook لا يمنع mod.rs (0.16 > 0.30 لا — wait, 0.16 < 0.30!)
لكن العتبة في الـ hook هي 0.30 → هذا الملف سيُرفض إذا staged
القرار: استثناء llm/mod.rs أو تحسين الكود

---

## 8. نتائج ec check (v1.6)
ec check /path/to/project
Files scanned:  129
Files passed:    71  ✅  (كانت 4 في v1.5)
Files failed:    58  ❌
Project score: 0.874  (كانت 0.774 في v1.5)

**الملفات الفاشلة المتبقية (طبيعية):**
- ملفات الاختبارات `tests/*.rs` — تستخدم println! + imports كثيرة (طبيعي)
- ملفات تعتمد على external crates كثيرة (graph.rs, pipeline.rs) — stability أقل من 0.50 لكن صحيح

---

## 9. قرارات التصميم (ADRs)

| ADR | العنوان | المرحلة |
|-----|---------|---------|
| 004-019 | Architecture, Sandbox, Memory, etc. | Phase 1-5 |
| 020 | Semantic Boundary Protection | Phase 6 |

---

## 10. الاختبارات — الإحصائيات النهائية

| Phase | Weeks | المحتوى | Tests |
|-------|-------|---------|-------|
| 1 | 1-6 | Fitness + Epistemic + Constitutional | ~90 |
| 2 | 7-18 | Sandbox + Integration | ~150 |
| 3 | 19-27 | Analysis + Memory + Codegen + SQLite | ~163 |
| 4 | 28-34 | syn AST + ConfidenceVector | +114 |
| 5 | 35-42 | Bayesian Intelligence | +89 |
| 6 | 43-56 | Governance + API + CLI | +56 |
| **v1.6** | **Post-56** | **Bug fixes من الاستخدام الفعلي** | **662 (unchanged)** |

---

## 11. أوامر الصيانة

```bash
# فحص البنية
cargo check --workspace

# clippy
cargo clippy --workspace --tests -- -D warnings

# اختبارات
cargo test --workspace

# مع Docker
cargo test --workspace --features ec-sandbox/slow_tests

# تحليل ملف
ec analyze src/main.rs
ec analyze src/main.rs --json
ec analyze src/main.rs --verbose

# فحص مشروع كامل
ec check /path/to/project
ec check /path/to/project --json | jq '{passed, failed, score}'

# تشغيل الخادم
EC_DB=/data/ec.db ec-server

# فحص grep بدون bash history issues
grep -cF 'println!' src/file.rs
```

---

## 12. ملخص سريع
v1.5 → v1.6: 3 إصلاحات من الاستخدام الفعلي
✅ CouplingVisitor: depth tracking (workspace crates معفاة)
✅ TestVisitor: no-fn → (1.0, 0.30)
✅ ec check: 4/129 → 71/129 · score 0.774 → 0.874
النظام:
11 crates · ~22,000 سطر · 662 tests · 0 failed
10 design invariants · 12 ADRs
pre-commit hook نشط على sel-agent-v4
ec في PATH — يعمل من أي مكان
الدرس: الاستخدام الفعلي كشف bugs لم تكتشفها 662 اختباراً

---

*نهاية الوثيقة المرجعية — Engineering Civilization v1.6*
