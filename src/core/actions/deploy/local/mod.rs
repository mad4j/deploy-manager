use anyhow::Result;

use crate::core::config::{DeployActionConfig, ExecType};
use crate::core::executor::ExecutionContext;

mod container;
mod executable;
mod script;
mod service;

pub async fn run(cfg: &DeployActionConfig, ctx: &ExecutionContext) -> Result<()> {
    match &cfg.r#type {
        ExecType::Executable => executable::run(cfg, ctx).await,
        ExecType::Service => service::run(cfg, ctx).await,
        ExecType::Container => container::run(cfg, ctx).await,
        ExecType::Script => script::run(cfg, ctx).await,
    }
}
