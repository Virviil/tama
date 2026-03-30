use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_http_post")
        .with_description("Send an HTTP POST request with a JSON body. Use ${ENV_VAR} in url or header values to inject environment variables without exposing them to the model.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "url":  { "type": "string", "description": "URL to post to. Supports ${ENV_VAR} substitution." },
                "body": { "type": "string", "description": "JSON body as string" },
                "headers": {
                    "type": "array",
                    "description": "Optional HTTP headers. Each entry is a {\"name\": \"value\"} object. Values support ${ENV_VAR} substitution.",
                    "items": {
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    }
                }
            },
            "required": ["url", "body"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let url = crate::runtime::tools::resolve(
        args["url"].as_str().context("http_post: missing 'url'")?,
    );
    let body = args["body"].as_str().context("http_post: missing 'body'")?;
    let mut req = reqwest::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string());
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
        .context("http_post failed")?
        .text()
        .await
        .context("http_post: failed to read body")
}
