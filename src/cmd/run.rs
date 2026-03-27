use anyhow::Result;
use std::sync::Arc;

use crate::runtime::debug_hook::{CliDebugger, DebugHook};
use crate::runtime::duckdb_tracer::DuckDbTracer;
use crate::runtime::tracer::{CompositeTracer, OtelTracer};

const DB_DIR: &str = ".tama";
const DB_PATH: &str = ".tama/runs.duckdb";

pub async fn run(
    task: &str,
    agent: Option<&str>,
    debug: bool,
    breakpoints: Vec<String>,
) -> Result<()> {
    if let Some(a) = agent {
        std::env::set_var("TAMA_ENTRYPOINT_AGENT", a);
    }

    std::fs::create_dir_all(DB_DIR)?;

    let duckdb = DuckDbTracer::new(DB_PATH)?;
    let tracer = CompositeTracer::new(vec![Box::new(OtelTracer::new()), Box::new(duckdb)]);

    let debug_hook: Option<Arc<dyn DebugHook + Send + Sync>> = if debug {
        if !breakpoints.is_empty() {
            eprintln!("debug: breakpoints on agents: {}", breakpoints.join(", "));
        } else {
            eprintln!("debug: step-through enabled (pause at every LLM call)");
        }
        Some(Arc::new(CliDebugger::new(breakpoints)))
    } else {
        None
    };

    crate::runtime::run(task, Box::new(tracer), debug_hook).await
}
