#![forbid(unsafe_code)]

//! `ec adjudge` — shadow mode (ADR-033 §6).
//!
//! يطبع الحكم والدليل ويخرج 0 دائمًا، أيًّا كان الحكم. سياسة الحجب
//! قرار لاحق منفصل.

use ec_app::adjudication::{adjudicate_executed, AdjudicationVerdict, ExecutedAdjudication};
use std::path::PathBuf;

/// مصيّر JSON نقي — قابل للاختبار بلا CLI وبلا Docker.
pub fn render_json(executed: &ExecutedAdjudication) -> serde_json::Value {
    let (verdict, reason, additional) = match &executed.outcome.verdict {
        AdjudicationVerdict::Accepted => ("Accepted", None, Vec::new()),
        AdjudicationVerdict::Rejected {
            primary,
            additional,
        } => ("Rejected", Some(primary.clone()), additional.clone()),
        AdjudicationVerdict::Unverified { reason } => {
            ("Unverified", Some(reason.clone()), Vec::new())
        }
    };
    let evidence = executed
        .evidence
        .as_ref()
        .map(|e| serde_json::to_value(e).expect("validated MeasuredEvidence must serialize"));
    serde_json::json!({
        "verdict": verdict,
        "reason": reason,
        "additional": additional,
        "artifact_sha256": executed.outcome.artifact_sha256.clone(),
        "evidence": evidence,
    })
}

pub fn cmd_adjudge(path: PathBuf, json: bool) {
    let code = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };

    let executed = adjudicate_executed(&code);

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&render_json(&executed)).unwrap()
        );
        return;
    }

    println!("Artifact: {}", path.display());
    println!("SHA-256:  {}", executed.outcome.artifact_sha256);
    match &executed.outcome.verdict {
        AdjudicationVerdict::Accepted => println!("Verdict:  Accepted"),
        AdjudicationVerdict::Rejected {
            primary,
            additional,
        } => {
            println!("Verdict:  Rejected");
            println!("Reason:   {}", primary);
            for r in additional {
                println!("          {}", r);
            }
        }
        AdjudicationVerdict::Unverified { reason } => {
            println!("Verdict:  Unverified");
            println!("Reason:   {}", reason);
        }
    }
    if let Some(e) = &executed.evidence {
        println!("Evidence:");
        println!("  provenance:     {:?}", e.provenance());
        println!("  oracle:         {:?}", e.oracle());
        println!("  sandbox image:  {}", e.sandbox_image());
        println!("  exit codes:     {:?}", e.exit_codes());
        println!("  reproducibility: {:.4}", e.reproducibility());
    }
    // shadow mode: خروج 0 regardless of verdict.
}

#[cfg(test)]
mod tests {
    use super::*;
    use ec_app::adjudication::{AdjudicationOutcome, ExecutedAdjudication};
    use ec_epistemic::{ExecutionProvenance, MeasuredEvidence, OracleId};

    #[test]
    fn render_json_preserves_verdict_reasons_and_evidence() {
        let evidence = MeasuredEvidence::new(
            ExecutionProvenance::Docker,
            "a".repeat(64),
            "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523"
                .to_string(),
            vec![0, 0, 0],
            0.99,
            OracleId::ExitCodeZero,
        )
        .expect("valid evidence");

        let executed = ExecutedAdjudication {
            outcome: AdjudicationOutcome {
                verdict: AdjudicationVerdict::Rejected {
                    primary: "primary reason".to_string(),
                    additional: vec!["second reason".to_string()],
                },
                evaluation: None,
                artifact_sha256: "a".repeat(64),
                oracle: Some(OracleId::ExitCodeZero),
            },
            evidence: Some(evidence.clone()),
        };

        let v = render_json(&executed);
        assert_eq!(v["verdict"], "Rejected");
        assert_eq!(v["reason"], "primary reason");
        assert_eq!(v["additional"][0], "second reason");

        let ev = &v["evidence"];
        assert_eq!(ev["exit_codes"].as_array().map(|a| a.len()), Some(3));
        assert_eq!(ev["exit_codes"][0], 0);
        assert_eq!(
            ev["sandbox_image"],
            "rust@sha256:31ee7fc65186be7e0e0ccb3f2ca305f14e4739e7642a1ae65753aa5d7b874523"
        );
        assert_eq!(ev["provenance"], "Docker");
        assert_eq!(ev["oracle"], "ExitCodeZero");
        assert!((ev["reproducibility"].as_f64().unwrap() - 0.99).abs() < 1e-9);
        assert_eq!(ev["artifact_sha256"], "a".repeat(64));

        let unverified = ExecutedAdjudication {
            outcome: AdjudicationOutcome {
                verdict: AdjudicationVerdict::Unverified {
                    reason: "docker unavailable: execution could not start".to_string(),
                },
                evaluation: None,
                artifact_sha256: "b".repeat(64),
                oracle: None,
            },
            evidence: None,
        };
        let u = render_json(&unverified);
        assert_eq!(u["verdict"], "Unverified");
        assert_eq!(u["reason"], "docker unavailable: execution could not start");
        assert!(u["evidence"].is_null());

        let accepted = ExecutedAdjudication {
            outcome: AdjudicationOutcome {
                verdict: AdjudicationVerdict::Accepted,
                evaluation: None,
                artifact_sha256: "c".repeat(64),
                oracle: Some(OracleId::ExitCodeZero),
            },
            evidence: Some(evidence),
        };
        let a = render_json(&accepted);
        assert_eq!(a["verdict"], "Accepted");
        assert!(a["reason"].is_null());
        assert!(a["evidence"].is_object());
    }
}
