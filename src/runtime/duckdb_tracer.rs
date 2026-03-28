use anyhow::Result;
use rusqlite::{params, Connection};

use crate::runtime::tracer::{TraceCtx, Tracer};

/// Persists all trace events to a local SQLite file.
/// Used only by `tama run`.
pub struct SqliteTracer {
    conn: Connection,
    seq: u64,
    // agent span_ids that have already had system_prompt stored — skip on subsequent LLM calls
    stored_prompts: std::collections::HashSet<String>,
}

impl SqliteTracer {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Self::create_schema(&conn)?;
        Ok(SqliteTracer {
            conn,
            seq: 0,
            stored_prompts: std::collections::HashSet::new(),
        })
    }

    fn create_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS runs (
                trace_id    TEXT PRIMARY KEY,
                timestamp   INTEGER,
                entrypoint  TEXT,
                task        TEXT,
                status      TEXT,
                output      TEXT,
                duration_ms INTEGER
            );

            CREATE TABLE IF NOT EXISTS spans (
                span_id        TEXT PRIMARY KEY,
                parent_span_id TEXT,
                prev_span_id   TEXT,
                node_id        TEXT,
                trace_id       TEXT,
                name           TEXT,
                kind           TEXT,
                start_ms       INTEGER,
                end_ms         INTEGER,
                seq            INTEGER
            );

            CREATE TABLE IF NOT EXISTS llm_calls (
                span_id       TEXT PRIMARY KEY,
                trace_id      TEXT,
                model         TEXT,
                temperature   REAL,
                system_prompt TEXT,
                response      TEXT,
                input_tokens  INTEGER,
                output_tokens INTEGER,
                duration_ms   INTEGER,
                role          TEXT
            );

            CREATE TABLE IF NOT EXISTS tool_calls (
                span_id     TEXT PRIMARY KEY,
                trace_id    TEXT,
                tool_name   TEXT,
                args_json   TEXT,
                result      TEXT,
                duration_ms INTEGER
            );
        ",
        )?;
        Ok(())
    }

    fn next_seq(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }

    fn now_ms() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
}

impl Tracer for SqliteTracer {
    fn on_run_start(&mut self, ctx: &TraceCtx, entrypoint: &str, task: &str) {
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO runs (trace_id, timestamp, entrypoint, task, status, output, duration_ms) VALUES (?1,?2,?3,?4,'running','',0)",
            params![ctx.trace_id, Self::now_ms(), entrypoint, task],
        );
    }

    fn on_run_end(&mut self, ctx: &TraceCtx, status: &str, output: &str, duration_ms: u128) {
        let _ = self.conn.execute(
            "UPDATE runs SET status=?1, output=?2, duration_ms=?3 WHERE trace_id=?4",
            params![status, output, duration_ms as i64, ctx.trace_id],
        );
    }

    fn on_agent_start(
        &mut self,
        ctx: &TraceCtx,
        agent: &str,
        pattern: &str,
        _input: &str,
        prev_span_id: Option<&str>,
        node_id: &str,
    ) {
        let name = format!("agent:{agent}:{pattern}");
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, prev_span_id, node_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?1,?2,?3,?4,?5,?6,'agent',?7,0,?8)",
            params![ctx.span_id, ctx.parent_span_id, prev_span_id, node_id, ctx.trace_id, name, Self::now_ms(), seq],
        );
    }

    fn on_agent_end(&mut self, ctx: &TraceCtx, _key: &str, _output: &str, duration_ms: u128) {
        let _ = self.conn.execute(
            "UPDATE spans SET end_ms = start_ms + ?1 WHERE span_id = ?2",
            params![duration_ms as i64, ctx.span_id],
        );
    }

    fn on_llm_call(
        &mut self,
        ctx: &TraceCtx,
        step: &str,
        model: &str,
        role: &str,
        temperature: Option<f32>,
        system: &str,
        response: &str,
        input_tokens: u32,
        output_tokens: u32,
        duration_ms: u128,
    ) {
        let end_ms = Self::now_ms();
        let start_ms = end_ms - duration_ms as i64;
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?1,?2,?3,?4,'llm',?5,?6,?7)",
            params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, format!("llm:{step}"), start_ms, end_ms, seq],
        );
        let agent_span = ctx.parent_span_id.as_deref().unwrap_or("");
        let prompt_to_store: Option<&str> = if self.stored_prompts.insert(agent_span.to_string()) {
            Some(system)
        } else {
            None
        };
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO llm_calls (span_id, trace_id, model, role, temperature, system_prompt, response, input_tokens, output_tokens, duration_ms) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![ctx.span_id, ctx.trace_id, model, role, temperature.map(|t| t as f64), prompt_to_store, response, input_tokens as i32, output_tokens as i32, duration_ms as i64],
        );
    }

    fn on_tool_call(
        &mut self,
        ctx: &TraceCtx,
        tool: &str,
        args_json: &str,
        result: &str,
        duration_ms: u128,
    ) {
        let end_ms = Self::now_ms();
        let start_ms = end_ms - duration_ms as i64;
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?1,?2,?3,?4,'tool',?5,?6,?7)",
            params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, format!("tool:{tool}"), start_ms, end_ms, seq],
        );
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO tool_calls (span_id, trace_id, tool_name, args_json, result, duration_ms) VALUES (?1,?2,?3,?4,?5,?6)",
            params![ctx.span_id, ctx.trace_id, tool, args_json, result, duration_ms as i64],
        );
    }

    fn on_synthetic_start(&mut self, ctx: &TraceCtx, input: &str) {
        let now_ms = Self::now_ms();
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?1,?2,?3,'tool:start','synthetic',?4,?4,?5)",
            params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, now_ms, seq],
        );
        let args = serde_json::json!({});
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO tool_calls (span_id, trace_id, tool_name, args_json, result, duration_ms) VALUES (?1,?2,'start',?3,?4,0)",
            params![ctx.span_id, ctx.trace_id, args.to_string(), input],
        );
    }

    fn on_synthetic_finish(&mut self, ctx: &TraceCtx, args_json: &str, result: &str) {
        let now_ms = Self::now_ms();
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?1,?2,?3,'tool:finish','synthetic',?4,?4,?5)",
            params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, now_ms, seq],
        );
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO tool_calls (span_id, trace_id, tool_name, args_json, result, duration_ms) VALUES (?1,?2,'finish',?3,?4,0)",
            params![ctx.span_id, ctx.trace_id, args_json, result],
        );
    }
}
