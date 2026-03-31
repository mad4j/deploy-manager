use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;

mod cli;
mod core;
mod frontend;

use cli::Cli;
use core::config::parse_deploy_file;
use core::executor::execute;
use frontend::{logger, progress::ProgressTracker};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let content = std::fs::read_to_string(&cli.file)
        .with_context(|| format!("Cannot read deploy file: {}", cli.file.display()))?;

    let deploy = parse_deploy_file(&content)
        .with_context(|| format!("Invalid deploy file: {}", cli.file.display()))?;

    let tracker = ProgressTracker::new(deploy.actions.len());

    logger::init(cli.verbose, Some(tracker.log_handle()));

    info!(actions = deploy.actions.len(), "Actions to execute");

    if cli.dry_run {
        info!("Dry-run mode: actions will be validated but not executed");
    }

    execute(&deploy, cli.dry_run, &tracker).await?;

    info!("All actions completed successfully");
    Ok(())
}
