#![forbid(unsafe_code)]

//! Integration Pipeline — القلب الذي يربط كل الطبقات.
//!
//! Week 17: IntegrationPipeline (sandbox → constitution → feedback)
//! Week 22: IterativePipeline (codegen → analysis → memory → learn)
//!
//! **العقد:** IntegrationPipeline لا يتغير — الاختبارات هي المرجع.

use ec_analysis::analyze_code;
use ec_constitutional::evaluation::ConstitutionalEvaluation;
use ec_constitutional::{
    Constitution, ConstitutionalEngine, EvaluationContext,
};
use ec_epistemic::{
    CalibrationState, EpistemicResult, EpistemicState, Evidence,
    UncertaintyDecomposition,
};
use ec_fitness::fitness::{CatastropheThresholds, FitnessVector};
use ec_sandbox::config::SandboxMode;
use ec_sandbox::executor::ExecutionResult;
use ec_sandbox::executor::SandboxExecutor;
use ec_sandbox::feedback::{PredictionError, PredictionRecord, RealityFeedback};
use ec_sandbox::reality::RealityVector;
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════
// PipelineVerdict
// ═══════════════════════════════════════════════════════════════════

/// حكم الـ pipeline على الكود.
#[derive(Debug, Clone)]
pub enum PipelineVerdict {
    /// قُبل — الكود صالح دستورياً وواقعياً.
    Accepted,
    /// رُفض بسبب فشل التنفيذ أو عدم موثوقية النتائج.
    RejectedByReality {
        /// سبب الرفض.
        reason: String,
    },
    /// رُفض بسبب انتهاك دستوري.
    RejectedByConstitution {
        /// سبب الرفض.
        reason: String,
    },
    /// فشل التنفيذ (انتهاك أمني أو خطأ).
    ExecutionFailed {
        /// سبب الفشل.
        reason: String,
    },
    /// فشل التوليد.
    GenerationFailed {
        /// سبب الفشل.
        reason: String,
    },
    /// استنفد المحاولات (IterativePipeline فقط).
    RejectedAfterMaxAttempts {
        /// سبب الرفض.
        reason: String,
    },
}

// ═══════════════════════════════════════════════════════════════════
// PipelineResult — العقد الأصلي (Week 17)
// ═══════════════════════════════════════════════════════════════════

/// نتيجة تشغيل الـ pipeline.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    /// معرف فريد للتشغيل.
    pub run_id: Uuid,
    /// الكود المُدخل.
    pub code: String,
    /// الحكم النهائي.
    pub verdict: PipelineVerdict,
    /// نتيجة التنفيذ.
    pub execution: ExecutionResult,
    /// خطأ التوقع.
    pub prediction_error: PredictionError,
    /// التقييم الدستوري.
    pub evaluation: ConstitutionalEvaluation,
}

impl PipelineResult {
    /// هل قُبل الكود؟
    pub fn is_accepted(&self) -> bool {
        matches!(self.verdict, PipelineVerdict::Accepted)
    }

