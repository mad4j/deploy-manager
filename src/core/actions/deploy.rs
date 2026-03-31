use anyhow::Result;
use tracing::info;

use crate::core::config::DeployActionConfig;
use crate::core::executor::ExecutionContext;

mod local;
mod remote;
mod target_dispatch;

/// Execute a `deploy` action.
pub async fn run(cfg: &DeployActionConfig, ctx: &ExecutionContext) -> Result<()> {
    info!(
        file = %cfg.file.display(),
        target = ?cfg.target,
        type = ?cfg.r#type,
        "Deploy action"
    );

    // Validate that the file exists (unless dry-run).
    if !ctx.dry_run {
        anyhow::ensure!(
            cfg.file.exists(),
            "File not found: {}",
            cfg.file.display()
        );
    }

    target_dispatch::run(cfg, ctx).await
}
