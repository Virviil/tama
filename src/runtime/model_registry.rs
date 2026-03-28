use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;

use crate::config::TomlConfig;
use crate::skill::manifest::{ModelConfig, Provider};

// ── Built-in role temperature defaults ───────────────────────────────────────

fn role_default_temperature(role: &str) -> Option<f64> {
    match role {
        "thinker" => Some(1.0),
        "worker" => Some(0.0),
        "default" => Some(0.8),
        _ => None,
    }
}

/// Pattern default role: which role to use when no `model:` is set in frontmatter.
pub fn pattern_default_role(pattern: &str) -> &'static str {
    match pattern {
        "react" => "thinker",
        "oneshot" => "worker",
        _ => "default",
    }
}

// ── ResolvedModel ─────────────────────────────────────────────────────────────

/// Fully resolved, immutable model config — ready to build an LlmClient.
/// Built once at startup; all env vars and toml values already applied.
#[derive(Clone)]
pub struct ResolvedModel {
    pub role: String,
    pub provider: Provider,
    /// Bare model name passed to the LLM provider (no prefix).
    pub model_name: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    /// API key (always present; Ollama uses dummy value "ollama").
    pub api_key: String,
    /// Custom base URL for self-hosted / proxy endpoints. None = provider default.
    pub base_url: Option<String>,
}

// ── ModelRegistry ─────────────────────────────────────────────────────────────

/// Immutable registry of resolved models. Built once at startup.
/// Validates all roles before any LLM call is made.
pub struct ModelRegistry {
    models: HashMap<String, ResolvedModel>,
}

impl ModelRegistry {
    /// Build from config + current environment. Fails if any role cannot be fully resolved.
    pub fn build(config: &TomlConfig) -> Result<Self> {
        let mut models = HashMap::new();
        for (role, entry) in &config.models {
            let resolved = resolve_role(
                role,
                entry.name(),
                entry.temperature(),
                entry.max_tokens(),
                entry.base_url(),
                &config.providers,
            )
            .with_context(|| format!("failed to configure model role '{role}'"))?;
            models.insert(role.clone(), resolved);
        }
        Ok(ModelRegistry { models })
    }

    /// Get a model by exact role name. Error if role not in registry.
    pub fn get(&self, role: &str) -> Result<&ResolvedModel> {
        self.models
            .get(role)
            .ok_or_else(|| anyhow!("model role '{}' is not defined in tama.toml [models]", role))
    }

    /// Resolve the model for an agent or step call.
    ///
    /// Resolution order:
    /// 1. `model_config.name` — direct provider:model spec
    /// 2. `model_config.role` — named role lookup in registry
    /// 3. `None` — pattern default role (`react`→`thinker`, everything else→`worker`)
    ///    falling back to `default` if the pattern default isn't defined
    ///
    /// Local `temperature`/`max_tokens` overrides in `model_config` are applied on top.
    pub fn resolve(
        &self,
        model_config: Option<&ModelConfig>,
        pattern: &str,
    ) -> Result<ResolvedModel> {
        let (mut resolved, local_temp, local_max_tokens) = match model_config {
            Some(mc) if mc.name.is_some() => {
                // Direct spec — resolve from provider:model string + env for key/url
                let name = mc.name.as_ref().unwrap();
                let r = resolve_direct_spec(name)
                    .with_context(|| format!("in model: name: '{name}'"))?;
                (r, mc.temperature.map(|t| t as f64), mc.max_tokens)
            }
            Some(mc) if mc.role.is_some() => {
                let role = mc.role.as_ref().unwrap();
                let r = self.get_or_default(role)?;
                (r.clone(), mc.temperature.map(|t| t as f64), mc.max_tokens)
            }
            _ => {
                // Pattern default
                let default_role = pattern_default_role(pattern);
                let r = self.get_or_default(default_role)?;
                (r.clone(), None, None)
            }
        };

        // Apply local per-call overrides
        if let Some(t) = local_temp {
            resolved.temperature = Some(t);
        }
        if let Some(mt) = local_max_tokens {
            resolved.max_tokens = Some(mt);
        }

        Ok(resolved)
    }

    /// Look up role; if not found, fall back to `default`; error if neither exists.
    fn get_or_default(&self, role: &str) -> Result<&ResolvedModel> {
        if let Some(m) = self.models.get(role) {
            return Ok(m);
        }
        if let Some(m) = self.models.get("default") {
            return Ok(m);
        }
        bail!(
            "model role '{}' is not defined in tama.toml [models], and no 'default' role is defined.\n  \
            Add a [models] section to tama.toml or set TAMA_MODEL_{}_NAME.",
            role,
            role.to_uppercase().replace('-', "_")
        )
    }
}

// ── Resolution helpers ────────────────────────────────────────────────────────