    /// ملخص نصي.
    pub fn summary(&self) -> String {
        match &self.verdict {
            PipelineVerdict::Accepted => {
                format!("Pipeline: ACCEPTED (run={})", self.run_id)
            }
            PipelineVerdict::RejectedByReality { reason } => {
                format!(
                    "Pipeline: REJECTED BY REALITY — {} (run={})",
                    reason, self.run_id
                )
            }
            PipelineVerdict::RejectedByConstitution { reason } => {
                format!(
                    "Pipeline: REJECTED BY CONSTITUTION — {} (run={})",
                    reason, self.run_id
                )
            }
            PipelineVerdict::ExecutionFailed { reason } => {
                format!(
                    "Pipeline: EXECUTION FAILED — {} (run={})",
                    reason, self.run_id
                )
            }
            PipelineVerdict::GenerationFailed { reason } => {
                format!(
                    "Pipeline: GENERATION FAILED — {} (run={})",
                    reason, self.run_id
                )
            }
            PipelineVerdict::RejectedAfterMaxAttempts { reason } => {
                format!(
                    "Pipeline: REJECTED AFTER MAX ATTEMPTS — {} (run={})",
                    reason, self.run_id
                )
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// IntegrationPipeline (Week 17 — العقد الأصلي، لا يُغيَّر)
// ═══════════════════════════════════════════════════════════════════

/// Pipeline التكاملي — يربط Constitution → Sandbox → Feedback.
///
/// هذا هو العقد الأصلي من Phase 2 (Weeks 13-18).
/// لا يُغيَّر — الاختبارات تعتمد عليه.
pub struct IntegrationPipeline {
    engine: ConstitutionalEngine,
    executor: SandboxExecutor,
    feedback: RealityFeedback,
}

impl IntegrationPipeline {
    /// إنشاء pipeline بوضع simulated.
    pub fn new_simulated(
        constitution: Constitution,
        _thresholds: CatastropheThresholds,
    ) -> anyhow::Result<Self> {
        let config = ec_sandbox::config::SandboxConfig::default();
        let executor = SandboxExecutor::new(config)?;
        let engine = ConstitutionalEngine::with_default_cache(constitution);
        Ok(Self {
            engine,
            executor,
            feedback: RealityFeedback::new(),
        })
    }

    /// تشغيل الكود عبر كل الطبقات.
    pub fn run(&mut self, artifact_id: &str, code: &str) -> PipelineResult {
        let run_id = Uuid::new_v4();

        // ─── Step 1: Static Analysis → FitnessVector ───────────
        let fitness = analyze_code(code);

        // ─── Step 2: Epistemic State ────────────────────────────
        let epistemic = build_epistemic_from_fitness(&fitness);

        // ─── Step 3: Constitutional Evaluation ──────────────────
        let artifact_hash = hash_code(code);
        let evaluation = self.engine.evaluate(
            artifact_id,
            artifact_hash,
            &fitness,
            &epistemic,
            &EvaluationContext::default(),
        );

        // ─── Step 4: Sandbox Execution ──────────────────────────
        let execution = self.executor.execute(artifact_id, code);

        // ─── Step 5: Determine Verdict ──────────────────────────
        let verdict = determine_verdict(&evaluation, &execution);

        // ─── Step 6: Prediction Error ───────────────────────────
        let prediction = PredictionRecord {
            artifact_id: artifact_id.to_string(),
            predicted_validity: evaluation.is_valid,
            predicted_confidence: evaluation.epistemic.confidence,
        };

        let reality = execution
            .reality
            .as_ref()
            .cloned()
            .unwrap_or_else(RealityVector::dummy);

        let prediction_error = self.feedback.learn(&prediction, &reality);

        PipelineResult {
            run_id,
            code: code.to_string(),
            verdict,
            execution,
            prediction_error,
            evaluation,
        }
    }

    /// هل النظام يتحسن؟
    pub fn is_improving(&self) -> bool {
        self.feedback.is_improving()
    }

    /// هل يحتاج مراجعة دستورية؟
    pub fn needs_review(&self) -> bool {
        self.feedback.needs_constitutional_review()
    }

    /// متوسط خطأ التوقع.
    pub fn mean_validity_error(&self) -> f64 {
        self.feedback.mean_validity_error()
    }

    /// وضع الـ sandbox.
    pub fn sandbox_mode(&self) -> SandboxMode {
        SandboxMode::Simulated
    }
}

// ═══════════════════════════════════════════════════════════════════
// IterativePipeline (Week 22 — إضافة جديدة فوق العقد)
// ═══════════════════════════════════════════════════════════════════

/// سجل محاولة واحدة.
#[derive(Debug, Clone)]
pub struct AttemptRecord {
    /// رقم المحاولة.
    pub attempt_number: usize,
    /// معرف العقدة في الذاكرة.
    pub node_id: ec_memory::NodeId,
    /// الكود المُولَّد.
    pub code: String,
    /// التقييم.
    pub fitness: FitnessVector,
    /// هل اجتاز الدستور؟
    pub is_constitutional: bool,
    /// درجة correctness من sandbox.
    pub sandbox_correctness: Option<f64>,
}

/// نتيجة الـ IterativePipeline.
#[derive(Debug)]
pub struct IterativePipelineResult {
    /// العقدة المقبولة.
    pub accepted_node: Option<ec_memory::DecisionNode>,
    /// كل المحاولات.
    pub attempts: Vec<AttemptRecord>,
    /// عدد المحاولات.
    pub total_iterations: usize,
    /// الحكم.
    pub verdict: PipelineVerdict,
}

/// Pipeline تكراري — يُولّد → يُقيّم → يُنفّذ → يُخزّن → يُكرّر.
///
/// إضافة Week 22 فوق العقد الأصلي.
/// لا يُغيّر IntegrationPipeline.
pub struct IterativePipeline {
    generator: ec_codegen::CodeGenerator,
    engine: ConstitutionalEngine,
    executor: SandboxExecutor,
    memory: ec_memory::CausalMemoryGraph,
    feedback: RealityFeedback,
    max_iterations: usize,
}

impl IterativePipeline {
    /// إنشاء pipeline تكراري.
    pub fn new(
        constitution: Constitution,
        max_iterations: usize,
    ) -> anyhow::Result<Self> {
        let config = ec_sandbox::config::SandboxConfig::default();
        let executor = SandboxExecutor::new(config)?;
        let engine = ConstitutionalEngine::with_default_cache(constitution);
        Ok(Self {
            generator: ec_codegen::CodeGenerator::new(),
            engine,
            executor,
            memory: ec_memory::CausalMemoryGraph::new(),
            feedback: RealityFeedback::new(),
            max_iterations,
        })
    }

    /// تشغيل الدورة التكرارية.
    pub fn run(
        &mut self,
        spec: &ec_codegen::GenerationSpec,
    ) -> IterativePipelineResult {
        let mut attempts: Vec<AttemptRecord> = Vec::new();
        let mut previous_node_id: Option<ec_memory::NodeId> = None;

        for iteration in 0..self.max_iterations {
            let attempt_num = iteration + 1;

            // ─── Step 1: Generate ──────────────────────────────
            let code = match self.generator.generate(spec) {
                ec_codegen::GenerationResult::Success(s) => s.code,
                ec_codegen::GenerationResult::Failed { reason } => {
                    return IterativePipelineResult {
                        accepted_node: None,
                        attempts,
                        total_iterations: iteration,
                        verdict: PipelineVerdict::GenerationFailed { reason },
                    };
                }
            };

            // ─── Step 2: Static Analysis ───────────────────────
            let fitness = analyze_code(&code);
            let artifact = ec_memory::ArtifactSnapshot::new(&code);

            // ─── Step 3: Constitutional Evaluation ─────────────
            let epistemic = build_epistemic_from_fitness(&fitness);
            let eval = self.engine.evaluate(
                &spec.function_name,
                artifact.hash,
                &fitness,
                &epistemic,
                &EvaluationContext::default(),
            );

            // ─── Step 4: Sandbox Execution ─────────────────────
            let execution =
                self.executor.execute(&spec.function_name, &code);

            // ─── Step 5: Determine verdict ─────────────────────
            let verdict = determine_verdict(&eval, &execution);
            let is_accepted = matches!(verdict, PipelineVerdict::Accepted);

            // ─── Step 6: Record in Memory ──────────────────────
            let sandbox_outcome = execution.reality.as_ref().map(|r| {
                ec_memory::SandboxOutcome {
                    correctness: r.correctness,
                    reproducibility: r.reproducibility,
                    empirical_confidence: r.empirical_confidence,
                }
            });

            let mut builder = ec_memory::DecisionNodeBuilder::new(
                format!("{}-iter-{}", spec.function_name, attempt_num),
                artifact.clone(),
                fitness.clone(),
            )
            .constitutional_valid(eval.is_valid)
            .sandbox_outcome(sandbox_outcome)
            .causal_parents(previous_node_id.into_iter().collect());

            if !eval.is_valid {
                if let Some(cat_dim) = &eval.catastrophic {
                    let alt = ec_memory::RejectedAlternative::new(
                        artifact.clone(),
                        fitness.clone(),
                        ec_memory::RejectionReason::CatastrophicFailure {
                            dimension: format!("{:?}", cat_dim),
                        },
                    );
                    builder = builder.add_alternative(alt);
                }
            }

            let node_id = self
                .memory
                .record_from_builder(builder)
                .expect("record in memory");

            let node = self.memory.get(node_id).expect("node just recorded must exist").clone();

            // ─── Step 7: Track attempt ─────────────────────────
            attempts.push(AttemptRecord {
                attempt_number: attempt_num,
                node_id,
                code: code.clone(),
                fitness: fitness.clone(),
                is_constitutional: eval.is_valid,
                sandbox_correctness: execution
                    .reality
                    .as_ref()
                    .map(|r| r.correctness),
            });

            // ─── Step 8: Learn from feedback ───────────────────
            let prediction = PredictionRecord {
                artifact_id: format!(
                    "{}-iter-{}",
                    spec.function_name, attempt_num
                ),
                predicted_validity: eval.is_valid,
                predicted_confidence: eval.epistemic.confidence,
            };
            let reality = execution
                .reality
                .as_ref()
                .cloned()
                .unwrap_or_else(RealityVector::dummy);
            self.feedback.learn(&prediction, &reality);

            // ─── Step 9: Check success ─────────────────────────
            if is_accepted {
                return IterativePipelineResult {
                    accepted_node: Some(node),
                    attempts,
                    total_iterations: attempt_num,
                    verdict: PipelineVerdict::Accepted,
                };
            }

            previous_node_id = Some(node_id);
        }

        IterativePipelineResult {
            accepted_node: None,
            attempts,
            total_iterations: self.max_iterations,
            verdict: PipelineVerdict::RejectedAfterMaxAttempts {
                reason: format!(
                    "Failed after {} attempts",
                    self.max_iterations
                ),
            },
        }
    }

    /// الذاكرة السببية.
    pub fn memory(&self) -> &ec_memory::CausalMemoryGraph {
        &self.memory
    }

    /// الذاكرة السببية (mutable).
    pub fn memory_mut(&mut self) -> &mut ec_memory::CausalMemoryGraph {
        &mut self.memory
    }

    /// هل يتحسن؟
    pub fn is_improving(&self) -> bool {
        self.feedback.is_improving()
    }

    /// تحديث retrospective.
    pub fn learn_from_history(
        &mut self,
        node_id: ec_memory::NodeId,
        assessment: ec_memory::RetrospectiveAssessment,
    ) -> Result<(), ec_memory::MemoryError> {
        self.memory.update_retrospective(node_id, assessment)
    }

    /// تتبع السلسلة السببية.
    pub fn trace_causal_chain(
        &self,
        node_id: ec_memory::NodeId,
    ) -> Vec<&ec_memory::DecisionNode> {
        self.memory.causal_chain(node_id)
    }
}

// ═══════════════════════════════════════════════════════════════════
// Helpers — مشتركة
// ═══════════════════════════════════════════════════════════════════

fn hash_code(code: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

fn build_epistemic_from_fitness(fitness: &FitnessVector) -> EpistemicState {
    let avg_score = (fitness.security
        + fitness.reversibility
        + fitness.test_coverage
        + fitness.maintainability
        + fitness.performance
        + fitness.architectural_stability)
        / 6.0;

    EpistemicState::new(
        avg_score.clamp(0.3, 0.95),
        Evidence::new(1, 0, avg_score, 0.8).expect("valid evidence"),
        UncertaintyDecomposition::new(0.2, 0.2, 0.1).expect("valid uncertainty"),
        CalibrationState::default(),
    )
    .expect("valid epistemic state")
}

/// بناء EpistemicState من RealityVector.
pub fn build_epistemic_from_reality(
    reality: &RealityVector,
) -> EpistemicResult<EpistemicState> {
    let confidence = reality.empirical_confidence;

    EpistemicState::new(
        confidence.clamp(0.3, 0.95),
        Evidence::new(1, 0, reality.correctness, reality.reproducibility)?,
        UncertaintyDecomposition::new(0.1, 0.2, 0.1)?,
        CalibrationState::default(),
    )
}

fn determine_verdict(
    evaluation: &ConstitutionalEvaluation,
    execution: &ExecutionResult,
) -> PipelineVerdict {
    // الأمن أولاً
    if !execution.is_secure() {
        return PipelineVerdict::ExecutionFailed {
            reason: "Security violation detected".to_string(),
        };
    }

    // ثم الدستور
    if !evaluation.is_valid {
        return PipelineVerdict::RejectedByConstitution {
            reason: evaluation.explanation.clone(),
        };
    }

    // ثم نجاح التنفيذ
    if !execution.success {
        return PipelineVerdict::RejectedByReality {
            reason: execution
                .error_message
                .clone()
                .unwrap_or_else(|| "Execution failed".to_string()),
        };
    }

    // ثم موثوقية الواقع
    if let Some(ref reality) = execution.reality {
        if !reality.is_trustworthy() {
            return PipelineVerdict::RejectedByReality {
                reason: format!(
                    "Not trustworthy: correctness={}, reproducibility={}, confidence={}",
                    reality.correctness,
                    reality.reproducibility,
                    reality.empirical_confidence
                ),
            };
        }
    } else {
        return PipelineVerdict::ExecutionFailed {
            reason: "No reality vector produced".to_string(),
        };
    }

    PipelineVerdict::Accepted
}

// ═══════════════════════════════════════════════════════════════════
// Tests — IntegrationPipeline (العقد الأصلي)
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests_integration {
    use super::*;
    use ec_constitutional::invariant::{Invariant, ViolationReport};
    use std::sync::Arc;

    #[derive(Debug)]
    struct AlwaysAccept;
    impl Invariant for AlwaysAccept {
        fn name(&self) -> &'static str {
            "AlwaysAccept"
        }
        fn check(
            &self,
            _: &FitnessVector,
            _: &EpistemicState,
        ) -> Result<(), ViolationReport> {
            Ok(())
        }
    }

    fn permissive_constitution() -> Constitution {
        Constitution::new(
            vec![Arc::new(AlwaysAccept)],
            CatastropheThresholds {
                min_security: 0.0,
                min_reversibility: 0.0,
                min_test_coverage: 0.0,
                min_maintainability: 0.0,
                min_performance: 0.0,
                min_architectural_stability: 0.0,
            },
        )
    }

    fn simulated_pipeline() -> IntegrationPipeline {
        IntegrationPipeline::new_simulated(
            permissive_constitution(),
            CatastropheThresholds::default(),
        )
        .unwrap()
    }

    #[test]
    fn integration_pipeline_creates() {
        assert!(IntegrationPipeline::new_simulated(
            permissive_constitution(),
            CatastropheThresholds::default(),
        )
        .is_ok());
    }

    #[test]
    fn integration_pipeline_accepts_good_code() {
        let mut p = simulated_pipeline();
        let result = p.run("test-artifact", "fn main() {}");
        assert!(result.is_accepted(), "verdict: {:?}", result.verdict);
        assert!(result.execution.success);
        assert!(result.execution.reality.is_some());
    }

    #[test]
    fn integration_pipeline_rejects_fail_artifact() {
        let mut p = simulated_pipeline();
        let result = p.run("fail-artifact", "");
        assert!(!result.is_accepted());
        assert!(matches!(
            result.verdict,
            PipelineVerdict::RejectedByReality { .. }
        ));
    }

    #[test]
    fn integration_pipeline_feedback_improves() {
        let mut p = simulated_pipeline();
        for i in 0..20 {
            p.run(&format!("artifact-{}", i), "fn main() {}");
        }
        assert!(p.is_improving());
        assert!(!p.needs_review());
    }

    #[test]
    fn integration_pipeline_mean_error_low() {
        let mut p = simulated_pipeline();
        for i in 0..10 {
            p.run(&format!("artifact-{}", i), "fn main() {}");
        }
        assert!(p.mean_validity_error() < 0.1);
    }

    #[test]
    fn integration_pipeline_unique_run_ids() {
        let mut p = simulated_pipeline();
        let r1 = p.run("a", "fn main() {}");
        let r2 = p.run("b", "fn main() {}");
        assert_ne!(r1.run_id, r2.run_id);
    }

    #[test]
    fn integration_pipeline_summary_informative() {
        let mut p = simulated_pipeline();
        let result = p.run("test", "fn main() {}");
        let summary = result.summary();
        assert!(!summary.is_empty());
        assert!(summary.contains("Pipeline"));
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests — IterativePipeline (Week 22)
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests_iterative {
    use super::*;
    use ec_constitutional::{
        Invariant, ReversibilityInvariant, SecurityInvariant,
        TestCoverageInvariant, TypeSafetyInvariant,
    };
    use std::sync::Arc;

    fn make_constitution() -> Constitution {
        let invariants: Vec<Arc<dyn Invariant>> = vec![
            Arc::new(SecurityInvariant::default()),
            Arc::new(TestCoverageInvariant::default()),
            Arc::new(ReversibilityInvariant::default()),
            Arc::new(TypeSafetyInvariant::default()),
        ];
        Constitution::new(invariants, CatastropheThresholds::default())
    }

    #[test]
    fn iterative_pipeline_runs_and_stores_in_memory() {
        let constitution = make_constitution();
        let mut pipeline =
            IterativePipeline::new(constitution, 3).unwrap();

        let spec =
            ec_codegen::GenerationSpec::simple("add", vec!["i32", "i32"], "i32");
        let result = pipeline.run(&spec);

        assert!(result.total_iterations > 0);
        assert!(!result.attempts.is_empty());
        assert_eq!(pipeline.memory().len(), result.attempts.len());

        if result.attempts.len() >= 2 {
            let chain = pipeline
                .trace_causal_chain(result.attempts.last().unwrap().node_id);
            assert!(!chain.is_empty());
        }
    }

    #[test]
    fn iterative_pipeline_multiple_artifacts() {
        let constitution = make_constitution();
        let mut pipeline =
            IterativePipeline::new(constitution, 2).unwrap();

        let spec1 = ec_codegen::GenerationSpec::simple(
            "multiply",
            vec!["f64", "f64"],
            "f64",
        );
        let spec2 = ec_codegen::GenerationSpec::simple(
            "divide",
            vec!["f64", "f64"],
            "f64",
        );

        let r1 = pipeline.run(&spec1);
        let r2 = pipeline.run(&spec2);

        let total_attempts = r1.attempts.len() + r2.attempts.len();
        assert!(pipeline.memory().len() >= 2);
        assert_eq!(pipeline.memory().len(), total_attempts);

        // verify artifact_ids contain the function name
        for attempt in &r1.attempts {
            let node = pipeline.memory().get(attempt.node_id).unwrap();
            assert!(node.artifact_id.starts_with("multiply"));
        }
        for attempt in &r2.attempts {
            let node = pipeline.memory().get(attempt.node_id).unwrap();
            assert!(node.artifact_id.starts_with("divide"));
        }
    }

    #[test]
    fn iterative_pipeline_learn_from_history() {
        let constitution = make_constitution();
        let mut pipeline =
            IterativePipeline::new(constitution, 3).unwrap();

        let spec =
            ec_codegen::GenerationSpec::simple("test_fn", vec!["i32"], "i32");
        let result = pipeline.run(&spec);

        if let Some(first_attempt) = result.attempts.first() {
            let assessment =
                ec_memory::RetrospectiveAssessment::new(true, 0.85, "was good")
                    .expect("assessment");
            pipeline
                .learn_from_history(first_attempt.node_id, assessment)
                .unwrap();

            let node = pipeline
                .memory()
                .get(first_attempt.node_id)
                .unwrap();
            assert!(!node.retrospective.is_empty());
        }
    }

    #[test]
    fn iterative_pipeline_pure_function() {
        let constitution = make_constitution();
        let mut pipeline =
            IterativePipeline::new(constitution, 3).unwrap();

        let mut spec =
            ec_codegen::GenerationSpec::simple("square", vec!["f64"], "f64");
        spec.constraints.push("pure".into());
        spec.constraints.push("no_side_effects".into());

        let result = pipeline.run(&spec);
        assert!(result.total_iterations > 0);
    }
}
