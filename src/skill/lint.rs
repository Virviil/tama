use anyhow::{bail, Result};
use std::path::Path;

use super::manifest::AgentPattern;
use super::parser::parse_agent;

/// Validate that an agent directory has all required prompt files.
pub fn lint_agent(dir: &Path) -> Result<()> {
    let agent_md = dir.join("AGENT.md");
    let agent = parse_agent(&agent_md)?;

    // System prompt always comes from AGENT.md body — no separate system.md.
    // Only patterns that read additional .md files for intermediate steps are listed here.
    let required: &[&str] = match &agent.pattern {
        AgentPattern::React => &[],
        AgentPattern::Scatter { .. } => &["reduce.md"],
        AgentPattern::Critic => &["draft.md", "critique.md", "refine.md"],
        AgentPattern::Parallel { .. } => &[],
        AgentPattern::Fsm { .. } => &[],
        AgentPattern::Debate { .. } => &[],
        AgentPattern::Reflexion => &["act.md", "reflect.md"],
        AgentPattern::Constitutional => &["critique.md", "revise.md"],
        AgentPattern::ChainOfVerification => &["verify.md", "check.md", "revise.md"],
        AgentPattern::PlanExecute => &["execute.md", "verify.md"],
        AgentPattern::BestOfN { .. } => &["judge.md"],
        AgentPattern::Human => &["resume.md"],
        AgentPattern::Oneshot => &[],
    };

    let mut missing = vec![];
    for file in required {
        if !dir.join(file).exists() {
            missing.push(*file);
        }
    }

    if !missing.is_empty() {
        bail!(
            "agent '{}': missing required files: {}",
            agent.name,
            missing.join(", ")
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn tmp() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tama_lint_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    fn write_agent_md(dir: &Path, pattern_yaml: &str) {
        let content = format!(
            "---\nname: test\ndescription: test agent\nversion: \"1.0.0\"\n{pattern_yaml}\ncall:\n  model:\n    role: thinker\n---\n\nSystem prompt here.\n"
        );
        write(dir, "AGENT.md", &content);
    }

    // ── react: no required files ─────────────────────────────────────────────

    #[test]
    fn lint_react_ok_with_no_extra_files() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: react");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    // ── scatter: requires reduce.md ──────────────────────────────────────────

    #[test]
    fn lint_scatter_ok_with_reduce() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: scatter\nworker: my-worker");
        write(&dir, "reduce.md", "Synthesise results.");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_scatter_missing_reduce_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: scatter\nworker: my-worker");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("reduce.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── parallel / fsm / debate: no required files ───────────────────────────

    #[test]
    fn lint_parallel_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: parallel\nworkers: [a, b]");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_fsm_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: fsm\ninitial: start\nstates:\n  start:");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_debate_ok() {
        let dir = tmp();
        write_agent_md(
            &dir,
            "pattern: debate\nagents: [pro, con]\nrounds: 1\njudge: judge",
        );
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    // ── critic: requires draft.md, critique.md, refine.md ────────────────────

    #[test]
    fn lint_critic_all_files_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: critic");
        write(&dir, "draft.md", "Draft system prompt.");
        write(&dir, "critique.md", "Critique system prompt.");
        write(&dir, "refine.md", "Refine system prompt.");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_critic_missing_all_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: critic");
        let err = lint_agent(&dir).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("draft.md"));
        assert!(msg.contains("critique.md"));
        assert!(msg.contains("refine.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── reflexion: requires act.md + reflect.md ──────────────────────────────

    #[test]
    fn lint_reflexion_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: reflexion\nmax_iter: 3");
        write(&dir, "act.md", "Act prompt.");
        write(&dir, "reflect.md", "Reflect prompt.");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_reflexion_missing_act_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: reflexion\nmax_iter: 3");
        write(&dir, "reflect.md", "Reflect prompt.");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("act.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_reflexion_missing_reflect_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: reflexion\nmax_iter: 3");
        write(&dir, "act.md", "Act prompt.");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("reflect.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── constitutional: requires critique.md, revise.md ──────────────────────

    #[test]
    fn lint_constitutional_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: constitutional");
        write(&dir, "critique.md", ".");
        write(&dir, "revise.md", ".");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_constitutional_missing_files_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: constitutional");
        write(&dir, "critique.md", ".");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("revise.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── chain-of-verification: requires verify.md, check.md, revise.md ───────

    #[test]
    fn lint_cov_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: chain-of-verification");
        write(&dir, "verify.md", ".");
        write(&dir, "check.md", ".");
        write(&dir, "revise.md", ".");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_cov_missing_check_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: chain-of-verification");
        write(&dir, "verify.md", ".");
        write(&dir, "revise.md", ".");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("check.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── plan-execute: requires execute.md, verify.md ─────────────────────────

    #[test]
    fn lint_plan_execute_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: plan-execute");
        write(&dir, "execute.md", ".");
        write(&dir, "verify.md", ".");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_plan_execute_missing_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: plan-execute");
        write(&dir, "verify.md", ".");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("execute.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── best-of-n: requires judge.md ─────────────────────────────────────────

    #[test]
    fn lint_best_of_n_ok() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: best-of-n\nn: 3");
        write(&dir, "judge.md", ".");
        assert!(lint_agent(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn lint_best_of_n_missing_judge_fails() {
        let dir = tmp();
        write_agent_md(&dir, "pattern: best-of-n\nn: 3");
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("judge.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    // ── error message contains agent name ────────────────────────────────────

    #[test]
    fn lint_error_includes_agent_name() {
        let dir = tmp();
        let content = "---\nname: my-critic\ndescription: test\nversion: \"1.0.0\"\npattern: critic\ncall:\n  model:\n    role: thinker\n---\n\nbody\n";
        fs::write(dir.join("AGENT.md"), content).unwrap();
        let err = lint_agent(&dir).unwrap_err();
        assert!(err.to_string().contains("my-critic"));
        let _ = fs::remove_dir_all(&dir);
    }
}
