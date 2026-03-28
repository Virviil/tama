use anyhow::Result;
use duckdb::Connection;

use crate::runtime::tracer::{TraceCtx, Tracer};

/// Persists all trace events to a local DuckDB file.
/// Used only by `tama run`.
pub struct DuckDbTracer {
    conn: Connection,
    seq: u64,
    // agent span_ids that have already had system_prompt stored — skip on subsequent LLM calls
    stored_prompts: std::collections::HashSet<String>,
}

impl DuckDbTracer {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Self::create_schema(&conn)?;
        Ok(DuckDbTracer {
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
                timestamp   BIGINT,
                entrypoint  TEXT,
                task        TEXT,
                status      TEXT,
                output      TEXT,
                duration_ms BIGINT
            );

            CREATE TABLE IF NOT EXISTS spans (
                span_id        TEXT PRIMARY KEY,
                parent_span_id TEXT,
                prev_span_id   TEXT,
                node_id        TEXT,
                trace_id       TEXT,
                name           TEXT,
                kind           TEXT,
                start_ms       BIGINT,
                end_ms         BIGINT,
                seq            BIGINT
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
                duration_ms   BIGINT
            );
            ALTER TABLE llm_calls ADD COLUMN IF NOT EXISTS temperature REAL;
            ALTER TABLE llm_calls ADD COLUMN IF NOT EXISTS role TEXT;

            CREATE TABLE IF NOT EXISTS tool_calls (
                span_id   TEXT PRIMARY KEY,
                trace_id  TEXT,
                tool_name TEXT,
                args_json TEXT,
                result    TEXT,
                duration_ms BIGINT
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

impl Tracer for DuckDbTracer {
    fn on_run_start(&mut self, ctx: &TraceCtx, entrypoint: &str, task: &str) {
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO runs (trace_id, timestamp, entrypoint, task, status, output, duration_ms) VALUES (?,?,?,?,'running','',0)",
            duckdb::params![ctx.trace_id, Self::now_ms(), entrypoint, task],
        );
    }

    fn on_run_end(&mut self, ctx: &TraceCtx, status: &str, output: &str, duration_ms: u128) {
        let _ = self.conn.execute(
            "UPDATE runs SET status=?, output=?, duration_ms=? WHERE trace_id=?",
            duckdb::params![status, output, duration_ms as i64, ctx.trace_id],
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
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, prev_span_id, node_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?,?,?,?,?,?,'agent',?,0,?)",
            duckdb::params![ctx.span_id, ctx.parent_span_id, prev_span_id, node_id, ctx.trace_id, name, Self::now_ms(), seq],
        );
    }

    fn on_agent_end(&mut self, ctx: &TraceCtx, _key: &str, _output: &str, duration_ms: u128) {
        let _ = self.conn.execute(
            "UPDATE spans SET end_ms = start_ms + ? WHERE span_id = ?",
            duckdb::params![duration_ms as i64, ctx.span_id],
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
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?,?,?,?,'llm',?,?,?)",
            duckdb::params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, format!("llm:{step}"), start_ms, end_ms, seq],
        );
        // Store system_prompt only on the first LLM call for each agent span
        let agent_span = ctx.parent_span_id.as_deref().unwrap_or("");
        let prompt_to_store: Option<&str> = if self.stored_prompts.insert(agent_span.to_string()) {
            Some(system)
        } else {
            None
        };
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO llm_calls (span_id, trace_id, model, role, temperature, system_prompt, response, input_tokens, output_tokens, duration_ms) VALUES (?,?,?,?,?,?,?,?,?,?)",
            duckdb::params![ctx.span_id, ctx.trace_id, model, role, temperature.map(|t| t as f64), prompt_to_store, response, input_tokens as i32, output_tokens as i32, duration_ms as i64],
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
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?,?,?,?,'tool',?,?,?)",
            duckdb::params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, format!("tool:{tool}"), start_ms, end_ms, seq],
        );
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO tool_calls (span_id, trace_id, tool_name, args_json, result, duration_ms) VALUES (?,?,?,?,?,?)",
            duckdb::params![ctx.span_id, ctx.trace_id, tool, args_json, result, duration_ms as i64],
        );
    }

    fn on_synthetic_start(&mut self, ctx: &TraceCtx, input: &str) {
        let now_ms = Self::now_ms();
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?,?,?,?,'synthetic',?,?,?)",
            duckdb::params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, "tool:start", now_ms, now_ms, seq],
        );
        let args = serde_json::json!({});
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO tool_calls (span_id, trace_id, tool_name, args_json, result, duration_ms) VALUES (?,?,?,?,?,0)",
            duckdb::params![ctx.span_id, ctx.trace_id, "start", args.to_string(), input],
        );
    }

    fn on_synthetic_finish(&mut self, ctx: &TraceCtx, args_json: &str, result: &str) {
        let now_ms = Self::now_ms();
        let seq = self.next_seq() as i64;
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO spans (span_id, parent_span_id, trace_id, name, kind, start_ms, end_ms, seq) VALUES (?,?,?,?,'synthetic',?,?,?)",
            duckdb::params![ctx.span_id, ctx.parent_span_id, ctx.trace_id, "tool:finish", now_ms, now_ms, seq],
        );
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO tool_calls (span_id, trace_id, tool_name, args_json, result, duration_ms) VALUES (?,?,?,?,?,0)",
            duckdb::params![ctx.span_id, ctx.trace_id, "finish", args_json, result],
        );
    }
}
