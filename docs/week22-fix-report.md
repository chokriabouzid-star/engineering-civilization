# وثيقة إصلاح Week 22 — التقرير الكامل

**التاريخ:** Week 22 — Phase 3
**الحالة:** ✅ 326 اختبار نجح · 0 فاشل · 2 متجاهل
**المبدأ:** الاختبارات هي العقد — الكود يتبعها، لا العكس

---

## 1. المشكلة الأصلية

عند محاولة بناء المشروع بعد إضافة Week 21 (ec-codegen) و Week 22 (ec-memory)،
ظهرت **4 أخطاء تجميعية** تمنع `cargo check` من النجاح:
error 1: ec-memory يستخدم ec_constitutional لكن Cargo.toml لا يعتمد عليه
error 2: ec-memory يستخدم anyhow لكن Cargo.toml لا يعتمد عليه
error 3: ec-constitutional يستخدم tracing, dashmap, tokio, toml لكن Cargo.toml لا يعتمد عليها
error 4: ec-analysis يستدعي FitnessVector::new() لكن هذه الدالة غير موجودة

text


بعد إصلاح الأخطاء الأربعة، ظهرت **أخطاء إضافية** في ec-app:
- `IntegrationPipeline` اختفى واستُبدل بـ `IterativePipeline`
- `PipelineVerdict` فقد عدة متغيرات (RejectedByReality, RejectedByConstitution, ExecutionFailed)
- `build_epistemic_from_reality` اختفى
- `ec-sandbox` لم يُصدّر وحداته العامة

---

## 2. المبدأ التوجيهي

> **الاختبارات هي العقد. الكود يجب أن يطابقها، لا العكس.**

هذا يعني:
- لا نُعدّل اختباراً أصلياً لينجح مع الكود الجديد
- نُعدّل الكود لينجح مع الاختبارات الأصلية
- إذا أضفنا ميزة جديدة، نضيفها **فوق** العقد القائم بدون كسره
- الـ 280+ اختبار التي كانت تمر هي المرجع الحقيقي

---

## 3. بنية المشروع — الخريطة الكاملة
engineering-civilization/
├── crates/
│ ├── ec-fitness/ ← الأساس: FitnessVector + Pareto + CatastropheThresholds
│ ├── ec-epistemic/ ← نموذج عدم اليقين: Evidence + UncertaintyDecomposition
│ ├── ec-constitutional/ ← المحرك الدستوري: Constitution + Invariants + Engine
│ ├── ec-sandbox/ ← التنفيذ الآمن: SandboxExecutor + RealityVector + Feedback
│ ├── ec-analysis/ ← التحليل الثابت: analyze_code() → FitnessVector
│ ├── ec-memory/ ← الذاكرة السببية: CausalMemoryGraph + DecisionNode
│ ├── ec-codegen/ ← توليد الكود: CodeGenerator + Templates + Spec
│ └── ec-app/ ← التكامل: IntegrationPipeline + IterativePipeline
└── docs/
└── adr/ ← قرارات التصميم

text


### 3.1 ec-fitness (الأساس)

**الدور:** يُعرّف الأبعاد الستة للياقة الدستورية.

