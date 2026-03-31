use anyhow::Result;
use tracing::info;

use crate::core::config::DeployActionConfig;
use crate::core::executor::ExecutionContext;

pub async fn run(_cfg: &DeployActionConfig, _ctx: &ExecutionContext) -> Result<()> {
    info!("Service deployment noted (platform-specific install not performed)");
    Ok(())
}
