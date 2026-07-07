#![forbid(unsafe_code)]

//! Memory Storage — SQLite persistence for causal memory.
//!
//! Week 27 — Phase 4
//!
//! **Design:**
//! - `MemoryStorage` trait: واجهة عامة
//! - `SqliteStorage`: تنفيذ SQLite مع `Mutex<Connection>`
//! - `in_memory()`: للاختبارات (اتصال واحد يبقى حياً)
//! - `new(path)`: للإنتاج

use crate::graph::CausalMemoryGraph;
use crate::node::DecisionNode;
use crate::types::{NodeId, RetrospectiveAssessment};
use std::path::Path;
use std::sync::Mutex;

// ─── الأخطاء ────────────────────────────────────────────

/// أخطاء التخزين.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// خطأ SQLite.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// خطأ Serialization.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    /// خطأ في المسار.
    #[error("Path error: {0}")]
    Path(String),
    /// خطأ قفل.
    #[error("Lock error")]
    Lock,
}

// ─── الـ Trait ──────────────────────────────────────────

/// واجهة التخزين — قد تكون SQLite أو أي شيء آخر.
pub trait MemoryStorage: Send + Sync {
    /// حفظ الـ graph كاملاً.
    fn save(&self, graph: &CausalMemoryGraph) -> Result<(), StorageError>;

    /// تحميل الـ graph كاملاً.
    fn load(&self) -> Result<CausalMemoryGraph, StorageError>;

    /// إضافة عقدة واحدة (incremental).
    fn append_node(&self, node: &DecisionNode) -> Result<(), StorageError>;

    /// إضافة تقييم استعادي.
    fn append_retrospective(
        &self,
        node_id: NodeId,
        assessment: &RetrospectiveAssessment,
    ) -> Result<(), StorageError>;
}

// ─── Schema ─────────────────────────────────────────────

