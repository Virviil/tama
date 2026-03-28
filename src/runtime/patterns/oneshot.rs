use anyhow::Result;
use std::time::Instant;

use super::AgentOutput;
use crate::runtime::llm::LlmClient;
use crate::runtime::tracer::{TraceCtx, Tracer};

/// Single LLM call with synthetic start/finish events so the trace shows input and output.
/// Used directly by patterns that run the AGENT.md body as a oneshot step (constitutional
/// generate phase, plan_execute plan phase, chain_of_verification generate phase, etc.).
pub async fn call(
    system: &str,
    input: &str,
    step_name: &str,
    client: &LlmClient,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<String> {
    // Synthetic start — shows what input was passed to this step
    let start_ctx = ctx.child();
    tracer.on_synthetic_start(&start_ctx, input);

    let t = Instant::now();
    let (response, input_tokens, output_tokens) = client
        .call_system_user_tracked(system, input, step_name, &ctx.trace_id, &ctx.span_id, crumb)
        .await?;
    let duration_ms = t.elapsed().as_millis();

    let llm_ctx = ctx.child();
    tracer.on_llm_call(
        &llm_ctx,
        step_name,
        client.model_name(),
        client.role(),
        client.temperature(),
        system,
        &response,
        input_tokens,
        output_tokens,
        duration_ms,
    );

    // Synthetic finish — shows output as a finish(key="done") call
    let finish_ctx = ctx.child();
    let args = serde_json::json!({"key": "done", "value": &response});
    tracer.on_synthetic_finish(&finish_ctx, &args.to_string(), &response);

    Ok(response)
}

/// Single LLM call: system prompt from AGENT.md body, input as user message.
/// Returns key="result", value=<llm response>.
pub async fn run(
    body: &str,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    let value = call(body, input, "oneshot", client, tracer, ctx, crumb).await?;
    Ok(AgentOutput {
        key: "result".to_string(),
        value,
        span_id: String::new(),
    })
}
