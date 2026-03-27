use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Default)]
pub struct TomlConfig {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub models: HashMap<String, ModelEntry>,
    #[serde(default)]
    pub providers: HashMap<String, ProviderEntry>,
}

#[derive(Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub entrypoint: String,
}

/// A model role entry in `[models]`. Supports shorthand (`"provider:model"`) or extended form.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ModelEntry {
    /// Shorthand: `default = "anthropic:claude-sonnet-4-6"`
    Shorthand(String),
    /// Extended: `thinker = { name = "anthropic:claude-opus-4-6", temperature = 1.0 }`
    Extended(ModelEntryConfig),
}

impl ModelEntry {
    pub fn name(&self) -> &str {
        match self {
            ModelEntry::Shorthand(s) => s,
            ModelEntry::Extended(e) => &e.name,
        }
    }
    pub fn temperature(&self) -> Option<f64> {
        match self {
            ModelEntry::Shorthand(_) => None,
            ModelEntry::Extended(e) => e.temperature,
        }
    }
    pub fn max_tokens(&self) -> Option<u32> {
        match self {
            ModelEntry::Shorthand(_) => None,
            ModelEntry::Extended(e) => e.max_tokens,
        }
    }
    pub fn base_url(&self) -> Option<&str> {
        match self {
            ModelEntry::Shorthand(_) => None,
            ModelEntry::Extended(e) => e.base_url.as_deref(),
        }
    }
}

/// Extended model entry configuration. No API keys — env vars only.
#[derive(Deserialize, Clone)]
pub struct ModelEntryConfig {
    pub name: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    /// Base URL override for self-hosted / proxy endpoints (non-secret, safe to commit).
    /// Prefer `TAMA_MODEL_{ROLE}_BASE_URL` env var for dynamic overrides.
    pub base_url: Option<String>,
}

/// Provider-level infrastructure config. Non-secret fields only — no API keys.
#[derive(Deserialize, Default, Clone)]
pub struct ProviderEntry {
    pub base_url: Option<String>,
}

impl TomlConfig {
    pub fn load() -> Result<Self> {
        let path = std::path::Path::new("tama.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }
}
