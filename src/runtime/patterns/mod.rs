pub mod best_of_n;
pub mod chain_of_verification;
pub mod constitutional;
pub mod critic;
pub mod debate;
pub mod fsm;
pub mod human;
pub mod oneshot;
pub mod parallel;
pub mod plan_execute;
pub mod react;
pub mod reflexion;
pub mod scatter;
pub mod step;

use anyhow::{Context, Result};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use crate::runtime::debug_hook::AfterAgentDecision;
use crate::runtime::graph::{AgentGraph, AgentNode};
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{TraceCtx, Tracer};
use crate::skill::manifest::AgentPattern;

/// Output from an agent run.
pub struct AgentOutput {
    /// Routing key; "done" for terminal/non-routing patterns.
    pub key: String,
    /// The actual result content passed to the next agent.
    pub value: String,
    /// The span_id of this agent's execution (used by FSM to chain prev_span_id).
    pub span_id: String,
}

/// Run a named agent from the graph.
/// Returns a boxed future to allow recursive async calls.
/// `parent_crumb` is the breadcrumb of the calling agent (empty for the root).
pub fn run_node<'a>(
    graph: &'a AgentGraph,
    name: &'a str,
    registry: &'a Arc<ModelRegistry>,
    fallback_client: &'a LlmClient,
    input: &'a str,
    tracer: &'a mut dyn Tracer,
    parent_ctx: &'a TraceCtx,
    parent_crumb: &'a str,
    // The span that completed immediately before this one in the parent's scope.
    // Pass Some(parent_ctx.span_id) for parallel workers (all start from parent).
    // Pass Some(previous_sibling_span_id) for sequential chains (FSM states).
    prev_span_id: Option<String>,
) -> Pin<Box<dyn Future<Output = Result<AgentOutput>> + 'a>> {
    Box::pin(async move {
        let node = graph
            .get(name)
            .with_context(|| format!("agent '{name}' not in graph"))?;

        let owned_client;
        let call = node.agent.call.as_ref();
        let model_config = call.and_then(|c| c.model.as_ref());
        let pattern_str = pattern_name(&node.agent.pattern);
        let client = {
            let resolved = registry.resolve(model_config, pattern_str)?;
            let c = LlmClient::from_resolved(&resolved, Some(fallback_client.debug_hook()))?
                .with_agent_name(name);
            owned_client = c;
            &owned_client
        };

        let max_iter = node.agent.max_iter.unwrap_or(10);

        let pattern = pattern_name(&node.agent.pattern);

        // Build the display breadcrumb: "parent→name" or just "name" at the root.
        let crumb = if parent_crumb.is_empty() {
            name.to_string()
        } else {
            format!("{parent_crumb}→{name}")
        };

        // Each attempt gets its own child span. On retry, old span is closed and a
        // new one opened — rollbacker uses the span_id to undo side-effects.
        // node_id is stable across retries — uniquely identifies this node's position in the tree.
        let node_id = crate::runtime::tracer::new_node_id();
        let mut agent_ctx = parent_ctx.child();
        let mut prev = prev_span_id.clone();
        eprintln!("tamad: run '{}' pattern={}", node.name, pattern);
        tracer.on_agent_start(&agent_ctx, name, pattern, input, prev.as_deref(), &node_id);

        loop {
            let t = Instant::now();
            let result = dispatch(
                graph, node, registry, client, input, max_iter, tracer, &agent_ctx, &crumb,
            )
            .await;
            let duration_ms = t.elapsed().as_millis();

            match result {
                Ok(output) => {
                    let span_id = agent_ctx.span_id.clone();
                    tracer.on_agent_end(&agent_ctx, &output.key, &output.value, duration_ms);
                    match client.after_agent(
                        pattern,
                        &output.key,
                        &output.value,
                        &agent_ctx.trace_id,
                        &agent_ctx.span_id,
                        &crumb,
                    ) {
                        AfterAgentDecision::Proceed => break Ok(AgentOutput { span_id, ..output }),
                        AfterAgentDecision::Retry => {
                            eprintln!("  debug: retrying agent '{name}'");
                            crate::runtime::rollbacker::rollback(&agent_ctx.span_id);
                            // Retry is sequential: prev = the failed attempt's span
                            prev = Some(agent_ctx.span_id.clone());
                            agent_ctx = parent_ctx.child();
                            tracer.on_agent_start(
                                &agent_ctx,
                                name,
                                pattern,
                                input,
                                prev.as_deref(),
                                &node_id,
                            );
                            continue;
                        }
                    }
                }
                Err(e) => {
                    tracer.on_agent_end(&agent_ctx, "error", &e.to_string(), duration_ms);
                    break Err(e);
                }
            }
        }
    })
}

