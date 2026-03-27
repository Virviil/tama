use anyhow::Result;
use genai::chat::{ChatMessage, ChatRequest, Tool, ToolResponse};
use std::collections::HashSet;
use std::time::Instant;

use super::AgentOutput;
use crate::runtime::llm::{extract_usage, LlmClient};
use crate::runtime::tools;
use crate::runtime::tracer::{TraceCtx, Tracer};

pub async fn run(
    body: &str,
    uses: &[String],
    max_iter: u32,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    react_loop(body, uses, max_iter, client, input, tracer, ctx, &[], crumb).await
}

/// React loop with optional extra always-on tools (used by scatter for parallel_run).
pub async fn react_loop(
    body: &str,
    uses: &[String],
    max_iter: u32,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    extra_tools: &[Tool],
    crumb: &str,
) -> Result<AgentOutput> {
    let system = tools::build_system(body, uses);
    let mut unlocked: HashSet<String> = HashSet::new();
    let mut prev_tool_results = String::new();

    // Anthropic requires at least one user message. The agent calls start() to
    // receive the actual task — this message just triggers the first tool call.
    let mut req = ChatRequest::from_system(&system)
        .append_message(ChatMessage::user("Begin."))
        .with_tools(tools::build_active_tools(uses, &unlocked, extra_tools));

    for iter in 0..max_iter {
        let step = format!("react_{}", iter + 1);
        let t = Instant::now();
        let response = client
            .chat_with_step(
                req.clone(),
                &step,
                &prev_tool_results,
                &ctx.trace_id,
                &ctx.span_id,
                crumb,
            )
            .await?;
        let duration_ms = t.elapsed().as_millis();

        let (input_tokens, output_tokens) = extract_usage(&response);
        let response_text = response
            .first_text()
            .map(str::to_string)
            .unwrap_or_default();

        // Implicit finish: if the model returns plain text with no tool calls,
        // treat it as finish(key="done", value=<text>).
        //
        // This works correctly for patterns that only care about the value
        // (critic, reflexion actor, oneshot-like steps).
        //
        // WARNING: patterns that route on the key (FSM states, scatter map)
        // require an explicit finish(key="<routing-word>", value="...") call.
        // A plain-text response here will always produce key="done" and break routing.
        if response.tool_calls().is_empty() {
            let llm_ctx = ctx.child();
            tracer.on_llm_call(
                &llm_ctx,
                &step,
                client.model_name(),
                client.temperature(),
                &system,
                &response_text,
                input_tokens,
                output_tokens,
                duration_ms,
            );
            // Model returned plain text — synthesize a finish(key="done") tool call
            // so the trace is complete. kind='synthetic' distinguishes it from
            // an explicit finish call made by the model.
            let finish_ctx = ctx.child();
            let args = serde_json::json!({"key": "done", "value": &response_text});
            tracer.on_synthetic_finish(&finish_ctx, &args.to_string(), &response_text);
            return Ok(AgentOutput {
                key: "done".to_string(),
                value: response_text,
                span_id: String::new(),
            });
        }

        let tool_calls = response.into_tool_calls();
        let llm_ctx = ctx.child();
        tracer.on_llm_call(
            &llm_ctx,
            &step,
            client.model_name(),
            client.temperature(),
            &system,
            &response_text,
            input_tokens,
            output_tokens,
            duration_ms,
        );

        req = req.append_message(ChatMessage::from(tool_calls.clone()));

        let mut iter_results: Vec<String> = Vec::new();
        for tc in &tool_calls {
            let tool_t = Instant::now();

            // finish is always last — trace it then return.
            if tc.fn_name == "finish" {
                let key = tc.fn_arguments["key"]
                    .as_str()
                    .unwrap_or("done")
                    .to_string();
                let value = tc.fn_arguments["value"].as_str().unwrap_or("").to_string();
                let tool_ctx = ctx.child();
                tracer.on_tool_call(
                    &tool_ctx,
                    "finish",
                    &tc.fn_arguments.to_string(),
                    &value,
                    tool_t.elapsed().as_millis(),
                );
                return Ok(AgentOutput {
                    key,
                    value,
                    span_id: String::new(),
                });
            }

            let result = if tc.fn_name == "start" {
                input.to_string()
            } else if tc.fn_name == "read_skill" {
                let name = tc.fn_arguments["name"].as_str().unwrap_or("");
                match tools::load_skill(name) {
                    Ok((body, skill_tools)) => {
                        let newly: Vec<_> = skill_tools
                            .iter()
                            .filter(|t| unlocked.insert((*t).clone()))
                            .cloned()
                            .collect();
                        if !newly.is_empty() {
                            eprintln!("    unlocked tools: {}", newly.join(", "));
                        }
                        body
                    }
                    Err(e) => format!("[error: {e}]"),
                }
            } else {
                tools::execute_tool(&tc.fn_name, &tc.fn_arguments, &ctx.span_id)
                    .await
                    .unwrap_or_else(|e| format!("[error: {e}]"))
            };

            let result = tools::truncate(result, 12_000);

            let tool_ctx = ctx.child();
            tracer.on_tool_call(
                &tool_ctx,
                &tc.fn_name,
                &tc.fn_arguments.to_string(),
                &result,
                tool_t.elapsed().as_millis(),
            );

            iter_results.push(format!("{}({}) →\n{}", tc.fn_name, tc.fn_arguments, result));
            req = req.append_message(ChatMessage::from(ToolResponse::new(&tc.call_id, result)));
        }
        prev_tool_results = iter_results.join("\n\n");

        req = ChatRequest {
            tools: Some(tools::build_active_tools(uses, &unlocked, extra_tools)),
            ..req
        };
    }

    anyhow::bail!("react agent reached max_iter ({max_iter}) without calling finish")
}
