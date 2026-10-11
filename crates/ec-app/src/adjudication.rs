#![forbid(unsafe_code)]

//! مسار التحكيم المعتمد — القلب النقي المستقل لتقييم الأثر البرمجي بالواقع المقاس.
//!
//! ADR-033. لا يغيّر هذا الملف `pipeline.rs`، ولا يُستدعى منه في PR1.

use ec_constitutional::{
    Constitution, ConstitutionalEvaluation, MeasuredRealityInvariant, ReversibilityInvariant,
    SecurityInvariant, TestCoverageInvariant, MIN_RUNS_FOR_AUTHORITY,
};
use ec_epistemic::{
    CalibrationState, EpistemicState, Evidence, ExecutionProvenance, MeasuredEvidence, OracleId,
    UncertaintyDecomposition,
};
use ec_fitness::{CatastropheThresholds, CatastrophicDimension, FitnessVector};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// حكم التحكيم النهائي. ثلاثة فروع فقط في PR1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjudicationVerdict {
    /// قُبل بعد استيفاء الدستور والدليل المقاس.
    Accepted,
    /// رُفض بسبب مقيس أو دستوري، والسبب إلزامي وآتٍ من `evaluation`.
    Rejected {
        /// السبب الأول — الأقوى.
        primary: String,
        /// بقية الأسباب المقيسة.
        additional: Vec<String>,
    },
    /// تعذّر التحقق: الدليل غائب أو غير مؤهَّل. **ليس رفضًا.**
    Unverified {
        /// سبب تعذّر التحقق.
        reason: String,
    },
}

/// نتيجة التحكيم الكاملة.
#[derive(Debug, Clone)]
pub struct AdjudicationOutcome {
    /// الحكم.
    pub verdict: AdjudicationVerdict,
    /// التقييم الدستوري — `Some` في كل مسار وصل إلى الدستور (الخيار ب).
    pub evaluation: Option<ConstitutionalEvaluation>,
    /// بصمة SHA-256 المحسوبة من الكود المفحوص.
    pub artifact_sha256: String,
    /// أوراكل التحكيم — `Some` فقط إذا كان الأوراكل قد استُشير فعلًا.
    pub oracle: Option<OracleId>,
}

/// مسار `Unverified`: لا تقييم، ولا أوراكل مُبلَّغ عنه.
fn unverified(reason: String, artifact_sha256: String) -> AdjudicationOutcome {
    AdjudicationOutcome {
        verdict: AdjudicationVerdict::Unverified { reason },
        evaluation: None,
        artifact_sha256,
        oracle: None,
    }
}

/// `CatastrophicDimension` لا يطبّق `Display` ⇒ مطابقة صريحة بستة فروع
/// تُرجع (اسم البعد، القيمة المقيسة، العتبة) ليحملها سبب الرفض.
fn catastrophe_parts(
    d: &CatastrophicDimension,
    f: &FitnessVector,
    t: &CatastropheThresholds,
) -> (&'static str, f64, f64) {
    // `CatastrophicDimension` لا يشتق `Copy` ⇒ يُطابَق على `*d` بلا إزاحة.
    match *d {
        CatastrophicDimension::Security => ("security", f.security, t.min_security),
        CatastrophicDimension::Reversibility => {
            ("reversibility", f.reversibility, t.min_reversibility)
        }
        CatastrophicDimension::TestCoverage => {
            ("test_coverage", f.test_coverage, t.min_test_coverage)
        }
        CatastrophicDimension::Maintainability => {
            ("maintainability", f.maintainability, t.min_maintainability)
        }
        CatastrophicDimension::Performance => ("performance", f.performance, t.min_performance),
        CatastrophicDimension::ArchitecturalStability => (
            "architectural_stability",
            f.architectural_stability,
            t.min_architectural_stability,
        ),
    }
}

