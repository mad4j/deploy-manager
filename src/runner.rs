use anyhow::{Context, Result};
use tracing::info;

use crate::core::config::parse_deploy_file;
use crate::core::executor::execute;
use crate::frontend::progress::ProgressTracker;

/// Parse and execute a deploy file path. The caller is responsible for
/// initialising the logging/tracing subscriber before calling this function.
pub async fn run_deploy(file_path: &str, dry_run: bool) -> Result<()> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Cannot read deploy file: {file_path}"))?;
    run_deploy_from_content(&content, dry_run).await.map(|_| ())
}

/// Parse and execute a deploy plan from raw YAML content. Returns a report
/// listing the name of each action that was processed.
///
/// The caller is responsible for initialising the logging/tracing subscriber
/// and for setting any required environment variables before calling this
/// function (they are used during YAML `${VAR}` interpolation).
pub async fn run_deploy_from_content(yaml_content: &str, dry_run: bool) -> Result<Vec<String>> {
    let deploy = parse_deploy_file(yaml_content)
        .context("Invalid deploy file content")?;

    let tracker = ProgressTracker::new(deploy.actions.len());

    info!(actions = deploy.actions.len(), "Actions to execute");

    if dry_run {
        info!("Dry-run mode: actions will be validated but not executed");
    }

    execute(&deploy, dry_run, &tracker).await?;

    let report: Vec<String> = deploy
        .actions
        .iter()
        .map(|a| format!("action '{}' completed", a.name()))
        .collect();
    Ok(report)
}
