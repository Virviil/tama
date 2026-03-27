use anyhow::Result;
use clap::Parser;

use tama::runtime::tracer::OtelTracer;

#[derive(Parser)]
#[command(name = "tamad", about = "tama runtime — executes agents via LLM")]
struct Cli {
    /// Task to run
    task: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    tama::runtime::run(&cli.task, Box::new(OtelTracer::new()), None).await
}
