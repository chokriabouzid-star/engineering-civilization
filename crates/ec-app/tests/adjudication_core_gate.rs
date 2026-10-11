//! بوابة قلب التحكيم — PR1 / المرحلة R.
//!
//! الأسماء عقد بين R وG: لا تُعاد صياغتها، ولا يتغيّر ترتيبها.

use ec_app::adjudication::{adjudicate, AdjudicationVerdict};
use ec_epistemic::{ExecutionProvenance, MeasuredEvidence, OracleId};
use sha2::{Digest, Sha256};

/// الصورة المثبَّتة المعتمدة — نفس `ec_sandbox::docker::DEFAULT_IMAGE`.
const IMAGE: &str = "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523";
/// صورة مثبَّتة ببصمة صالحة لكنها **غير مطابقة** للسياسة.
const OTHER_IMAGE: &str =
    "rust@sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn sha256_hex(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

fn evidence(
    code_hash: String,
    provenance: ExecutionProvenance,
    image: &str,
    exit_codes: Vec<i32>,
) -> MeasuredEvidence {
    MeasuredEvidence::new(
        provenance,
        code_hash,
        image.to_string(),
        exit_codes,
        0.99,
        OracleId::ExitCodeZero,
    )
    .unwrap()
}

fn avg(f: &ec_fitness::FitnessVector) -> f64 {
    (f.security
        + f.reversibility
        + f.test_coverage
        + f.maintainability
        + f.performance
        + f.architectural_stability)
        / 6.0
}

// ─────────────────────────────────────────────────────────────────────────────
// العيّنات
// ─────────────────────────────────────────────────────────────────────────────

const HEALTHY_CODE: &str = r#"/// A pure add function.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-2, 2), 0);
    }
}
"#;

// `count_functions` يعدّ "fn " و`count_test_functions` يعدّ "#[test]"
// ⇒ 1 دالة إنتاج + 2 اختبار = تغطية 2/3 = 0.667 ≥ min_test_coverage (0.60)
// ⇒ لا كارثة تغطية في العيّنة السليمة، فتكون `Accepted` ممكنة في G.

/// ت5: ≥30 تخصيصًا ⇒ `performance` يهبط إلى حدّه الأدنى 0.30،
/// مع `panic!` وغياب اختبارات ⇒ متوسط منخفض خارج نطاق تصادم `clamp` مع العيّنة السليمة.
const UNHEALTHY_CODE: &str = r#"pub fn risky(x: i32) -> i32 {
    let s00 = String::from("x");
    let s01 = String::from("x");
    let s02 = String::from("x");
    let s03 = String::from("x");
    let s04 = String::from("x");
    let s05 = String::from("x");
    let s06 = String::from("x");
    let s07 = String::from("x");
    let s08 = String::from("x");
    let s09 = String::from("x");
    let s10 = String::from("x");
    let s11 = String::from("x");
    let s12 = String::from("x");
    let s13 = String::from("x");
    let s14 = String::from("x");
    let s15 = String::from("x");
    let s16 = String::from("x");
    let s17 = String::from("x");
    let s18 = String::from("x");
    let s19 = String::from("x");
    let s20 = String::from("x");
    let s21 = String::from("x");
    let s22 = String::from("x");
    let s23 = String::from("x");
    let s24 = String::from("x");
    let s25 = String::from("x");
    let s26 = String::from("x");
    let s27 = String::from("x");
    let s28 = String::from("x");
    let s29 = String::from("x");
    if x < 0 {
        panic!("negative value");
    }
    let _ = (s00, s01, s02, s03, s04, s05, s06, s07, s08, s09);
    let _ = (s10, s11, s12, s13, s14, s15, s16, s17, s18, s19);
    let _ = (s20, s21, s22, s23, s24, s25, s26, s27, s28, s29);
    x
}
"#;

