#![forbid(unsafe_code)]

//! GovernanceStorage — SQLite persistence للحوكمة

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::audit::AuditEntry;
use crate::proposal::{ConstitutionalProposal, GovernanceError};

/// تخزين الحوكمة — proposals + audit
pub struct GovernanceStorage {
    conn: Mutex<Connection>,
}

const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS proposals (
        id          TEXT PRIMARY KEY,
        created_at  TEXT NOT NULL,
        data        TEXT NOT NULL,
        status      TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS audit_log (
        id          TEXT PRIMARY KEY,
        timestamp   TEXT NOT NULL,
        event_type  TEXT NOT NULL,
        actor       TEXT NOT NULL,
        data        TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_proposals_status ON proposals(status);
    CREATE INDEX IF NOT EXISTS idx_audit_timestamp  ON audit_log(timestamp);
"#;

impl GovernanceStorage {
    /// فتح تخزين في ملف
    pub fn open(path: &Path) -> Result<Self, GovernanceError> {
        let conn = Connection::open(path)
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        conn.execute_batch(SCHEMA)
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// تخزين في الذاكرة (للاختبارات)
    pub fn in_memory() -> Result<Self, GovernanceError> {
        let conn = Connection::open(":memory:")
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        conn.execute_batch(SCHEMA)
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// حفظ اقتراح
    pub fn save_proposal(
        &self,
        p: &ConstitutionalProposal,
    ) -> Result<(), GovernanceError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        let data = serde_json::to_string(p)
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        let status = format!("{:?}", p.status)
            .split_whitespace()
            .next()
            .unwrap_or("Unknown")
            .to_string();
        conn.execute(
            "INSERT OR REPLACE INTO proposals (id, created_at, data, status)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                p.id.to_string(),
                p.created_at.to_rfc3339(),
                data,
                status,
            ],
        )
        .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        Ok(())
    }

    /// تحميل كل الاقتراحات
    pub fn load_proposals(
        &self,
    ) -> Result<Vec<ConstitutionalProposal>, GovernanceError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT data FROM proposals ORDER BY created_at ASC")
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;

        let proposals: Vec<ConstitutionalProposal> = stmt
            .query_map([], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })
            .map_err(|e| GovernanceError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|data| serde_json::from_str(&data).ok())
            .collect();

        Ok(proposals)
    }

    /// حفظ مدونة تدقيق
    pub fn save_audit(&self, entry: &AuditEntry) -> Result<(), GovernanceError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        let data = serde_json::to_string(entry)
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        let event_type = format!("{:?}", entry.event)
            .split_whitespace()
            .next()
            .unwrap_or("Unknown")
            .to_string();
        conn.execute(
            "INSERT INTO audit_log (id, timestamp, event_type, actor, data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                entry.id.to_string(),
                entry.timestamp.to_rfc3339(),
                event_type,
                entry.actor,
                data,
            ],
        )
        .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        Ok(())
    }

    /// تحميل سجل التدقيق
    pub fn load_audit(&self) -> Result<Vec<AuditEntry>, GovernanceError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT data FROM audit_log ORDER BY timestamp ASC")
            .map_err(|e| GovernanceError::Storage(e.to_string()))?;

        let entries: Vec<AuditEntry> = stmt
            .query_map([], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })
            .map_err(|e| GovernanceError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .filter_map(|data| serde_json::from_str(&data).ok())
            .collect();

        Ok(entries)
    }
}
