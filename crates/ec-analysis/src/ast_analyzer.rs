#![forbid(unsafe_code)]

//! AstAnalyzer — يحلل AST بدلاً من keyword counting

use syn::{File, visit::Visit};
use ec_fitness::FitnessVector;
use crate::report::{AnalysisReport, ConfidenceVector, AnalysisWarning};
use crate::visitors::{
    UnsafeVisitor, ComplexityVisitor, TestVisitor,
    CouplingVisitor, SideEffectVisitor, PerformanceVisitor,
};

pub struct AstAnalyzer;

impl Default for AstAnalyzer {
    fn default() -> Self { Self::new() }
}

impl AstAnalyzer {
    pub fn new() -> Self { Self }

    pub fn analyze_file(&self, ast: &File) -> AnalysisReport {
        let mut unsafe_v   = UnsafeVisitor::new();
        let mut complex_v  = ComplexityVisitor::new();
        let mut test_v     = TestVisitor::new();
        let mut coupling_v = CouplingVisitor::new();
        let mut side_v     = SideEffectVisitor::new();
        let mut perf_v     = PerformanceVisitor::new();

        unsafe_v.visit_file(ast);
        complex_v.visit_file(ast);
        test_v.visit_file(ast);
        coupling_v.visit_file(ast);
        side_v.visit_file(ast);
        perf_v.visit_file(ast);

        let (security,   sec_conf)   = unsafe_v.score();
        let (maint,      maint_conf) = complex_v.score();
        let (coverage,   cov_conf)   = test_v.score();
        let (stability,  stab_conf)  = coupling_v.score();
        let (revers,     rev_conf)   = side_v.score();
        let (perf,       perf_conf)  = perf_v.score();

        let fitness = FitnessVector {
            security,
            reversibility: revers,
            test_coverage: coverage,
            maintainability: maint,
            performance: perf,
            architectural_stability: stability,
        };

        let confidence = ConfidenceVector {
            security: sec_conf,
            test_coverage: cov_conf,
            maintainability: maint_conf,
            performance: perf_conf,
            architectural_stability: stab_conf,
            reversibility: rev_conf,
        };

        let mut warnings = vec![];

        if unsafe_v.unjustified_count() > 0 {
            warnings.push(AnalysisWarning::UnsafeWithoutComment {
                count: unsafe_v.unjustified_count(),
            });
        }

        for (name, cc) in complex_v.high_complexity() {
            if cc > 20 {
                warnings.push(AnalysisWarning::HighComplexity {
                    function: name,
                    cc,
                });
            }
        }

        if confidence.overall() < 0.50 {
            warnings.push(AnalysisWarning::LowConfidence {
                dimension: "overall".into(),
                value: confidence.overall(),
            });
        }

        AnalysisReport {
            fitness,
            confidence,
            warnings,
            parse_successful: true,
        }
    }
}
