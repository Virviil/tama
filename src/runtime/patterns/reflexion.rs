use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use super::AgentOutput;
use super::step::{Step, Step::React};
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{new_node_id, TraceCtx, Tracer};

/// Reflexion: act → reflect → loop.
///
/// Files:
///   act.md     → actor system prompt; supports `pattern: react` (default) or `pattern: oneshot`
///   reflect.md → reflector system prompt; supports `pattern: react` frontmatter
///
/// The reflector should output either:
///   "DONE: <explanation>" — to stop the loop
///   Any other text       — treated as feedback for the next iteration
pub async fn run(
    agent_dir: &Path,
    max_iter: u32,
    registry: &Arc<ModelRegistry>,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    let act_step = Step::from_file(&agent_dir.join("act.md"))?;
    let reflect_step = Step::from_file(&agent_dir.join("reflect.md"))?;
    let mut last_result = String::new();
    let mut feedback = String::new();
    // Each step chains from the previous one so the UI shows them sequentially.
    let mut prev_span_id = ctx.span_id.clone();

    for iter in 0..max_iter {
        let actor_input = if feedback.is_empty() {
            input.to_string()
        } else {
            format!("Original task: {input}\n\nPrevious attempt:\n{last_result}\n\nFeedback:\n{feedback}\n\nPlease improve your answer based on this feedback.")
        };

        eprintln!("  → reflexion act iter {}", iter + 1);
        let act_name = format!("act_{}", iter + 1);
        let act_ctx = ctx.child();
        let act_crumb = format!("{crumb}→{act_name}");
        let t = Instant::now();
        tracer.on_agent_start(
            &act_ctx,
            &act_name,
            act_step.pattern_name(),
            &actor_input,
            Some(&prev_span_id),
            &new_node_id(),
        );
        let act_out = act_step
            .run(&actor_input, &act_name, registry, client, tracer, &act_ctx, &act_crumb)
            .await?;
        tracer.on_agent_end(&act_ctx, &act_out.key, &act_out.value, t.elapsed().as_millis());
        last_result = act_out.value;
        prev_span_id = act_ctx.span_id.clone();

        if iter + 1 >= max_iter {
            break;
        }

        eprintln!("  → reflexion reflect iter {}", iter + 1);
        let reflect_name = format!("reflect_{}", iter + 1);
        let reflect_input = format!("Task: {input}\n\nResult:\n{last_result}");
        let reflect_ctx = ctx.child();
        let reflect_crumb = format!("{crumb}→{reflect_name}");
        let t = Instant::now();
        tracer.on_agent_start(
            &reflect_ctx,
            &reflect_name,
            reflect_step.pattern_name(),
            &reflect_input,
            Some(&prev_span_id),
            &new_node_id(),
        );
        let reflection = reflect_step
            .run(&reflect_input, &reflect_name, registry, client, tracer, &reflect_ctx, &reflect_crumb)
            .await?;
        tracer.on_agent_end(&reflect_ctx, &reflection.key, &reflection.value, t.elapsed().as_millis());
        prev_span_id = reflect_ctx.span_id.clone();

        let satisfied = if matches!(reflect_step, React(_)) {
            reflection.key == "done"
        } else {
            reflection.value.trim().to_uppercase().starts_with("DONE")
        };
        if satisfied {
            eprintln!("  → reflexion: satisfied after {} iterations", iter + 1);
            break;
        }

        feedback = reflection.value;
    }

    Ok(AgentOutput {
        key: "done".to_string(),
        value: last_result,
        span_id: String::new(),
    })
}
