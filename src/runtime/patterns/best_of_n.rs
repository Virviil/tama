use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::step::Step;
use super::AgentOutput;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{BufferedTracer, TraceCtx, Tracer};

/// Best-of-N: generate N variants in parallel → judge picks best.
///
/// Files:
///   AGENT.md body  → system prompt for all N variants (always react agents)
///   judge.md       → judge system prompt; supports `pattern: react` frontmatter
pub async fn run(
    body: &str,
    uses: &[String],
    n: u32,
    agent_dir: &Path,
    registry: &Arc<ModelRegistry>,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    eprintln!("  → best_of_n: generating {n} variants");

    let variant_ctxs: Vec<TraceCtx> = (0..n).map(|_| ctx.child()).collect();

    let futs: Vec<_> = (0..n as usize)
        .zip(variant_ctxs.into_iter())
        .map(|(i, variant_ctx)| async move {
            let mut sub_tracer = BufferedTracer::new();
            let result = super::react::run(
                body,
                uses,
                15,
                client,
                input,
                &mut sub_tracer,
                &variant_ctx,
                crumb,
            )
            .await;
            (i, result, sub_tracer)
        })
        .collect();

    let variants = futures::future::join_all(futs).await;

    let mut variant_texts: Vec<String> = vec!["".to_string(); n as usize];
    for (i, result, sub_tracer) in variants {
        sub_tracer.flush_into(tracer);
        variant_texts[i] = result
            .map(|o| o.value)
            .unwrap_or_else(|e| format!("[error: {e}]"));
    }

    eprintln!("  → best_of_n: judging");
    let judge_step = Step::from_file(&agent_dir.join("judge.md"))?;
    let variants_text = variant_texts
        .iter()
        .enumerate()
        .map(|(i, v)| format!("## Variant {}\n\n{v}", i + 1))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    let judge_input = format!(
        "Task: {input}\n\nVariants:\n\n{variants_text}\n\nSelect the best variant and return it (or synthesize if appropriate)."
    );

    let best = judge_step
        .run(&judge_input, "judge", registry, client, tracer, ctx, crumb)
        .await?
        .value;

    Ok(AgentOutput {
        key: "done".to_string(),
        value: best,
        span_id: String::new(),
    })
}
