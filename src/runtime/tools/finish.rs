use genai::chat::Tool;
use serde_json::json;

pub fn definition() -> Tool {
    Tool::new("finish")
        .with_description("Call when the task is complete. Provide a routing key and the final result.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "key":   { "type": "string", "description": "Routing word (e.g. \"done\", \"needs-work\", \"good-enough\")" },
                "value": { "type": "string", "description": "The result to pass to the next agent" }
            },
            "required": ["key", "value"]
        }))
}
