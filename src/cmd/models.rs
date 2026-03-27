use anyhow::{Context, Result};
use std::collections::BTreeMap;

const LITELLM_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

const PROVIDERS: &[(&str, &str)] = &[
    ("Anthropic", "anthropic"),
    ("OpenAI", "openai"),
    ("Gemini", "gemini"),
];

pub async fn run() -> Result<()> {
    eprint!("fetching model list... ");
    let resp = reqwest::get(LITELLM_URL)
        .await
        .context("failed to fetch LiteLLM model list")?
        .json::<serde_json::Value>()
        .await
        .context("failed to parse response")?;
    eprintln!("done");

    let obj = resp.as_object().context("unexpected JSON shape")?;

    // Group by provider: BTreeMap keeps insertion order per provider
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (label, provider_id) in PROVIDERS {
        groups.insert(label, vec![]);
        for (model_name, meta) in obj {
            if model_name == "sample_spec" {
                continue;
            }
            let is_provider =
                meta.get("litellm_provider").and_then(|v| v.as_str()) == Some(provider_id);
            let is_chat = meta.get("mode").and_then(|v| v.as_str()) == Some("chat");
            if is_provider && is_chat {
                groups.get_mut(label).unwrap().push(model_name.as_str());
            }
        }
        groups.get_mut(label).unwrap().sort_unstable();
    }

    for (label, provider_id) in PROVIDERS {
        let models = &groups[label];
        println!("\n{label} ({} models)", models.len());
        for name in models {
            // strip "provider/" prefix if present
            let display = name
                .strip_prefix(provider_id)
                .and_then(|s| s.strip_prefix('/'))
                .unwrap_or(name);
            println!("  {display}");
        }
    }

    Ok(())
}
