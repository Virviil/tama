use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use genai::chat::Tool;
use serde_json::json;

// ── Store ─────────────────────────────────────────────────────────────────────

static STORE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Clear the store at the start of each run.
pub fn clear() {
    STORE.lock().unwrap().clear();
}

pub fn set(key: &str, value: &str) {
    STORE
        .lock()
        .unwrap()
        .insert(key.to_string(), value.to_string());
}

pub fn delete(key: &str) {
    STORE.lock().unwrap().remove(key);
}

/// Returns the current value for `key`, or `None` if not set.
pub fn get_opt(key: &str) -> Option<String> {
    STORE.lock().unwrap().get(key).cloned()
}

pub fn get(key: &str) -> String {
    STORE
        .lock()
        .unwrap()
        .get(key)
        .cloned()
        .unwrap_or_else(|| format!("[no value stored for key '{key}']"))
}

/// Appends `item` to the JSON array stored at `key`.
/// If the key doesn't exist, creates a new array `[item]`.
/// `item` may be any JSON value (object, string, number); if it can't be parsed
/// as JSON it is stored as a JSON string.
pub fn append(key: &str, item: &str) -> String {
    let mut store = STORE.lock().unwrap();
    let item_val: serde_json::Value =
        serde_json::from_str(item).unwrap_or_else(|_| serde_json::Value::String(item.to_string()));

    let mut arr: Vec<serde_json::Value> = store
        .get(key)
        .and_then(|v| serde_json::from_str(v).ok())
        .unwrap_or_default();

    arr.push(item_val);
    let serialized = serde_json::to_string(&arr).unwrap();
    store.insert(key.to_string(), serialized.clone());
    serialized
}

// ── Tool definitions ──────────────────────────────────────────────────────────

pub fn definition_set() -> Tool {
    Tool::new("tama_mem_set")
        .with_description("Store a value in shared memory so other agents can retrieve it.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "key":   { "type": "string", "description": "Storage key, e.g. \"poem\"" },
                "value": { "type": "string", "description": "Value to store" }
            },
            "required": ["key", "value"]
        }))
}

pub fn definition_get() -> Tool {
    Tool::new("tama_mem_get")
        .with_description("Retrieve a value previously stored by another agent.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "key": { "type": "string", "description": "Storage key, e.g. \"poem\"" }
            },
            "required": ["key"]
        }))
}

pub fn definition_append() -> Tool {
    Tool::new("tama_mem_append")
        .with_description("Append an item to a shared array. Creates the array if it doesn't exist yet. The item may be a JSON object, string, or number.")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "key":  { "type": "string", "description": "Array key, e.g. \"pipeline_errors\"" },
                "item": { "type": "string", "description": "JSON-encoded item to append, e.g. '{\"phase\":\"fixer\",\"error\":\"build failed\"}'" }
            },
            "required": ["key", "item"]
        }))
}
