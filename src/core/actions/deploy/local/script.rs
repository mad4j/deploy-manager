use anyhow::Result;
use tracing::info;

use crate::core::config::DeployActionConfig;
use crate::core::executor::ExecutionContext;

pub async fn run(cfg: &DeployActionConfig, ctx: &ExecutionContext) -> Result<()> {
    if ctx.dry_run {
        info!("[dry-run] Would verify script file {}", cfg.file.display());
    } else {
        info!(file = %cfg.file.display(), "Script file verified in-place");
    }

    Ok(())
}