/// ت5/G8: 26 سطر `use` ⇒ `architectural_stability = 1.0 − 26×0.02 = 0.48 < 0.50`
/// ⇒ كارثة `ArchitecturalStability`، مع `#[cfg(test)]` لرفع التغطية فوق 0.60
/// فلا تتقدّم كارثة أخرى ولا مخالفة.
const G8_CODE: &str = r#"use std::fmt as p01;
use std::io as p02;
use std::mem as p03;
use std::cmp as p04;
use std::ptr as p05;
use std::vec as p06;
use std::str as p07;
use std::ops as p08;
use std::iter as p09;
use std::time as p10;
use std::sync as p11;
use std::cell as p12;
use std::char as p13;
use std::hash as p14;
use std::num as p15;
use std::env as p16;
use std::ffi as p17;
use std::fs as p18;
use std::net as p19;
use std::pin as p20;
use std::rc as p21;
use std::any as p22;
use std::borrow as p23;
use std::boxed as p24;
use std::clone as p25;
use std::default as p26;

pub fn noop() -> i32 {
    42
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        assert_eq!(super::noop(), 42);
    }

    #[test]
    fn t2() {
        assert!(super::noop() > 0);
    }
}
"#;

// التغطية 2/3 = 0.667 ≥ 0.60 ⇒ كارثة التغطية لا تتقدّم،
// فتكون `ArchitecturalStability` (0.48 < 0.50) هي الكارثة الوحيدة كما خُطِّط.

// ─────────────────────────────────────────────────────────────────────────────
// البوابات التسع
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn g1_divergent_evidence_divergent_verdicts() {
    let hash = sha256_hex(HEALTHY_CODE);
    let good = evidence(
        hash.clone(),
        ExecutionProvenance::Docker,
        IMAGE,
        vec![0, 0, 0],
    );
    let bad = evidence(hash, ExecutionProvenance::Docker, IMAGE, vec![3]);

    let og = adjudicate(HEALTHY_CODE, Some(&good));
    let ob = adjudicate(HEALTHY_CODE, Some(&bad));

    assert_eq!(
        og.verdict,
        AdjudicationVerdict::Accepted,
        "{:?}",
        og.verdict
    );
    assert!(
        matches!(ob.verdict, AdjudicationVerdict::Rejected { .. }),
        "a measured failing run must be Rejected, got {:?}",
        ob.verdict
    );

    // الخيار (ب): كلا الذراعين يصل إلى الدستور.
    let eg = og
        .evaluation
        .as_ref()
        .expect("precondition: good arm must reach the constitution");
    let eb = ob
        .evaluation
        .as_ref()
        .expect("precondition: bad arm must reach the constitution");
    assert_ne!(
        eg.epistemic.measured, eb.epistemic.measured,
        "the epistemic state must carry the divergent measured evidence"
    );
    assert!(
        eb.violations
            .iter()
            .any(|v| v.invariant_name == "MeasuredRealityInvariant"),
        "the failing arm must be rejected BY THE MEASURED-REALITY LAW inside evaluation, \
         not by a rule that pre-empts the constitution"
    );
}

#[test]
fn g2_simulated_provenance_is_unverified() {
    let hash = sha256_hex(HEALTHY_CODE);
    let ev = evidence(hash, ExecutionProvenance::Simulated, IMAGE, vec![0, 0, 0]);
    match adjudicate(HEALTHY_CODE, Some(&ev)).verdict {
        AdjudicationVerdict::Unverified { reason } => assert!(
            reason.contains("Docker"),
            "reason must name the required provenance: {}",
            reason
        ),
        other => panic!(
            "expected Unverified for Simulated provenance, got {:?}",
            other
        ),
    }
}

#[test]
fn g3_no_evidence_is_unverified() {
    match adjudicate(HEALTHY_CODE, None).verdict {
        AdjudicationVerdict::Unverified { reason } => assert!(
            reason.contains("evidence"),
            "reason must name the missing evidence: {}",
            reason
        ),
        other => panic!("expected Unverified when evidence is None, got {:?}", other),
    }
}

