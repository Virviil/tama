use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_http_get")
        .with_description("Fetch content from a URL via HTTP GET.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to fetch" }
            },
            "required": ["url"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let url = args["url"].as_str().context("http_get: missing 'url'")?;
    reqwest::get(url)
        .await
        .context("http_get failed")?
        .text()
        .await
        .context("http_get: failed to read body")
}
