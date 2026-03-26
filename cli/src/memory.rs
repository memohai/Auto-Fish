use anyhow::Context;
use rusqlite::{Connection, params};
use std::path::PathBuf;

#[derive(Debug)]
pub struct TraceStore {
    connection: Connection,
}

#[derive(Debug, Clone)]
pub struct TraceRecord {
    pub created_at: String,
    pub session: String,
    pub trace_id: String,
    pub command: String,
    pub status: String,
    pub output_json: String,
    pub duration_ms: u128,
}

impl TraceStore {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let connection = Connection::open(&path)
            .with_context(|| format!("failed to connect to sqlite database at {:?}", path))?;
        init_trace_table(&connection)?;
        init_ref_version_table(&connection)?;
        Ok(Self { connection })
    }

    pub fn record(&self, record: &TraceRecord) -> anyhow::Result<()> {
        self.connection
            .execute(
                "INSERT INTO traces (
                    created_at, session, trace_id, command, status, output_json, duration_ms
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    record.created_at,
                    record.session,
                    record.trace_id,
                    record.command,
                    record.status,
                    record.output_json,
                    record.duration_ms as i64
                ],
            )
            .with_context(|| "failed to insert trace record")?;
        Ok(())
    }

    pub fn upsert_ref_version(&self, scope: &str, version: u64) -> anyhow::Result<()> {
        self.connection
            .execute(
                "INSERT INTO ref_versions (scope, version, updated_at) VALUES (?1, ?2, datetime('now'))
                 ON CONFLICT(scope) DO UPDATE SET version = excluded.version, updated_at = excluded.updated_at",
                params![scope, version as i64],
            )
            .with_context(|| "failed to upsert ref version")?;
        Ok(())
    }

    pub fn get_ref_version(&self, scope: &str) -> anyhow::Result<Option<u64>> {
        let mut stmt = self
            .connection
            .prepare("SELECT version FROM ref_versions WHERE scope = ?1 LIMIT 1")
            .with_context(|| "failed to prepare ref_version lookup")?;
        let mut rows = stmt
            .query(params![scope])
            .with_context(|| "failed to query ref_version")?;
        if let Some(row) = rows.next().with_context(|| "failed to read ref_version row")? {
            let value: i64 = row.get(0).with_context(|| "failed to decode ref_version")?;
            Ok(Some(value.max(0) as u64))
        } else {
            Ok(None)
        }
    }
}

fn init_trace_table(connection: &Connection) -> anyhow::Result<()> {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS traces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL,
                session TEXT NOT NULL,
                trace_id TEXT NOT NULL,
                command TEXT NOT NULL,
                status TEXT NOT NULL,
                output_json TEXT NOT NULL,
                duration_ms INTEGER NOT NULL
            )",
            [],
        )
        .with_context(|| "failed to create traces table")?;
    Ok(())
}

fn init_ref_version_table(connection: &Connection) -> anyhow::Result<()> {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS ref_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scope TEXT NOT NULL UNIQUE,
                version INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .with_context(|| "failed to create ref_versions table")?;
    Ok(())
}
