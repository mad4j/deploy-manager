use anyhow::{Context, Result};
use tracing::info;

use crate::core::config::parse_deploy_file;
use crate::core::executor::execute;
use crate::frontend::progress::ProgressTracker;

/// Parse and execute a deploy file. The caller is responsible for initialising
/// the logging/tracing subscriber before calling this function.
pub async fn run_deploy(file_path: &str, dry_run: bool) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Cannot read deploy file: {file_path}"))?;

    let deploy = parse_deploy_file(&content)
        .with_context(|| format!("Invalid deploy file: {file_path}"))?;

    let tracker = ProgressTracker::new(deploy.actions.len());

    info!(actions = deploy.actions.len(), "Actions to execute");

    if dry_run {
        info!("Dry-run mode: actions will be validated but not executed");
    }

    execute(&deploy, dry_run, &tracker).await?;
    Ok(())
}
