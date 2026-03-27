use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_http_post")
        .with_description("Send an HTTP POST request with a JSON body.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "url":  { "type": "string", "description": "URL to post to" },
                "body": { "type": "string", "description": "JSON body as string" }
            },
            "required": ["url", "body"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let url = args["url"].as_str().context("http_post: missing 'url'")?;
    let body = args["body"].as_str().context("http_post: missing 'body'")?;
    reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .context("http_post failed")?
        .text()
        .await
        .context("http_post: failed to read body")
}
