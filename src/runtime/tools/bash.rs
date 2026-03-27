use anyhow::{Context, Result};
use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("tama_bash")
        .with_description("Run a shell command.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to run" }
            },
            "required": ["command"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let cmd = args["command"]
        .as_str()
        .context("bash: missing 'command'")?;
    run_bash(cmd).await
}

pub async fn run_bash(cmd: &str) -> Result<String> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .context("bash: spawn failed")?;

    let mut result = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.stderr.is_empty() {
        result.push_str("\n[stderr]\n");
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    if !output.status.success() {
        result.push_str(&format!("\n[exit {}]", output.status));
    }
    Ok(result)
}
