use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::AgentOutput;
use crate::runtime::graph::AgentGraph;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{BufferedTracer, TraceCtx, Tracer};

/// Run a fixed list of different agents in parallel with the same input.
/// Results are collected into a JSON map keyed by agent name.
pub async fn run(
    graph: &AgentGraph,
    registry: &Arc<ModelRegistry>,
    workers: &[String],
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    eprintln!("  → parallel: {} agents", workers.len());

    // All workers start from parent simultaneously → prev = parent span_id → marks them parallel
    let parent_span = ctx.span_id.clone();
    let futs: Vec<_> = workers
        .iter()
        .map(|name| {
            let prev = Some(parent_span.clone());
            async move {
                let mut sub_tracer = BufferedTracer::new();
                let result = super::run_node(
                    graph,
                    name,
                    registry,
                    client,
                    input,
                    &mut sub_tracer,
                    ctx,
                    crumb,
                    prev,
                )
                .await;
                (name.as_str(), result, sub_tracer)
            }
        })
        .collect();

    let results = futures::future::join_all(futs).await;

    let mut map: HashMap<String, String> = HashMap::new();
    for (name, result, sub_tracer) in results {
        sub_tracer.flush_into(tracer);
        let value = match result {
            Ok(r) => r.value,
            Err(e) => format!("[error: {e}]"),
        };
        map.insert(name.to_string(), value);
    }

    let json_string = serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string());

    Ok(AgentOutput {
        key: "parallel".to_string(),
        value: json_string,
        span_id: String::new(),
    })
}
