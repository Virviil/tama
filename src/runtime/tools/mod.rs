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

/// A segment of a parsed template string.
#[derive(Debug, PartialEq)]
pub enum Segment {
    /// Literal text, passed through as-is.
    Literal(String),
    /// `${VAR_NAME}` — to be resolved from the environment.
    Var(String),
}

/// Parse a template string into segments. Pure function, no env access.
///
/// Syntax:
/// - `${VAR}` → `Segment::Var("VAR")`
/// - `$$`     → `Segment::Literal("$")`
/// - anything else → `Segment::Literal(...)`
/// - unclosed `${` or bare `$` → treated as literal
pub fn parse_template(s: &str) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut literal = String::new();
    let mut chars = s.chars().peekable();

    let mut push_literal = |buf: &mut String, segs: &mut Vec<Segment>| {
        if !buf.is_empty() {
            segs.push(Segment::Literal(std::mem::take(buf)));
        }
    };

    while let Some(c) = chars.next() {
        if c != '$' {
            literal.push(c);
            continue;
        }
        match chars.peek() {
            Some('$') => {
                chars.next();
                literal.push('$');
            }
            Some('{') => {
                chars.next();
                let mut var_name = String::new();
                let mut closed = false;
                for vc in chars.by_ref() {
                    if vc == '}' {
                        closed = true;
                        break;
                    }
                    var_name.push(vc);
                }
                if closed {
                    push_literal(&mut literal, &mut segments);
                    segments.push(Segment::Var(var_name));
                } else {
                    // unclosed — treat as literal
                    literal.push_str("${");
                    literal.push_str(&var_name);
                }
            }
            _ => literal.push('$'),
        }
    }
    push_literal(&mut literal, &mut segments);
    segments
}

/// Resolve a parsed template by substituting `Var` segments from the environment.
/// Unknown variables are rendered as `${VAR_NAME}`.
pub fn resolve_env(segments: &[Segment]) -> String {
    let mut result = String::new();
    for seg in segments {
        match seg {
            Segment::Literal(s) => result.push_str(s),
            Segment::Var(name) => match std::env::var(name) {
                Ok(val) => result.push_str(&val),
                Err(_) => {
                    result.push_str("${");
                    result.push_str(name);
                    result.push('}');
                }
            },
        }
    }
    result
}

/// Convenience: parse and resolve in one call.
pub fn resolve(s: &str) -> String {
    resolve_env(&parse_template(s))
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

    // ── parse_template ───────────────────────────────────────────────────────

    #[test]
    fn parse_plain_literal() {
        assert_eq!(parse_template("hello"), vec![Segment::Literal("hello".into())]);
    }

    #[test]
    fn parse_single_var() {
        assert_eq!(
            parse_template("${FOO}"),
            vec![Segment::Var("FOO".into())]
        );
    }

    #[test]
    fn parse_var_with_surrounding_text() {
        assert_eq!(
            parse_template("Bearer ${TOKEN} rest"),
            vec![
                Segment::Literal("Bearer ".into()),
                Segment::Var("TOKEN".into()),
                Segment::Literal(" rest".into()),
            ]
        );
    }

    #[test]
    fn parse_multiple_vars() {
        assert_eq!(
            parse_template("${A}/${B}"),
            vec![
                Segment::Var("A".into()),
                Segment::Literal("/".into()),
                Segment::Var("B".into()),
            ]
        );
    }

    #[test]
    fn parse_dollar_dollar_becomes_literal_dollar() {
        assert_eq!(parse_template("$$"), vec![Segment::Literal("$".into())]);
    }

    #[test]
    fn parse_escaped_var_not_substituted() {
        // $$ → literal $, then {FOO} continues as literal → merged into "${FOO}"
        assert_eq!(
            parse_template("$${FOO}"),
            vec![Segment::Literal("${FOO}".into())]
        );
    }

    #[test]
    fn parse_unclosed_brace_is_literal() {
        assert_eq!(
            parse_template("${UNCLOSED"),
            vec![Segment::Literal("${UNCLOSED".into())]
        );
    }

    #[test]
    fn parse_bare_dollar_is_literal() {
        // bare $ not followed by { or $ → merged into surrounding literal
        assert_eq!(
            parse_template("price is $10"),
            vec![Segment::Literal("price is $10".into())]
        );
    }

    #[test]
    fn parse_no_dollar_empty_string() {
        assert_eq!(parse_template(""), vec![]);
    }

    // ── resolve_env (impure — tests use sentinel env vars) ───────────────────

    #[test]
    fn resolve_known_var() {
        std::env::set_var("_TAMA_TEST_FOO", "hello");
        assert_eq!(resolve_env(&[Segment::Var("_TAMA_TEST_FOO".into())]), "hello");
    }

    #[test]
    fn resolve_unknown_var_left_as_is() {
        assert_eq!(
            resolve_env(&[Segment::Var("_TAMA_NO_SUCH_VAR_XYZ".into())]),
            "${_TAMA_NO_SUCH_VAR_XYZ}"
        );
    }

    #[test]
    fn resolve_mixed_segments() {
        std::env::set_var("_TAMA_TEST_KEY", "secret");
        assert_eq!(
            resolve_env(&[
                Segment::Literal("Bearer ".into()),
                Segment::Var("_TAMA_TEST_KEY".into()),
            ]),
            "Bearer secret"
        );
    }

    // ── resolve (integration) ────────────────────────────────────────────────

    #[test]
    fn resolve_end_to_end() {
        std::env::set_var("_TAMA_TEST_E2E", "works");
        assert_eq!(resolve("result: ${_TAMA_TEST_E2E}"), "result: works");
    }

    #[test]
    fn resolve_no_vars_unchanged() {
        assert_eq!(resolve("https://example.com/path"), "https://example.com/path");
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
