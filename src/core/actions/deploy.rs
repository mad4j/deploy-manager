use anyhow::Result;
use tracing::info;

use crate::core::config::{DeployActionConfig, DeployTarget, ExecType};
use crate::core::executor::ExecutionContext;

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

    match &cfg.target {
        DeployTarget::Local => deploy_local(cfg, ctx).await,
        DeployTarget::Remote(host) => {
            anyhow::bail!("Remote deployment to '{}' is not yet supported", host);
        }
    }
}

async fn deploy_local(cfg: &DeployActionConfig, ctx: &ExecutionContext) -> Result<()> {
    match &cfg.r#type {
        ExecType::Executable | ExecType::Script => {
            if ctx.dry_run {
                info!("[dry-run] Would verify file {}", cfg.file.display());
            } else {
                info!(file = %cfg.file.display(), "File verified in-place");
            }
        }
        ExecType::Service => {
            info!(
                "Service deployment noted (platform-specific install not performed)"
            );
        }
        ExecType::Container => {
            info!(
                "Container deployment noted (container runtime not invoked)"
            );
        }
    }

    Ok(())
}