/// تحكيم الأثر البرمجي بناءً على الكود والدليل المقاس.
///
/// ## بنية المسار
///
/// 1. **الفحص القبلي** — أهلية التحكيم فقط: لا دليل، أو مصدر غير Docker،
///    أو دليل مرتبط بأثر آخر أو بصورة غير معتمدة، أو عينة ناجحة أصغر من 3.
///    كلها `Unverified`، ولا واحدة منها `Rejected`.
/// 2. **الخيار (ب):** وُجد رمز خروج غير صفري ⇒ **لا** يُعترض هنا؛ يتابع إلى
///    الدستور فيرفضه `MeasuredRealityInvariant` من داخل `evaluation`.
/// 3. المعرفة تُبنى من **الدليل** (`runs / (runs + 0.5)`، الصيغة الموروثة
///    من `ec-sandbox/src/reality.rs:115`)، لا من اللياقة.
/// 4. سبب الرفض من `evaluation.violations` + بُعد الكارثة، لا من
///    `evaluation.explanation` (قيمتها ثابتة `"Evaluation complete"`).
///
/// **EC-ADJ-01:** في PR2 تُربط مخرجات Docker بحيث تتحوّل `-1` وأعطال الحاوية
/// إلى خطأ تنفيذ صريح، لا إلى دليل مقاس. انظر `MeasuredEvidence::new`.
pub fn adjudicate(code: &str, evidence: Option<&MeasuredEvidence>) -> AdjudicationOutcome {
    let artifact_sha256 = format!("{:x}", Sha256::digest(code.as_bytes()));

    // ── 1. الفحص القبلي: أهلية التحكيم فقط ───────────────────────────────────
    let ev = match evidence {
        Some(ev) => ev,
        None => {
            return unverified(
                "no measured evidence: acceptance requires real execution".to_string(),
                artifact_sha256,
            )
        }
    };

    if ev.provenance() != ExecutionProvenance::Docker {
        return unverified(
            format!(
                "provenance {:?} confers no authority; only Docker execution does",
                ev.provenance()
            ),
            artifact_sha256,
        );
    }

    if ev.artifact_sha256() != artifact_sha256 {
        return unverified(
            format!(
                "evidence hash {} does not match computed sha256 {} for this artifact",
                ev.artifact_sha256(),
                artifact_sha256
            ),
            artifact_sha256,
        );
    }

    if ev.sandbox_image() != ec_sandbox::docker::DEFAULT_IMAGE {
        return unverified(
            format!(
                "sandbox image {} is not the approved pinned image {}",
                ev.sandbox_image(),
                ec_sandbox::docker::DEFAULT_IMAGE
            ),
            artifact_sha256,
        );
    }

    if ev.all_succeeded() && ev.runs_completed() < MIN_RUNS_FOR_AUTHORITY {
        return unverified(
            format!(
                "insufficient runs: {} of {} required for authority",
                ev.runs_completed(),
                MIN_RUNS_FOR_AUTHORITY
            ),
            artifact_sha256,
        );
    }

    // ── 2. اللياقة تُحسب داخليًا من الكود ────────────────────────────────────
    let fitness = ec_analysis::analyze_code(code);

    // ── 3. المعرفة تُبنى من الدليل لا من اللياقة ─────────────────────────────
    let runs = ev.runs_completed();
    let confidence = (runs as f64 / (runs as f64 + 0.5)).min(0.99);
    let epistemic = EpistemicState::with_measured(
        confidence,
        Evidence::new(runs as u64, 0, ev.reproducibility(), 1.0)
            .expect("reproducibility validated at construction"),
        UncertaintyDecomposition::new(0.1, 0.2, 0.1).expect("valid uncertainty"),
        CalibrationState::default(),
        ev.clone(),
    )
    .expect("confidence in [0,1] and evidence validated at construction");

    // ── 4. الدستور — القانون المقاس أولًا فيأتي سببه أولًا ───────────────────
    let thresholds = CatastropheThresholds::default();
    let constitution = Constitution::new(
        vec![
            Arc::new(MeasuredRealityInvariant),
            Arc::new(SecurityInvariant::default()),
            Arc::new(TestCoverageInvariant::default()),
            Arc::new(ReversibilityInvariant::default()),
        ],
        thresholds.clone(),
    );

    let evaluation = constitution.evaluate(&artifact_sha256, &fitness, &epistemic);

    // ── 5. الحكم ────────────────────────────────────────────────────────────
    let verdict = if evaluation.is_valid {
        AdjudicationVerdict::Accepted
    } else {
        let mut parts: Vec<String> = evaluation
            .violations
            .iter()
            .map(|v| format!("{}: {}", v.invariant_name, v.reason))
            .collect();
        // `ref` لا `move`: `evaluation` يُستخدم كاملًا بعد هذا (E0382).
        if let Some(ref dim) = evaluation.catastrophic {
            let (name, value, limit) = catastrophe_parts(dim, &fitness, &thresholds);
            parts.push(format!(
                "catastrophic {} = {:.2} below {:.2}",
                name, value, limit
            ));
        }
        if parts.is_empty() {
            // فرع مجمَّد موثَّق: `constitution.rs:53` يجعل
            // is_valid == catastrophic.is_none() && Accepted،
            // فـ !is_valid يلزم عنه مخالفة أو كارثة. الوصول إلى هنا عطل صريح،
            // ولا يجوز تصنيع سبب رفض.
            unreachable!(
                "invalid evaluation with neither violation nor catastrophe (constitution.rs:53)"
            );
        }
        let mut it = parts.into_iter();
        AdjudicationVerdict::Rejected {
            primary: it.next().unwrap(),
            additional: it.collect(),
        }
    };

    AdjudicationOutcome {
        verdict,
        evaluation: Some(evaluation),
        artifact_sha256,
        oracle: Some(ev.oracle()),
    }
}