async fn dispatch(
    graph: &AgentGraph,
    node: &AgentNode,
    registry: &Arc<ModelRegistry>,
    client: &LlmClient,
    input: &str,
    max_iter: u32,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    let uses = node
        .agent
        .call
        .as_ref()
        .map(|c| c.uses.as_slice())
        .unwrap_or(&[]);
    match &node.agent.pattern {
        AgentPattern::Critic => critic::run(&node.dir, registry, client, input, tracer, ctx, crumb).await,
        AgentPattern::React => {
            react::run(
                &node.agent.body,
                uses,
                max_iter,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::Scatter { worker } => {
            scatter::run(
                graph,
                registry,
                &node.dir,
                &node.agent.body,
                worker,
                uses,
                max_iter,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::Parallel { workers } => {
            parallel::run(graph, registry, workers, client, input, tracer, ctx, crumb).await
        }
        AgentPattern::Fsm { initial, states } => {
            fsm::run(graph, registry, initial, states, client, input, tracer, ctx, crumb).await
        }
        AgentPattern::Reflexion => {
            let reflexion_iter = node.agent.max_iter.unwrap_or(4);
            reflexion::run(
                &node.dir,
                reflexion_iter,
                registry,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::Debate {
            agents,
            rounds,
            judge,
        } => {
            debate::run(
                graph, registry, agents, *rounds, judge, client, input, tracer, ctx, crumb,
            )
            .await
        }
        AgentPattern::Constitutional => {
            constitutional::run(
                &node.dir,
                &node.agent.body,
                registry,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::BestOfN { n } => {
            best_of_n::run(
                &node.agent.body,
                uses,
                *n,
                &node.dir,
                registry,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::ChainOfVerification => {
            chain_of_verification::run(
                &node.dir,
                &node.agent.body,
                registry,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::PlanExecute => {
            plan_execute::run(
                &node.dir,
                &node.agent.body,
                registry,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::Human => {
            human::run(
                &node.dir,
                &node.agent.body,
                uses,
                max_iter,
                registry,
                client,
                input,
                tracer,
                ctx,
                crumb,
            )
            .await
        }
        AgentPattern::Oneshot => {
            oneshot::run(&node.agent.body, client, input, tracer, ctx, crumb).await
        }
    }
}

fn pattern_name(p: &AgentPattern) -> &'static str {
    match p {
        AgentPattern::Critic => "critic",
        AgentPattern::React => "react",
        AgentPattern::Scatter { .. } => "scatter",
        AgentPattern::Parallel { .. } => "parallel",
        AgentPattern::Fsm { .. } => "fsm",
        AgentPattern::Reflexion => "reflexion",
        AgentPattern::Debate { .. } => "debate",
        AgentPattern::Constitutional => "constitutional",
        AgentPattern::BestOfN { .. } => "best_of_n",
        AgentPattern::ChainOfVerification => "chain_of_verification",
        AgentPattern::PlanExecute => "plan_execute",
        AgentPattern::Human => "human",
        AgentPattern::Oneshot => "oneshot",
    }
}
