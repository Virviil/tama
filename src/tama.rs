use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use tama::cmd;
use tama::skill::lint::lint_agent;
use tama::skill::parser::{parse_agent, parse_skill};

#[derive(Parser)]
#[command(
    name = "tama",
    about = "Markdown-native agent framework — write agents in .md, run in Docker"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum RunsCommand {
    /// Show span tree for a run
    Show {
        trace_id: String,
        /// Include full LLM prompts and responses
        #[arg(long)]
        llm: bool,
    },
    /// Re-run with the same task
    Retry { trace_id: String },
    /// Open browser dashboard (runs list + span graph)
    Serve,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new tama project
    ///
    /// Creates a project directory with tama.toml, .env.example,
    /// and a starter react agent. Name must be kebab-case.
    ///
    /// Example:
    ///   tama init my-project
    Init { name: String },

    /// Add an agent or skill scaffold to the current project
    ///
    /// AGENT PATTERNS:
    ///   react                 Tool-use loop until finish. Core workhorse.
    ///   critic                draft → critique → refine.
    ///   orchestrator          LLM decomposes → parallel workers → LLM merges.
    ///   parallel              Fixed list of agents run simultaneously.
    ///   fsm                   Finite state machine: states + word-based routing.
    ///   scatter               map → parallel workers → reduce. Worker declared in YAML.
    ///   debate                position-a → position-b → judge synthesizes.
    ///   reflexion             act → evaluate → reflect → loop.
    ///   constitutional        generate → check principles → revise.
    ///   chain-of-verification generate → verify claims → revise.
    ///   plan-execute          plan → execute → verify → loop.
    ///   best-of-n             scatter N variants → judge picks best.
    ///
    /// SKILL:
    ///   skill                 Create a SKILL.md (Anthropic-compatible tool definition).
    ///
    /// Names must be kebab-case. Examples:
    ///   tama add react my-agent
    ///   tama add critic essay-critic
    ///   tama add skill search-web
    Add { pattern: String, name: String },

    /// Collect skill dependencies and build Docker image
    ///
    /// Reads tama.toml, scans all SKILL.md files for tama.depends.apt/uv/bins,
    /// and builds a distroless Docker image with all dependencies baked in.
    ///
    /// Requires Docker to be running.
    Brew,

    /// Validate an agent directory (checks required prompt files exist)
    ///
    /// Example:
    ///   tama lint agents/my-agent
    Lint { path: PathBuf },

    /// List available models from configured providers
    Models,

    /// Run a task through the agent graph, recording to .tama/runs.duckdb
    ///
    /// Example:
    ///   tama run "summarise the README"
    ///   tama run --agent my-agent "write a blog post about Rust"
    Run {
        task: String,
        /// Override the entrypoint agent
        #[arg(long)]
        agent: Option<String>,
        /// Enable interactive step-through debugger (pause before/after each LLM call)
        #[arg(long)]
        debug: bool,
        /// Only pause at this agent (may be repeated; default: pause at all agents)
        #[arg(long = "break", value_name = "AGENT")]
        breakpoints: Vec<String>,
    },

    /// Inspect past runs recorded in .tama/runs.duckdb
    ///
    /// Examples:
    ///   tama runs                        list recent runs
    ///   tama runs show <id>              span tree with timing
    ///   tama runs show <id> --llm        include full prompts and responses
    ///   tama runs retry <id>             re-run with same task
    Runs {
        #[command(subcommand)]
        cmd: Option<RunsCommand>,
    },

    /// Open an interactive browser preview of the agent graph.
    View,

    /// Parse and print a SKILL.md (debug)
    #[command(hide = true)]
    Skill { path: PathBuf },

    /// Parse and print an AGENT.md (debug)
    #[command(hide = true)]
    Agent { path: PathBuf },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => cmd::init::run(&name)?,
        Commands::Add { pattern, name } => cmd::add::run(&pattern, &name)?,
        Commands::Brew => cmd::brew::run()?,
        Commands::Lint { path } => {
            lint_agent(&path)?;
            println!("ok");
        }
        Commands::Run {
            task,
            agent,
            debug,
            breakpoints,
        } => cmd::run::run(&task, agent.as_deref(), debug, breakpoints).await?,
        Commands::Runs { cmd } => match cmd {
            None => cmd::runs::list()?,
            Some(RunsCommand::Show { trace_id, llm }) => cmd::runs::show(&trace_id, llm)?,
            Some(RunsCommand::Retry { trace_id }) => cmd::runs::retry(&trace_id).await?,
            Some(RunsCommand::Serve) => cmd::runs::serve().await?,
        },
        Commands::Models => cmd::models::run().await?,
        Commands::View => cmd::view::run().await?,
        Commands::Skill { path } => println!("{:#?}", parse_skill(&path)?),
        Commands::Agent { path } => println!("{:#?}", parse_agent(&path)?),
    }

    Ok(())
}
