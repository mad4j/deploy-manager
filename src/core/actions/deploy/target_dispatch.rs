use anyhow::Result;

use super::local;
use super::remote;
use crate::core::config::{DeployActionConfig, DeployTarget};
use crate::core::executor::ExecutionContext;

pub async fn run(cfg: &DeployActionConfig, ctx: &ExecutionContext) -> Result<()> {
    match &cfg.target {
        DeployTarget::Local => local::run(cfg, ctx).await,
        DeployTarget::Remote(host) => remote::run(host, cfg, ctx).await,
    }
}
