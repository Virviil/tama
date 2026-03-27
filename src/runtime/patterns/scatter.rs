use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use super::AgentOutput;
use super::step::Step;
use crate::runtime::graph::AgentGraph;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{BufferedTracer, TraceCtx, Tracer};

/// Scatter = map → parallel → reduce.
///
/// Files:
///   AGENT.md body  → map system prompt (react atom that decides what to fan out)
///   reduce.md      → reduce system prompt (react atom that synthesizes worker results)
///
/// Flow:
///   Phase 1 (map):      react atom runs with the task as input.
///                       calls finish(key="parallel", value='["item1","item2",...]')
///   Phase 2 (parallel): runtime fans each item out to the worker agent concurrently.
///   Phase 3 (reduce):   react atom receives all worker results and synthesizes a final answer.
///
///   If the map phase calls finish with any key other than "parallel", the result
///   is returned directly without fanning out.
pub async fn run(
    graph: &AgentGraph,
    registry: &Arc<ModelRegistry>,
    agent_dir: &Path,
    body: &str,
    worker: &str,
    uses: &[String],
    max_iter: u32,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    // Phase 1: Map — react atom decides what to fan out
    eprintln!("  → scatter: map");
    let map_system = build_map_system(body, uses);
    let map_ctx = ctx.child();
    let map_crumb = format!("{crumb}→map");
    let t = Instant::now();
    tracer.on_agent_start(
        &map_ctx,
        "map",
        "react",
        input,
        Some(ctx.span_id.as_str()),
        &crate::runtime::tracer::new_node_id(),
    );
    let map_result = super::react::react_loop(
        &map_system,
        uses,
        max_iter,
        client,
        input,
        tracer,
        &map_ctx,
        &[],
        &map_crumb,
    )
    .await?;
    tracer.on_agent_end(
        &map_ctx,
        &map_result.key,
        &map_result.value,
        t.elapsed().as_millis(),
    );

    if map_result.key != "parallel" {
        // Map finished without requesting fan-out — return directly
        return Ok(map_result);
    }

    let items: Vec<String> = serde_json::from_str(&map_result.value).with_context(|| {
        format!(
            "scatter map phase must finish(key='parallel', value='[\"item1\",...]') — got: {}",
            map_result.value
        )
    })?;

    eprintln!("  → scatter: parallel — {} items → '{worker}'", items.len());

    // Phase 2: Parallel — run worker on each item concurrently.
    // Workers use map_ctx.span_id as prev: multiple siblings with the same prev → detected as parallel.
    let map_span = map_ctx.span_id.clone();
    let futs: Vec<_> = items
        .iter()
        .map(|item| {
            let prev = Some(map_span.clone());
            async move {
                let mut sub_tracer = BufferedTracer::new();
                let result = super::run_node(
                    graph,
                    worker,
                    registry,
                    client,
                    item,
                    &mut sub_tracer,
                    ctx,
                    crumb,
                    prev,
                )
                .await;
                (item.as_str(), result, sub_tracer)
            }
        })
        .collect();

    let results = futures::future::join_all(futs).await;

    let mut combined = String::new();
    let mut last_worker_span: Option<String> = None;
    for (item, result, sub_tracer) in results {
        sub_tracer.flush_into(tracer);
        match result {
            Ok(r) => {
                last_worker_span = Some(r.span_id.clone());
                combined.push_str(&format!("[Item: {item}]\n{}\n\n", r.value));
            }
            Err(e) => combined.push_str(&format!("[Item: {item}]\n[error: {e}]\n\n")),
        }
    }

    // Phase 3: Reduce — synthesizes all worker results.
    // reduce.md may declare `pattern: react` (default) or `pattern: oneshot`.
    // prev = last worker's span_id so reduce appears sequential after the parallel block.
    eprintln!("  → scatter: reduce");
    let reduce_step = Step::from_file(&agent_dir.join("reduce.md"))?;
    let reduce_input = format!("Original task: {input}\n\nWorker results:\n{combined}");
    let reduce_ctx = ctx.child();
    let reduce_crumb = format!("{crumb}→reduce");
    let t = Instant::now();
    let reduce_prev = last_worker_span.or_else(|| Some(map_span.clone()));
    tracer.on_agent_start(
        &reduce_ctx,
        "reduce",
        reduce_step.pattern_name(),
        &reduce_input,
        reduce_prev.as_deref(),
        &crate::runtime::tracer::new_node_id(),
    );
    let reduce_out = reduce_step.run(&reduce_input, "reduce", registry, client, tracer, &reduce_ctx, &reduce_crumb).await?;
    let reduce_result = AgentOutput { key: reduce_out.key, value: reduce_out.value, span_id: String::new() };
    tracer.on_agent_end(
        &reduce_ctx,
        &reduce_result.key,
        &reduce_result.value,
        t.elapsed().as_millis(),
    );

    Ok(reduce_result)
}

/// Prepend scatter-specific instructions to the map system prompt.
fn build_map_system(body: &str, _: &[String]) -> String {
    let scatter_instruction =
        "When you have determined the full list of items to process in parallel, \
         call finish(key=\"parallel\", value='[\"item1\",\"item2\",...]') with a JSON array of strings.\n\
         If no fan-out is needed, call finish(key=\"done\", value=\"...\") as usual.\n\n";
    format!("{scatter_instruction}{body}")
}

