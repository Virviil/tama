use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_files_write")
        .with_description("Write content to a file in the workspace.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "path":    { "type": "string", "description": "File path" },
                "content": { "type": "string", "description": "Content to write" }
            },
            "required": ["path", "content"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let path = args["path"]
        .as_str()
        .context("write_file: missing 'path'")?;
    let content = args["content"]
        .as_str()
        .context("write_file: missing 'content'")?;
    std::fs::write(path, content).context("write_file failed")?;
    Ok("written".to_string())
}
