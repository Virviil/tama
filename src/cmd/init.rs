use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

fn validate_kebab_case(s: &str, label: &str) -> Result<()> {
    if s.is_empty() {
        bail!("{label} cannot be empty");
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("{label} '{s}' must be kebab-case (lowercase letters, digits, hyphens only)");
    }
    Ok(())
}

pub fn run(name: &str) -> Result<()> {
    validate_kebab_case(name, "project name")?;

    let root = Path::new(name);
    if root.exists() {
        bail!("'{}' already exists", name);
    }

    let agent_name = format!("{name}-agent");

    fs::create_dir_all(root.join("agents").join(&agent_name))?;
    fs::create_dir_all(root.join("skills"))?;

    fs::write(
        root.join(".env.example"),
        "ANTHROPIC_API_KEY=sk-ant-...\n\
         \n\
         # Model roles: TAMA_MODEL_{ROLE}=provider:model-name\n\
         TAMA_MODEL_THINKER=anthropic:claude-opus-4-6\n\
         TAMA_MODEL_WORKER=anthropic:claude-sonnet-4-6\n\
         TAMA_MODEL_CRITIC=anthropic:claude-sonnet-4-6\n\
         \n\
         # Custom roles example:\n\
         # TAMA_MODEL_FAST=openai:gpt-4o-mini\n",
    )?;

    fs::write(root.join(".gitignore"), ".env\ntarget/\n")?;

    fs::write(
        root.join("tama.toml"),
        format!(
            "[project]\n\
             name = \"{name}\"\n\
             entrypoint = \"{agent_name}\"\n\
             \n\
             [models]\n\
             thinker = \"anthropic:claude-opus-4-6\"\n\
             worker  = \"anthropic:claude-sonnet-4-6\"\n\
             critic  = \"anthropic:claude-sonnet-4-6\"\n"
        ),
    )?;

    fs::write(
        root.join("agents").join(&agent_name).join("AGENT.md"),
        format!(
            "---\n\
             name: {agent_name}\n\
             description: \"\"\n\
             \n\
             tama:\n\
               version: 1.0.0\n\
               pattern: react\n\
               uses: []\n\
               model:\n\
                 role: thinker\n\
             ---\n\
             \n\
             You are a helpful assistant.\n\
             When done, call finish with your result.\n\
             \n\
             Task: {{{{input}}}}\n"
        ),
    )?;

    println!("created project '{name}'");
    println!();
    println!("  cd {name}");
    println!("  cp .env.example .env  # add your API key");
    println!("  tama run \"hello world\"");

    Ok(())
}
