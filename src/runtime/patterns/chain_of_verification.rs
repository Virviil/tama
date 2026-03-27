use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use super::AgentOutput;
use super::oneshot;
use super::step::Step;
use crate::runtime::llm::LlmClient;
use crate::runtime::model_registry::ModelRegistry;
use crate::runtime::tracer::{TraceCtx, Tracer};

/// Chain-of-Verification: generate → extract claims → verify each → revise.
///
/// Files:
///   AGENT.md body  → initial answer generation system prompt
///   verify.md      → claim extraction system prompt (output: one claim per line)
///   check.md       → individual claim checking system prompt
///   revise.md      → final revision system prompt
///
/// Each step file supports optional `pattern: react` frontmatter.
pub async fn run(
    agent_dir: &Path,
    body: &str,
    registry: &Arc<ModelRegistry>,
    client: &LlmClient,
    input: &str,
    tracer: &mut dyn Tracer,
    ctx: &TraceCtx,
    crumb: &str,
) -> Result<AgentOutput> {
    eprintln!("  → cov: generate");
    let initial = oneshot::call(body, input, "generate", client, tracer, ctx, crumb).await?;

    let verify_step = Step::from_file(&agent_dir.join("verify.md"))?;
    eprintln!("  → cov: extract claims");
    let claims_text = verify_step
        .run(
            &format!("Answer to verify:\n{initial}"),
            "extract_claims",
            registry,
            client,
            tracer,
            ctx,
            crumb,
        )
        .await?
        .value;

    let claims: Vec<&str> = claims_text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    eprintln!("  → cov: {} claims to check", claims.len());

    let check_step = Step::from_file(&agent_dir.join("check.md"))?;
    let mut verifications = Vec::new();
    for (i, claim) in claims.iter().enumerate() {
        eprintln!("  → cov: checking claim {}/{}", i + 1, claims.len());
        let check_result = check_step
            .run(
                &format!("Claim: {claim}\n\nContext: {input}"),
                &format!("check_{}", i + 1),
                registry,
                client,
                tracer,
                ctx,
                crumb,
            )
            .await?
            .value;
        verifications.push(format!("Claim: {claim}\nVerification: {check_result}"));
    }

    let revise_step = Step::from_file(&agent_dir.join("revise.md"))?;
    eprintln!("  → cov: revise");
    let revise_input = format!(
        "Original question: {input}\n\nInitial answer:\n{initial}\n\nVerification results:\n{}",
        verifications.join("\n\n")
    );
    let revised = revise_step.run(&revise_input, "revise", registry, client, tracer, ctx, crumb).await?.value;

    Ok(AgentOutput {
        key: "done".to_string(),
        value: revised,
        span_id: String::new(),
    })
}
