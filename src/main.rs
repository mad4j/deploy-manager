use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;

mod cli;
mod core;
mod frontend;
mod grpc;

use cli::{Cli, Command};
use core::config::parse_deploy_file;
use core::executor::execute;
use frontend::{logger, progress::ProgressTracker};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { file, dry_run } => {
            let content = std::fs::read_to_string(&file)
                .with_context(|| format!("Cannot read deploy file: {}", file.display()))?;

            let deploy = parse_deploy_file(&content)
                .with_context(|| format!("Invalid deploy file: {}", file.display()))?;

            let tracker = ProgressTracker::new(deploy.actions.len());

            logger::init(cli.verbose, Some(tracker.log_handle()));

            info!(actions = deploy.actions.len(), "Actions to execute");

            if dry_run {
                info!("Dry-run mode: actions will be validated but not executed");
            }

            execute(&deploy, dry_run, &tracker).await?;

            info!("All actions completed successfully");
        }
        Command::Serve { listen } => {
            logger::init(cli.verbose, None);

            info!(address = %listen, "Starting gRPC server");
            grpc::serve(&listen).await?;
        }
    }

    Ok(())
}

