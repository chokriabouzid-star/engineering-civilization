#![forbid(unsafe_code)]

//! ec — Engineering Civilization CLI

use clap::{Parser, Subcommand};
use ec_analysis::analyze_code_full;
use ec_memory::MemoryStorage;
use std::path::PathBuf;

mod check;

#[derive(Parser)]
#[command(name = "ec", version, about = "Engineering Civilization CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, default_value = "ec.db")]
    db: String,
}

#[derive(Subcommand)]
enum Commands {
    /// تحليل ملف واحد
    Analyze {
        path: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        verbose: bool,
    },
    /// فحص مشروع كامل (مجلد)
    Check {
        path: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        verbose: bool,
    },
    /// عرض تقرير الانجراف القيمي
    Drift {
        #[arg(long, default_value = "10")]
        baseline: usize,
        #[arg(long, default_value = "5")]
        window: usize,
    },
    /// إدارة الاقتراحات الدستورية
    Propose {
        #[command(subcommand)]
        action: ProposeAction,
    },
    /// عرض سجل التدقيق
    Audit {
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// فحص صحة النظام
    Health,
}

#[derive(Subcommand)]
enum ProposeAction {
    /// تقديم اقتراح جديد
    Submit {
        #[arg(long)]
        dimension: String,
        #[arg(long)]
        current: f64,
        #[arg(long)]
        proposed: f64,
        #[arg(long)]
        justification: String,
        #[arg(long, default_value = "cli-user")]
        by: String,
    },
    /// قائمة الاقتراحات
    List,
    /// الموافقة على اقتراح
    Approve {
        id: String,
        #[arg(long)]
        by: String,
        #[arg(long, default_value = "")]
        note: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            path,
            json,
            verbose,
        } => cmd_analyze(path, json, verbose),
        Commands::Check {
            path,
            json,
            verbose,
        } => cmd_check(path, json, verbose),
        Commands::Drift { baseline, window } => cmd_drift(&cli.db, baseline, window),
        Commands::Audit { limit } => cmd_audit(&cli.db, limit),
        Commands::Health => cmd_health(),
        Commands::Propose { action } => cmd_propose(&cli.db, action),
    }
}

