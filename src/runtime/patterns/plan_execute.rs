use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::AgentOutput;
use super::oneshot;
use super::step::{Step, Step::React};
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{TraceCtx, Tracer};

/// Plan-Execute: plan (JSON steps) → execute each step → verify → loop.
///
/// Files:
///   AGENT.md body  → planning system prompt (produce JSON array of steps)
///   execute.md     → step execution system prompt
///   verify.md      → verification system prompt
///               outputs "DONE: <reason>" or "RETRY: <feedback>"
///
/// execute.md and verify.md support optional `pattern: react` frontmatter.
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
    let execute_step = Step::from_file(&agent_dir.join("execute.md"))?;
    let verify_step = Step::from_file(&agent_dir.join("verify.md"))?;

    const MAX_PLAN_CYCLES: u32 = 3;
    let mut feedback = String::new();

    for cycle in 0..MAX_PLAN_CYCLES {
        eprintln!("  → plan_execute: planning (cycle {})", cycle + 1);
        let plan_input = if feedback.is_empty() {
            format!("Task: {input}\n\nRespond with a JSON array of step strings only.")
        } else {
            format!("Task: {input}\n\nFeedback from previous attempt:\n{feedback}\n\nRevise your plan. Respond with a JSON array of step strings only.")
        };

        let plan_text = oneshot::call(
            body,
            &plan_input,
            &format!("plan_{}", cycle + 1),
            client,
            tracer,
            ctx,
            crumb,
        )
        .await?;
        let steps = parse_json_array(&plan_text);
        eprintln!("  → plan_execute: {} steps", steps.len());

        let mut step_results: Vec<String> = Vec::new();
        let mut context = format!("Task: {input}");

        for (i, step) in steps.iter().enumerate() {
            eprintln!("  → plan_execute: step {}/{}", i + 1, steps.len());
            let step_input = format!("{context}\n\nCurrent step: {step}");
            let result = execute_step
                .run(
                    &step_input,
                    &format!("execute_{}_{}", cycle + 1, i + 1),
                    registry,
                    client,
                    tracer,
                    ctx,
                    crumb,
                )
                .await?
                .value;
            step_results.push(format!("Step: {step}\nResult: {result}"));
            context = format!("{context}\n\n{}", step_results.last().unwrap());
        }

        let execution_summary = step_results.join("\n\n");

        eprintln!("  → plan_execute: verify (cycle {})", cycle + 1);
        let verify_input = format!("Task: {input}\n\nExecution summary:\n{execution_summary}");
        let verdict = verify_step
            .run(
                &verify_input,
                &format!("verify_{}", cycle + 1),
                registry,
                client,
                tracer,
                ctx,
                crumb,
            )
            .await?;

        let done = if matches!(verify_step, React(_)) {
            verdict.key == "done"
        } else {
            verdict.value.trim().to_uppercase().starts_with("DONE")
        };
        if done {
            eprintln!("  → plan_execute: done after {} cycles", cycle + 1);
            return Ok(AgentOutput {
                key: "done".to_string(),
                value: context,
                span_id: String::new(),
            });
        }

        feedback = verdict.value;
    }

    anyhow::bail!("plan_execute reached {MAX_PLAN_CYCLES} cycles without DONE verdict")
}

fn parse_json_array(text: &str) -> Vec<String> {
    let cleaned = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("```"))
        .collect::<Vec<_>>()
        .join("\n");

    if let Ok(arr) = serde_json::from_str::<Vec<String>>(cleaned.trim()) {
        return arr;
    }
    if let Some(start) = cleaned.find('[') {
        if let Some(end) = cleaned.rfind(']') {
            let slice = &cleaned[start..=end];
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(slice) {
                return arr;
            }
        }
    }
    vec![text.trim().to_string()]
}
