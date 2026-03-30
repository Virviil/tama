use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_http_get")
        .with_description("Fetch content from a URL via HTTP GET. Use ${ENV_VAR} in url or header values to inject environment variables without exposing them to the model.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to fetch. Supports ${ENV_VAR} substitution." },
                "headers": {
                    "type": "array",
                    "description": "Optional HTTP headers. Each entry is a {\"name\": \"value\"} object. Values support ${ENV_VAR} substitution.",
                    "items": {
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    }
                }
            },
            "required": ["url"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let url = crate::runtime::tools::resolve(
        args["url"].as_str().context("http_get: missing 'url'")?,
    );
    let mut req = reqwest::Client::new().get(&url);
    if let Some(headers) = args["headers"].as_array() {
        for entry in headers {
            if let Some(map) = entry.as_object() {
                for (k, v) in map {
                    if let Some(v) = v.as_str() {
                        req = req.header(k.as_str(), crate::runtime::tools::resolve(v));
                    }
                }
            }
        }
    }
    req.send()
        .await
        .context("http_get failed")?
        .text()
        .await
        .context("http_get: failed to read body")
}
