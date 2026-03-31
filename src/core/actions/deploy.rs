use anyhow::{Context, Result};
use tracing::info;

use crate::core::config::{DeployActionConfig, DeployTarget, ExecType};
use crate::core::executor::ExecutionContext;

/// Execute a `deploy` action.
pub async fn run(cfg: &DeployActionConfig, ctx: &ExecutionContext) -> Result<()> {
    info!(
        app = %cfg.app,
        binary = %cfg.binary.display(),
        target = ?cfg.target,
        exec_type = ?cfg.exec_type,
        "Deploy action"
    );

    // Validate that the binary exists (unless dry-run).
    if !ctx.dry_run {
        anyhow::ensure!(
            cfg.binary.exists(),
            "Binary not found: {}",
            cfg.binary.display()
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
    match &cfg.exec_type {
        ExecType::Executable | ExecType::Script => {
            if let Some(dest) = &cfg.destination {
                if ctx.dry_run {
                    info!(
                        "[dry-run] Would copy {} → {}",
                        cfg.binary.display(),
                        dest.display()
                    );
                    return Ok(());
                }

                // Ensure destination directory exists.
                if dest.is_dir() || dest.to_string_lossy().ends_with('/') {
                    std::fs::create_dir_all(dest).with_context(|| {
                        format!("Failed to create destination directory: {}", dest.display())
                    })?;
                    let file_name = cfg
                        .binary
                        .file_name()
                        .context("Binary path has no file name")?;
                    let target_path = dest.join(file_name);
                    std::fs::copy(&cfg.binary, &target_path).with_context(|| {
                        format!(
                            "Failed to copy {} to {}",
                            cfg.binary.display(),
                            target_path.display()
                        )
                    })?;
                    info!(
                        app = %cfg.app,
                        dest = %target_path.display(),
                        "Binary deployed"
                    );
                } else {
                    std::fs::create_dir_all(dest.parent().unwrap_or(dest)).with_context(|| {
                        format!("Failed to create parent directory for: {}", dest.display())
                    })?;
                    std::fs::copy(&cfg.binary, dest).with_context(|| {
                        format!(
                            "Failed to copy {} to {}",
                            cfg.binary.display(),
                            dest.display()
                        )
                    })?;
                    info!(
                        app = %cfg.app,
                        dest = %dest.display(),
                        "Binary deployed"
                    );
                }
            } else {
                info!(
                    app = %cfg.app,
                    binary = %cfg.binary.display(),
                    "No destination specified; binary verified in-place"
                );
            }
        }
        ExecType::Service => {
            info!(
                app = %cfg.app,
                "Service deployment noted (platform-specific install not performed)"
            );
        }
        ExecType::Container => {
            info!(
                app = %cfg.app,
                "Container deployment noted (container runtime not invoked)"
            );
        }
    }

    Ok(())
}
