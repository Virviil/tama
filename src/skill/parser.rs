use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::Path;

use super::manifest::*;

// ── Frontmatter extraction ────────────────────────────────────────────────────

pub fn split_frontmatter(source: &str) -> Result<(&str, &str)> {
    let source = source.trim_start();
    if !source.starts_with("---") {
        bail!("file does not start with '---' frontmatter");
    }
    let rest = &source[3..];
    let end = rest
        .find("\n---")
        .context("frontmatter closing '---' not found")?;
    let yaml = rest[..end].trim();
    let body = rest[end + 4..].trim_start_matches('\n');
    Ok((yaml, body))
}

// ── Raw frontmatter struct for SKILL.md ──────────────────────────────────────

#[derive(Deserialize)]
struct RawSkillFrontmatter {
    name: String,
    description: String,
    license: Option<String>,
    tama: serde_yaml::Value,
}

/// Flat tama: block for SKILL.md — pattern is a plain string, fields are flat.
#[derive(Deserialize)]
struct RawSkillTama {
    version: String,
    pattern: String,
    tool: Option<String>,
    depends: Option<Depends>,
    #[serde(default)]
    env: Vec<String>,
}

fn build_skill_pattern(raw: &RawSkillTama) -> Result<SkillPattern> {
    match raw.pattern.as_str() {
        "tool" => Ok(SkillPattern::Tool {
            tool: raw
                .tool
                .clone()
                .context("pattern: tool requires `tool:` field")?,
        }),
        other => bail!("unknown skill pattern: '{}'", other),
    }
}

// ── Raw frontmatter struct for AGENT.md (new flat format) ────────────────────

/// Raw deserialization target for the new flat AGENT.md frontmatter.
/// Does NOT include `body` — that comes from the content after `---`.
#[derive(Deserialize)]
struct RawAgentFrontmatter {
    name: String,
    description: String,
    version: String,
    env: Option<Vec<String>>,
    call: Option<CallConfig>,
    max_iter: Option<u32>,
    #[serde(flatten)]
    pattern: AgentPattern,
}

/// Raw deserialization target for step file frontmatter.
#[derive(Deserialize)]
struct RawStepFrontmatter {
    #[serde(default)]
    pattern: String,
    call: Option<CallConfig>,
}

// ── Public parse functions ────────────────────────────────────────────────────

pub fn parse_skill(path: &Path) -> Result<SkillFile> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;

    let (yaml, _body) = split_frontmatter(&source)
        .with_context(|| format!("bad frontmatter in {}", path.display()))?;

    let raw: RawSkillFrontmatter = serde_yaml::from_str(yaml)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;

    let tama: RawSkillTama = serde_yaml::from_value(raw.tama)
        .with_context(|| format!("invalid tama: block in {}", path.display()))?;

    let version = tama.version.clone();
    let depends = tama.depends.clone();
    let env = tama.env.clone();
    let pattern = build_skill_pattern(&tama).with_context(|| format!("in {}", path.display()))?;

    Ok(SkillFile {
        name: raw.name,
        description: raw.description,
        license: raw.license,
        tama: TamaSkillMeta {
            version,
            pattern,
            depends,
            env: Some(env),
        },
    })
}

/// Agent AGENT.md: deserialize frontmatter directly into the new flat format.
/// AgentPattern uses #[serde(tag = "pattern")] so pattern is a flat field.
pub fn parse_agent(path: &Path) -> Result<AgentFile> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;

    let (yaml, body) = split_frontmatter(&source)
        .with_context(|| format!("bad frontmatter in {}", path.display()))?;

    let raw: RawAgentFrontmatter = serde_yaml::from_str(yaml)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;

    Ok(AgentFile {
        name: raw.name,
        description: raw.description,
        version: raw.version,
        pattern: raw.pattern,
        env: raw.env,
        call: raw.call,
        max_iter: raw.max_iter,
        body: body.to_string(),
    })
}

