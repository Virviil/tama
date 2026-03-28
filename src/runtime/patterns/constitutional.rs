use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::oneshot;
use super::step::Step;
use super::AgentOutput;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{TraceCtx, Tracer};

/// Constitutional AI: generate → critique against principles → revise.
///
/// Files:
///   AGENT.md body  → generation system prompt
///   critique.md    → constitutional critique system prompt (includes principles)
///   revise.md      → revision system prompt
pub async fn run(
    agent_dir: &Path,
    body: &str,
    registry: &Arc<ModelRegistry>,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    eprintln!("  → constitutional: generate");
    let draft = oneshot::call(body, input, "generate", client, tracer, ctx, crumb).await?;

    let critique_step = Step::from_file(&agent_dir.join("critique.md"))?;
    eprintln!("  → constitutional: critique");
    let critique = critique_step
        .run(&draft, "critique", registry, client, tracer, ctx, crumb)
        .await?
        .value;

    let revise_step = Step::from_file(&agent_dir.join("revise.md"))?;
    eprintln!("  → constitutional: revise");
    let revise_user = format!(
        "Original request: {input}\n\nInitial response:\n{draft}\n\nConstitutional critique:\n{critique}"
    );
    let revised = revise_step
        .run(&revise_user, "revise", registry, client, tracer, ctx, crumb)
        .await?
        .value;

    Ok(AgentOutput {
        key: "done".to_string(),
        value: revised,
        span_id: String::new(),
    })
}
