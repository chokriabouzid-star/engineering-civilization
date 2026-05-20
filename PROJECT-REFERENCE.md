# Engineering Civilization — الوثيقة المرجعية

> **آخر تحديث:** Phase 3 Gate — Commit `1e6b56c`  
> **حالة المشروع:** 376 اختبار · 0 فاشل · 16 متجاهل · 0 clippy warning  
> **8 crates · 13,047 سطر · 6 ADRs**

---

## جدول المحتويات

1. [الرؤية والمبادئ](#1-الرؤية-والمبادئ)
2. [البنية المعمارية](#2-البنية-المعمارية)
3. [الـ Crates — مرجع كامل](#3-الـ-crates--مرجع-كامل)
4. [الأنواع الأساسية](#4-الأنواع-الأساسية)
5. [ثوابت التصميم (Design Invariants)](#5-ثوابت-التصميم-design-invariants)
6. [تدفق البيانات](#6-تدفق-البيانات)
7. [الاختبارات — خريطة كاملة](#7-الاختبارات--خريطة-كاملة)
8. [إصلاحات Week 25-26](#8-إصلاحات-week-25-26)
9. [قرارات التصميم (ADRs)](#9-قرارات-التصميم-adrs)
10. [دليل الصيانة](#10-دليل-الصيانة)
11. [خريطة الطريق](#11-خريطة-الطريق)

---

## 1. الرؤية والمبادئ

### الفكرة الأساسية
"نظام هندسي يتبنى قيمًا دستورية — يُولّد الكود، يُقيّمه، يُنفّذه،
يتعلم من الواقع، ويحتفظ بذاكرة سببية لقراراته."

text


### المبادئ الخمسة

| # | المبدأ | المعنى | أين يُطبَّق |
|---|--------|--------|-----------|
| 1 | **الدستور أولاً** | كل كود يُقيَّم دستورياً قبل القبول | `ec-constitutional` |
| 2 | **الحقيقة ≠ اللياقة** | Fitness تنبؤ، Reality حقيقة | `ec-sandbox` + `ec-epistemic` |
| 3 | **الماضي لا يتغيّر** | الذاكرة append-only | `ec-memory` |
| 4 | **التعلم من الخطأ** | Prediction error → calibration | `ec-sandbox::feedback` |
| 5 | **باريتو للأفضليات** | لا بُعد واحد يسيطر | `ec-fitness::pareto` |

---

## 2. البنية المعمارية

### الطبقات
┌─────────────────────────────────────────────────────────────┐
│ ec-app (Integration) │
│ IntegrationPipeline · IterativePipeline │
├──────────┬──────────┬──────────┬──────────┬────────────────┤
│ ec-codegen│ec-analysis│ec-memory│ec-sandbox│ec-constitutional│
│ توليد │ تحليل │ ذاكرة │ تنفيذ │ تقييم دستوري │
│ الكود │ ثابت │ سببية │ مُعزول │ │
├──────────┴──────────┴──────────┴──────────┴────────────────┤
│ ec-epistemic · ec-fitness │
│ حالة معرفية · متجه لياقة │
└─────────────────────────────────────────────────────────────┘

text


### Dependency Graph (فعلي من Cargo.toml)
ec-app
├── ec-fitness
├── ec-epistemic
├── ec-constitutional → ec-fitness, ec-epistemic
├── ec-sandbox → ec-fitness, ec-constitutional
├── ec-analysis → ec-fitness
├── ec-codegen
└── ec-memory → ec-fitness

ec-epistemic → (لا يعتمد على شيء داخلي)
ec-fitness → (لا يعتمد على شيء — الأساس)

text


**ملاحظة مهمة:** `ec-memory` لا يعتمد على `ec-constitutional` (حُذف الـ dependency في Week 24).

### Workspace Cargo.toml

```toml
[workspace]
members = [
    "crates/ec-fitness",
    "crates/ec-epistemic",
    "crates/ec-constitutional",
    "crates/ec-sandbox",
    "crates/ec-app",
    "crates/ec-analysis",
    "crates/ec-memory",
    "crates/ec-codegen",
]
resolver = "2"

[workspace.package]
version = "0.3.0"
edition = "2021"
license = "MIT"
3. الـ Crates — مرجع كامل
3.1 ec-fitness — متجه اللياقة
الدور: يُعرّف الأبعاد الستة للجودة ويُوفر عمليات باريتو وتشابه جيب التمام.

الملفات:

text

ec-fitness/src/
├── fitness.rs    ← FitnessVector + CatastropheThresholds
├── pareto.rs     ← ParetoOrdering + pareto_compare()
└── lib.rs        ← re-exports
FitnessVector:

Rust

pub struct FitnessVector {
    pub security: f64,                 // أمن
    pub reversibility: f64,           // قابلية العكس
    pub test_coverage: f64,           // تغطية الاختبارات
    pub maintainability: f64,         // قابلية الصيانة
    pub performance: f64,             // أداء
    pub architectural_stability: f64, // استقرار معماري
}
الطرق (على FitnessVector):

الطريقة	التوقيع	الوصف
validate()	&self -> Result<(), &'static str>	كل بُعد في [0, 1] ومحدود
cosine_similarity()	&self, &Self -> f64	تشابه جيب التمام (1.0=متطابق، 0.0=متعامد)
cosine_angle_degrees()	&self, &Self -> f64	الزاوية بالدرجات (0°=متطابق، 90°=متعامد)
magnitude()	&self -> f64	طول المتجه (private — fn ليس pub)
pareto_compare()	&self, &Self -> ParetoOrdering	مقارنة باريتو
ParetoOrdering:

Rust

pub enum ParetoOrdering {
    Dominates,    // a يسيطر على b (كل أبعاد a ≥ b وبعضها >)
    Dominated,    // a مُسيطَر عليه من b
    Equal,        // متساويان في كل الأبعاد
    NonDominated, // لا يسيطر أحدهما على الآخر
}
CatastropheThresholds (القيم الافتراضية):

Rust

min_security: 0.70,
min_reversibility: 0.30,
min_test_coverage: 0.60,
min_maintainability: 0.40,
min_performance: 0.20,
min_architectural_stability: 0.50,
التصديرات:

Rust

pub use fitness::{CatastropheThresholds, CatastrophicDimension, FitnessVector};
pub use pareto::ParetoOrdering;
لا اختبارات خاصة — يُختبر عبر crates أخرى.

3.2 ec-epistemic — الحالة المعرفية
الدور: يُمثّل عدم اليقين في التقييم — يفصل بين ما نعرفه وما لا نعرفه.

الملفات:

text

ec-epistemic/src/
├── state.rs        ← EpistemicState, Evidence, UncertaintyDecomposition
├── calibration.rs  ← CalibrationState, Brier score
├── decay.rs        ← تلاشي زمني للثقة
├── propagation.rs  ← ConservativeCombiner (دمج محافظ)
├── error.rs        ← EpistemicError
└── lib.rs          ← re-exports الكل
EpistemicState:

Rust

pub struct EpistemicState {
    pub confidence: f64,
    pub evidence: Evidence,
    pub uncertainty: UncertaintyDecomposition,
    pub calibration: CalibrationState,
}
Evidence:

Rust

pub struct Evidence {
    pub successes: u32,
    pub failures: u32,
    pub mean_score: f64,
    pub variance: f64,
}
impl Evidence {
    pub fn new(successes: u32, failures: u32, mean: f64, variance: f64) -> EpistemicResult<Self>;
    pub fn weakest(evidence: &[Evidence]) -> Option<Evidence>;
}
UncertaintyDecomposition:

Rust

pub struct UncertaintyDecomposition {
    pub aleatoric: f64,    // عشوائي (لا يُقلَّص)
    pub epistemic: f64,    // معرفي (يُقلَّص بالتعلم)
    pub model: f64,        // وجودي (حدود النموذج) — اسمه `model` ليس `ontological`
}
CalibrationState:

Rust

pub struct CalibrationState {
    pub predictions: Vec<(f64, bool)>,  // (predicted_confidence, was_correct)
    pub brier_score: f64,
}
ConservativeCombiner:

Rust

pub trait ConservativePropagation {
    fn combine(states: &[EpistemicState]) -> EpistemicResult<EpistemicState>;
}
// التنفيذ: يأخذ min(confidence)، weakest(evidence)، RSS(uncertainty)
الاختبارات: 18 (8 وحدة + 10 property-based)

3.3 ec-constitutional — المحرك الدستوري
الدور: يُقيّم الكود ضد مجموعة ثوابت دستورية.

الملفات:

text

ec-constitutional/src/
├── constitution.rs  ← Constitution struct
├── engine.rs        ← ConstitutionalEngine + EvaluationContext
├── evaluation.rs    ← ConstitutionalEvaluation
├── invariant.rs     ← Invariant trait + ViolationReport
├── security.rs      ← SecurityInvariant
├── coverage.rs      ← TestCoverageInvariant
├── reversibility.rs ← ReversibilityInvariant
├── type_safety.rs   ← TypeSafetyInvariant
├── verdict.rs       ← ConstitutionalVerdict
├── frontier.rs      ← Frontier distance
├── compare.rs       ← مقارنة تقييمات
├── meta.rs          ← OssificationDetector + ValueDriftDetector
├── policy.rs        ← سياسات
└── lib.rs           ← re-exports
Constitution:

Rust

pub struct Constitution {
    invariants: Vec<Arc<dyn Invariant>>,
    thresholds: CatastropheThresholds,
}
Invariant trait:

Rust

pub trait Invariant: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, fitness: &FitnessVector, epistemic: &EpistemicState) 
        -> Result<(), ViolationReport>;
}
الثوابت المتوفرة:

الثابت	يفحص
SecurityInvariant	أمن الكود (عتبة 0.70)
TestCoverageInvariant	تغطية الاختبارات (عتبة 0.60)
ReversibilityInvariant	قابلية العكس (عتبة 0.30)
TypeSafetyInvariant	أمن الأنواع
ConstitutionalEngine:

Rust

pub struct ConstitutionalEngine {
    constitution: Constitution,
    cache: Option<EvaluationCache>,
}
impl ConstitutionalEngine {
    pub fn with_default_cache(constitution: Constitution) -> Self;
    pub fn evaluate(&self, artifact_id: &str, hash: u64, fitness: &FitnessVector,
                    epistemic: &EpistemicState, ctx: &EvaluationContext) -> ConstitutionalEvaluation;
}
ConstitutionalEvaluation:

Rust

pub struct ConstitutionalEvaluation {
    pub is_valid: bool,
    pub explanation: String,
    pub catastrophic: Option<CatastrophicDimension>,
    pub epistemic: EpistemicState,
    pub frontier_distance: f64,
}
الاختبارات: 38 (17 وحدة + 21 تكامل) + benchmarks

Benchmarks: ec-constitutional/benches/engine_benchmarks.rs

3.4 ec-sandbox — بيئة التنفيذ المعزولة
الدور: يُنفّذ الكود في بيئة آمنة ويُعيد نتائج واقعية.

الملفات:

text

ec-sandbox/src/
├── executor.rs   ← SandboxExecutor (الواجهة الرئيسية)
├── config.rs     ← SandboxConfig, SandboxMode, ResourceLimits
├── compiler.rs   ← compile Rust code
├── docker.rs     ← Docker execution
├── hardened.rs   ← Hardened execution
├── security.rs   ← SecurityViolation + فحص
├── reality.rs    ← RealityVector + LatencyMeasurement
├── metrics.rs    ← ExecutionMetrics
├── feedback.rs   ← RealityFeedback + PredictionError
└── lib.rs        ← re-exports
SandboxMode:

Rust

pub enum SandboxMode {
    Simulated,  // محاكاة (للتطوير — الافتراضي)
    Docker,     // Docker (للإنتاج)
    Hardened,   // معزز أمنياً
}
SandboxConfig:

Rust

pub struct SandboxConfig {
    pub mode: SandboxMode,
    pub runs_for_reproducibility: usize,
    pub limits: ResourceLimits,
    pub max_execution_time: Duration,
}
impl SandboxConfig {
    pub fn new(mode: SandboxMode) -> Self;
    pub fn default() -> Self;  // Simulated
    pub fn validate(&self) -> Result<(), SandboxError>;
}
SandboxExecutor:

Rust

pub struct SandboxExecutor { config: SandboxConfig }
impl SandboxExecutor {
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError>;
    pub fn execute(&self, artifact_id: &str, code: &str) -> ExecutionResult;
}
ExecutionResult:

Rust

pub struct ExecutionResult {
    pub success: bool,
    pub is_secure: bool,  // method: fn is_secure(&self) -> bool
    pub reality: Option<RealityVector>,
    pub error_message: Option<String>,
    pub violations: Vec<SecurityViolation>,
}
RealityVector:

Rust

pub struct RealityVector {
    pub correctness: f64,
    pub reproducibility: f64,
    pub empirical_confidence: f64,
    pub runs_completed: usize,
    pub latency: Option<LatencyMeasurement>,
}
impl RealityVector {
    pub fn new(correctness, reproducibility, confidence, runs, latency) -> Result<Self, ...>;
    pub fn is_correct(&self) -> bool;          // correctness ≥ 0.99
    pub fn is_reproducible(&self) -> bool;     // reproducibility ≥ 0.95
    pub fn is_trustworthy(&self) -> bool;      // confidence ≥ 0.8
    pub fn dummy() -> Self;                    // للاختبارات
}
RealityFeedback:

Rust

pub struct RealityFeedback { records: Vec<PredictionRecord> }
impl RealityFeedback {
    pub fn new() -> Self;
    pub fn learn(&mut self, prediction: &PredictionRecord, reality: &RealityVector) -> PredictionError;
    pub fn is_improving(&self) -> bool;
    pub fn needs_constitutional_review(&self) -> bool;
    pub fn mean_validity_error(&self) -> f64;
}
PredictionError:

Rust

pub struct PredictionError {
    pub validity_error: f64,
    pub confidence_error: f64,
}
Feature flag: slow_tests — اختبارات Docker تُتجاهل افتراضياً.

الاختبارات: 68 وحدة + 15 تكامل (14 ignored بدون --features slow_tests)

3.5 ec-analysis — التحليل الثابت
الدور: يحلّل الكود بدون تنفيذ → يُنتج FitnessVector.

الملفات:

text

ec-analysis/src/
├── lib.rs          ← analyze_code() + كل الدوال
├── security.rs     ← SecurityMetrics (مستخدم داخلياً)
├── complexity.rs
├── coverage.rs
├── reversibility.rs
└── metrics.rs
الواجهة الوحيدة:

Rust

pub fn analyze_code(code: &str) -> FitnessVector
خوارزميات التقييم:

البُعد	الخوارزمية
security	1.0 - (unsafe×0.5 + unwrap×0.2 + expect×0.1 + panic×0.3).min(1.0)
reversibility	1.0 - (println + eprintln + static_mut + unsafe + Mutex).×0.15.min(1.0)
test_coverage	(#[test] count / fn count).min(1.0)، 0.5 إذا لا دوال
maintainability	1.0 - (branches × 0.1).min(1.0)
performance	1.0 - (alloc + Vec + String + Box + HashMap).×0.2.min(1.0)
architectural_stability	1.0 - (use count × 0.1).min(1.0)
الاختبارات: 10 وحدة + 10 تكامل (week19_gate)

3.6 ec-codegen — مولّد الكود
الدور: يُولّد كود Rust من مواصفات عالية المستوى.

الملفات:

text

ec-codegen/src/
├── generator.rs  ← CodeGenerator
├── template.rs   ← CodeTemplate trait
├── rust.rs       ← RustFunctionTemplate, RustPureTemplate, RustStructTemplate
├── spec.rs       ← GenerationSpec + FailureContext
├── result.rs     ← GenerationResult + GenerationSuccess
└── lib.rs
CodeGenerator:

Rust

pub struct CodeGenerator {
    templates: Vec<Box<dyn CodeTemplate>>,
}
impl CodeGenerator {
    pub fn new() -> Self;     // 3 templates مرتبة بالأولوية
    pub fn generate(&self, spec: &GenerationSpec) -> GenerationResult;
    pub fn template_count(&self) -> usize;
    pub fn template_names(&self) -> Vec<&'static str>;
}
impl Default for CodeGenerator;
CodeTemplate trait:

Rust

pub trait CodeTemplate: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u32;
    fn matches(&self, spec: &GenerationSpec) -> bool;
    fn generate(&self, spec: &GenerationSpec) -> GenerationResult;
}
القوالب (مرتبة بالأولوية التنازلية):

القالب	الأولوية	يطابق
RustPureTemplate	3	pure, no_side_effects في constraints
RustFunctionTemplate	2	أي spec (default fallback)
RustStructTemplate	1	struct, data في constraints
GenerationSpec:

Rust

pub struct GenerationSpec {
    pub function_name: String,
    pub input_types: Vec<String>,
    pub output_type: String,
    pub description: String,
    pub constraints: Vec<String>,
    pub include_tests: bool,
    pub previous_failures: Vec<FailureContext>,
}
impl GenerationSpec {
    pub fn simple(name, inputs, output) -> Self;
    pub fn with_description(self, desc) -> Self;
    pub fn with_failure(self, failure) -> Self;
    pub fn is_first_attempt(&self) -> bool;
    pub fn attempt_number(&self) -> usize;
    pub fn format_params(&self) -> String;  // "a: i32, b: String"
    pub fn requires_unsafe(&self) -> bool;
}
GenerationResult:

Rust

pub enum GenerationResult {
    Success(GenerationSuccess),
    Failed { reason: String },
}
impl GenerationResult {
    pub fn succeeded(&self) -> bool;
    pub fn code(&self) -> Option<&str>;
    pub fn success(&self) -> Option<&GenerationSuccess>;
}
GenerationSuccess:

Rust

pub struct GenerationSuccess {
    pub code: String,
    pub template_name: &'static str,
    pub generation_id: Uuid,
    pub attempt_number: usize,
}
الاختبارات: 14 (4 وحدة + 10 تكامل)

3.7 ec-memory — الذاكرة السببية
الدور: يحتفظ بسجل append-only لكل القرارات مع علاقات سببية.

الملفات:

text

ec-memory/src/
├── types.rs   ← NodeId, ArtifactSnapshot, RejectionReason, RetrospectiveAssessment
├── node.rs    ← DecisionNode, DecisionNodeBuilder, RejectedAlternative, SandboxOutcome
├── graph.rs   ← CausalMemoryGraph, MemoryError
├── query.rs   ← MemoryQuery, CounterfactualGain, FitnessSnapshot, SimilarDecision
├── drift.rs   ← HistoricalDriftAnalyzer, DriftReport, DriftClassification, DriftAction
└── lib.rs     ← re-exports الكل
الأنواع الأساسية (types.rs)
NodeId:

Rust

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);
impl NodeId {
    pub fn new() -> Self;          // Uuid::new_v4()
    pub fn from_uuid(uuid: Uuid) -> Self;
}
impl Display for NodeId {
    fn fmt(&self, f) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])  // أول 8 أحرف
    }
}
ArtifactSnapshot:

Rust

pub type ArtifactHash = u64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSnapshot {
    pub hash: ArtifactHash,
    #[serde(skip)]
    pub code: Arc<str>,       // shared ownership
}
impl ArtifactSnapshot {
    pub fn new(code: impl Into<String>) -> Self;  // DefaultHasher
    pub fn code(&self) -> &str;
}
ضمان: clone() يشارك Arc — Arc::ptr_eq(&s1.code, &s2.code) == true.

RejectionReason:

Rust

pub enum RejectionReason {
    CatastrophicFailure { dimension: String },
    ParetoDominated { dominated_by: NodeId },
    ConstitutionalViolation { violations: Vec<String> },
    SandboxFailure { correctness: f64 },
    MaxIterationsReached { attempts: usize },
}
RetrospectiveAssessment:

Rust

pub struct RetrospectiveAssessment {
    pub assessed_at: DateTime<Utc>,
    pub was_better_choice: bool,
    pub confidence: f64,       // [0, 1]
    pub reasoning: String,
}
impl RetrospectiveAssessment {
    pub fn new(was_better: bool, confidence: f64, reasoning: impl Into<String>) 
        -> anyhow::Result<Self>;  // يفحص confidence ∈ [0, 1]
}
العقدة (node.rs)
DecisionNode:

Rust

pub struct DecisionNode {
    pub id: NodeId,
    pub created_at: DateTime<Utc>,
    pub artifact_id: String,
    pub artifact: ArtifactSnapshot,
    pub fitness: FitnessVector,
    pub constitutional_valid: bool,
    pub sandbox_outcome: Option<SandboxOutcome>,
    pub alternatives: Vec<RejectedAlternative>,
    pub causal_parents: Vec<NodeId>,
    pub retrospective: Vec<RetrospectiveAssessment>,
}
قاعدة الإنشاء:

Rust

// داخل ec-memory فقط:
DecisionNode::new(...)  // pub(crate) — لا يُستدعى من خارج

// من خارج ec-memory:
DecisionNodeBuilder::new(artifact_id, artifact, fitness)
    .constitutional_valid(true)
    .sandbox_outcome(Some(...))
    .causal_parents(vec![...])
    .add_alternative(alt)

// ثم:
memory.record_from_builder(builder)  // id + created_at يُنشآن هنا
DecisionNodeBuilder:

Rust

pub struct DecisionNodeBuilder {
    pub artifact_id: String,
    pub artifact: ArtifactSnapshot,
    pub fitness: FitnessVector,
    pub constitutional_valid: bool,        // default: false
    pub sandbox_outcome: Option<SandboxOutcome>,
    pub causal_parents: Vec<NodeId>,       // default: vec![]
    pub alternatives: Vec<RejectedAlternative>,
}
impl DecisionNodeBuilder {
    pub fn new(artifact_id, artifact, fitness) -> Self;
    pub fn constitutional_valid(self, bool) -> Self;
    pub fn sandbox_outcome(self, Option) -> Self;
    pub fn causal_parents(self, Vec) -> Self;
    pub fn add_alternative(self, alt) -> Self;
    pub(crate) fn build(self) -> DecisionNode;  // يستدعي DecisionNode::new()
}
SandboxOutcome (في ec-memory):

Rust

pub struct SandboxOutcome {
    pub correctness: f64,
    pub reproducibility: f64,
    pub empirical_confidence: f64,
}
RejectedAlternative:

Rust

pub struct RejectedAlternative {
    pub id: NodeId,                     // يُنشأ تلقائياً
    pub artifact: ArtifactSnapshot,
    pub fitness: FitnessVector,
    pub reason: RejectionReason,
    pub rejected_at: DateTime<Utc>,     // Utc::now()
    pub retrospective: Vec<RetrospectiveAssessment>,
}
impl RejectedAlternative {
    pub fn new(artifact, fitness, reason) -> Self;
    pub fn add_retrospective(&mut self, assessment);
}
الرسم البياني (graph.rs)
CausalMemoryGraph:

Rust

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalMemoryGraph {
    nodes: Vec<DecisionNode>,  // append-only, ترتيب زمني
}
العمليات المسموحة:

العملية	النوع	التوقيع
new()	إنشاء	-> Self
len()	قراءة	-> usize
is_empty()	قراءة	-> bool
record()	كتابة (pub(crate))	(DecisionNode) -> Result<NodeId, MemoryError>
record_from_builder()	كتابة (pub)	(DecisionNodeBuilder) -> Result<NodeId, MemoryError>
update_retrospective()	كتابة استعادية	(NodeId, RetrospectiveAssessment) -> Result<(), MemoryError>
update_alternative_retrospective()	كتابة استعادية	(NodeId, NodeId, RetrospectiveAssessment) -> Result<(), MemoryError>
get()	قراءة	(NodeId) -> Option<&DecisionNode>
all()	قراءة	-> &[DecisionNode]
causal_chain()	قراءة	(NodeId) -> Vec<&DecisionNode>
decisions_for_artifact()	قراءة	(&str) -> Vec<&DecisionNode>
latest_n()	قراءة	(usize) -> &[DecisionNode]
العمليات الممنوعة (لا وجود لها في الكود):

text

delete() · update_fitness() · clear() · remove() · update_node()
MemoryError:

Rust

pub enum MemoryError {
    NodeNotFound(NodeId),
    CycleDetected { node: NodeId },
}
validate_acyclic() — التحقق من الدورات:

Rust

fn validate_acyclic(&self, node: &DecisionNode) -> Result<(), MemoryError> {
    // 1. node.id موجود بالفعل → CycleDetected (تسجيل مكرر)
    if self.get(node.id).is_some() {
        return Err(MemoryError::CycleDetected { node: node.id });
    }
    for parent_id in &node.causal_parents {
        // 2. parent == node.id → CycleDetected (إشارة ذاتية)
        if *parent_id == node.id {
            return Err(MemoryError::CycleDetected { node: node.id });
        }
        // 3. parent غير موجود → NodeNotFound
        if self.get(*parent_id).is_none() {
            return Err(MemoryError::NodeNotFound(*parent_id));
        }
    }
    Ok(())
}
الاستعلامات (query.rs)
MemoryQuery:

Rust

pub struct MemoryQuery<'a> {
    graph: &'a CausalMemoryGraph,
}
الطريقة	التوقيع	الوصف
new()	&CausalMemoryGraph -> Self	إنشاء
best_rejected_alternative()	(NodeId) -> Option<&RejectedAlternative>	أعلى security+coverage
counterfactual_gain()	(&FitnessVector, &FitnessVector) -> CounterfactualGain	هل البديل كان أفضل؟
fitness_evolution()	(&str) -> Vec<FitnessSnapshot>	تطور اللياقة لـ artifact
find_similar()	(&FitnessVector, usize) -> Vec<SimilarDecision>	أشبه k قرارات
cosine_similarity()	(&FitnessVector, &FitnessVector) -> f64	delegate إلى FitnessVector
CounterfactualGain:

Rust

pub enum CounterfactualGain {
    AlternativeWasBetter { dimensions_better: usize },
    ChoiceWasCorrect,
    NoMeaningfulDifference,
    TradeoffDependent,
}
FitnessSnapshot:

Rust

pub struct FitnessSnapshot {
    pub iteration: usize,
    pub fitness: FitnessVector,
    pub was_accepted: bool,
    pub created_at: DateTime<Utc>,
}
SimilarDecision:

Rust

pub struct SimilarDecision {
    pub node_id: NodeId,
    pub similarity: f64,
    pub fitness: FitnessVector,
    pub was_accepted: bool,
}
خوارزمية best_rejected_alternative:

text

max_by: security + test_coverage
خوارزمية counterfactual_gain:

text

pareto_compare(alternative, chosen)
  → Dominates → AlternativeWasBetter { count_better_dimensions }
  → Dominated → ChoiceWasCorrect
  → Equal → NoMeaningfulDifference
  → NonDominated → TradeoffDependent
خوارزمية find_similar:

text

1. لكل عقدة: cosine_similarity(target, node.fitness)
2. sort تنازلياً
3. take(k)
كشف الانجراف (drift.rs)
HistoricalDriftAnalyzer:

Rust

pub struct HistoricalDriftAnalyzer<'a> {
    memory: &'a CausalMemoryGraph,
    baseline_size: usize,
    current_window: usize,
}
خوارزمية analyze():

text

1. total = memory.len()
2. required = baseline_size + current_window
3. if total < required → InsufficientData
4. baseline = first baseline_size nodes
5. current = last current_window nodes
6. baseline_avg = average_fitness(baseline)
7. current_avg = average_fitness(current)
8. angle = baseline_avg.cosine_angle_degrees(current_avg)
9. baseline_rate = rejections / baseline_size
10. current_rate = rejections / current_window
11. rejection_increase = current_rate - baseline_rate
12. pareto_improved = current_avg.security > baseline_avg.security
                     OR current_avg.test_coverage > baseline_avg.test_coverage
13. classify(angle, rejection_increase, pareto_improved)
14. recommend(classification)
DriftClassification:

التصنيف	الشرط
Stable	زاوية < 10°
LearningProgress { angle }	زاوية ≥ 10° + باريتو تحسّن
ValueShift { angle }	زاوية ≥ 10° + بدون تحسن
Corruption { rejection_increase }	rejection_increase > 0.2
InsufficientData { available, required }	total < required
DriftAction:

الإجراء	متى
None	Stable أو InsufficientData
Monitor	LearningProgress
ReviewConstitution { reason }	ValueShift ≤ 45°
HumanIntervention { reason }	ValueShift > 45° أو Corruption
DriftReport:

Rust

pub struct DriftReport {
    pub drift_angle_degrees: f64,
    pub baseline_count: usize,
    pub current_count: usize,
    pub total_decisions: usize,
    pub classification: DriftClassification,
    pub recommended_action: DriftAction,
}
impl DriftReport {
    pub fn requires_action(&self) -> bool;  // None أو Monitor → false
}
average_fitness():

text

لكل بُعد: sum(values) / count
إذا فارغ → FitnessVector::default()
التصديرات (lib.rs):

Rust

pub use graph::{CausalMemoryGraph, MemoryError};
pub use node::{DecisionNode, DecisionNodeBuilder, RejectedAlternative, SandboxOutcome};
pub use query::{CounterfactualGain, FitnessSnapshot, MemoryQuery, SimilarDecision};
pub use types::{ArtifactHash, ArtifactSnapshot, NodeId, RejectionReason, RetrospectiveAssessment};
pub use drift::{DriftAction, DriftClassification, DriftReport, HistoricalDriftAnalyzer};
الاختبارات: 75 (26 وحدة + 49 تكامل)

3.8 ec-app — طبقة التكامل
الدور: يربط كل الطبقات في pipelines.

الملفات:

text

ec-app/src/
├── lib.rs       ← (فارغ تقريباً)
└── pipeline.rs  ← IntegrationPipeline + IterativePipeline + helpers
IntegrationPipeline (Week 17 — العقد الأصلي)
Rust

pub struct IntegrationPipeline {
    engine: ConstitutionalEngine,
    executor: SandboxExecutor,
    feedback: RealityFeedback,
}
التدفق:

text

Code → analyze_code() → FitnessVector
     → build_epistemic_from_fitness()
     → ConstitutionalEngine.evaluate()
     → SandboxExecutor.execute()
     → determine_verdict()
     → RealityFeedback.learn()
     → PipelineResult
الواجهة:

Rust

impl IntegrationPipeline {
    pub fn new_simulated(constitution, thresholds) -> anyhow::Result<Self>;
    pub fn run(&mut self, artifact_id: &str, code: &str) -> PipelineResult;
    pub fn is_improving(&self) -> bool;
    pub fn needs_review(&self) -> bool;
    pub fn mean_validity_error(&self) -> f64;
    pub fn sandbox_mode(&self) -> SandboxMode;
}
IterativePipeline (Week 22)
Rust

pub struct IterativePipeline {
    generator: CodeGenerator,
    engine: ConstitutionalEngine,
    executor: SandboxExecutor,
    memory: CausalMemoryGraph,
    feedback: RealityFeedback,
    max_iterations: usize,
}
التدفق التكراري:

text

For iteration in 0..max_iterations:
  1. Generate code from spec
  2. analyze_code() → FitnessVector
  3. build_epistemic_from_fitness()
  4. ConstitutionalEngine.evaluate()
  5. SandboxExecutor.execute()
  6. determine_verdict()
  7. DecisionNodeBuilder → memory.record_from_builder()
     - if !constitutional_valid → add RejectedAlternative (CatastrophicFailure)
  8. Track AttemptRecord
  9. RealityFeedback.learn()
  10. If Accepted → return
  11. Else → previous_node_id = Some(node_id), next iteration
الواجهة:

Rust

impl IterativePipeline {
    pub fn new(constitution, max_iterations) -> anyhow::Result<Self>;
    pub fn run(&mut self, spec: &GenerationSpec) -> IterativePipelineResult;
    pub fn memory(&self) -> &CausalMemoryGraph;
    pub fn memory_mut(&mut self) -> &mut CausalMemoryGraph;
    pub fn is_improving(&self) -> bool;
    pub fn learn_from_history(&mut self, node_id, assessment) -> Result<(), MemoryError>;
    pub fn trace_causal_chain(&self, node_id) -> Vec<&DecisionNode>;
}
الأنواع المشتركة
PipelineVerdict:

Rust

pub enum PipelineVerdict {
    Accepted,
    RejectedByReality { reason: String },
    RejectedByConstitution { reason: String },
    ExecutionFailed { reason: String },
    GenerationFailed { reason: String },
    RejectedAfterMaxAttempts { reason: String },
}
PipelineResult:

Rust

pub struct PipelineResult {
    pub run_id: Uuid,
    pub code: String,
    pub verdict: PipelineVerdict,
    pub execution: ExecutionResult,
    pub prediction_error: PredictionError,
    pub evaluation: ConstitutionalEvaluation,
}
impl PipelineResult {
    pub fn is_accepted(&self) -> bool;
    pub fn summary(&self) -> String;
}
AttemptRecord:

Rust

pub struct AttemptRecord {
    pub attempt_number: usize,
    pub node_id: NodeId,
    pub code: String,
    pub fitness: FitnessVector,
    pub is_constitutional: bool,
    pub sandbox_correctness: Option<f64>,
}
IterativePipelineResult:

Rust

pub struct IterativePipelineResult {
    pub accepted_node: Option<DecisionNode>,
    pub attempts: Vec<AttemptRecord>,
    pub total_iterations: usize,
    pub verdict: PipelineVerdict,
}
Helpers
determine_verdict():

text

1. !execution.is_secure() → ExecutionFailed
2. !evaluation.is_valid → RejectedByConstitution
3. !execution.success → RejectedByReality
4. reality.is_none() → ExecutionFailed
5. !reality.is_trustworthy() → RejectedByReality
6. → Accepted
build_epistemic_from_fitness():

text

avg = mean(all 6 dimensions)
confidence = avg.clamp(0.3, 0.95)
evidence = Evidence::new(1, 0, avg, 0.8)
uncertainty = UncertaintyDecomposition::new(0.2, 0.2, 0.1)
build_epistemic_from_reality() (pub):

text

confidence = reality.empirical_confidence.clamp(0.3, 0.95)
evidence = Evidence::new(1, 0, correctness, reproducibility)
uncertainty = UncertaintyDecomposition::new(0.1, 0.2, 0.1)
الاختبارات: 11 وحدة (7 integration + 4 iterative)

4. الأنواع الأساسية
ملخص الأنواع عبر Crates
النوع	Crate	الوصف
FitnessVector	ec-fitness	6 أبعاد [0,1]
CatastropheThresholds	ec-fitness	عتبات الكوارث
CatastrophicDimension	ec-fitness	البُعد الفاشل
ParetoOrdering	ec-fitness	نتيجة باريتو
EpistemicState	ec-epistemic	حالة معرفية
Evidence	ec-epistemic	أدلة (نجاح/فشل)
UncertaintyDecomposition	ec-epistemic	تحلل عدم اليقين
CalibrationState	ec-epistemic	حالة المعايرة
EpistemicError	ec-epistemic	خطأ نطاق
Constitution	ec-constitutional	مجموعة ثوابت
ConstitutionalEngine	ec-constitutional	محرك التقييم
ConstitutionalEvaluation	ec-constitutional	نتيجة التقييم
Invariant	ec-constitutional	trait للثوابت
ViolationReport	ec-constitutional	تقرير انتهاك
SandboxConfig	ec-sandbox	إعدادات التنفيذ
SandboxMode	ec-sandbox	Simulated/Docker/Hardened
ExecutionResult	ec-sandbox	نتيجة التنفيذ
RealityVector	ec-sandbox	حقيقة من تنفيذ
PredictionError	ec-sandbox	خطأ التنبؤ
NodeId	ec-memory	معرف فريد (UUID)
ArtifactSnapshot	ec-memory	كود + hash
ArtifactHash	ec-memory	u64 (DefaultHasher)
DecisionNode	ec-memory	عقدة قرار
DecisionNodeBuilder	ec-memory	باني العقد
RejectedAlternative	ec-memory	بديل مرفوض
SandboxOutcome	ec-memory	نتيجة sandbox (مبسطة)
RejectionReason	ec-memory	سبب الرفض
RetrospectiveAssessment	ec-memory	تقييم استعادي
MemoryError	ec-memory	خطأ الذاكرة
CounterfactualGain	ec-memory	نتيجة what-if
FitnessSnapshot	ec-memory	لقطة لياقة
SimilarDecision	ec-memory	قرار مشابه
DriftClassification	ec-memory	تصنيف الانجراف
DriftAction	ec-memory	إجراء موصى به
DriftReport	ec-memory	تقرير الانجراف
GenerationSpec	ec-codegen	مواصفات التوليد
GenerationResult	ec-codegen	نتيجة التوليد
GenerationSuccess	ec-codegen	تفاصيل النجاح
FailureContext	ec-codegen	سياق فشل سابق
PipelineVerdict	ec-app	حكم الـ pipeline
PipelineResult	ec-app	نتيجة تشغيل
AttemptRecord	ec-app	سجل محاولة
IterativePipelineResult	ec-app	نتيجة تكرارية
5. ثوابت التصميم (Design Invariants)
D1: Append-Only Memory
text

الماضي لا يتغيّر.
CausalMemoryGraph لا تحتوي على delete(), update_fitness(), clear().
فقط retrospective assessments تُضاف.
تُطبق compile-time: الـ methods غير موجودة.
D2: Truth ≠ Fitness
text

FitnessVector = تقييم دستوري (تنبؤ) — من ec-analysis
RealityVector = نتيجة تنفيذ (حقيقة) — من ec-sandbox
لا يُخلط بينهما.
PredictionError = |predicted_validity - actual_correctness|
D3: DAG Enforcement
text

العلاقات السببية تشكّل DAG.
validate_acyclic() تُنفَّذ قبل كل record().
3 حالات مرفوضة:
  - node.id موجود بالفعل → CycleDetected
  - parent == node.id → CycleDetected
  - parent غير موجود → NodeNotFound
D4: Builder Pattern for External Construction
text

DecisionNode::new() → pub(crate)
DecisionNodeBuilder → pub
record_from_builder() → pub
id و created_at يُنشآن داخل .build() فقط.
لا يمكن لأي crate خارجي التحكم فيهما.
D5: Single Source of Truth for Similarity
text

FitnessVector::cosine_similarity() — الطريقة الوحيدة.
FitnessVector::cosine_angle_degrees() — الطريقة الوحيدة.
FitnessVector::magnitude() — private.
لا تكرار في ملفات أخرى.
drift.rs يستخدم baseline_avg.cosine_angle_degrees(&current_avg).
query.rs يستخدم fitness.cosine_similarity(target).
D6: Constitutional Primacy
text

كل كود يُقيَّم دستورياً قبل القبول.
الفشل الدستوري = رفض نهائي.
determine_verdict() يفحص الأمن أولاً → الدستور → التنفيذ → الواقع.
لا استثناءات.
6. تدفق البيانات
تدفق القرار الواحد (IntegrationPipeline)
text

                    ┌──────────────┐
                    │  code: &str  │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ analyze_code │
                    └──────┬───────┘
                           │ FitnessVector
                    ┌──────▼──────────────┐
                    │ build_epistemic_from │
                    │     _fitness()       │
                    └──────┬──────────────┘
                           │ EpistemicState
              ┌────────────┼────────────┐
              │            │            │
     ┌────────▼───┐  ┌─────▼─────┐  ┌──▼──────────┐
     │            │  │Constitut- │  │   Sandbox   │
     │            │  │  ional    │  │  Executor   │
     │            │  │  Engine   │  │             │
     └────────────┘  └─────┬─────┘  └──┬──────────┘
                           │            │
                 ConstitutionalEval   ExecutionResult
                           │            │
                    ┌──────▼───────┐    │
                    │ determine_  │◄───┘
                    │  verdict()  │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ RealityFeed- │
                    │ back.learn() │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ PipelineResult│
                    └──────────────┘
تدفق التكرار (IterativePipeline)
text

┌─────────────────────────────────────────────────┐
│ for iteration in 0..max_iterations              │
│                                                  │
│  1. CodeGenerator.generate(spec) ──→ code       │
│  2. analyze_code(code) ──→ FitnessVector        │
│  3. build_epistemic_from_fitness()              │
│  4. ConstitutionalEngine.evaluate()             │
│  5. SandboxExecutor.execute()                   │
│  6. determine_verdict()                         │
│  7. DecisionNodeBuilder                         │
│     .constitutional_valid(eval.is_valid)        │
│     .sandbox_outcome(...)                       │
│     .causal_parents([previous_node_id])         │
│     → memory.record_from_builder()              │
│  8. RealityFeedback.learn()                     │
│  9. if Accepted → return                        │
│ 10. previous_node_id = Some(node_id)            │
│                                                  │
│ → next iteration                                │
└─────────────────────────────────────────────────┘
تدفق التعلم
text

PredictionRecord ──→ RealityFeedback.learn() ──→ PredictionError
     │                                          │
     │ predicted_validity                 validity_error
     │ predicted_confidence               confidence_error
     │                                    │
     │                              is_improving()
     │                              needs_constitutional_review()
     │                              mean_validity_error()
تدفق الانجراف
text

CausalMemoryGraph
    │
    ├── baseline (أول N قرار)
    ├── current (آخر M قرار)
    │
    ▼
average_fitness(baseline) ──→ baseline_avg.cosine_angle_degrees(current_avg)
average_fitness(current)  ──→ angle
                              │
                    rejection_increase = current_rate - baseline_rate
                    pareto_improved = security↑ OR coverage↑
                              │
                    classify(angle, rejection_increase, pareto_improved)
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
      Stable          LearningProgress      ValueShift
           │                  │                  │
      None            Monitor         ReviewConstitution
                                                      │
                                               angle > 45°?
                                                      │
                                               HumanIntervention
تدفق الاستعلامات
text

MemoryQuery
    │
    ├── find_similar(target, k)
    │   → لكل node: fitness.cosine_similarity(target)
    │   → sort تنازلياً
    │   → take(k)
    │
    ├── counterfactual_gain(chosen, alternative)
    │   → alternative.pareto_compare(chosen)
    │   → Dominates → AlternativeWasBetter
    │   → Dominated → ChoiceWasCorrect
    │   → Equal → NoMeaningfulDifference
    │   → NonDominated → TradeoffDependent
    │
    ├── fitness_evolution(artifact_id)
    │   → decisions_for_artifact()
    │   → enumerate → FitnessSnapshot
    │
    └── best_rejected_alternative(node_id)
        → node.alternatives
        → max_by(security + test_coverage)
7. الاختبارات — خريطة كاملة
حسب الـ Crate
Crate	وحدة	تكامل	المجموع
ec-fitness	0	0	0
ec-epistemic	8	10	18
ec-analysis	10	10	20
ec-constitutional	17	21	38
ec-sandbox	68	15	83
ec-codegen	4	10	14
ec-memory	49	26	75
ec-app	11	26	37
المجموع	167	118	~376
حسب الـ Week Gate
Gate	الملف	الاختبارات
Week 3	ec-constitutional/tests/week3_gate.rs	10
Week 4	ec-constitutional/tests/week4_integration.rs	20
Week 5	ec-constitutional/tests/week5_engine.rs	8
Week 6	ec-constitutional/tests/week6_meta.rs	10
Week 7	ec-constitutional/tests/week7_hybrid.rs	10
Week 9	ec-constitutional/tests/week9_edge_cases.rs	4
Week 10	ec-constitutional/tests/week10_integration.rs	4
Week 11	ec-constitutional/tests/week11_fuzzing.rs	5
Week 13	ec-sandbox/tests/week13_gate.rs	10
Week 14	ec-sandbox/tests/week14_gate.rs	15 (14 ignored بدون slow_tests)
Week 15	ec-sandbox/tests/week15_gate.rs	14
Week 16	ec-sandbox/tests/week16_gate.rs	14
Week 17	ec-app/tests/week17_integration.rs	3
Week 18	ec-app/tests/week18_phase2_gate.rs	4
Week 19	ec-analysis/tests/week19_gate.rs	10
Week 20	ec-memory/tests/week20_gate.rs	16
Week 21	ec-codegen/tests/week21_gate.rs	23
Week 23	ec-memory/tests/week23_gate.rs	23
Week 24	ec-memory/tests/week24_gate.rs	10
Week 25	ec-app/tests/week25_gate.rs	8
Phase 3	ec-app/tests/phase3_gate.rs	18
اختبارات ec-memory التفصيلية
graph.rs (14 اختبار):

text

graph_starts_empty
graph_record_adds_node
graph_record_multiple_nodes
graph_prevents_cycles_self_reference
graph_prevents_missing_parent
graph_prevents_duplicate_registration
graph_get_returns_none_for_missing
graph_update_retrospective
graph_update_retrospective_missing_node
graph_causal_chain
graph_decisions_for_artifact
graph_latest_n
graph_all_returns_slice
node.rs (8 اختبارات):

text

decision_node_creates
decision_node_has_unique_id
decision_node_add_retrospective
rejected_alternative_creates
rejected_alternative_add_retrospective
builder_creates_node
builder_with_alternative
builder_id_is_controlled_by_memory
types.rs (5 اختبارات):

text

node_id_is_unique
artifact_snapshot_same_code_same_hash
artifact_snapshot_different_code_different_hash
artifact_snapshot_shares_arc
retrospective_assessment_validates_confidence
اختبارات Phase 3 Gate (18 اختبار)
text

gate_static_analysis_produces_valid_fitness
gate_static_analysis_detects_unsafe
gate_memory_append_only
gate_memory_dag_enforced
gate_memory_prevents_cycles
gate_memory_retrospective_append
gate_code_generation_works
gate_generated_code_passes_analysis
gate_counterfactual_gain
gate_find_similar
gate_fitness_evolution
gate_drift_stable
gate_drift_value_shift
gate_drift_insufficient_data
gate_iterative_pipeline_stores_in_memory
gate_phase3_adrs_exist
gate_analysis_performance
phase3_gate_complete
Feature Flags
Crate	Feature	الوصف
ec-sandbox	slow_tests	يُفعّل اختبارات Docker (14 test)
بدون slow_tests: 376 passed, 16 ignored
مع --features slow_tests: 376+ passed, 0-2 ignored

8. إصلاحات Week 25-26
الإصلاح 1: validate_acyclic() في graph.rs
المشكلة الأصلية: الدالة لم تكشف أي دورة. is_ancestor_of() تبحث عن العقدة الجديدة في graph قبل تسجيلها — دائماً تُرجع false.

الحل:

Rust

// قبل (معطوب):
fn validate_acyclic(&self, node: &DecisionNode) -> Result<(), MemoryError> {
    for parent_id in &node.causal_parents {
        if !self.is_ancestor_of(*parent_id, node.id) { continue; }
        return Err(MemoryError::CycleDetected { node: node.id });
    }
    Ok(())
}

// بعد (يعمل):
fn validate_acyclic(&self, node: &DecisionNode) -> Result<(), MemoryError> {
    if self.get(node.id).is_some() {
        return Err(MemoryError::CycleDetected { node: node.id });
    }
    for parent_id in &node.causal_parents {
        if *parent_id == node.id {
            return Err(MemoryError::CycleDetected { node: node.id });
        }
        if self.get(*parent_id).is_none() {
            return Err(MemoryError::NodeNotFound(*parent_id));
        }
    }
    Ok(())
}
الإصلاح 2: cosine_similarity — Single Source of Truth
المشكلة: نفس حساب cosine_angle_degrees مكرّر في drift.rs.

الحل: نقل إلى FitnessVector كطرق عامة:

cosine_similarity(&self, other: &Self) -> f64
cosine_angle_degrees(&self, other: &Self) -> f64
magnitude(&self) -> f64 (private)
drift.rs الآن يستخدم: baseline_avg.cosine_angle_degrees(&current_avg)
query.rs الآن يستخدم: n.fitness.cosine_similarity(target)

الإصلاح 3: Builder Pattern
المشكلة: DecisionNode::new() كان pub — أي crate يُنشئ عقد ويتحكم في id و created_at.

الحل:

DecisionNode::new() → pub(crate)
DecisionNodeBuilder → pub (builder pattern)
record_from_builder() → pub
builder.build() → pub(crate)
الإصلاح 4: فصل ec-constitutional
المشكلة: ec-memory يعتمد على ec-constitutional لكن لا يستخدمه.

الحل: حذف السطر من Cargo.toml.

الإصلاح 5: unwrap() → expect() في الإنتاج
المواقع المُصلحة:

الملف	السطر	التغيير
pipeline.rs	380	unwrap() → .expect("node just recorded must exist")
pipeline.rs	493	Evidence::new().unwrap() → .expect("valid evidence")
pipeline.rs	494	UncertaintyDecomposition::new().unwrap() → .expect("valid uncertainty")
pipeline.rs	497	EpistemicState::new().unwrap() → .expect("valid epistemic state")
propagation.rs	34	Evidence::weakest().unwrap() → .ok_or(EpistemicError::OutOfRange{...})?
الإصلاح 6: Docker Tests Feature Flag
المشكلة: اختبارات Docker تستغرق 154 ثانية.

الحل: إضافة [features] slow_tests = [] في ec-sandbox/Cargo.toml + #[cfg_attr(not(feature = "slow_tests"), ignore)] على 14 اختبار Docker.

الإصلاح 7: حذف graph.rs.orig
ملف متروك من git merge — حُذف.

الإصلاح 8: ADRs
ADR-017-causal-memory-model.md → ADR-015-causal-memory.md (إعادة تسمية)
ADR-016-code-generation.md (جديد)
ADR-017-iterative-pipeline.md (جديد)
9. قرارات التصميم (ADRs)
ADR	العنوان	المسار
004	Integration Architecture	docs/adr/ADR-004-integration-architecture.md
009	Sandbox Foundation	docs/adr/ADR-009-sandbox-foundation.md
010	Docker Execution Strategy	docs/adr/ADR-010-docker-execution-strategy.md
012	Security Hardening	docs/adr/ADR-012-security-hardening.md
013	Integration Pipeline	docs/adr/ADR-013-integration-pipeline.md
014	Static Code Analysis	docs/adr/ADR-014-static-code-analysis.md
015	Causal Memory	docs/adr/ADR-015-causal-memory.md
016	Code Generation	docs/adr/ADR-016-code-generation.md
017	Iterative Pipeline	docs/adr/ADR-017-iterative-pipeline.md
018	Counterfactual Query	docs/adr/ADR-018-counterfactual-query.md
019	Value Drift Enhanced	docs/adr/ADR-019-value-drift-enhanced.md
10. دليل الصيانة
إضافة بُعد جديد لـ Fitness
أضف الحقل إلى FitnessVector في ec-fitness/src/fitness.rs
حدّث validate() و magnitude() و cosine_similarity()
حدّث CatastropheThresholds (أضف عتبة + default)
حدّث CatastrophicDimension (أضف متغير)
حدّث pareto_compare() في ec-fitness/src/pareto.rs
حدّث count_better_dimensions() في ec-memory/src/query.rs
حدّث average_fitness() في ec-memory/src/drift.rs
أضف محلل في ec-analysis/src/
أضف ثابت دستوري في ec-constitutional/src/
شغّل cargo test --workspace
إضافة ثابت دستوري جديد
أنشئ struct يُنفّذ Invariant trait
أضفه إلى Constitution::new() في الاختبارات والـ pipeline
أضف اختبارات للثابت الجديد
إضافة قالب كود جديد
أنشئ struct يُنفّذ CodeTemplate trait
أضفه إلى CodeGenerator::new() (رتب بالأولوية)
أضف اختبارات
إضافة نوع رفض جديد
أضف متغير إلى RejectionReason في ec-memory/src/types.rs
حدّث classify() في drift.rs إن لزم
حدّث IterativePipeline في pipeline.rs إن لزم
أضف اختبارات
إضافة crate جديد
أضفه إلى Cargo.toml الرئيسي ([workspace].members)
أضف dependency في [workspace.dependencies]
تأكد من اتجاه الـ dependency (لا دورات بين crates)
أضف اختبارات week gate
شغّل cargo clippy --workspace --tests -- -D warnings
عند تعديل ec-memory
لا تُضف delete(), update_fitness(), clear() — يكسر D1
لا تجعل DecisionNode::new() عامة — يكسر D4
لا تُضف حساب similarity محلي — يكسر D5
لا تُضف dependency على ec-constitutional — لا حاجة
أوامر التحقق
Bash

# بنية كاملة
cargo check --workspace

# clippy (مع اختبارات)
cargo clippy --workspace --tests -- -D warnings

# اختبارات (سريعة)
cargo test --workspace

# اختبارات (مع Docker)
cargo test --workspace --features ec-sandbox/slow_tests

# عدد الأسطر
find crates/ -name "*.rs" | xargs wc -l | tail -1
11. خريطة الطريق
المُنجز
Phase	Weeks	المحتوى	الاختبارات
1	1-6	الدستور + اللياقة + المعرفية	~90
2	7-18	Sandbox + Integration + Feedback	~150
3	19-26	Analysis + Memory + Codegen + Drift	376
التالي
Phase	Weeks	المحتوى	الهدف
4	27-32	Governance: SQLite + Proposals + Audit	450+
5	33-40	Agents: trait + runtime + orchestration	500+
6	41-44	Server: REST API + CLI + demo	550+
Phase 4 — أولوية Week 27
text

المشكلة: الذاكرة تختفي عند إغلاق البرنامج
الحل: MemoryStorage trait + SqliteStorage
التبعية الجديدة: rusqlite
ملخص سريع
text

8 crates · 67 ملف .rs · 13,047 سطر
376 tests · 0 failed · 16 ignored
0 clippy warnings · 0 unwrap() in production
6 design invariants · 11 ADRs
8 architectural fixes applied
Commit: 1e6b56c
نهاية الوثيقة المرجعية
