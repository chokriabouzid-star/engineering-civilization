#![forbid(unsafe_code)]

use ec_analysis::analyze_code_full;
use std::path::Path;

/// نتيجة فحص مشروع كامل
pub struct WorkspaceReport {
    pub files_scanned: usize,
    pub files_passed: usize,
    pub files_failed: usize,
    pub violations: Vec<FileViolation>,
    pub project_score: f64,
}

pub struct FileViolation {
    pub path: String,
    pub dimension: String,
    pub value: f64,
    pub threshold: f64,
}

/// فحص مجلد كامل (بشكل متكرر)
pub fn check_workspace(root: &Path) -> WorkspaceReport {
    let mut report = WorkspaceReport {
        files_scanned: 0,
        files_passed: 0,
        files_failed: 0,
        violations: vec![],
        project_score: 0.0,
    };

    let mut total_security = 0.0;
    let mut total_coverage = 0.0;
    let mut total_maintain = 0.0;
    let mut total_perf = 0.0;
    let mut total_stability = 0.0;
    let mut total_revers = 0.0;

    scan_dir(root, &mut report, &mut total_security, &mut total_coverage,
             &mut total_maintain, &mut total_perf, &mut total_stability, &mut total_revers);

    if report.files_scanned > 0 {
        let n = report.files_scanned as f64;
        report.project_score = (total_security + total_coverage + total_maintain
            + total_perf + total_stability + total_revers) / (6.0 * n);
    }

    report.files_passed = report.files_scanned - report.files_failed;
    report
}

fn scan_dir(
    dir: &Path,
    report: &mut WorkspaceReport,
    total_security: &mut f64,
    total_coverage: &mut f64,
    total_maintain: &mut f64,
    total_perf: &mut f64,
    total_stability: &mut f64,
    total_revers: &mut f64,
) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // تجاهل target/ و .git/
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                scan_dir(&path, report, total_security, total_coverage,
                         total_maintain, total_perf, total_stability, total_revers);
            } else if path.extension().map_or(false, |e| e == "rs") {
                analyze_file(&path, report, total_security, total_coverage,
                             total_maintain, total_perf, total_stability, total_revers);
            }
        }
    }
}

fn analyze_file(
    path: &Path,
    report: &mut WorkspaceReport,
    total_security: &mut f64,
    total_coverage: &mut f64,
    total_maintain: &mut f64,
    total_perf: &mut f64,
    total_stability: &mut f64,
    total_revers: &mut f64,
) {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let result = analyze_code_full(&code);
    report.files_scanned += 1;

    let f = &result.fitness;
    *total_security += f.security;
    *total_coverage += f.test_coverage;
    *total_maintain += f.maintainability;
    *total_perf += f.performance;
    *total_stability += f.architectural_stability;
    *total_revers += f.reversibility;

    let path_str = path.to_string_lossy().to_string();

    // فحص العتبات الدستورية
    let thresholds = [
        ("security", f.security, 0.70),
        ("test_coverage", f.test_coverage, 0.60),
        ("maintainability", f.maintainability, 0.40),
        ("performance", f.performance, 0.20),
        ("reversibility", f.reversibility, 0.30),
        ("architectural_stability", f.architectural_stability, 0.50),
    ];

    for (dim, value, threshold) in thresholds {
        if value < threshold {
            report.files_failed += 1;
            report.violations.push(FileViolation {
                path: path_str.clone(),
                dimension: dim.to_string(),
                value,
                threshold,
            });
            break; // نسجل الملف مرة واحدة فقط
        }
    }
}
