use anyhow::Result;

use crate::core::config::DeployActionConfig;
use crate::core::executor::ExecutionContext;

pub async fn run(host: &str, _cfg: &DeployActionConfig, _ctx: &ExecutionContext) -> Result<()> {
    anyhow::bail!("Remote deployment to '{}' is not yet supported", host);
}
