#![forbid(unsafe_code)]

use ec_analysis::analyze_code_full;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

/// ملف تم مسحه في التمريرة الأولى
struct ScannedFile {
    path: PathBuf,
    crate_root: PathBuf,
    is_test_or_bench: bool,
    analysis: ec_analysis::AnalysisReport,
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

/// فحص مجلد كامل (بشكل متكرر) — تمريرتان
pub fn check_workspace(root: &Path) -> WorkspaceReport {
    // ── تمريرة 1: مسح وتجميع ──
    let mut files: Vec<ScannedFile> = Vec::new();
    collect_files(root, &mut files);

    // تجميع test_fns / production_fns لكل crate
    let mut crate_counts: HashMap<PathBuf, (usize, usize)> = HashMap::new();
    for sf in &files {
        let entry = crate_counts.entry(sf.crate_root.clone()).or_insert((0, 0));
        entry.0 += sf.analysis.test_fns;
        entry.1 += sf.analysis.production_fns;
    }

    // حساب تغطية كل crate
    let crate_coverage: HashMap<PathBuf, f64> = crate_counts
        .iter()
        .map(|(root, (test_fns, prod_fns))| {
            let cov = if *prod_fns == 0 {
                // crate بلا دوال إنتاجية — حيادي
                1.0
            } else {
                (*test_fns as f64 / *prod_fns as f64).min(1.0)
            };
            (root.clone(), cov)
        })
        .collect();

    // ── تمريرة 2: تقييم وتطبيق العتبات ──
    let mut report = WorkspaceReport {
        files_scanned: files.len(),
        files_passed: 0,
        files_failed: 0,
        violations: vec![],
        project_score: 0.0,
    };
    let mut totals = DimensionTotals::new();

    for sf in &files {
        let f = &sf.analysis.fitness;

        // تغطية فعّالة: ملفات src تأخذ تغطية الـcrate، ملفات tests/benches تبقى كما هي
        let effective_coverage = if sf.is_test_or_bench {
            f.test_coverage
        } else {
            crate_coverage
                .get(&sf.crate_root)
                .copied()
                .unwrap_or(f.test_coverage)
        };

        totals.add(
            f.security,
            effective_coverage,
            f.maintainability,
            f.performance,
            f.architectural_stability,
            f.reversibility,
        );

        let path_str = sf.path.to_string_lossy().to_string();

        // بناء قائمة العتبات — مع استثناء test_coverage و reversibility لملفات الاختبار
        let mut thresholds: Vec<(&str, f64, f64)> = vec![
            ("security", f.security, 0.70),
            ("maintainability", f.maintainability, 0.40),
            ("performance", f.performance, 0.20),
            ("architectural_stability", f.architectural_stability, 0.50),
        ];

        if sf.is_test_or_bench {
            // ملفات tests/benches: لا فحص عتبة لـ test_coverage و reversibility
            // القيم تُحسب وتُعرض للشفافية، لكن لا تُسجَّل كانتهاك
        } else {
            // ملفات إنتاج: فحص كامل مع التغطية الفعّالة
            thresholds.insert(1, ("test_coverage", effective_coverage, 0.60));
            thresholds.push(("reversibility", f.reversibility, 0.30));
        }

        let mut failed = false;
        for (dim, value, threshold) in thresholds {
            if value < threshold {
                failed = true;
                report.violations.push(FileViolation {
                    path: path_str.clone(),
                    dimension: dim.to_string(),
                    value,
                    threshold,
                });
                break;
            }
        }
        if failed {
            report.files_failed += 1;
        }
    }

    report.project_score = totals.project_score(report.files_scanned);
    report.files_passed = report.files_scanned - report.files_failed;
    report
}

/// مسح متكرر لجمع الملفات مع تحليلها
fn collect_files(dir: &Path, files: &mut Vec<ScannedFile>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "target" || name == ".git" || name == "node_modules" {
                continue;
            }
            collect_files(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            if let Some(sf) = scan_file(&path) {
                files.push(sf);
            }
        }
    }
}

/// تحليل ملف واحد وإرجاع بياناته
fn scan_file(path: &Path) -> Option<ScannedFile> {
    let code = std::fs::read_to_string(path).ok()?;
    let analysis = analyze_code_full(&code);
    let crate_root = find_crate_root(path);
    let is_test_or_bench = path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some("tests") | Some("benches")
        )
    });
    Some(ScannedFile {
        path: path.to_path_buf(),
        crate_root,
        is_test_or_bench,
        analysis,
    })
}

/// أقرب مجلد أب يحوي Cargo.toml
fn find_crate_root(path: &Path) -> PathBuf {
    let mut dir = path.parent();
    while let Some(d) = dir {
        if d.join("Cargo.toml").exists() {
            return d.to_path_buf();
        }
        dir = d.parent();
    }
    // fallback: مجلد الملف نفسه
    path.parent().unwrap_or(path).to_path_buf()
}