#[test]
fn g4_artifact_hash_or_image_mismatch_is_unverified() {
    // الحالة 1 — هاش أثر مخالف
    let ev = evidence(
        "0".repeat(64),
        ExecutionProvenance::Docker,
        IMAGE,
        vec![0, 0, 0],
    );
    match adjudicate(HEALTHY_CODE, Some(&ev)).verdict {
        AdjudicationVerdict::Unverified { reason } => assert!(
            reason.contains("hash") || reason.contains("SHA"),
            "reason must name the hash mismatch: {}",
            reason
        ),
        other => panic!(
            "expected Unverified for artifact hash mismatch, got {:?}",
            other
        ),
    }

    // الحالة 2 — صورة مثبَّتة ببصمة صالحة لكنها غير مطابقة للسياسة
    let hash = sha256_hex(HEALTHY_CODE);
    let ev2 = evidence(
        hash,
        ExecutionProvenance::Docker,
        OTHER_IMAGE,
        vec![0, 0, 0],
    );
    match adjudicate(HEALTHY_CODE, Some(&ev2)).verdict {
        AdjudicationVerdict::Unverified { reason } => assert!(
            reason.contains("image"),
            "reason must name the image policy mismatch: {}",
            reason
        ),
        other => panic!("expected Unverified for non-policy image, got {:?}", other),
    }
}

#[test]
fn g5_identical_evidence_yields_identical_reality_epistemic() {
    let fa = ec_analysis::analyze_code(HEALTHY_CODE);
    let fb = ec_analysis::analyze_code(UNHEALTHY_CODE);
    println!("precondition_healthy_code_fitness={:?}", fa);
    println!("precondition_unhealthy_code_fitness={:?}", fb);

    // ت5: الشرط المسبق على اللياقة الخام لا يكفي — الاصطدام بسقف `clamp`
    // يجعل الثقتين متساويتين فيمرّ الاختبار بلا معنى.
    assert_ne!(
        fa, fb,
        "precondition: samples must produce different fitness"
    );
    let ca = avg(&fa).clamp(0.3, 0.95);
    let cb = avg(&fb).clamp(0.3, 0.95);
    println!("precondition_clamped_confidence_healthy={:.4}", ca);
    println!("precondition_clamped_confidence_unhealthy={:.4}", cb);
    assert_ne!(
        ca, cb,
        "precondition: clamped confidences must differ, got {:.4} and {:.4}",
        ca, cb
    );

    let ea = evidence(
        sha256_hex(HEALTHY_CODE),
        ExecutionProvenance::Docker,
        IMAGE,
        vec![0, 0, 0],
    );
    let eb = evidence(
        sha256_hex(UNHEALTHY_CODE),
        ExecutionProvenance::Docker,
        IMAGE,
        vec![0, 0, 0],
    );

    let oa = adjudicate(HEALTHY_CODE, Some(&ea));
    let ob = adjudicate(UNHEALTHY_CODE, Some(&eb));

    let pa = oa
        .evaluation
        .as_ref()
        .expect("evaluation a")
        .epistemic
        .clone();
    let pb = ob
        .evaluation
        .as_ref()
        .expect("evaluation b")
        .epistemic
        .clone();

    // مكوّنات المشاهدة متطابقة ⇒ ما اشتُقّ منها يجب أن يتطابق، أيًّا كانت اللياقة.
    assert_eq!(
        pa.confidence, pb.confidence,
        "confidence derived from identical execution evidence must match regardless of fitness"
    );
    assert_eq!(
        pa.evidence, pb.evidence,
        "evidence derived from identical execution records must match regardless of fitness"
    );
}

#[test]
fn g6_insufficient_successful_runs_is_unverified() {
    let hash = sha256_hex(HEALTHY_CODE);
    let ev = evidence(hash, ExecutionProvenance::Docker, IMAGE, vec![0]);
    match adjudicate(HEALTHY_CODE, Some(&ev)).verdict {
        AdjudicationVerdict::Unverified { reason } => assert!(
            reason.contains("runs"),
            "reason must name the insufficient runs: {}",
            reason
        ),
        other => panic!(
            "expected Unverified for a single successful run, got {:?}",
            other
        ),
    }
}

#[test]
fn g7_no_rejected_with_empty_or_bogus_explanation() {
    let code = "pub fn do_panic() {\n    panic!(\"fatal\");\n}\n";
    let ev = evidence(
        sha256_hex(code),
        ExecutionProvenance::Docker,
        IMAGE,
        vec![1],
    );
    match adjudicate(code, Some(&ev)).verdict {
        AdjudicationVerdict::Rejected { primary, .. } => {
            assert!(
                !primary.trim().is_empty(),
                "the rejection reason cannot be empty"
            );
            assert_ne!(
                primary, "Evaluation complete",
                "the legacy content-free explanation is forbidden as a rejection reason"
            );
        }
        other => panic!(
            "expected Rejected for a measured failing run, got {:?}",
            other
        ),
    }
}

