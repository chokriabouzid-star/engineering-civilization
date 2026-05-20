#![forbid(unsafe_code)]

//! الأنواع الأساسية للذاكرة السببية.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// معرف فريد لكل عقدة في الذاكرة.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// إنشاء معرف جديد.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// معرف من UUID محدد (للاختبارات).
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// UUID الكامل كنص (للتخزين).
    pub fn to_uuid_string(&self) -> String {
        self.0.to_string()
    }

    /// تحليل UUID من نص.
    pub fn parse_uuid_str(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// تجزئة artifact (SHA256 أو مشابه).
pub type ArtifactHash = u64;

/// لقطة artifact — الكود + التجزئة.
///
/// **Design:** نستخدم Arc<str> لتجنب تكرار الكود.
/// آلاف القرارات قد تشير لنفس الكود.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSnapshot {
    /// تجزئة المحتوى.
    pub hash: ArtifactHash,
    /// الكود الفعلي (shared ownership).
    pub code: Arc<str>,
}

impl Serialize for ArtifactSnapshot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ArtifactSnapshot", 2)?;
        s.serialize_field("hash", &self.hash)?;
        s.serialize_field("code", &*self.code)?;
        s.end()
    }
}

impl<'de> Deserialize<'de> for ArtifactSnapshot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            hash: ArtifactHash,
            code: String,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(Self {
            hash: h.hash,
            code: Arc::from(h.code.as_str()),
        })
    }
}

impl ArtifactSnapshot {
    /// إنشاء snapshot جديد.
    pub fn new(code: impl Into<String>) -> Self {
        let code_str: String = code.into();
        let hash = Self::compute_hash(&code_str);
        Self {
            hash,
            code: Arc::from(code_str.as_str()),
        }
    }

    /// حساب hash بسيط (للتطوير — في الإنتاج: SHA256).
    fn compute_hash(code: &str) -> ArtifactHash {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
        hasher.finish()
    }

    /// استرجاع الكود.
    pub fn code(&self) -> &str {
        &self.code
    }
}

/// سبب رفض بديل.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RejectionReason {
    /// فشل catastrophic.
    CatastrophicFailure {
        /// البُعد الفاشل.
        dimension: String,
    },
    /// مُسيطَر عليه من باريتو.
    ParetoDominated {
        /// معرف الذي سيطر عليه.
        dominated_by: NodeId,
    },
    /// انتهاك دستوري.
    ConstitutionalViolation {
        /// الانتهاكات.
        violations: Vec<String>,
    },
    /// فشل sandbox.
    SandboxFailure {
        /// درجة correctness.
        correctness: f64,
    },
    /// وصلنا للحد الأقصى من المحاولات.
    MaxIterationsReached {
        /// عدد المحاولات.
        attempts: usize,
    },
}

/// تقييم استعادي — الشيء الوحيد القابل للتغيير.
///
/// **Design Invariant:**
/// - الماضي لا يتغير
/// - فقط فهمنا له يتغير
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetrospectiveAssessment {
    /// متى تم التقييم.
    pub assessed_at: DateTime<Utc>,
    /// هل اتضح لاحقاً أن هذا كان الخيار الأفضل؟
    pub was_better_choice: bool,
    /// مستوى الثقة في التقييم.
    pub confidence: f64,
    /// سبب التقييم.
    pub reasoning: String,
}

impl RetrospectiveAssessment {
    /// إنشاء تقييم جديد.
    pub fn new(
        was_better: bool,
        confidence: f64,
        reasoning: impl Into<String>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            (0.0..=1.0).contains(&confidence),
            "confidence must be in [0, 1], got {}",
            confidence
        );

        Ok(Self {
            assessed_at: Utc::now(),
            was_better_choice: was_better,
            confidence,
            reasoning: reasoning.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_is_unique() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn artifact_snapshot_same_code_same_hash() {
        let s1 = ArtifactSnapshot::new("fn main() {}");
        let s2 = ArtifactSnapshot::new("fn main() {}");
        assert_eq!(s1.hash, s2.hash);
    }

    #[test]
    fn artifact_snapshot_different_code_different_hash() {
        let s1 = ArtifactSnapshot::new("fn main() {}");
        let s2 = ArtifactSnapshot::new("fn foo() {}");
        assert_ne!(s1.hash, s2.hash);
    }

    #[test]
    fn artifact_snapshot_shares_arc() {
        let s1 = ArtifactSnapshot::new("fn main() {}");
        let s2 = s1.clone();
        assert!(Arc::ptr_eq(&s1.code, &s2.code));
    }

    #[test]
    fn retrospective_assessment_validates_confidence() {
        assert!(RetrospectiveAssessment::new(true, 0.5, "ok").is_ok());
        assert!(RetrospectiveAssessment::new(true, 1.5, "bad").is_err());
        assert!(RetrospectiveAssessment::new(true, -0.1, "bad").is_err());
    }
}
