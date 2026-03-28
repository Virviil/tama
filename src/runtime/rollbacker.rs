use std::sync::{LazyLock, Mutex};

#[cfg(feature = "cli")]
use anyhow::Result;
#[cfg(feature = "cli")]
use rusqlite::{params, Connection};

// ── Trait ─────────────────────────────────────────────────────────────────────

/// Records side-effectful actions during an agent span so they can be undone on retry.
pub trait Rollbacker: Send {
    fn record_tool_call(
        &mut self,
        span_id: &str,
        tool_name: &str,
        key: &str,
        old_value: Option<&str>,
    );
    fn rollback(&mut self, span_id: &str);
    fn clear(&mut self);
}

// ── NoopRollbacker ────────────────────────────────────────────────────────────

/// Zero-cost. Used in production (`tamad` without a debug hook) where retries never occur.
pub struct NoopRollbacker;

impl Rollbacker for NoopRollbacker {
    fn record_tool_call(&mut self, _: &str, _: &str, _: &str, _: Option<&str>) {}
    fn rollback(&mut self, _: &str) {}
    fn clear(&mut self) {}
}

// ── SqliteRollbacker ──────────────────────────────────────────────────────────

/// Persists the ordered action log to SQLite — the single source of truth for rollback.
///
/// Schema: `action_log(seq, span_id, tool_name, key, old_value)`
#[cfg(feature = "cli")]
pub struct SqliteRollbacker {
    conn: Connection,
    seq: i64,
}

#[cfg(feature = "cli")]
impl SqliteRollbacker {
    pub fn new(db_path: &str) -> Result<Self> {
        std::fs::create_dir_all(
            std::path::Path::new(db_path)
                .parent()
                .unwrap_or(std::path::Path::new(".")),
        )?;
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS action_log (
                 seq       INTEGER,
                 span_id   TEXT,
                 tool_name TEXT,
                 key       TEXT,
                 old_value TEXT
             );",
        )?;
        Ok(SqliteRollbacker { conn, seq: 0 })
    }
}

#[cfg(feature = "cli")]
impl Rollbacker for SqliteRollbacker {
    fn record_tool_call(
        &mut self,
        span_id: &str,
        tool_name: &str,
        key: &str,
        old_value: Option<&str>,
    ) {
        let _ = self.conn.execute(
            "INSERT INTO action_log (seq, span_id, tool_name, key, old_value) VALUES (?1,?2,?3,?4,?5)",
            params![self.seq, span_id, tool_name, key, old_value],
        );
        self.seq += 1;
    }

    fn rollback(&mut self, span_id: &str) {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT tool_name, key, old_value FROM action_log WHERE span_id=?1 ORDER BY seq DESC",
        ) else {
            return;
        };

        let rows: Vec<(String, String, Option<String>)> = stmt
            .query_map(params![span_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        for (tool_name, key, old_value) in rows {
            match tool_name.as_str() {
                "tama_mem_set" => match old_value {
                    Some(ref v) => {
                        eprintln!("  rollback: mem[{key}] ← {v:?}");
                        crate::runtime::tools::inmemory::set(&key, v);
                    }
                    None => {
                        eprintln!("  rollback: mem[{key}] ← (deleted)");
                        crate::runtime::tools::inmemory::delete(&key);
                    }
                },
                other => {
                    eprintln!("  rollback: {other} — no handler, skipping");
                }
            }
        }

        let _ = self
            .conn
            .execute("DELETE FROM action_log WHERE span_id=?1", params![span_id]);
        eprintln!("  rollback: completed for span {span_id}");
    }

    fn clear(&mut self) {
        self.seq = 0;
    }
}

// ── Global registry ───────────────────────────────────────────────────────────

static ROLLBACKER: LazyLock<Mutex<Box<dyn Rollbacker + Send>>> =
    LazyLock::new(|| Mutex::new(Box::new(NoopRollbacker)));

pub fn install(r: impl Rollbacker + 'static) {
    *ROLLBACKER.lock().unwrap() = Box::new(r);
}

pub fn record_tool_call(span_id: &str, tool_name: &str, key: &str, old_value: Option<&str>) {
    ROLLBACKER
        .lock()
        .unwrap()
        .record_tool_call(span_id, tool_name, key, old_value);
}

pub fn rollback(span_id: &str) {
    ROLLBACKER.lock().unwrap().rollback(span_id);
}

pub fn clear() {
    ROLLBACKER.lock().unwrap().clear();
}
