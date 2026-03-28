use anyhow::{Context, Result};
use genai::adapter::AdapterKind;
use genai::chat::{ChatOptions, ChatRequest, ChatResponse};
use genai::resolver::{AuthData, AuthResolver, Endpoint, ServiceTargetResolver};
use genai::{Client, ModelIden, ServiceTarget};
use std::sync::Arc;
use std::time::Instant;

use crate::runtime::debug_hook::{AfterAgentDecision, DebugHook, NoopHook, PreCallDecision};
use crate::runtime::model_registry::ResolvedModel;
use crate::skill::manifest::{ModelRef, Provider};

pub struct LlmClient {
    model_name: String,
    role: String,
    client: Client,
    agent_name: String,
    debug_hook: Arc<dyn DebugHook + Send + Sync>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl LlmClient {
    pub fn from_model_ref(
        model_ref: &ModelRef,
        debug_hook: Option<Arc<dyn DebugHook + Send + Sync>>,
    ) -> Result<Self> {
        let api_key = if model_ref.provider == Provider::Ollama {
            "ollama".to_string()
        } else {
            let env_var = match model_ref.provider {
                Provider::Anthropic => "ANTHROPIC_API_KEY",
                Provider::OpenAi => "OPENAI_API_KEY",
                Provider::Google => "GEMINI_API_KEY",
                Provider::Ollama => unreachable!(),
            };
            std::env::var(env_var).with_context(|| format!("{env_var} is not set"))?
        };
        let resolved = ResolvedModel {
            role: String::new(),
            provider: model_ref.provider.clone(),
            model_name: model_ref.model.clone(),
            temperature: None,
            max_tokens: None,
            api_key,
            base_url: None,
        };
        Self::from_resolved(&resolved, debug_hook)
    }

    /// Build from a fully resolved model config (primary path).
    /// Handles all providers, API keys, base URL overrides, temperature, and max_tokens.
    pub fn from_resolved(
        model: &ResolvedModel,
        debug_hook: Option<Arc<dyn DebugHook + Send + Sync>>,
    ) -> Result<Self> {
        let genai_model_name = match model.provider {
            Provider::Ollama => format!("ollama::{}", model.model_name),
            _ => model.model_name.clone(),
        };

        let client = build_genai_client(model)?;

        Ok(LlmClient {
            model_name: genai_model_name,
            role: if model.role.is_empty() { "custom".to_string() } else { model.role.clone() },
            client,
            agent_name: String::new(),
            debug_hook: debug_hook.unwrap_or_else(|| Arc::new(NoopHook)),
            temperature: model.temperature.map(|t| t as f32),
            max_tokens: model.max_tokens,
        })
    }

    pub fn with_agent_name(mut self, name: &str) -> Self {
        self.agent_name = name.to_string();
        self
    }

    pub fn with_temperature(mut self, t: f32) -> Self {
        self.temperature = Some(t);
        self
    }

    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    /// Returns a clone of the debug hook for propagation to sub-clients.
    pub fn debug_hook(&self) -> Arc<dyn DebugHook + Send + Sync> {
        self.debug_hook.clone()
    }

    /// Single user-message call, returns text.
    pub async fn call(
        &self,
        user: &str,
        trace_id: &str,
        span_id: &str,
        crumb: &str,
    ) -> Result<String> {
        let req = ChatRequest::from_user(user);
        let response = self.chat(req, trace_id, span_id, crumb).await?;
        Ok(response.first_text().unwrap_or_default().to_string())
    }

    /// System prompt + user message call, returns text.
    pub async fn call_system_user(
        &self,
        system: &str,
        user: &str,
        trace_id: &str,
        span_id: &str,
        crumb: &str,
    ) -> Result<String> {
        let (text, _, _) = self
            .call_system_user_tracked(system, user, "", trace_id, span_id, crumb)
            .await?;
        Ok(text)
    }

    /// System prompt + user message call, returns (text, input_tokens, output_tokens).
    /// `step` is displayed in the debugger header; pass an empty string when not in a named loop.
    pub async fn call_system_user_tracked(
        &self,
        system: &str,
        user: &str,
        step: &str,
        trace_id: &str,
        span_id: &str,
        crumb: &str,
    ) -> Result<(String, u32, u32)> {
        use genai::chat::ChatMessage;
        let req = ChatRequest::from_system(system).append_message(ChatMessage::user(user));
        let response = self
            .chat_with_step(req, step, "", trace_id, span_id, crumb)
            .await?;
        let (input_tokens, output_tokens) = extract_usage(&response);
        let text = response.first_text().unwrap_or_default().to_string();
        Ok((text, input_tokens, output_tokens))
    }

    /// Full chat call — caller controls messages, tools, system prompt.
    /// Use `chat_with_step` when a step name is available (react, scatter loops).
    pub async fn chat(
        &self,
        req: ChatRequest,
        trace_id: &str,
        span_id: &str,
        crumb: &str,
    ) -> Result<ChatResponse> {
        self.chat_with_step(req, "", "", trace_id, span_id, crumb)
            .await
    }

    /// Full chat call with a step name for the debug hook display.
    /// `context` is shown in the debugger before the call — pass tool results from the previous iteration.
    pub async fn chat_with_step(
        &self,
        mut req: ChatRequest,
        step: &str,
        context: &str,
        trace_id: &str,
        span_id: &str,
        crumb: &str,
    ) -> Result<ChatResponse> {
        let system = req.system.as_deref().unwrap_or("").to_string();

        match self.debug_hook.before_call(
            &self.agent_name,
            step,
            &self.model_name,
            &system,
            context,
            trace_id,
            span_id,
            crumb,
        ) {
            PreCallDecision::Proceed {
                system_override: Some(new_sys),
            } => {
                req.system = Some(new_sys);
            }
            PreCallDecision::Proceed {
                system_override: None,
            } => {}
            PreCallDecision::Quit => anyhow::bail!("debug: user quit"),
        }

        // Build ChatOptions applying temperature and max_tokens if set
        let mut opts = ChatOptions::default();
        if let Some(t) = self.temperature {
            opts = opts.with_temperature(t as f64);
        }
        if let Some(n) = self.max_tokens {
            opts = opts.with_max_tokens(n);
        }
        let use_opts = self.temperature.is_some() || self.max_tokens.is_some();

        let t = Instant::now();
        let response = self
            .client
            .exec_chat(
                &self.model_name,
                req,
                if use_opts { Some(&opts) } else { None },
            )
            .await
            .context("LLM call failed")?;
        let duration_ms = t.elapsed().as_millis();

        let (input_tokens, output_tokens) = extract_usage(&response);
        let response_text = response
            .first_text()
            .map(str::to_string)
            .unwrap_or_default();
        let tool_lines: Vec<String> = response
            .tool_calls()
            .iter()
            .map(|tc| format!("  tool: {}({})", tc.fn_name, tc.fn_arguments))
            .collect();
        let display = match (response_text.is_empty(), tool_lines.is_empty()) {
            (true, true) => String::new(),
            (true, false) => tool_lines.join("\n"),
            (false, true) => response_text.clone(),
            (false, false) => format!("{}\n{}", response_text, tool_lines.join("\n")),
        };
        self.debug_hook.after_call(
            &self.agent_name,
            step,
            &display,
            input_tokens,
            output_tokens,
            duration_ms,
            trace_id,
            span_id,
            crumb,
        );

        Ok(response)
    }

    /// Called after the agent's full run completes. Returns whether to retry the whole agent.
    pub fn after_agent(
        &self,
        pattern: &str,
        key: &str,
        value: &str,
        trace_id: &str,
        span_id: &str,
        crumb: &str,
    ) -> AfterAgentDecision {
        self.debug_hook.after_agent(
            &self.agent_name,
            pattern,
            key,
            value,
            trace_id,
            span_id,
            crumb,
        )
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    pub fn role(&self) -> &str {
        &self.role
    }
}

/// Extract (input_tokens, output_tokens) from a ChatResponse.
pub fn extract_usage(response: &ChatResponse) -> (u32, u32) {
    let input = response.usage.prompt_tokens.unwrap_or(0).max(0) as u32;
    let output = response.usage.completion_tokens.unwrap_or(0).max(0) as u32;
    (input, output)
}

/// Build a genai Client for the given resolved model, handling base_url overrides.
fn build_genai_client(model: &ResolvedModel) -> Result<Client> {
    if let Some(base_url) = &model.base_url {
        // Custom endpoint: use ServiceTargetResolver to override endpoint + auth + adapter kind
        let url = base_url.clone();
        let api_key = model.api_key.clone();
        let adapter_kind = provider_to_adapter_kind(&model.provider);
        let model_name = model.model_name.clone();
        let target_resolver = ServiceTargetResolver::from_resolver_fn(
            move |_service_target: ServiceTarget| {
                let endpoint = Endpoint::from_owned(url.clone());
                let auth = AuthData::from_single(api_key.clone());
                let model_iden = ModelIden::new(adapter_kind, model_name.clone());
                Ok(ServiceTarget { endpoint, auth, model: model_iden })
            },
        );
        Ok(Client::builder().with_service_target_resolver(target_resolver).build())
    } else if model.provider == Provider::Ollama {
        // Ollama default: genai auto-detects from "ollama::" model name prefix
        Ok(Client::builder().build())
    } else {
        // Standard API: AuthResolver with provider key
        let key = model.api_key.clone();
        let auth =
            AuthResolver::from_resolver_fn(move |_model_iden| Ok(Some(AuthData::from_single(key.clone()))));
        Ok(Client::builder().with_auth_resolver(auth).build())
    }
}

fn provider_to_adapter_kind(provider: &Provider) -> AdapterKind {
    match provider {
        Provider::Anthropic => AdapterKind::Anthropic,
        Provider::OpenAi => AdapterKind::OpenAI,
        Provider::Google => AdapterKind::Gemini,
        Provider::Ollama => AdapterKind::Ollama,
    }
}
