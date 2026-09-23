#![forbid(unsafe_code)]

//! ec-analysis — Static Code Analysis
//! Week 19: analyze_code() — keyword heuristic (لا تتغير)
//! Week 28: analyze_code_full() + AnalysisReport + syn AST (إضافة فقط)
//! Sprint 1: F1 Depth Guard — حماية من تجاوز الـ stack عند التعشيق العميق

pub mod analyzer;
pub mod complexity;
pub mod coverage;
pub mod depth_guard;
pub mod metrics;
pub mod reversibility;
pub mod security;

// Week 28: جديد — إضافي فقط
pub mod ast_analyzer;
pub mod report;
pub mod visitors;

pub use ast_analyzer::AstAnalyzer;
pub use depth_guard::{check_nesting_safety, DepthError, MAX_DELIMITER_DEPTH, MAX_UNARY_CHAIN};
pub use report::{AnalysisReport, AnalysisWarning, ConfidenceVector};

use ec_fitness::FitnessVector;

/// الواجهة القديمة — لا تتغير أبداً (Week 19)
/// جميع الاختبارات الموجودة تعتمد عليها
pub fn analyze_code(code: &str) -> FitnessVector {
    analyzer::analyze_code(code)
}

/// الواجهة الجديدة — إضافية (Week 28 + Sprint 1 Hardened)
/// تُنتج FitnessVector + ConfidenceVector + warnings
/// تستخدم syn AST بدلاً من keyword counting مع فاحص عمق وقائي لمنع stack overflow
pub fn analyze_code_full(code: &str) -> AnalysisReport {
    if let Err(depth_err) = check_nesting_safety(code) {
        return AnalysisReport::unparseable(depth_err.to_string());
    }

    match syn::parse_str::<syn::File>(code) {
        Ok(ast) => AstAnalyzer::new().analyze_file(&ast),
        Err(e) => AnalysisReport::unparseable(e.to_string()),
    }
}
