use anyhow::Result;
use std::sync::Arc;

use super::AgentOutput;
use crate::runtime::graph::AgentGraph;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{BufferedTracer, TraceCtx, Tracer};

/// Debate: position agents argue in rounds, judge synthesizes.
///
/// Round 1: each agent sees only the original input.
/// Round N>1: each agent sees all previous arguments.
/// Judge: sees all rounds and produces the final synthesis.
pub async fn run(
    graph: &AgentGraph,
    registry: &Arc<ModelRegistry>,
    agents: &[String],
    rounds: u32,
    judge: &str,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    let mut round_transcripts: Vec<String> = vec![];

    for round in 0..rounds {
        eprintln!("  → debate round {}/{rounds}", round + 1);

        let round_context = if round_transcripts.is_empty() {
            format!("Topic: {input}")
        } else {
            format!(
                "Topic: {input}\n\nPrevious arguments:\n{}",
                round_transcripts.join("\n\n---\n\n")
            )
        };

        let parent_span = ctx.span_id.clone();
        let futs: Vec<_> = agents
            .iter()
            .enumerate()
            .map(|(i, agent_name)| {
                let context = round_context.clone();
                let prev = Some(parent_span.clone());
                async move {
                    let mut sub_tracer = BufferedTracer::new();
                    let result = super::run_node(
                        graph,
                        agent_name,
                        registry,
                        client,
                        &context,
                        &mut sub_tracer,
                        ctx,
                        crumb,
                        prev,
                    )
                    .await;
                    (i, agent_name.as_str(), result, sub_tracer)
                }
            })
            .collect();

        let mut positions = futures::future::join_all(futs).await;
        positions.sort_by_key(|(i, _, _, _)| *i);

        let mut round_text = format!("## Round {}\n\n", round + 1);
        for (_, agent_name, result, sub_tracer) in positions {
            sub_tracer.flush_into(tracer);
            let arg = result
                .map(|o| o.value)
                .unwrap_or_else(|e| format!("[error: {e}]"));
            round_text.push_str(&format!("**{agent_name}**: {arg}\n\n"));
        }

        round_transcripts.push(round_text);
    }

    eprintln!("  → debate: judge '{judge}'");
    let debate_summary = format!("Topic: {input}\n\n{}", round_transcripts.join("\n\n"));

    let mut judge_tracer = BufferedTracer::new();
    let result = super::run_node(
        graph,
        judge,
        registry,
        client,
        &debate_summary,
        &mut judge_tracer,
        ctx,
        crumb,
        Some(ctx.span_id.clone()),
    )
    .await;
    judge_tracer.flush_into(tracer);
    result
}
