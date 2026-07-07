#![forbid(unsafe_code)]

//! OutcomeStorage — تخزين outcomes Bayesian في SQLite
//! Week 37 — إضافة فقط

use crate::storage::{SqliteStorage, StorageError};
use ec_epistemic::BayesianEvidence;

/// واجهة تخزين الـ outcomes
pub trait OutcomeStorage: Send + Sync {
    /// تسجيل outcome واحد
    fn record_outcome(
        &self,
        artifact_id: &str,
        was_correct: bool,
        score: f64,
    ) -> Result<(), StorageError>;

    /// تحميل BayesianEvidence لـ artifact معيّن
    fn load_evidence(&self, artifact_id: &str) -> Result<BayesianEvidence, StorageError>;

    /// تحميل BayesianEvidence من كل الـ outcomes
    fn load_all_evidence(&self) -> Result<BayesianEvidence, StorageError>;

    /// عدد الـ outcomes المخزّنة
    fn outcome_count(&self) -> Result<u32, StorageError>;
}

const OUTCOME_SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS bayesian_outcomes (
        id          TEXT PRIMARY KEY,
        artifact_id TEXT NOT NULL,
        was_correct INTEGER NOT NULL,
        score       REAL NOT NULL,
        recorded_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_outcomes_artifact_id
        ON bayesian_outcomes(artifact_id);
"#;

impl SqliteStorage {
    /// تهيئة جداول الـ outcomes
    pub fn init_outcome_schema(&self) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            conn.execute_batch(OUTCOME_SCHEMA)?;
            Ok(())
        })
    }

    /// إنشاء مع schema كامل (decisions + outcomes)
    pub fn new_with_outcomes(path: impl AsRef<std::path::Path>) -> Result<Self, StorageError> {
        let storage = Self::new(path)?;
        storage.init_outcome_schema()?;
        Ok(storage)
    }

    /// إنشاء in-memory مع schema كامل
    pub fn in_memory_with_outcomes() -> Result<Self, StorageError> {
        let storage = Self::in_memory()?;
        storage.init_outcome_schema()?;
        Ok(storage)
    }
}

fn to_storage_error(e: ec_epistemic::EpistemicError) -> StorageError {
    StorageError::Path(e.to_string())
}

impl OutcomeStorage for SqliteStorage {
    fn record_outcome(
        &self,
        artifact_id: &str,
        was_correct: bool,
        score: f64,
    ) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT INTO bayesian_outcomes
                   (id, artifact_id, was_correct, score, recorded_at)
                   VALUES (?1, ?2, ?3, ?4, ?5)"#,
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    artifact_id,
                    was_correct as i32,
                    score,
                    chrono::Utc::now().to_rfc3339(),
                ],
            )?;
            Ok(())
        })
    }

    fn load_evidence(&self, artifact_id: &str) -> Result<BayesianEvidence, StorageError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"SELECT was_correct, score FROM bayesian_outcomes
                   WHERE artifact_id = ?1 ORDER BY rowid ASC"#,
            )?;

            let mut evidence = BayesianEvidence::initial_prior().map_err(to_storage_error)?;

            let rows: Vec<(bool, f64)> = stmt
                .query_map(rusqlite::params![artifact_id], |row| {
                    let was_correct: i32 = row.get(0)?;
                    let score: f64 = row.get(1)?;
                    Ok((was_correct != 0, score))
                })?
                .filter_map(|r| r.ok())
                .collect();

            for (was_correct, score) in rows {
                evidence = evidence
                    .update_with_outcome(was_correct, score)
                    .map_err(to_storage_error)?;
            }

            Ok(evidence)
        })
    }

    fn load_all_evidence(&self) -> Result<BayesianEvidence, StorageError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                r#"SELECT was_correct, score FROM bayesian_outcomes
                   ORDER BY rowid ASC"#,
            )?;

            let mut evidence = BayesianEvidence::initial_prior().map_err(to_storage_error)?;

            let rows: Vec<(bool, f64)> = stmt
                .query_map([], |row| {
                    let was_correct: i32 = row.get(0)?;
                    let score: f64 = row.get(1)?;
                    Ok((was_correct != 0, score))
                })?
                .filter_map(|r| r.ok())
                .collect();

            for (was_correct, score) in rows {
                evidence = evidence
                    .update_with_outcome(was_correct, score)
                    .map_err(to_storage_error)?;
            }

            Ok(evidence)
        })
    }

    fn outcome_count(&self) -> Result<u32, StorageError> {
        self.with_conn(|conn| {
            let count: u32 =
                conn.query_row("SELECT COUNT(*) FROM bayesian_outcomes", [], |row| {
                    row.get(0)
                })?;
            Ok(count)
        })
    }
}
