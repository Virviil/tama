use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use super::step::Step;
use super::AgentOutput;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{new_node_id, TraceCtx, Tracer};

pub async fn run(
    agent_dir: &Path,
    registry: &Arc<ModelRegistry>,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    let draft_step = Step::from_file(&agent_dir.join("draft.md"))?;
    let critique_step = Step::from_file(&agent_dir.join("critique.md"))?;
    let refine_step = Step::from_file(&agent_dir.join("refine.md"))?;

    // ── draft ─────────────────────────────────────────────────────────────────
    let draft_ctx = ctx.child();
    let t = Instant::now();
    tracer.on_agent_start(
        &draft_ctx,
        "draft",
        draft_step.pattern_name(),
        input,
        Some(ctx.span_id.as_str()),
        &new_node_id(),
    );
    let draft = draft_step
        .run(
            input,
            "draft",
            registry,
            client,
            tracer,
            &draft_ctx,
            &format!("{crumb}→draft"),
        )
        .await?;
    tracer.on_agent_end(
        &draft_ctx,
        &draft.key,
        &draft.value,
        t.elapsed().as_millis(),
    );

    // ── critique ──────────────────────────────────────────────────────────────
    let critique_ctx = ctx.child();
    let t = Instant::now();
    tracer.on_agent_start(
        &critique_ctx,
        "critique",
        critique_step.pattern_name(),
        &draft.value,
        Some(draft_ctx.span_id.as_str()),
        &new_node_id(),
    );
    let critique = critique_step
        .run(
            &draft.value,
            "critique",
            registry,
            client,
            tracer,
            &critique_ctx,
            &format!("{crumb}→critique"),
        )
        .await?;
    tracer.on_agent_end(
        &critique_ctx,
        &critique.key,
        &critique.value,
        t.elapsed().as_millis(),
    );

    // ── refine ────────────────────────────────────────────────────────────────
    let refine_user = format!("Original task:\n{input}\n\nCritique:\n{}", critique.value);
    let refine_ctx = ctx.child();
    let t = Instant::now();
    tracer.on_agent_start(
        &refine_ctx,
        "refine",
        refine_step.pattern_name(),
        &refine_user,
        Some(critique_ctx.span_id.as_str()),
        &new_node_id(),
    );
    let refined = refine_step
        .run(
            &refine_user,
            "refine",
            registry,
            client,
            tracer,
            &refine_ctx,
            &format!("{crumb}→refine"),
        )
        .await?;
    tracer.on_agent_end(
        &refine_ctx,
        &refined.key,
        &refined.value,
        t.elapsed().as_millis(),
    );

    Ok(AgentOutput {
        key: "done".to_string(),
        value: refined.value,
        span_id: String::new(),
    })
}
