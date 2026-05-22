#![forbid(unsafe_code)]

//! AnalysisReport + ConfidenceVector — Week 28
//! إضافة فقط — لا تكسر analyze_code() القديم

use ec_fitness::FitnessVector;

/// تقرير التحليل الكامل (يشمل FitnessVector + confidence)
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    /// للتوافق مع الكود القديم — نفس ما تُنتجه analyze_code()
    pub fitness: FitnessVector,
    /// جديد — منفصل تماماً عن fitness (D8)
    pub confidence: ConfidenceVector,
    /// تحذيرات من التحليل
    pub warnings: Vec<AnalysisWarning>,
    /// هل نجح parse الـ AST؟
    pub parse_successful: bool,
}

impl AnalysisReport {
    /// عند فشل الـ parse — fallback آمن
    pub fn unparseable(reason: String) -> Self {
        Self {
            fitness: FitnessVector::default(),
            confidence: ConfidenceVector::zero(),
            warnings: vec![AnalysisWarning::ParseFailed(reason)],
            parse_successful: false,
        }
    }
}

/// مستوى الثقة في كل بُعد من أبعاد FitnessVector
/// D8: لا تُعدَّل FitnessVector — هذا منفصل تماماً
#[derive(Debug, Clone)]
pub struct ConfidenceVector {
    pub security: f64,
    pub test_coverage: f64,
    pub maintainability: f64,
    pub performance: f64,
    pub architectural_stability: f64,
    pub reversibility: f64,
}

impl ConfidenceVector {
    /// صفر — عند فشل الـ parse
    pub fn zero() -> Self {
        Self {
            security: 0.0,
            test_coverage: 0.0,
            maintainability: 0.0,
            performance: 0.0,
            architectural_stability: 0.0,
            reversibility: 0.0,
        }
    }

    /// أدنى confidence — المحافظ دائماً
    pub fn overall(&self) -> f64 {
        [
            self.security,
            self.test_coverage,
            self.maintainability,
            self.performance,
            self.architectural_stability,
            self.reversibility,
        ]
        .iter()
        .cloned()
        .fold(f64::MAX, f64::min)
    }
}

/// تحذيرات من التحليل
#[derive(Debug, Clone)]
pub enum AnalysisWarning {
    ParseFailed(String),
    LowConfidence { dimension: String, value: f64 },
    UnsafeWithoutComment { count: usize },
    HighComplexity { function: String, cc: u32 },
}