fn resolve_role(
    role: &str,
    entry_name: &str,
    entry_temperature: Option<f64>,
    entry_max_tokens: Option<u32>,
    entry_base_url: Option<&str>,
    providers: &HashMap<String, crate::config::ProviderEntry>,
) -> Result<ResolvedModel> {
    let role_upper = role.to_uppercase().replace('-', "_");

    // Model name: env override > entry name
    let name_str = std::env::var(format!("TAMA_MODEL_{role_upper}_NAME"))
        .unwrap_or_else(|_| entry_name.to_string());

    let (provider, model_name) = parse_provider_model(&name_str)
        .with_context(|| format!("invalid model spec '{name_str}'"))?;

    // Temperature: env > entry > role built-in default
    let env_temp: Option<f64> = std::env::var(format!("TAMA_MODEL_{role_upper}_TEMPERATURE"))
        .ok()
        .and_then(|s| s.parse().ok());
    let temperature = env_temp
        .or(entry_temperature)
        .or_else(|| role_default_temperature(role));

    // Max tokens: env > entry (no built-in default — ∞)
    let env_max_tokens: Option<u32> = std::env::var(format!("TAMA_MODEL_{role_upper}_MAX_TOKENS"))
        .ok()
        .and_then(|s| s.parse().ok());
    let max_tokens = env_max_tokens.or(entry_max_tokens);

    // API key (Ollama doesn't need one)
    let api_key = if provider == Provider::Ollama {
        "ollama".to_string()
    } else {
        let provider_env = provider_api_key_env(&provider);
        std::env::var(format!("TAMA_MODEL_{role_upper}_API_KEY"))
            .or_else(|_| std::env::var(provider_env))
            .with_context(|| {
                format!(
                    "model role '{}' (provider: {}) is missing an API key.\n  \
                    Set TAMA_MODEL_{role_upper}_API_KEY or {}.",
                    role, provider, provider_env
                )
            })?
    };

    // Base URL: env (role-specific) > env (provider-wide) > entry > provider config in tama.toml
    let provider_str = provider.to_string().to_uppercase();
    let base_url = std::env::var(format!("TAMA_MODEL_{role_upper}_BASE_URL"))
        .ok()
        .or_else(|| std::env::var(format!("TAMA_PROVIDER_{provider_str}_BASE_URL")).ok())
        .or_else(|| entry_base_url.map(str::to_string))
        .or_else(|| {
            providers
                .get(&provider.to_string())
                .and_then(|p| p.base_url.clone())
        });

    Ok(ResolvedModel {
        role: role.to_string(),
        provider,
        model_name,
        temperature,
        max_tokens,
        api_key,
        base_url,
    })
}

/// Resolve a direct `name: provider:model` spec (used in agent/step frontmatter).
/// No role-specific env overrides — uses provider standard key only.
fn resolve_direct_spec(name: &str) -> Result<ResolvedModel> {
    let (provider, model_name) = parse_provider_model(name)?;

    let api_key = if provider == Provider::Ollama {
        "ollama".to_string()
    } else {
        let provider_env = provider_api_key_env(&provider);
        let provider_str = provider.to_string().to_uppercase();
        std::env::var(format!("TAMA_PROVIDER_{provider_str}_API_KEY"))
            .or_else(|_| std::env::var(provider_env))
            .with_context(|| {
                format!(
                    "direct model spec '{}' is missing an API key.\n  Set {}.",
                    name, provider_env
                )
            })?
    };

    let provider_str = provider.to_string().to_uppercase();
    let base_url = std::env::var(format!("TAMA_PROVIDER_{provider_str}_BASE_URL")).ok();

    Ok(ResolvedModel {
        role: String::new(),
        provider,
        model_name,
        temperature: None,
        max_tokens: None,
        api_key,
        base_url,
    })
}

/// Parse "provider:model-name" → (Provider, model_name).
/// For Ollama, model may contain colons (e.g., "ollama:qwen2.5:14b" → model = "qwen2.5:14b").
fn parse_provider_model(s: &str) -> Result<(Provider, String)> {
    let (provider_str, model) = s
        .split_once(':')
        .with_context(|| format!("invalid model spec '{}': expected 'provider:model'", s))?;

    if model.is_empty() {
        bail!("model name cannot be empty in '{}'", s);
    }

    use crate::skill::manifest::Provider::*;
    let provider = match provider_str {
        "anthropic" => Anthropic,
        "openai" => OpenAi,
        "google" => Google,
        "ollama" => Ollama,
        other => bail!(
            "unknown provider '{}': supported providers: anthropic, openai, google, ollama",
            other
        ),
    };

    Ok((provider, model.to_string()))
}

fn provider_api_key_env(provider: &Provider) -> &'static str {
    match provider {
        Provider::Anthropic => "ANTHROPIC_API_KEY",
        Provider::OpenAi => "OPENAI_API_KEY",
        Provider::Google => "GEMINI_API_KEY",
        Provider::Ollama => unreachable!("Ollama doesn't use an API key"),
    }
}
