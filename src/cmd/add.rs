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

pub fn run(pattern: &str, name: &str) -> Result<()> {
    if !Path::new("tama.toml").exists() {
        bail!("no tama.toml found — run this command inside a tama project (or run 'tama init <name>' first)");
    }
    validate_kebab_case(name, "agent name")?;

    if pattern == "skill" {
        return add_skill(name);
    }

    let dir = Path::new("agents").join(name);
    if dir.exists() {
        bail!("agent '{}' already exists at {}", name, dir.display());
    }
    fs::create_dir_all(&dir)?;

    match pattern {
        "react" => scaffold_react(&dir, name)?,
        "critic" => scaffold_critic(&dir, name)?,
        "parallel" => scaffold_parallel(&dir, name)?,
        "fsm" => scaffold_fsm(&dir, name)?,
        "scatter" => scaffold_scatter(&dir, name)?,
        "debate" => scaffold_debate(&dir, name)?,
        "reflexion" => scaffold_reflexion(&dir, name)?,
        "constitutional" => scaffold_constitutional(&dir, name)?,
        "chain-of-verification" => scaffold_chain_of_verification(&dir, name)?,
        "plan-execute" => scaffold_plan_execute(&dir, name)?,
        "best-of-n" => scaffold_best_of_n(&dir, name)?,
        "human" => scaffold_human(&dir, name)?,
        "oneshot" => scaffold_oneshot(&dir, name)?,
        other => bail!(
            "unknown pattern '{}'. Available:\n  \
             skill, react, critic, parallel, fsm, scatter, human,\n  \
             debate, reflexion, constitutional, chain-of-verification, plan-execute, best-of-n,\n  \
             oneshot",
            other
        ),
    }

    println!("created agent '{}'", name);
    print_files(&dir);
    Ok(())
}

fn add_skill(name: &str) -> Result<()> {
    let dir = Path::new("skills").join(name);
    if dir.exists() {
        bail!("skill '{}' already exists at {}", name, dir.display());
    }
    fs::create_dir_all(&dir)?;

    fs::write(
        dir.join("SKILL.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this skill does and when to use it.\n\
             metadata:\n\
               tama.depends.apt: ~      # e.g. poppler-utils\n\
               tama.depends.bins: ~     # e.g. pdftotext\n\
               tama.depends.uv: ~       # e.g. requests>=2.31.0\n\
             ---\n\
             \n\
             TODO: describe how to use this skill.\n\
             \n\
             Use bash to run `command \"$INPUT\"` and return the result.\n"
        ),
    )?;

    println!("created skill '{}'", name);
    print_files(&dir);
    Ok(())
}

fn print_files(dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            println!("  {}", entry.path().display());
        }
    }
}

// ── Scaffolds ─────────────────────────────────────────────────────────────────

fn scaffold_react(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this agent does.\n\
             version: 1.0.0\n\
             pattern: react\n\
             call:\n\
               model:\n\
                 role: thinker\n\
               uses: []  # add skill names, e.g. [search-web]\n\
             ---\n\
             \n\
             You are a helpful assistant.\n\
             Use the available tools to complete the task.\n\
             When done, call finish with your result.\n"
        ),
    )?;
    Ok(())
}

fn scaffold_critic(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this agent reviews and improves.\n\
             version: 1.0.0\n\
             pattern: critic\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("draft.md"),
        "Write a thorough response to the task given to you.\n",
    )?;
    fs::write(
        dir.join("critique.md"),
        "Review the response given to you. Identify specific weaknesses:\n\
         missing information, incorrect claims, poor structure.\n",
    )?;
    fs::write(
        dir.join("refine.md"),
        "Rewrite the response fixing all issues from the critique.\n\
         You will receive the original task and the critique as context.\n\
         Produce only the final improved response.\n",
    )?;
    Ok(())
}

fn scaffold_parallel(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this parallel agent does.\n\
             version: 1.0.0\n\
             pattern: parallel\n\
             workers: []  # add agent names, e.g. [agent-a, agent-b]\n\
             ---\n"
        ),
    )?;
    Ok(())
}

fn scaffold_fsm(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe the workflow this FSM implements.\n\
             version: 1.0.0\n\
             pattern: fsm\n\
             initial: step-a\n\
             states:\n\
               step-a:              # unconditional → always goes to step-b\n\
                 - done: ~          # conditional → finish word 'done' = terminal\n\
                 - next: step-b     # conditional → finish word 'next' = step-b\n\
               step-b:              # terminal (no transitions)\n\
             ---\n"
        ),
    )?;
    Ok(())
}

fn scaffold_scatter(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this scatter agent fans out.\n\
             version: 1.0.0\n\
             pattern: scatter\n\
             worker: my-worker  # replace with the worker agent name\n\
             call:\n\
               model:\n\
                 role: thinker\n\
               uses: []  # skills available to the map phase\n\
             ---\n\
             \n\
             You are a planning agent. Analyse the task and determine the list of items to process.\n\
             When ready, call finish(key=\"parallel\", value='[\"item1\",\"item2\",...]') with a JSON array.\n\
             If no fan-out is needed, call finish(key=\"done\", value=\"...\") directly.\n"
        ),
    )?;
    fs::write(
        dir.join("reduce.md"),
        "You are a synthesis agent.\n\
         You will receive the original task and the results from all parallel workers.\n\
         Synthesize the results into a single coherent final answer.\n\
         When done, call finish(key=\"done\", value=\"...\") with your synthesized result.\n",
    )?;
    Ok(())
}

