use anyhow::Result;
use tracing::{error, info, warn};

use crate::core::config::{LogActionConfig, LogLevel};
use crate::core::executor::ExecutionContext;

/// Execute a `log` action.
pub async fn run(cfg: &LogActionConfig, ctx: &ExecutionContext) -> Result<()> {
    if ctx.dry_run {
        info!("[dry-run] Would log (level={:?}): {}", cfg.level, cfg.message);
        return Ok(());
    }

    match cfg.level {
        LogLevel::Info => info!(message = %cfg.message, "Log action"),
        LogLevel::Warn => warn!(message = %cfg.message, "Log action"),
        LogLevel::Error => error!(message = %cfg.message, "Log action"),
    }

    Ok(())
}
