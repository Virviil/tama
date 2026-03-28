pub mod debug_hook;
pub mod duckdb_tracer;
pub mod graph;
pub mod llm;
pub mod model_registry;
pub mod patterns;
pub mod rollbacker;
pub mod tools;
pub mod tracer;

use anyhow::{Context, Result};
use std::sync::Arc;

use debug_hook::DebugHook;
use graph::AgentGraph;
use llm::LlmClient;
use model_registry::ModelRegistry;
use tracer::{TraceCtx, Tracer};

pub async fn run(
    task: &str,
    mut tracer: Box<dyn Tracer>,
    debug_hook: Option<Arc<dyn DebugHook + Send + Sync>>,
) -> Result<()> {
    tools::inmemory::clear();
    // Install the rollbacker. DuckDbRollbacker when a debug hook is active (retries
    // possible); NoopRollbacker in production (zero cost — no DuckDB, no recording).
    if debug_hook.is_some() {
        rollbacker::install(rollbacker::DuckDbRollbacker::new(".tama/rollback.duckdb")?);
    } else {
        rollbacker::install(rollbacker::NoopRollbacker);
    }
    rollbacker::clear();
    let config = crate::config::TomlConfig::load()?;
    let registry = Arc::new(ModelRegistry::build(&config)?);

    let agent_name = std::env::var("TAMA_ENTRYPOINT_AGENT")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| Some(config.project.entrypoint).filter(|s| !s.is_empty()))
        .context("TAMA_ENTRYPOINT_AGENT is not set and tama.toml has no entrypoint")?;

    let graph = AgentGraph::build(&agent_name)?;
    eprintln!("tamad: graph has {} agent(s)", graph.nodes.len());

    // Build the fallback client from the root agent's model config, falling back to
    // the registry's pattern default. Used as debug-hook source and for agents that
    // don't define their own model (resolved via registry in run_node).
    let root = graph.root_node();
    let root_model_config = root.agent.call.as_ref().and_then(|c| c.model.as_ref());
    let root_resolved = registry.resolve(root_model_config, "agent")?;
    let client = LlmClient::from_resolved(&root_resolved, debug_hook)?.with_agent_name(&agent_name);

    eprintln!(
        "tamad: entrypoint={agent_name} model={}",
        root_resolved.model_name
    );

    let trace_id = uuid::Uuid::new_v4().to_string();
    let root_ctx = TraceCtx::new_root(trace_id);

    tracer.on_run_start(&root_ctx, &agent_name, task);

    let t = std::time::Instant::now();
    let result = patterns::run_node(
        &graph,
        &agent_name,
        &registry,
        &client,
        task,
        tracer.as_mut(),
        &root_ctx,
        "",
        None,
    )
    .await;
    let duration_ms = t.elapsed().as_millis();

    match &result {
        Ok(output) => tracer.on_run_end(&root_ctx, "ok", &output.value, duration_ms),
        Err(e) => tracer.on_run_end(&root_ctx, "error", &e.to_string(), duration_ms),
    }

    let output = result?;
    println!("{}", output.value);
    Ok(())
}
