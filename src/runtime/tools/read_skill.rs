use anyhow::{Context, Result};
use genai::chat::Tool;
use serde::Deserialize;
use serde_json::json;

use crate::skill::parser::split_frontmatter;

pub fn definition() -> Tool {
    Tool::new("read_skill")
        .with_description(
            "Load full instructions for a skill. Call this first before using any skill.",
        )
        .with_schema(json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Skill name" }
            },
            "required": ["name"]
        }))
}

pub async fn execute(args: &serde_json::Value) -> Result<String> {
    let name = args["name"]
        .as_str()
        .context("read_skill: missing 'name'")?;
    let (body, _) = load_skill(name)?;
    Ok(body)
}

#[derive(Deserialize)]
struct SkillFrontmatter {
    #[serde(default)]
    description: String,
    #[serde(default)]
    tools: Vec<String>,
}

pub fn load_skill(name: &str) -> Result<(String, Vec<String>)> {
    let path = format!("skills/{name}/SKILL.md");
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("skill '{name}' not found at {path}"))?;

    let (yaml, body) =
        split_frontmatter(&content).with_context(|| format!("bad frontmatter in {path}"))?;
    let fm: SkillFrontmatter =
        serde_yaml::from_str(yaml).with_context(|| format!("invalid YAML in {path}"))?;

    Ok((format!("# Skill: {name}\n\n{body}"), fm.tools))
}

pub fn load_skill_description(skill_name: &str) -> String {
    let path = format!("skills/{skill_name}/SKILL.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return format!("(no description for {skill_name})");
    };
    let Ok((yaml, _)) = split_frontmatter(&content) else {
        return format!("(no description for {skill_name})");
    };
    let Ok(fm) = serde_yaml::from_str::<SkillFrontmatter>(yaml) else {
        return format!("(no description for {skill_name})");
    };
    if fm.description.is_empty() {
        format!("(no description for {skill_name})")
    } else {
        fm.description
    }
}

pub fn extract_body(md: &str) -> &str {
    let s = md.trim_start();
    if let Some(rest) = s.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            return rest[end + 4..].trim_start_matches('\n');
        }
    }
    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_body_after_frontmatter() {
        let md = "---\nname: s\n---\n\nThis is the body.\n";
        assert_eq!(extract_body(md), "This is the body.\n");
    }

    #[test]
    fn extract_body_no_frontmatter_returns_whole() {
        let md = "no frontmatter here";
        assert_eq!(extract_body(md), "no frontmatter here");
    }

    #[test]
    fn extract_body_empty_body() {
        let md = "---\nname: s\n---\n";
        assert_eq!(extract_body(md), "");
    }

    #[test]
    fn extract_body_leading_newlines_stripped() {
        let md = "---\nk: v\n---\n\n\nActual content";
        assert_eq!(extract_body(md), "Actual content");
    }

    #[test]
    fn parse_tools_block_sequence() {
        let md =
            "---\nname: mem-set\ndescription: Store a value.\ntools:\n  - mem_set\n---\n\nBody.\n";
        let (yaml, _) = split_frontmatter(md).unwrap();
        let fm: SkillFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.tools, vec!["mem_set"]);
    }

    #[test]
    fn parse_tools_inline_sequence() {
        let md = "---\nname: mem-get\ndescription: Get a value.\ntools: [mem_get]\n---\n\nBody.\n";
        let (yaml, _) = split_frontmatter(md).unwrap();
        let fm: SkillFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.tools, vec!["mem_get"]);
    }

    #[test]
    fn parse_tools_multiple() {
        let md = "---\nname: mem\ndescription: Memory.\ntools:\n  - mem_set\n  - mem_get\n---\n\nBody.\n";
        let (yaml, _) = split_frontmatter(md).unwrap();
        let fm: SkillFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.tools, vec!["mem_set", "mem_get"]);
    }

    #[test]
    fn parse_tools_absent_returns_empty() {
        let md = "---\nname: no-tools\ndescription: Nothing.\n---\n\nBody.\n";
        let (yaml, _) = split_frontmatter(md).unwrap();
        let fm: SkillFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert!(fm.tools.is_empty());
    }
}