const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS decisions (
        id          TEXT PRIMARY KEY,
        created_at  TEXT NOT NULL,
        artifact_id TEXT NOT NULL,
        code        TEXT NOT NULL,
        fitness     TEXT NOT NULL,
        constitutional_valid INTEGER NOT NULL,
        sandbox_outcome TEXT,
        causal_parents  TEXT NOT NULL,
        alternatives    TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS retrospective_assessments (
        id           TEXT PRIMARY KEY,
        decision_id  TEXT NOT NULL,
        assessed_at  TEXT NOT NULL,
        was_better   INTEGER NOT NULL,
        confidence   REAL NOT NULL,
        reasoning    TEXT NOT NULL,
        FOREIGN KEY (decision_id) REFERENCES decisions(id)
    );

    CREATE INDEX IF NOT EXISTS idx_decisions_artifact_id
        ON decisions(artifact_id);

    CREATE INDEX IF NOT EXISTS idx_retrospective_decision_id
        ON retrospective_assessments(decision_id);
"#;

// ─── SQLite Implementation ──────────────────────────────

/// تخزين SQLite — يحتفظ بـ connection واحد عبر Mutex.
pub struct SqliteStorage {
    conn: Mutex<rusqlite::Connection>,
}

impl SqliteStorage {
    /// إنشاء تخزين في ملف.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path_str = path
            .as_ref()
            .to_str()
            .ok_or_else(|| StorageError::Path("invalid path".into()))?;
        let conn = rusqlite::Connection::open(path_str)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// إنشاء تخزين في الذاكرة (للاختبارات).
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn = rusqlite::Connection::open(":memory:")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub(crate) fn with_conn<F, T>(&self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, StorageError>,
    {
        let conn = self.conn.lock().map_err(|_| StorageError::Lock)?;
        f(&conn)
    }
}

impl MemoryStorage for SqliteStorage {
    fn append_node(&self, node: &DecisionNode) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT OR IGNORE INTO decisions
                   (id, created_at, artifact_id, code, fitness,
                    constitutional_valid, sandbox_outcome,
                    causal_parents, alternatives)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                rusqlite::params![
                    node.id.to_uuid_string(),
                    node.created_at.to_rfc3339(),
                    node.artifact_id,
                    node.artifact.code(),
                    serde_json::to_string(&node.fitness)?,
                    node.constitutional_valid as i32,
                    serde_json::to_string(&node.sandbox_outcome)?,
                    serde_json::to_string(&node.causal_parents)?,
                    serde_json::to_string(&node.alternatives)?,
                ],
            )?;
            Ok(())
        })
    }

    fn append_retrospective(
        &self,
        node_id: NodeId,
        assessment: &RetrospectiveAssessment,
    ) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            conn.execute(
                r#"INSERT INTO retrospective_assessments
                   (id, decision_id, assessed_at, was_better,
                    confidence, reasoning)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    node_id.to_uuid_string(),
                    assessment.assessed_at.to_rfc3339(),
                    assessment.was_better_choice as i32,
                    assessment.confidence,
                    assessment.reasoning,
                ],
            )?;
            Ok(())
        })
    }

    fn save(&self, graph: &CausalMemoryGraph) -> Result<(), StorageError> {
        for node in graph.all() {
            self.append_node(node)?;
            for assessment in &node.retrospective {
                self.append_retrospective(node.id, assessment)?;
            }
        }
        Ok(())
    }

    fn load(&self) -> Result<CausalMemoryGraph, StorageError> {
        self.with_conn(|conn| {
            // ─── Phase 1: Load decisions ──────────
            let mut stmt = conn.prepare(
                r#"SELECT id, created_at, artifact_id, code, fitness,
                          constitutional_valid, sandbox_outcome,
                          causal_parents, alternatives
                   FROM decisions ORDER BY rowid ASC"#,
            )?;

            let nodes: Vec<DecisionNode> = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let created_at_str: String = row.get(1)?;
                    let artifact_id: String = row.get(2)?;
                    let code: String = row.get(3)?;
                    let fitness_json: String = row.get(4)?;
                    let valid: i32 = row.get(5)?;
                    let sandbox_json: Option<String> = row.get(6)?;
                    let parents_json: String = row.get(7)?;
                    let alts_json: String = row.get(8)?;
                    Ok((
                        id_str,
                        created_at_str,
                        artifact_id,
                        code,
                        fitness_json,
                        valid,
                        sandbox_json,
                        parents_json,
                        alts_json,
                    ))
                })?
                .filter_map(|r| r.ok())
                .filter_map(
                    |(
                        id_str,
                        created_str,
                        artifact_id,
                        code,
                        fitness_json,
                        valid,
                        sandbox_json,
                        parents_json,
                        alts_json,
                    )| {
                        let id = NodeId::parse_uuid_str(&id_str)?;
                        let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                            .ok()?
                            .with_timezone(&chrono::Utc);
                        let fitness: ec_fitness::FitnessVector =
                            serde_json::from_str(&fitness_json).ok()?;
                        let sandbox_outcome = sandbox_json
                            .as_deref()
                            .and_then(|j| serde_json::from_str(j).ok());
                        let causal_parents: Vec<NodeId> =
                            serde_json::from_str(&parents_json).ok()?;
                        let alternatives = serde_json::from_str(&alts_json).ok()?;

                        Some(DecisionNode {
                            id,
                            created_at,
                            artifact_id,
                            artifact: crate::types::ArtifactSnapshot::new(code),
                            fitness,
                            constitutional_valid: valid != 0,
                            sandbox_outcome,
                            alternatives,
                            causal_parents,
                            retrospective: vec![],
                        })
                    },
                )
                .collect();

            let mut graph = CausalMemoryGraph::from_nodes(nodes);

            // ─── Phase 2: Load retrospectives ─────
            let mut r_stmt = conn.prepare(
                r#"SELECT decision_id, assessed_at, was_better,
                          confidence, reasoning
                   FROM retrospective_assessments ORDER BY rowid ASC"#,
            )?;

            let assessments: Vec<(NodeId, RetrospectiveAssessment)> = r_stmt
                .query_map([], |row| {
                    let did_str: String = row.get(0)?;
                    let at_str: String = row.get(1)?;
                    let was_better: i32 = row.get(2)?;
                    let confidence: f64 = row.get(3)?;
                    let reasoning: String = row.get(4)?;
                    Ok((did_str, at_str, was_better, confidence, reasoning))
                })?
                .filter_map(|r| r.ok())
                .filter_map(|(did_str, at_str, was_better, confidence, reasoning)| {
                    let node_id = NodeId::parse_uuid_str(&did_str)?;
                    let _assessed_at = chrono::DateTime::parse_from_rfc3339(&at_str)
                        .ok()?
                        .with_timezone(&chrono::Utc);
                    let assessment =
                        RetrospectiveAssessment::new(was_better != 0, confidence, reasoning)
                            .ok()?;
                    Some((node_id, assessment))
                })
                .collect();

            for (node_id, assessment) in assessments {
                let _ = graph.update_retrospective(node_id, assessment);
            }

            Ok(graph)
        })
    }
}
