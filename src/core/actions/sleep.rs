use anyhow::Result;
use std::time::Duration;
use tracing::info;

use crate::core::config::SleepActionConfig;
use crate::core::executor::ExecutionContext;

/// Execute a `sleep` action.
pub async fn run(cfg: &SleepActionConfig, ctx: &ExecutionContext) -> Result<()> {
    let total_millis = cfg.secs.saturating_mul(1_000).saturating_add(cfg.millis);
    let duration = Duration::from_millis(total_millis);

    info!(
        secs = cfg.secs,
        millis = cfg.millis,
        "Sleep action"
    );

    if ctx.dry_run {
        info!("[dry-run] Would sleep for {}ms", total_millis);
        return Ok(());
    }

    tokio::time::sleep(duration).await;

    info!(total_millis, "Sleep complete");
    Ok(())
}