#[test]
fn g8_catastrophe_without_violations_has_explicit_reason() {
    let fit = ec_analysis::analyze_code(G8_CODE);
    println!("precondition_g8_sample_fitness={:?}", fit);
    println!(
        "precondition_g8_sample_architectural_stability={:.4}",
        fit.architectural_stability
    );
    assert!(
        fit.architectural_stability < 0.50,
        "precondition: the sample must trigger a catastrophe at the 0.50 threshold; got {:.4}",
        fit.architectural_stability
    );

    let ev = evidence(
        sha256_hex(G8_CODE),
        ExecutionProvenance::Docker,
        IMAGE,
        vec![0, 0, 0],
    );
    let outcome = adjudicate(G8_CODE, Some(&ev));
    let ev2 = outcome
        .evaluation
        .as_ref()
        .expect("precondition: evaluation must exist");
    println!(
        "precondition_g8_measured_catastrophic={:?}",
        ev2.catastrophic
    );
    println!("precondition_g8_measured_violations={:?}", ev2.violations);

    assert!(
        ev2.catastrophic.is_some(),
        "precondition: the sample must produce a catastrophic dimension"
    );
    assert!(
        ev2.violations.is_empty(),
        "precondition: G8 exercises catastrophe WITHOUT violations; got {:?}",
        ev2.violations
    );

    match outcome.verdict {
        AdjudicationVerdict::Rejected { primary, .. } => {
            assert_ne!(
                primary, "Evaluation complete",
                "the legacy explanation is forbidden"
            );
            assert!(
                primary.contains("catastrophic"),
                "the reason must name the catastrophe: {}",
                primary
            );
        }
        other => panic!(
            "expected Rejected for a catastrophic sample, got {:?}",
            other
        ),
    }
}

#[test]
fn g9_single_failing_run_is_rejected_with_evidence() {
    let hash = sha256_hex(HEALTHY_CODE);
    let ev = evidence(hash, ExecutionProvenance::Docker, IMAGE, vec![3]);
    match adjudicate(HEALTHY_CODE, Some(&ev)).verdict {
        AdjudicationVerdict::Rejected { primary, .. } => {
            assert!(
                primary.contains('3'),
                "reason must carry the exit code 3: {}",
                primary
            );
            assert!(
                primary.contains("1/1"),
                "reason must carry the run ratio: {}",
                primary
            );
        }
        other => panic!(
            "option (B): a single measured failing run must be Rejected BY THE LAW, got {:?}",
            other
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// الضوابط (تبقى خضراء في R)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn g10_reject_minus_one_exit_code() {
    let res = MeasuredEvidence::new(
        ExecutionProvenance::Docker,
        "a".repeat(64),
        IMAGE.to_string(),
        vec![0, -1, 0],
        0.99,
        OracleId::ExitCodeZero,
    );
    assert!(
        res.is_err(),
        "-1 marks a missing program result (signal/timeout/runner failure), not a program exit"
    );
}

#[test]
fn control_old_from_reality_remains_confidence_only() {
    let reality = ec_sandbox::RealityVector::test_fixture(1.0, 0.98, 0.95, 3);
    let ep = ec_app::build_epistemic_from_reality(&reality).unwrap();
    assert!(
        ep.measured.is_none(),
        "the legacy path must remain confidence-only; measured={:?}",
        ep.measured
    );
}

#[test]
fn control_pipeline_file_frozen() {
    const PIPELINE: &str = include_str!("../src/pipeline.rs");
    assert_eq!(
        PIPELINE.lines().count(),
        992,
        "pipeline.rs must remain untouched (992 lines)"
    );
    let hex = format!("{:x}", Sha256::digest(PIPELINE.as_bytes()));
    assert_eq!(
        hex, "d00fd0fd4523117e541cde191bcd90f86f0604e8f80af5a893c704d65b745a97",
        "pipeline.rs content must remain byte-identical"
    );
}

#[test]
fn control_from_reality_signature_pin() {
    let _f: fn(
        &ec_sandbox::RealityVector,
    ) -> ec_epistemic::EpistemicResult<ec_epistemic::EpistemicState> =
        ec_app::build_epistemic_from_reality;
}
