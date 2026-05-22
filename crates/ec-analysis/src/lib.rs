#![forbid(unsafe_code)]

//! ec-analysis — Static Code Analysis
//! Week 19: analyze_code() — keyword heuristic (لا تتغير)
//! Week 28: analyze_code_full() + AnalysisReport + syn AST (إضافة فقط)

pub mod analyzer;
pub mod complexity;
pub mod coverage;
pub mod metrics;
pub mod reversibility;
pub mod security;

// Week 28: جديد — إضافي فقط
pub mod report;
pub mod ast_analyzer;
pub mod visitors;

pub use report::{AnalysisReport, ConfidenceVector, AnalysisWarning};
pub use ast_analyzer::AstAnalyzer;

use ec_fitness::FitnessVector;

/// الواجهة القديمة — لا تتغير أبداً (Week 19)
/// جميع الاختبارات الموجودة تعتمد عليها
pub fn analyze_code(code: &str) -> FitnessVector {
    analyzer::analyze_code(code)
}

/// الواجهة الجديدة — إضافية (Week 28)
/// تُنتج FitnessVector + ConfidenceVector + warnings
/// تستخدم syn AST بدلاً من keyword counting
pub fn analyze_code_full(code: &str) -> AnalysisReport {
    match syn::parse_str::<syn::File>(code) {
        Ok(ast)  => AstAnalyzer::new().analyze_file(&ast),
        Err(e)   => AnalysisReport::unparseable(e.to_string()),
    }
}
