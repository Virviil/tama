use anyhow::{Context, Result};

use tama::runtime::tracer::OtelTracer;

#[tokio::main]
async fn main() -> Result<()> {
    let task = std::env::args()
        .nth(1)
        .context("usage: tamad <task>")?;
    tama::runtime::run(&task, Box::new(OtelTracer::new()), None).await
}
