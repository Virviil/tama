use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("start")
        .with_description("Receive your assigned task. Call this first before beginning work.")
        .with_schema(json!({ "type": "object", "properties": {} }))
}