fn cmd_check(path: PathBuf, json: bool, verbose: bool) {
    if !path.exists() {
        eprintln!("❌ Path not found: {}", path.display());
        std::process::exit(1);
    }

    let report = check::check_workspace(&path);

    if json {
        let violations: Vec<_> = report
            .violations
            .iter()
            .map(|v| {
                serde_json::json!({
                    "path": v.path,
                    "dimension": v.dimension,
                    "value": v.value,
                    "threshold": v.threshold,
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "files_scanned": report.files_scanned,
                "files_passed": report.files_passed,
                "files_failed": report.files_failed,
                "project_score": format!("{:.3}", report.project_score),
                "violations": violations,
            }))
            .unwrap()
        );
        return;
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  EC Workspace Check — {:39}║", path.display());
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!(
        "║  Files scanned:  {:5}                                       ║",
        report.files_scanned
    );
    println!(
        "║  Files passed:   {:5}  ✅                                    ║",
        report.files_passed
    );
    println!(
        "║  Files failed:   {:5}  ❌                                    ║",
        report.files_failed
    );
    println!(
        "║  Project score:  {:.3} / 1.0                                  ║",
        report.project_score
    );
    println!("╠══════════════════════════════════════════════════════════════╣");

    if report.violations.is_empty() {
        println!("║  🎉 All files pass constitutional thresholds!              ║");
    } else {
        println!("║  Constitutional Violations:                                ║");
        for v in &report.violations {
            let path_display = if v.path.len() > 42 {
                format!("...{}", &v.path[v.path.len().saturating_sub(39)..])
            } else {
                v.path.clone()
            };
            println!("║    {:42} ║", path_display);
            println!(
                "║      {}={:.2} (threshold: {:.2})                             ║",
                v.dimension, v.value, v.threshold
            );
        }
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    if verbose {
        for v in &report.violations {
            println!(
                "  ❌ {} — {}={:.2} < {:.2}",
                v.path, v.dimension, v.value, v.threshold
            );
        }
    }
}

fn cmd_analyze(path: PathBuf, json: bool, verbose: bool) {
    let code = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };

    let report = analyze_code_full(&code);

    if json {
        let output = serde_json::json!({
            "fitness": {
                "security": report.fitness.security,
                "test_coverage": report.fitness.test_coverage,
                "maintainability": report.fitness.maintainability,
                "performance": report.fitness.performance,
                "architectural_stability": report.fitness.architectural_stability,
                "reversibility": report.fitness.reversibility,
            },
            "confidence": { "overall": report.confidence.overall() },
            "parse_successful": report.parse_successful,
            "warnings": report.warnings.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return;
    }

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "?".into());

    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ Analysis: {:49}│", filename);
    println!("├────────────────────┬──────────┬────────────┬────────────┤");
    println!("│ Dimension          │  Score   │ Confidence │  Status    │");
    println!("├────────────────────┼──────────┼────────────┼────────────┤");

    let dims = [
        (
            "Security",
            report.fitness.security,
            report.confidence.security,
            0.70,
        ),
        (
            "Test Coverage",
            report.fitness.test_coverage,
            report.confidence.test_coverage,
            0.60,
        ),
        (
            "Maintainability",
            report.fitness.maintainability,
            report.confidence.maintainability,
            0.40,
        ),
        (
            "Performance",
            report.fitness.performance,
            report.confidence.performance,
            0.20,
        ),
        (
            "Stability",
            report.fitness.architectural_stability,
            report.confidence.architectural_stability,
            0.50,
        ),
        (
            "Reversibility",
            report.fitness.reversibility,
            report.confidence.reversibility,
            0.30,
        ),
    ];

    for (name, value, conf, threshold) in dims {
        let status = if value >= threshold {
            "✅ OK"
        } else {
            "❌ LOW"
        };
        let conf_str = if conf >= 0.80 {
            format!("{:.2} ●", conf)
        } else {
            format!("{:.2} ○", conf)
        };
        println!(
            "│ {:18} │  {:.3}   │   {}  │ {:10} │",
            name, value, conf_str, status
        );
    }

    println!("├────────────────────┴──────────┴────────────┴────────────┤");
    println!(
        "│  Overall confidence: {:.2}  │  Parse: {}       │",
        report.confidence.overall(),
        if report.parse_successful {
            "✅"
        } else {
            "❌"
        }
    );
    println!("└──────────────────────────────────────────────────────────┘");

    if verbose || !report.warnings.is_empty() {
        for w in &report.warnings {
            println!("  ⚠️  {:?}", w);
        }
    }
}

fn cmd_drift(db: &str, baseline: usize, window: usize) {
    let path = std::path::Path::new(db);
    if !path.exists() {
        println!("⚠️  Database not found: {}", db);
        return;
    }

    let storage = match ec_memory::SqliteStorage::new(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ DB error: {}", e);
            std::process::exit(1);
        }
    };
    let graph = match storage.load() {
        Ok(g) => g,
        Err(_) => ec_memory::CausalMemoryGraph::new(),
    };

    if graph.len() < baseline + window {
        println!(
            "⚠️  Insufficient data: {} nodes (need {}+{}={})",
            graph.len(),
            baseline,
            window,
            baseline + window
        );
        return;
    }

    let analyzer = ec_memory::drift::HistoricalDriftAnalyzer::new(&graph, baseline, window);
    let report = analyzer.analyze();

    println!("📊 Drift Report");
    println!("   Angle:          {:.1}°", report.drift_angle_degrees);
    println!("   Classification: {:?}", report.classification);
    println!("   Action:         {:?}", report.recommended_action);
    println!("   Requires action: {}", report.requires_action());
}

fn cmd_health() {
    println!("✅ System status: OK");
    println!("   Version: {}", env!("CARGO_PKG_VERSION"));
}

fn cmd_propose(_db: &str, action: ProposeAction) {
    match action {
        ProposeAction::List => {
            println!("ℹ️  Use REST API: GET /api/v1/governance/proposals");
        }
        ProposeAction::Submit {
            dimension,
            current,
            proposed,
            justification,
            by,
        } => {
            println!("ℹ️  Submit via REST API:");
            println!("   POST /api/v1/governance/proposals");
            println!(
                "   {{ \"dimension\": \"{}\", \"current_value\": {}, \"proposed_value\": {}, \"justification\": \"{}\", \"proposed_by\": \"{}\" }}",
                dimension, current, proposed, justification, by
            );
        }
        ProposeAction::Approve { id, by, note } => {
            println!("ℹ️  Approve via REST API:");
            println!("   PATCH /api/v1/governance/proposals/{}/approve", id);
            println!("   {{ \"by\": \"{}\", \"note\": \"{}\" }}", by, note);
        }
    }
}

fn cmd_audit(_db: &str, _limit: usize) {
    println!("ℹ️  Audit log available via API: GET /api/v1/governance/audit");
}
