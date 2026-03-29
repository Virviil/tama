use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_files_read")
        .with_description("Read a file from the workspace.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "File path" }
            },
            "required": ["path"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let path = args["path"].as_str().context("read_file: missing 'path'")?;
    std::fs::read_to_string(path).context("read_file failed")
}