```rust
pub struct FitnessVector {
    pub security: f64,               // الأمان
    pub reversibility: f64,          // القابلية للعكس
    pub test_coverage: f64,          // تغطية الاختبارات
    pub maintainability: f64,        // القابلية للصيانة
    pub performance: f64,            // الأداء
    pub architectural_stability: f64,// الاستقرار المعماري
}
ملاحظة مهمة: FitnessVector ليس له new() — يُنشأ بـ struct literal أو Default.
هذا كان سبب الخطأ الرابع: ec-analysis كان يستدعي FitnessVector::new().

كذلك يحتوي: CatastropheThresholds و CatastrophicDimension و ParetoOrdering.

3.2 ec-epistemic (نموذج عدم اليقين)
الدور: يُمثّل ثقتنا المعرفية في التقييم.

Rust

pub struct EpistemicState {
    pub confidence: f64,
    pub evidence: Evidence,
    pub uncertainty: UncertaintyDecomposition,
    pub calibration: CalibrationState,
}
نقطة مهمة: كل constructors تُرجع EpistemicResult<T> = Result<T, EpistemicError>.
لا تُرجع القيمة مباشرة. هذا كان سبب خطأ في pipeline القديم.

3.3 ec-constitutional (المحرك الدستوري)
الدور: يُقيّم الكود دستورياً — هل يُقبل أو يُرفض؟

Rust

pub struct ConstitutionalEvaluation {
    pub artifact_id: String,
    pub fitness: FitnessVector,
    pub epistemic: EpistemicState,
    pub violations: Vec<ViolationReport>,
    pub catastrophic: Option<CatastrophicDimension>,  // وليس Vec!
    pub is_valid: bool,
    pub explanation: String,
}
نقطة مهمة: catastrophic هو Option<CatastrophicDimension> وليس Vec.
هذا كان سبب خطأ في pipeline.

المكوّنات:

Constitution — الدستور: مجموعة Invariants + Thresholds
ConstitutionalEngine — المحرك: يُقيّم ويخزّن مؤقتاً
EvaluationContext — سياق التقييم
Invariant (trait) — العقد الدستورية (Security, TestCoverage, Reversibility, TypeSafety)
OssificationDetector + ValueDriftDetector — كشف انحراف القيم
3.4 ec-sandbox (التنفيذ الآمن)
الدور: يُنفّذ الكود في بيئة معزولة ويُقيس الواقع.

Rust

pub struct RealityVector {
    pub correctness: f64,           // هل نجح؟
    pub reproducibility: f64,       // هل يتكرر؟
    pub benchmark_validity: f64,    // هل القياسات صحيحة؟
    pub empirical_confidence: f64,  // ثقة تجريبية (تزيد مع التشغيلات)
    pub runs_completed: usize,
    pub latency: Option<LatencyMeasurement>,
}
نقطة مهمة: RealityVector::new() هو pub(crate) — لا يُنشأ إلا من SandboxExecutor.
اختبارات week18 تستخدم RealityVector::test_fixture().

المكوّنات:

SandboxExecutor — المنفّذ (Simulated / Local / Docker)
SandboxConfig + SandboxMode — التكوين
RealityFeedback — يتعلم من مقارنة التوقع بالواقع
PredictionError — خطأ التوقع
HardenedDockerRunner — تنفيذ مُشدّد أمنياً
SecurityViolation — انتهاكات الأمان
3.5 ec-analysis (التحليل الثابت)
الدور: يُنتج FitnessVector من الكود المصدري — بدون تنفيذ.

Rust

pub fn analyze_code(code: &str) -> FitnessVector
كيف تعمل كل بُعد:

البُعد	ما يُخفّضه
security	unsafe ×0.5, unwrap() ×0.2, expect( ×0.1, panic! ×0.3
reversibility	println! ×0.15, static mut ×0.15, unsafe ×0.15
test_coverage	#[test] / fn ratio
maintainability	1.0 - complexity (if/match/for/while/&&/|| ×0.1)
performance	alloc, Vec::new, String::from, Box::new, HashMap::new ×0.2
architectural_stability	use ×0.1
هذا كان سبب اختبارين فاشلين: performance_score و architectural_score
كانا يُرجعان ثوابت (0.7 و 0.8) بدلاً من أن يحللا الكود فعلياً.

3.6 ec-memory (الذاكرة السببية)
الدور: يُسجّل كل قرار في رسم بياني append-only.

Rust

pub struct CausalMemoryGraph {
    nodes: Vec<DecisionNode>,  // append-only
}
Design Invariants:

❌ لا delete() — الماضي لا يتغير
❌ لا update_fitness() — اللياقة لا تتغير
✅ record() — تسجيل قرار جديد
✅ update_retrospective() — فقط تفسيرنا يتغير
Rust

pub struct DecisionNode {
    pub id: NodeId,
    pub artifact_id: String,
    pub artifact: ArtifactSnapshot,
    pub fitness: FitnessVector,
    pub constitutional_valid: bool,
    pub sandbox_outcome: Option<SandboxOutcome>,
    pub alternatives: Vec<RejectedAlternative>,
    pub causal_parents: Vec<NodeId>,
    pub retrospective: Vec<RetrospectiveAssessment>,  // mutable
}
ملاحظة: SandboxOutcome هنا (في ec-memory) يختلف عن RealityVector (في ec-sandbox).
الأول بسيط: {correctness, reproducibility, empirical_confidence}.
الثاني غني: يشمل latency, runs_completed, benchmark_validity, validation.

3.7 ec-codegen (توليد الكود)
الدور: يُولّد كود Rust من مواصفات — بدون ذكاء اصطناعي.

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
القوالب (Templates):

RustPureTemplate (priority 80) — للدوال النقية (pure, no_side_effects)
RustFunctionTemplate (priority 50) — الدوال العادية
RustStructTemplate (priority 60) — الهياكل (أسماء تبدأ بحرف كبير)
3.8 ec-app (التكامل)
الدور: يربط كل الطبقات معاً.

يحتوي pipelineين:

IntegrationPipeline (Week 17 — العقد الأصلي)
text

الكود → Static Analysis → Constitutional Evaluation → Sandbox Execution → Verdict → Feedback
Rust

pub struct IntegrationPipeline {
    engine: ConstitutionalEngine,
    executor: SandboxExecutor,
    feedback: RealityFeedback,
}
العقد (ما تختبره الاختبارات):

new_simulated(constitution, thresholds) → Result<Self>
run(artifact_id, code) → PipelineResult
PipelineResult.run_id — UUID فريد
PipelineResult.code — الكود المُدخل
PipelineResult.verdict — Accepted / RejectedByReality / RejectedByConstitution / ExecutionFailed
PipelineResult.execution — ExecutionResult (لها .success, .reality, .is_secure())
PipelineResult.prediction_error — PredictionError (لها .validity_error, .confidence_gap)
PipelineResult.evaluation — ConstitutionalEvaluation (لها .fitness, .is_valid)
PipelineResult.is_accepted() — bool
PipelineResult.summary() — String تحتوي "Pipeline"
is_improving() → bool
needs_review() → bool
mean_validity_error() → f64
sandbox_mode() → SandboxMode::Simulated
IterativePipeline (Week 22 — الإضافة الجديدة)
text

Spec → Generate → Analyze → Evaluate → Execute → Store → Repeat (حتى القبول أو استنفاد المحاولات)
Rust

pub struct IterativePipeline {
    generator: CodeGenerator,
    engine: ConstitutionalEngine,
    executor: SandboxExecutor,
    memory: CausalMemoryGraph,
    feedback: RealityFeedback,
    max_iterations: usize,
}
يضيف فوق العقد الأصلي:

توليد كود من مواصفات (ec-codegen)
تخزين في ذاكرة سببية (ec-memory)
تكرار حتى القبول أو استنفاد المحاولات
memory() / memory_mut() — الوصول للذاكرة
learn_from_history() — تحديث retrospective
trace_causal_chain() — تتبع السلسلة السببية
مهم: IterativePipeline لا يُخرّب IntegrationPipeline — كلاهما يعيشان جنباً إلى جنب.

build_epistemic_from_reality (دالة عامة)
Rust

pub fn build_epistemic_from_reality(reality: &RealityVector) -> EpistemicResult<EpistemicState>
العقد: اختبارات week17 و week18 تستوردها من ec_app::pipeline.
يجب أن تكون عامة ومتاحة.

determine_verdict (المنطق المشترك)
ترتيب الأولويات:

الأمن أولاً — !execution.is_secure() → ExecutionFailed
الدستور ثانياً — !evaluation.is_valid → RejectedByConstitution
نجاح التنفيذ — !execution.success → RejectedByReality
موثوقية الواقع — !reality.is_trustworthy() → RejectedByReality
القبول — كل شيء سليم → Accepted
4. التغييرات بالتفصيل — ملف ملف
4.1 ec-memory/Cargo.toml
المشكلة: الكود يستخدم ec_constitutional و anyhow لكنهما لم يكونا في dependencies.

الإصلاح: أضفنا:

toml

ec-constitutional = { path = "../ec-constitutional" }
anyhow            = "1.0"
لماذا:

ec-memory/src/node.rs يستورد ec_constitutional::evaluation::ConstitutionalEvaluation
ec-memory/src/types.rs يستورد anyhow::ensure! في RetrospectiveAssessment::new()
4.2 ec-constitutional/Cargo.toml
المشكلة: الكود يستخدم tracing, dashmap, tokio, toml لكنها لم تكن في dependencies.

الإصلاح: أضفنا:

toml

tracing = "0.1"
dashmap = "5.5"
tokio   = { version = "1.35", features = ["full"] }
toml    = "0.8"
anyhow  = "1.0"
لماذا:

engine.rs يستخدم tracing للتسجيل، dashmap للذاكرة المؤقتة، tokio للتنفيذ غير المتزامن
policy.rs يستخدم toml لتحليل السياسات
4.3 ec-constitutional/src/lib.rs
المشكلة: الاختبارات تستورد Constitution, ConstitutionalEngine, EvaluationContext
من ec_constitutional مباشرة، لكن lib.rs لم يُصدّرها.

الإصلاح: أضفنا re-exports عامة:

Rust

pub use constitution::Constitution;
pub use engine::{ConstitutionalEngine, EvaluationContext};
pub use evaluation::ConstitutionalEvaluation;
pub use invariant::Invariant;
pub use security::SecurityInvariant;
pub use coverage::TestCoverageInvariant;
pub use reversibility::ReversibilityInvariant;
pub use type_safety::TypeSafetyInvariant;
pub use verdict::ConstitutionalVerdict;
pub use meta::{OssificationDetector, ValueDriftDetector};
4.4 ec-sandbox/src/lib.rs
المشكلة: lib.rs كان يحتوي فقط SandboxResult و SandboxError (placeholder).
الاختبارات تستورد SandboxConfig, SandboxMode, SandboxExecutor, RealityVector,
RealityFeedback, NetworkPolicy, SyscallPolicy, إلخ.

الإصلاح: أضفنا pub mod لكل الوحدات و re-exports:

Rust

pub mod compiler;
pub mod config;
pub mod docker;
pub mod executor;
pub mod feedback;
pub mod hardened;
pub mod metrics;
pub mod reality;
pub mod security;

pub use config::{NetworkPolicy, ResourceLimits, SandboxConfig, SandboxMode, SyscallPolicy};
pub use executor::{ExecutionResult, SandboxExecutor};
pub use feedback::{PredictionError, PredictionRecord, RealityFeedback};
pub use reality::{LatencyMeasurement, RealityVector};
pub use security::SecurityViolation;
4.5 ec-sandbox/Cargo.toml
المشكلة: اختبارات week15 تستورد ec_epistemic لكنه لم يكن في dev-dependencies.

الإصلاح: أضفنا:

toml

[dev-dependencies]
ec-epistemic = { path = "../ec-epistemic" }
4.6 ec-analysis/src/lib.rs
المشكلة 1: كان يستدعي FitnessVector::new() التي لا وجود لها.

الإصلاح: استخدمنا struct literal:

Rust

FitnessVector {
    security: security_score(code),
    reversibility: reversibility_score(code),
    test_coverage: coverage_score(code),
    maintainability: complexity_to_maintainability(complexity_score(code)),
    performance: performance_score(code),
    architectural_stability: architectural_score(code),
}
المشكلة 2: performance_score كان يُرجع 0.7 ثابت، و architectural_score كان يُرجع 0.8 ثابت.
اختبارات week19 تتوقع 1.0 عندما لا توجد allocations أو uses.

الإصلاح: جعلناهما يحللان الكود فعلياً:

Rust

fn performance_score(code: &str) -> f64 {
    let allocations = code.matches("alloc").count()
        + code.matches("Vec::new").count()
        + code.matches("String::from").count()
        + code.matches("Box::new").count()
        + code.matches("HashMap::new").count();
    1.0 - (allocations as f64 * 0.2).min(1.0)
}

fn architectural_score(code: &str) -> f64 {
    let uses = code.matches("use ").count();
    1.0 - (uses as f64 * 0.1).min(1.0)
}
4.7 ec-app/src/pipeline.rs — الإصلاح الأكبر
المشكلة الجذرية: الملف أُعيد كتابته بالكامل في Week 22:

IntegrationPipeline اختفى واستُبدل بـ IterativePipeline
PipelineVerdict فقد RejectedByReality, RejectedByConstitution, ExecutionFailed
PipelineResult فقد run_id, code, execution, prediction_error, evaluation
build_epistemic_from_reality اختفت
sandbox_mode() اختفت
الحل: أعدنا بناء الملف بحيث يحتوي كلا pipelineين:

IntegrationPipeline — العقد الأصلي (Week 17)، يُحقق كل ما تختبره week17 و week18
IterativePipeline — الإضافة الجديدة (Week 22)، يضيف ec-codegen + ec-memory
التفاصيل:

PipelineVerdict يضم كل المتغيرات القديمة + الجديدة:

Accepted
RejectedByReality { reason } — للعقد الأصلي
RejectedByConstitution { reason } — للعقد الأصلي
ExecutionFailed { reason } — للعقد الأصلي
GenerationFailed { reason } — جديد لـ IterativePipeline
RejectedAfterMaxAttempts { reason } — جديد لـ IterativePipeline
PipelineResult يُحقق العقد:

run_id: Uuid — فريد لكل تشغيل
code: String — الكود المُدخل
verdict: PipelineVerdict
execution: ExecutionResult — من SandboxExecutor
prediction_error: PredictionError — من RealityFeedback
evaluation: ConstitutionalEvaluation — من ConstitutionalEngine
is_accepted() — هل قُبل؟
summary() — نص يحتوي "Pipeline"
build_epistemic_from_reality() — عامة، تُرجع EpistemicResult<EpistemicState>

determine_verdict() — المنطق المشترك، يُرتّب: أمن ← دستور ← نجاح ← موثوقية ← قبول

4.8 ec-app/src/lib.rs
الإصلاح: صدّرنا كل ما تحتاجه الاختبارات:

Rust

pub use pipeline::{
    AttemptRecord, IntegrationPipeline, IterativePipeline,
    IterativePipelineResult, PipelineResult, PipelineVerdict,
    build_epistemic_from_reality,
};
5. الدروس المستفادة
5.1 لا تُعدّ اختباراً أصلياً
إذا كتبت IntegrationPipeline في Week 17 واختبرته 30 اختبار، ثم أردت إضافة
IterativePipeline في Week 22 — لا تحذف القديم. أضف الجديد جنبه.

5.2 الـ exports جزء من العقد
إذا كانت الاختبارات تستورد ec_sandbox::SandboxConfig، فهذا يعني أن lib.rs
يجب أن يُصدّره. حذف الـ export هو كسر للعقد حتى لو الكود الداخلي سليم.

5.3 الـ constructors جزء من العقد
إذا كان FitnessVector لا يملك new()، فلا تكتب كوداً يستدعي new().
إذا كان RealityVector::new() هو pub(crate)، فلا تحاول إنشاءه من خارج ec-sandbox.

5.4 الثوابت ليست تحليلاً
Rust

fn performance_score(_code: &str) -> f64 { 0.7 }  // ❌
fn performance_score(code: &str) -> f64 {          // ✅
    1.0 - (allocations as f64 * 0.2).min(1.0)
}
الاختبارات تتوقع أن الكود النقي يُرجع 1.0، والكود المُخصص يُرجع أقل.
الثابت لا يُحقق هذا.

5.5 لا تُعيد كتابة ما يعمل
إذا كانت 280 اختبار تمر، فالـ pipeline يعمل. إضافة ميزة جديدة تعني:

إضافة كود جديد
إضافة اختبارات جديدة
عدم لمس الكود القديم ما لم يكن ضرورياً
6. الخريطة المرجعية السريعة
التدفقات
text

IntegrationPipeline:
  code → analyze_code() → FitnessVector
       → EpistemicState
       → ConstitutionalEngine.evaluate() → ConstitutionalEvaluation
       → SandboxExecutor.execute() → ExecutionResult + RealityVector
       → determine_verdict() → PipelineVerdict
       → RealityFeedback.learn() → PredictionError

IterativePipeline:
  GenerationSpec → CodeGenerator.generate() → code
    ↳ نفس التدفق أعلاه +
    ↳ CausalMemoryGraph.record() → NodeId
    ↳ تكرار حتى القبول أو استنفاد المحاولات
أنواع النتائج
النوع	المصدر	المحتوى
FitnessVector	ec-analysis	6 أبعاد من الكود
RealityVector	ec-sandbox	قياسات من التنفيذ الفعلي
ConstitutionalEvaluation	ec-constitutional	حكم دستوري + لياقة + انتهاكات
ExecutionResult	ec-sandbox	نجاح + واقع + انتهاكات أمنية
PredictionError	ec-sandbox	فرق بين التوقع والواقع
PipelineResult	ec-app	كل ما سبق + حكم نهائي
DecisionNode	ec-memory	قرار مُسجّل في الذاكرة السببية
الحقيقة ≠ اللياقة
text

FitnessVector  → يُقاس من الكود المصدري (static analysis)
RealityVector  → يُقاس من التنفيذ الفعلي (sandbox execution)

لا يوجد تحويل مباشر بينهما.
كلاهما يُغذي ConstitutionalEvaluation.
كلاهما يُقارَن عبر PredictionError.
7. النتيجة النهائية
text

326 اختبار نجح ✅
0 اختبار فشل ✅
2 اختبار متجاهل (slow: 100 Docker execution) ✅
0 تحذير تجميعي حرج ✅
0 تعديل على اختبار أصلي ✅
الوثيقة أُعدّت كجزء من Week 22 — Phase 3: Causal Memory + Code Generation