/// Parse a step file (draft.md, critique.md, reflect.md, etc.).
///
/// Frontmatter is optional:
/// - No frontmatter → oneshot, no tools.
/// - `pattern: react` → react loop (with optional `call: uses/max_iter/model`).
/// - `pattern: oneshot` or omitted → single LLM call.
pub fn parse_step(path: &Path) -> Result<StepConfig> {
    let source =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;

    let trimmed = source.trim_start();
    if !trimmed.starts_with("---") {
        return Ok(StepConfig {
            react: false,
            call: None,
            body: source,
        });
    }

    let (yaml, body) = split_frontmatter(trimmed)
        .with_context(|| format!("bad frontmatter in {}", path.display()))?;

    let raw: RawStepFrontmatter = serde_yaml::from_str(yaml)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;

    let react = raw.pattern == "react"
        || raw.call.as_ref().map(|c| !c.uses.is_empty()).unwrap_or(false);

    Ok(StepConfig {
        react,
        call: raw.call,
        body: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn tmp() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tama_parser_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_agent(dir: &std::path::Path, yaml: &str) -> PathBuf {
        let content = format!("---\n{yaml}\n---\n\nAgent body text.\n");
        let path = dir.join("AGENT.md");
        fs::write(&path, content).unwrap();
        path
    }

    // ── split_frontmatter ────────────────────────────────────────────────────

    #[test]
    fn frontmatter_splits_correctly() {
        let src = "---\nfoo: bar\n---\n\nbody here";
        let (yaml, body) = split_frontmatter(src).unwrap();
        assert_eq!(yaml, "foo: bar");
        assert_eq!(body, "body here");
    }

    #[test]
    fn frontmatter_no_leading_dashes_fails() {
        assert!(split_frontmatter("no frontmatter here").is_err());
    }

    #[test]
    fn frontmatter_no_closing_dashes_fails() {
        assert!(split_frontmatter("---\nfoo: bar\n").is_err());
    }

    #[test]
    fn frontmatter_body_trimmed_leading_newline() {
        let src = "---\nk: v\n---\n\n\nActual body";
        let (_, body) = split_frontmatter(src).unwrap();
        assert_eq!(body, "Actual body");
    }

    // ── parse_agent: react ───────────────────────────────────────────────────

    #[test]
    fn parse_agent_react() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: react\ncall:\n  model:\n    role: thinker\n  uses: [search-web]");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert_eq!(agent.name, "test-agent");
        assert!(matches!(agent.pattern, AgentPattern::React));
        let uses = agent
            .call
            .as_ref()
            .map(|c| c.uses.as_slice())
            .unwrap_or(&[]);
        assert_eq!(uses, &["search-web"]);
        assert!(agent.body.contains("Agent body text."));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_react_empty_uses() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: react\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert!(matches!(agent.pattern, AgentPattern::React));
        let uses = agent
            .call
            .as_ref()
            .map(|c| c.uses.as_slice())
            .unwrap_or(&[]);
        assert!(uses.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_critic() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: critic\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert!(matches!(agent.pattern, AgentPattern::Critic));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_scatter() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: scatter\nworker: my-worker\ncall:\n  model:\n    role: thinker\n  uses: [fetch-url]");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        if let AgentPattern::Scatter { worker } = &agent.pattern {
            assert_eq!(worker, "my-worker");
        } else {
            panic!("expected Scatter");
        }
        let uses = agent
            .call
            .as_ref()
            .map(|c| c.uses.as_slice())
            .unwrap_or(&[]);
        assert_eq!(uses, &["fetch-url"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_parallel() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: parallel\nworkers: [agent-a, agent-b, agent-c]");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        if let AgentPattern::Parallel { workers } = &agent.pattern {
            assert_eq!(workers, &["agent-a", "agent-b", "agent-c"]);
        } else {
            panic!("expected Parallel");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_fsm() {
        let dir = tmp();
        let yaml = "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: fsm\ninitial: draft\nstates:\n  draft: critique\n  critique:";
        write_agent(&dir, yaml);
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        if let AgentPattern::Fsm { initial, states } = &agent.pattern {
            assert_eq!(initial, "draft");
            assert!(states.contains_key("draft"));
            assert!(states.contains_key("critique"));
        } else {
            panic!("expected Fsm");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_debate() {
        let dir = tmp();
        let yaml = "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: debate\nagents: [pro, con]\nrounds: 2\njudge: judge-agent\ncall:\n  model:\n    role: thinker";
        write_agent(&dir, yaml);
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        if let AgentPattern::Debate {
            agents,
            rounds,
            judge,
        } = &agent.pattern
        {
            assert_eq!(agents, &["pro", "con"]);
            assert_eq!(*rounds, 2);
            assert_eq!(judge, "judge-agent");
        } else {
            panic!("expected Debate");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_best_of_n() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: best-of-n\nn: 5\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        if let AgentPattern::BestOfN { n } = &agent.pattern {
            assert_eq!(*n, 5);
        } else {
            panic!("expected BestOfN");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_chain_of_verification() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: chain-of-verification\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert!(matches!(agent.pattern, AgentPattern::ChainOfVerification));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_plan_execute() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: plan-execute\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert!(matches!(agent.pattern, AgentPattern::PlanExecute));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_reflexion() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: reflexion\nmax_iter: 3\ncall:\n  model:\n    role: thinker\n  uses: [search-web]");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert!(matches!(agent.pattern, AgentPattern::Reflexion));
        assert_eq!(agent.max_iter, Some(3));
        let uses = agent
            .call
            .as_ref()
            .map(|c| c.uses.as_slice())
            .unwrap_or(&[]);
        assert_eq!(uses, &["search-web"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_constitutional() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: constitutional\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert!(matches!(agent.pattern, AgentPattern::Constitutional));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_model_name_override() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: react\ncall:\n  model:\n    name: anthropic:claude-opus-4-6");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        let mc = agent.call.as_ref().and_then(|c| c.model.as_ref()).unwrap();
        assert_eq!(mc.name.as_deref(), Some("anthropic:claude-opus-4-6"));
        assert!(mc.role.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_max_iter_at_root() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: react\nmax_iter: 25\ncall:\n  model:\n    role: thinker");
        let agent = parse_agent(&dir.join("AGENT.md")).unwrap();
        assert_eq!(agent.max_iter, Some(25));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_missing_file_fails() {
        let result = parse_agent(std::path::Path::new("/nonexistent/AGENT.md"));
        assert!(result.is_err());
    }

    #[test]
    fn parse_agent_invalid_pattern_fails() {
        let dir = tmp();
        write_agent(&dir, "name: test-agent\ndescription: A test agent.\nversion: \"1.0.0\"\npattern: unknown-pattern\ncall:\n  model:\n    role: thinker");
        assert!(parse_agent(&dir.join("AGENT.md")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_agent_body_extracted() {
        let dir = tmp();
        let path = write_agent(&dir, "name: test-agent\ndescription: Test.\nversion: \"1.0.0\"\npattern: react\ncall:\n  model:\n    role: thinker\nbody_marker: ignored");
        let agent = parse_agent(&path).unwrap();
        assert!(agent.body.contains("Agent body text."));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── parse_step ───────────────────────────────────────────────────────────

    #[test]
    fn parse_step_no_frontmatter() {
        let dir = tmp();
        let content = "This is the system prompt.\nWith multiple lines.\n";
        fs::write(dir.join("step.md"), content).unwrap();
        let step = parse_step(&dir.join("step.md")).unwrap();
        assert!(step.call.is_none());
        assert!(step.body.contains("This is the system prompt."));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_step_with_frontmatter() {
        let dir = tmp();
        let content = "---\ncall:\n  model:\n    role: worker\n    temperature: 0.2\n  uses: [bash]\n---\n\nSystem prompt here.\n";
        fs::write(dir.join("step.md"), content).unwrap();
        let step = parse_step(&dir.join("step.md")).unwrap();
        let call = step.call.unwrap();
        let model = call.model.unwrap();
        assert_eq!(model.role.as_deref(), Some("worker"));
        assert_eq!(model.temperature, Some(0.2));
        assert_eq!(call.uses, vec!["bash"]);
        assert!(step.body.contains("System prompt here."));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_step_empty_frontmatter() {
        let dir = tmp();
        let content = "---\n---\n\nJust a body.\n";
        fs::write(dir.join("step.md"), content).unwrap();
        let step = parse_step(&dir.join("step.md")).unwrap();
        assert!(step.call.is_none());
        assert!(step.body.contains("Just a body."));
        let _ = fs::remove_dir_all(&dir);
    }
}
