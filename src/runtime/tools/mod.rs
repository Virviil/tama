use anyhow::Result;
use genai::chat::Tool;
use std::collections::HashSet;

pub mod bash;
pub mod finish;
pub mod http_get;
pub mod http_post;
pub mod inmemory;
pub mod read_file;
pub mod read_skill;
pub mod start;
pub mod write_file;

pub use read_skill::extract_body;

// ── Tool registry ──────────────────────────────────────────────────────────────

pub fn all_tools() -> Vec<(&'static str, Tool)> {
    vec![
        ("tama_bash", bash::definition()),
        ("tama_http_get", http_get::definition()),
        ("tama_http_post", http_post::definition()),
        ("tama_files_read", read_file::definition()),
        ("tama_files_write", write_file::definition()),
        ("tama_mem_set", inmemory::definition_set()),
        ("tama_mem_get", inmemory::definition_get()),
        ("tama_mem_append", inmemory::definition_append()),
    ]
}

pub fn always_tools() -> Vec<Tool> {
    vec![start::definition(), finish::definition()]
}

/// Build the active tool list: start + finish + read_skill (only when skills exist) + extra + unlocked.
/// `read_skill` is omitted when `uses` is empty to prevent models from hallucinating skill calls.
pub fn build_active_tools(
    uses: &[String],
    unlocked: &HashSet<String>,
    extra: &[Tool],
) -> Vec<Tool> {
    let mut tools = always_tools();
    if !uses.is_empty() {
        tools.push(read_skill::definition());
    }
    for t in extra {
        tools.push(t.clone());
    }
    for (name, tool) in all_tools() {
        if unlocked.contains(name) {
            tools.push(tool);
        }
    }
    tools
}

// ── Tool execution ─────────────────────────────────────────────────────────────

pub async fn execute_tool(name: &str, args: &serde_json::Value, span_id: &str) -> Result<String> {
    match name {
        "tama_bash" => bash::execute(args).await,
        "tama_http_get" => http_get::execute(args).await,
        "tama_http_post" => http_post::execute(args).await,
        "tama_files_read" => read_file::execute(args).await,
        "tama_files_write" => write_file::execute(args).await,
        "tama_mem_set" => {
            let k = args["key"].as_str().unwrap_or("");
            let v = args["value"].as_str().unwrap_or("");
            let old = inmemory::get_opt(k);
            crate::runtime::rollbacker::record_tool_call(
                span_id,
                "tama_mem_set",
                k,
                old.as_deref(),
            );
            inmemory::set(k, v);
            Ok(format!("stored '{k}'"))
        }
        "tama_mem_get" => Ok(inmemory::get(args["key"].as_str().unwrap_or(""))),
        "tama_mem_append" => {
            let k = args["key"].as_str().unwrap_or("");
            let item = args["item"].as_str().unwrap_or("");
            let result = inmemory::append(k, item);
            Ok(format!("appended to '{k}': {result}"))
        }
        other => {
            anyhow::bail!("unknown tool '{other}' — was it declared in the skill's tools: list?")
        }
    }
}

// ── System prompt builder ──────────────────────────────────────────────────────

pub fn build_system(system_tmpl: &str, uses: &[String]) -> String {
    let mut system = system_tmpl.to_string();
    if !uses.is_empty() {
        system.push_str("\n\n## Available skills\n");
        system.push_str("Call read_skill(name) to load full instructions before using a skill.\n");
        for skill_name in uses {
            let description = read_skill::load_skill_description(skill_name);
            system.push_str(&format!("- **{skill_name}**: {description}\n"));
        }
    }
    system.push_str("\n\nCall start() to receive your task before beginning work.");
    system
}

pub fn truncate(s: String, max: usize) -> String {
    if s.len() > max {
        s[..max].to_string()
    } else {
        s
    }
}

// ── Skill loading (re-exported for callers that need load_skill) ───────────────

pub use read_skill::load_skill;

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_system ─────────────────────────────────────────────────────────

    #[test]
    fn build_system_no_uses() {
        let system = build_system("Be helpful.", &[]);
        assert!(system.starts_with("Be helpful."));
        assert!(system.contains("Call start()"));
    }

    #[test]
    fn build_system_with_uses_appends_section() {
        let system = build_system("Be helpful.", &["search-web".to_string()]);
        assert!(system.contains("## Available skills"));
        assert!(system.contains("search-web"));
        assert!(system.contains("read_skill"));
    }

    #[test]
    fn build_system_multiple_uses() {
        let system = build_system(
            "Prompt.",
            &["search-web".to_string(), "write-file".to_string()],
        );
        assert!(system.contains("search-web"));
        assert!(system.contains("write-file"));
    }

    // ── truncate ─────────────────────────────────────────────────────────────

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello".to_string(), 100), "hello");
    }

    #[test]
    fn truncate_exact_limit_unchanged() {
        assert_eq!(truncate("hello".to_string(), 5), "hello");
    }

    #[test]
    fn truncate_long_string_cut() {
        let result = truncate("hello world".to_string(), 5);
        assert_eq!(result, "hello");
    }
}
