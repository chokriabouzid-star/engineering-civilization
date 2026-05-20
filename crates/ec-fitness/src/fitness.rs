#![deny(warnings)]
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};

/// The six constitutional dimensions of fitness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitnessVector {
    pub security: f64,
    pub reversibility: f64,
    pub test_coverage: f64,
    pub maintainability: f64,
    pub performance: f64,
    pub architectural_stability: f64,
}

impl Default for FitnessVector {
    fn default() -> Self {
        Self {
            security: 0.0,
            reversibility: 0.0,
            test_coverage: 0.0,
            maintainability: 0.0,
            performance: 0.0,
            architectural_stability: 0.0,
        }
    }
}

/// Thresholds that trigger constitutional catastrophe.
#[derive(Debug, Clone)]
pub struct CatastropheThresholds {
    pub min_security: f64,
    pub min_reversibility: f64,
    pub min_test_coverage: f64,
    pub min_maintainability: f64,
    pub min_performance: f64,
    pub min_architectural_stability: f64,
}

impl Default for CatastropheThresholds {
    fn default() -> Self {
        Self {
            min_security: 0.70,
            min_reversibility: 0.30,
            min_test_coverage: 0.60,
            min_maintainability: 0.40,
            min_performance: 0.20,
            min_architectural_stability: 0.50,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CatastrophicDimension {
    Security,
    Reversibility,
    TestCoverage,
    Maintainability,
    Performance,
    ArchitecturalStability,
}

impl FitnessVector {
    /// Validate that all dimensions are finite and in [0, 1]
    pub fn validate(&self) -> Result<(), &'static str> {
        let dims = [
            ("security", self.security),
            ("reversibility", self.reversibility),
            ("test_coverage", self.test_coverage),
            ("maintainability", self.maintainability),
            ("performance", self.performance),
            ("architectural_stability", self.architectural_stability),
        ];

        for (_name, value) in &dims {
            if !value.is_finite() {
                return Err("FitnessVector contains non-finite value");
            }
            if *value < 0.0 || *value > 1.0 {
                return Err("FitnessVector dimension out of [0, 1] range");
            }
        }
        Ok(())
    }

    /// Cosine similarity بين متجهي لياقة.
    ///
    /// 1.0 = متطابقان، 0.0 = متعامدان، -1.0 = متعاكسان.
    pub fn cosine_similarity(&self, other: &Self) -> f64 {
        let dot = self.security * other.security
            + self.reversibility * other.reversibility
            + self.test_coverage * other.test_coverage
            + self.maintainability * other.maintainability
            + self.performance * other.performance
            + self.architectural_stability * other.architectural_stability;

        let mag_a = self.magnitude();
        let mag_b = other.magnitude();

        if mag_a < 1e-10 || mag_b < 1e-10 {
            return 0.0;
        }
        (dot / (mag_a * mag_b)).clamp(-1.0, 1.0)
    }

    /// الزاوية بالدرجات بين متجهي لياقة.
    ///
    /// 0° = متطابقان، 90° = متعامدان، 180° = متعاكسان.
    pub fn cosine_angle_degrees(&self, other: &Self) -> f64 {
        let cosine = self.cosine_similarity(other);
        cosine.acos().to_degrees()
    }

    /// طول المتجه (magnitude).
    fn magnitude(&self) -> f64 {
        (self.security.powi(2)
            + self.reversibility.powi(2)
            + self.test_coverage.powi(2)
            + self.maintainability.powi(2)
            + self.performance.powi(2)
            + self.architectural_stability.powi(2))
        .sqrt()
    }
}