fn scaffold_debate(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe the debate topic and goal.\n\
             version: 1.0.0\n\
             pattern: debate\n\
             agents:\n\
               - proponent\n\
               - skeptic\n\
             rounds: 2\n\
             judge: synthesis\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("position-a.md"),
        "You are arguing in favor of the topic given to you.\n\
         State your opening position clearly. Lead with your strongest argument.\n",
    )?;
    fs::write(
        dir.join("position-b.md"),
        "You are arguing against the topic given to you.\n\
         State your opening position clearly. Lead with your strongest argument.\n",
    )?;
    fs::write(
        dir.join("judge.md"),
        "You are an impartial judge.\n\
         You will receive the debate topic and the full debate transcript.\n\
         Give your verdict — not a winner, but the most defensible position.\n",
    )?;
    Ok(())
}

fn scaffold_reflexion(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe the task this agent iteratively attempts.\n\
             version: 1.0.0\n\
             pattern: reflexion\n\
             max_iter: 4\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("act.md"),
        "---\n\
         pattern: react\n\
         model:\n\
           role: thinker\n\
         uses: []  # tools needed to attempt the task, e.g. [run-tests]\n\
         ---\n\n\
         Complete the task given to you.\n\
         If previous attempts failed, you will receive their lessons — apply them.\n",
    )?;
    fs::write(
        dir.join("evaluate.md"),
        "Evaluate whether the attempt successfully solved the task.\n\
         You will receive the task and the attempt as context.\n\
         Call finish with 'continue' if it failed, 'stop' if it succeeded.\n",
    )?;
    fs::write(
        dir.join("reflect.md"),
        "The attempt failed. You will receive the task and the failed attempt.\n\
         Write 3-5 bullet points: what failed, root cause, what to do differently.\n",
    )?;
    Ok(())
}

fn scaffold_constitutional(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what content this agent generates and refines.\n\
             version: 1.0.0\n\
             pattern: constitutional\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("generate.md"),
        "Write a thorough response to the following request.\n\
         This is a first draft — do not self-censor excessively.\n\
         \n\
         Request: {{input}}\n",
    )?;
    fs::write(
        dir.join("principles.md"),
        "Be factually accurate: every claim must be verifiable or clearly labeled as opinion.\n\
         Be clear and concise: no jargon without explanation, no filler sentences.\n\
         Be balanced: present multiple perspectives on genuinely contested topics.\n\
         Be actionable: end with specific next steps or recommendations where relevant.\n",
    )?;
    fs::write(
        dir.join("revise.md"),
        "Rewrite the draft to fix all principle violations.\n\
         \n\
         Original request: {{input}}\n\
         Draft: {{draft}}\n\
         Critiques: {{critiques}}\n\
         \n\
         You will receive the original request, the draft, and the critiques as context.\n\
         Fix every flagged violation. Output only the final revised text.\n",
    )?;
    Ok(())
}

fn scaffold_chain_of_verification(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what claims this agent generates and verifies.\n\
             version: 1.0.0\n\
             pattern: chain-of-verification\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("generate.md"),
        "Answer the question given to you thoroughly.\n",
    )?;
    fs::write(
        dir.join("verify.md"),
        "List verification questions for every factual claim in the answer given to you.\n\
         Then answer each question independently.\n",
    )?;
    fs::write(
        dir.join("revise.md"),
        "Rewrite the original answer incorporating corrections from verification.\n\
         You will receive the original answer and the verification results as context.\n",
    )?;
    Ok(())
}

fn scaffold_plan_execute(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe the task this agent plans and executes.\n\
             version: 1.0.0\n\
             pattern: plan-execute\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("plan.md"),
        "Create a step-by-step plan to accomplish the task given to you.\n\
         Be specific and concrete. Number each step.\n",
    )?;
    fs::write(
        dir.join("execute.md"),
        "Execute the plan given to you step by step.\n\
         You will receive both the task and the plan as context.\n",
    )?;
    fs::write(
        dir.join("verify.md"),
        "Verify whether the execution successfully completed the task.\n\
         You will receive the task and the execution result as context.\n\
         Call finish with 'done' if successful, 'retry' if it needs another attempt.\n",
    )?;
    Ok(())
}

fn scaffold_human(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this agent does and what it asks the human.\n\
             version: 1.0.0\n\
             pattern: human\n\
             call:\n\
               model:\n\
                 role: thinker\n\
               uses: []  # tools for phase 1, e.g. [search-web]\n\
             ---\n\
             \n\
             Complete the preparation task, then ask the human a specific question.\n\
             When ready, call finish with the channel id as key and your question as value.\n\
             Example: finish(\"review\", \"Here is the draft: ...\\n\\nPlease review and approve.\")\n"
        ),
    )?;
    fs::write(
        dir.join("resume.md"),
        "The human has responded. Their response is your input.\n\
         Process the response and complete the task.\n\
         When done, call finish with your result.\n",
    )?;
    Ok(())
}

fn scaffold_oneshot(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this agent does.\n\
             version: 1.0.0\n\
             pattern: oneshot\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n\
             \n\
             You are a helpful assistant. Answer concisely and directly.\n"
        ),
    )?;
    Ok(())
}

fn scaffold_best_of_n(dir: &Path, name: &str) -> Result<()> {
    fs::write(
        dir.join("AGENT.md"),
        format!(
            "---\n\
             name: {name}\n\
             description: TODO describe what this agent generates and selects best of.\n\
             version: 1.0.0\n\
             pattern: best-of-n\n\
             n: 3  # number of variants to generate\n\
             call:\n\
               model:\n\
                 role: thinker\n\
             ---\n"
        ),
    )?;
    fs::write(
        dir.join("prompt.md"),
        "Complete the task given to you.\n\
         Be creative and thorough.\n",
    )?;
    fs::write(
        dir.join("judge.md"),
        "Select the best candidate response to the task.\n\
         You will receive the task and all candidate responses as context.\n\
         Explain your choice briefly, then output the selected response.\n",
    )?;
    Ok(())
}
