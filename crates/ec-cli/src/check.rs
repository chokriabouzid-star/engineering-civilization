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

/// حالة تجميع الأبعاد أثناء المسح
struct DimensionTotals {
    security: f64,
    coverage: f64,
    maintain: f64,
    perf: f64,
    stability: f64,
    revers: f64,
}

impl DimensionTotals {
    fn new() -> Self {
        Self {
            security: 0.0,
            coverage: 0.0,
            maintain: 0.0,
            perf: 0.0,
            stability: 0.0,
            revers: 0.0,
        }
    }

    fn add(
        &mut self,
        security: f64,
        coverage: f64,
        maintain: f64,
        perf: f64,
        stability: f64,
        revers: f64,
    ) {
        self.security += security;
        self.coverage += coverage;
        self.maintain += maintain;
        self.perf += perf;
        self.stability += stability;
        self.revers += revers;
    }

    fn project_score(&self, n: usize) -> f64 {
        if n == 0 {
            return 0.0;
        }
        let nf = n as f64;
        (self.security + self.coverage + self.maintain + self.perf + self.stability + self.revers)
            / (6.0 * nf)
    }
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
    let mut totals = DimensionTotals::new();

    scan_dir(root, &mut report, &mut totals);

    report.project_score = totals.project_score(report.files_scanned);
    report.files_passed = report.files_scanned - report.files_failed;
    report
}

fn scan_dir(dir: &Path, report: &mut WorkspaceReport, totals: &mut DimensionTotals) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                scan_dir(&path, report, totals);
            } else if path.extension().is_some_and(|e| e == "rs") {
                analyze_file(&path, report, totals);
            }
        }
    }
}

fn analyze_file(path: &Path, report: &mut WorkspaceReport, totals: &mut DimensionTotals) {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let result = analyze_code_full(&code);
    report.files_scanned += 1;

    let f = &result.fitness;
    totals.add(
        f.security,
        f.test_coverage,
        f.maintainability,
        f.performance,
        f.architectural_stability,
        f.reversibility,
    );

    let path_str = path.to_string_lossy().to_string();

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
            break;
        }
    }
}
