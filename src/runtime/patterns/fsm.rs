use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::AgentOutput;
use crate::runtime::graph::AgentGraph;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{TraceCtx, Tracer};
use crate::skill::manifest::FsmNext;

const MAX_STEPS: u32 = 50;

/// Finite state machine: run agents as states, route by finish key.
///
/// Each agent calls finish(key, value). The FSM matches `key` against the
/// transition table for the current state. `value` becomes the input to the
/// next state's agent via start().
pub async fn run(
    graph: &AgentGraph,
    registry: &Arc<ModelRegistry>,
    initial: &str,
    states: &HashMap<String, Option<FsmNext>>,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    let mut current_state = initial.to_string();
    let mut current_input = input.to_string();
    // First state starts from parent; subsequent states chain from the previous state's span
    let mut prev_span_id: Option<String> = Some(ctx.span_id.clone());

    for step in 0..MAX_STEPS {
        eprintln!("  → fsm step {} / state '{current_state}'", step + 1);

        let output = super::run_node(
            graph,
            &current_state,
            registry,
            client,
            &current_input,
            tracer,
            ctx,
            crumb,
            prev_span_id.clone(),
        )
        .await?;

        let next_def = states
            .get(&current_state)
            .with_context(|| format!("FSM state '{current_state}' not in states map"))?;

        match next_def {
            None => {
                eprintln!("  → fsm: terminal state '{current_state}'");
                return Ok(output);
            }
            Some(FsmNext::Unconditional(next_agent)) => {
                if matches!(states.get(next_agent.as_str()), Some(None)) {
                    eprintln!("  → fsm: → terminal '{next_agent}'");
                    return Ok(output);
                }
                eprintln!("  → fsm: → '{next_agent}'");
                prev_span_id = Some(output.span_id.clone());
                current_input = output.value;
                current_state = next_agent.clone();
            }
            Some(FsmNext::Conditional(conds)) => {
                let routing_key = output.key.to_lowercase();
                eprintln!("  → fsm: routing key = '{routing_key}'");

                let next_agent = resolve_conditional(conds, &routing_key).with_context(|| {
                    format!("FSM: no transition for key '{routing_key}' in state '{current_state}'")
                })?;

                if matches!(states.get(next_agent.as_str()), Some(None)) {
                    eprintln!("  → fsm: → terminal '{next_agent}'");
                    return Ok(output);
                }
                eprintln!("  → fsm: → '{next_agent}'");
                prev_span_id = Some(output.span_id.clone());
                current_input = output.value;
                current_state = next_agent.clone();
            }
        }
    }

    bail!("FSM exceeded {MAX_STEPS} steps without reaching a terminal state")
}

/// Returns the target agent name. `*` is a catch-all.
fn resolve_conditional<'a>(conds: &'a [HashMap<String, String>], key: &str) -> Option<&'a String> {
    let mut fallback: Option<&'a String> = None;
    for map in conds {
        if let Some(target) = map.get(key) {
            return Some(target);
        }
        if let Some(target) = map.get("*") {
            fallback = Some(target);
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_exact_match() {
        let conds = vec![
            std::collections::HashMap::from([("good-enough".to_string(), "done".to_string())]),
            std::collections::HashMap::from([("needs-work".to_string(), "critique".to_string())]),
        ];
        assert_eq!(
            resolve_conditional(&conds, "needs-work"),
            Some(&"critique".to_string())
        );
    }

    #[test]
    fn resolve_first_match_wins() {
        let conds = vec![
            std::collections::HashMap::from([("yes".to_string(), "state-a".to_string())]),
            std::collections::HashMap::from([("yes".to_string(), "state-b".to_string())]),
        ];
        assert_eq!(
            resolve_conditional(&conds, "yes"),
            Some(&"state-a".to_string())
        );
    }

    #[test]
    fn resolve_catchall_wildcard() {
        let conds = vec![
            std::collections::HashMap::from([("yes".to_string(), "accept".to_string())]),
            std::collections::HashMap::from([("*".to_string(), "error-handler".to_string())]),
        ];
        assert_eq!(
            resolve_conditional(&conds, "unknown-word"),
            Some(&"error-handler".to_string())
        );
    }

    #[test]
    fn resolve_exact_beats_catchall() {
        let conds = vec![
            std::collections::HashMap::from([("yes".to_string(), "accept".to_string())]),
            std::collections::HashMap::from([("*".to_string(), "fallback".to_string())]),
        ];
        assert_eq!(
            resolve_conditional(&conds, "yes"),
            Some(&"accept".to_string())
        );
    }

    #[test]
    fn resolve_no_match_no_catchall_returns_none() {
        let conds = vec![std::collections::HashMap::from([(
            "yes".to_string(),
            "accept".to_string(),
        )])];
        assert_eq!(resolve_conditional(&conds, "no"), None);
    }

    #[test]
    fn resolve_empty_conds_returns_none() {
        let conds: Vec<std::collections::HashMap<String, String>> = vec![];
        assert_eq!(resolve_conditional(&conds, "anything"), None);
    }
}
