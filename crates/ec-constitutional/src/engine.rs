#![deny(warnings)]
#![forbid(unsafe_code)]

use crate::constitution::Constitution;
use crate::evaluation::ConstitutionalEvaluation;
use ec_epistemic::state::EpistemicState;
use ec_fitness::fitness::FitnessVector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, info_span, warn};
use uuid::Uuid;

// ─── EvaluationContext ─────────────────────────────────────────────

/// سياق تقييم دستوري واحد.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    /// نية بشرية (اختياري — لتتبع القرار لاحقاً)
    pub intent: Option<String>,
    /// قيود إضافية فرضها الإنسان
    pub constraints: Option<String>,
    /// أدنى ثقة معرفية مقبولة
    pub min_epistemic_confidence: Option<f64>,
    /// طابع زمني للطلب
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Trace ID فريد لربط الطلبات
    pub trace_id: Uuid,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            intent: None,
            constraints: None,
            min_epistemic_confidence: None,
            timestamp: chrono::Utc::now(),
            trace_id: Uuid::new_v4(),
        }
    }
}

impl EvaluationContext {
    pub fn new(intent: impl Into<String>) -> Self {
        Self {
            intent: Some(intent.into()),
            trace_id: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

// ─── ConstitutionalCache ────────────────────────────────────────────

/// مفتاح التخزين المؤقت: تجزئة المحتوى + نسخة الدستور
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    artifact_hash: u64,
    constitution_version: String,
}

/// قيمة التخزين المؤقت: تقييم مع طابع زمني
#[derive(Debug, Clone)]
struct CacheEntry {
    evaluation: ConstitutionalEvaluation,
    inserted_at: Instant,
}

/// تخزين مؤقت للتقييمات الدستورية.
pub struct ConstitutionalCache {
    map: dashmap::DashMap<CacheKey, CacheEntry>,
    ttl: Duration,
}

impl ConstitutionalCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: dashmap::DashMap::new(),
            ttl,
        }
    }

    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(300))
    }

    pub fn get(
        &self,
        artifact_hash: u64,
        constitution_version: &str,
    ) -> Option<ConstitutionalEvaluation> {
        let key = CacheKey {
            artifact_hash,
            constitution_version: constitution_version.to_string(),
        };

        if let Some(entry) = self.map.get(&key) {
            if entry.inserted_at.elapsed() < self.ttl {
                debug!(
                    artifact_hash,
                    constitution_version,
                    "Cache hit"
                );
                return Some(entry.evaluation.clone());
            }
            drop(entry);
            self.map.remove(&key);
            debug!(artifact_hash, constitution_version, "Cache expired");
        }
        None
    }

    pub fn insert(
        &self,
        artifact_hash: u64,
        constitution_version: &str,
        evaluation: ConstitutionalEvaluation,
    ) {
        let key = CacheKey {
            artifact_hash,
            constitution_version: constitution_version.to_string(),
        };
        self.map.insert(
            key,
            CacheEntry {
                evaluation,
                inserted_at: Instant::now(),
            },
        );
    }

    pub fn purge_expired(&self) -> usize {
        let before = self.map.len();
        self.map
            .retain(|_, entry| entry.inserted_at.elapsed() < self.ttl);
        let removed = before - self.map.len();
        if removed > 0 {
            debug!(removed, "Purged expired cache entries");
        }
        removed
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for ConstitutionalCache {
    fn default() -> Self {
        Self::with_default_ttl()
    }
}

// ─── ConstitutionalEngine ───────────────────────────────────────────

/// محرك التقييم الدستوري.
pub struct ConstitutionalEngine {
    constitution: Arc<Constitution>,
    cache: ConstitutionalCache,
    constitution_version: String,
}

impl ConstitutionalEngine {
    pub fn new(constitution: Constitution, cache_ttl: Duration) -> Self {
        let version = uuid::Uuid::new_v4().to_string();
        info!(version = %version, "ConstitutionalEngine created");
        Self {
            constitution: Arc::new(constitution),
            cache: ConstitutionalCache::new(cache_ttl),
            constitution_version: version,
        }
    }

    pub fn with_default_cache(constitution: Constitution) -> Self {
        Self::new(constitution, Duration::from_secs(300))
    }

    pub fn constitution_version(&self) -> &str {
        &self.constitution_version
    }

    /// تقييم artifact — المسار الساخن، يجب أن يكون < 1ms p99
    pub fn evaluate(
        &self,
        artifact_id: &str,
        artifact_hash: u64,
        fitness: &FitnessVector,
        epistemic: &EpistemicState,
        context: &EvaluationContext,
    ) -> ConstitutionalEvaluation {
        let span = info_span!(
            "constitutional.evaluate",
            artifact_id = %artifact_id,
            artifact_hash = %artifact_hash,
            trace_id = %context.trace_id,
            intent = ?context.intent,
        );
        let _enter = span.enter();

        // Increment metric
        info!(counter.evaluations_total = 1);

        let start = Instant::now();
        let evaluation = self
            .constitution
            .evaluate(artifact_id, fitness, epistemic);
        let elapsed = start.elapsed();

        // Record duration metric
        info!(histogram.evaluation_duration_ms = elapsed.as_millis() as u64);

        debug!(
            artifact_id,
            elapsed_us = elapsed.as_micros(),
            is_valid = evaluation.is_valid,
            "Evaluation complete"
        );

        if elapsed > Duration::from_millis(1) {
            warn!(
                artifact_id,
                elapsed_ms = elapsed.as_millis(),
                "Slow evaluation: exceeded 1ms threshold"
            );
        }

        // Cache store
        self.cache
            .insert(artifact_hash, &self.constitution_version, evaluation.clone());

        evaluation
    }

    /// تقييم غير متزامن — جاهز للتكامل مع sandbox في Phase 2
    pub async fn evaluate_async(
        &self,
        artifact_id: String,
        artifact_hash: u64,
        fitness: FitnessVector,
        epistemic: EpistemicState,
        _context: EvaluationContext,
    ) -> ConstitutionalEvaluation {
        // Cache check (متزامن للسرعة)
        if let Some(cached) = self.cache.get(artifact_hash, &self.constitution_version) {
            info!(counter.cache_hits_total = 1);
            return cached;
        }

        let constitution = self.constitution.clone();
        let version = self.constitution_version.clone();
        let cache_ref = &self.cache;

        let start = Instant::now();
        let evaluation = tokio::task::spawn_blocking(move || {
            constitution.evaluate(&artifact_id, &fitness, &epistemic)
        })
        .await
        .expect("Evaluation task panicked");
        let elapsed = start.elapsed();

        info!(histogram.evaluation_duration_ms = elapsed.as_millis() as u64);
        info!(counter.evaluations_total = 1);

        cache_ref.insert(artifact_hash, &version, evaluation.clone());
        evaluation
    }

    pub fn compare(
        &self,
        left: &ConstitutionalEvaluation,
        right: &ConstitutionalEvaluation,
    ) -> ec_fitness::ParetoOrdering {
        self.constitution.compare(left, right)
    }

    pub fn build_frontier(
        &self,
        evaluations: &[ConstitutionalEvaluation],
    ) -> Vec<ConstitutionalEvaluation> {
        self.constitution.build_frontier(evaluations)
    }

    pub fn purge_cache(&self) -> usize {
        self.cache.purge_expired()
    }

    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
}
