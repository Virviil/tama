use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::AgentOutput;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{TraceCtx, Tracer};
use crate::skill::manifest::ModelConfig;
use crate::skill::parser::parse_step;

pub struct OneshotStep {
    pub body: String,
    pub model: Option<ModelConfig>,
}

pub struct ReactStep {
    pub body: String,
    pub uses: Vec<String>,
    pub max_iter: u32,
    pub model: Option<ModelConfig>,
}

/// A parsed, self-contained step built from a .md file.
/// Call `Step::from_file(path)` to construct; call `step.run(...)` to execute.
/// The step resolves its own LlmClient from the baked-in model config (falling back
/// to the provided client), and dispatches internally to the correct pattern.
pub enum Step {
    Oneshot(OneshotStep),
    React(ReactStep),
}

impl Step {
    /// Parse a step .md file and construct the appropriate variant.
    /// Frontmatter is optional — no frontmatter defaults to oneshot.
    /// Set `pattern: react` (or non-empty `call: uses`) to get a React step.
    pub fn from_file(path: &Path) -> Result<Self> {
        let cfg = parse_step(path)?;
        let model = cfg.call.as_ref().and_then(|c| c.model.clone());
        if cfg.react {
            let max_iter = cfg.max_iter();
            Ok(Step::React(ReactStep {
                body: cfg.body,
                uses: cfg.call.map(|c| c.uses).unwrap_or_default(),
                max_iter,
                model,
            }))
        } else {
            Ok(Step::Oneshot(OneshotStep {
                body: cfg.body,
                model,
            }))
        }
    }

    /// Pattern name string for use in tracer calls (on_agent_start, etc.).
    pub fn pattern_name(&self) -> &'static str {
        match self {
            Step::Oneshot(_) => "oneshot",
            Step::React(_) => "react",
        }
    }

    /// Run this step. Internally resolves the LlmClient (model override or fallback)
    /// and dispatches to the correct pattern.
    ///
    /// - Oneshot steps always return `key="done"`, value = model response.
    /// - React steps return the key and value from whichever `finish(key, value)` call the
    ///   model made. Callers that route on the key (e.g. reflexion, plan_execute) should
    ///   inspect `output.key`; callers that only need the text can take `output.value`.
    pub async fn run(
        &self,
        input: &str,
        name: &str,
        registry: &Arc<ModelRegistry>,
        fallback_client: &LlmClient,
        tracer: &mut dyn Tracer,
        ctx: &TraceCtx,
        crumb: &str,
    ) -> Result<AgentOutput> {
        let owned_client;
        let client: &LlmClient = {
            let resolved = registry.resolve(self.model(), self.pattern_name())?;
            let c = LlmClient::from_resolved(&resolved, Some(fallback_client.debug_hook()))?
                .with_agent_name(name);
            owned_client = c;
            &owned_client
        };

        eprintln!("  → {name}…");
        match self {
            Step::Oneshot(s) => {
                let value =
                    super::oneshot::call(&s.body, input, name, client, tracer, ctx, crumb).await?;
                Ok(AgentOutput { key: "done".to_string(), value, span_id: String::new() })
            }
            Step::React(s) => {
                super::react::react_loop(
                    &s.body,
                    &s.uses,
                    s.max_iter,
                    client,
                    input,
                    tracer,
                    ctx,
                    &[],
                    crumb,
                )
                .await
            }
        }
    }

    fn model(&self) -> Option<&ModelConfig> {
        match self {
            Step::Oneshot(s) => s.model.as_ref(),
            Step::React(s) => s.model.as_ref(),
        }
    }
}
