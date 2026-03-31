use anyhow::{Context, Result};
use tracing::info;

use crate::core::config::{FilesystemActionConfig, FsOperation};
use crate::core::executor::ExecutionContext;

/// Execute a `filesystem` action.
pub async fn run(cfg: &FilesystemActionConfig, ctx: &ExecutionContext) -> Result<()> {
    info!(operation = ?cfg.operation, "Filesystem action");

    if ctx.dry_run {
        info!("[dry-run] Would perform filesystem operation: {:?}", cfg.operation);
        return Ok(());
    }

    match &cfg.operation {
        FsOperation::Copy => {
            let src = cfg
                .source
                .as_ref()
                .context("'source' is required for the copy operation")?;
            let dst = cfg
                .destination
                .as_ref()
                .context("'destination' is required for the copy operation")?;

            if src.is_dir() {
                let mut copy_opts = fs_extra::dir::CopyOptions::new();
                copy_opts.overwrite = cfg.overwrite;
                copy_opts.copy_inside = true;
                fs_extra::dir::copy(src, dst, &copy_opts).with_context(|| {
                    format!("Failed to copy dir {} → {}", src.display(), dst.display())
                })?;
            } else {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file_opts = fs_extra::file::CopyOptions::new();
                file_opts.overwrite = cfg.overwrite;
                fs_extra::file::copy(src, dst, &file_opts).with_context(|| {
                    format!("Failed to copy {} → {}", src.display(), dst.display())
                })?;
            }
            info!(src = %src.display(), dst = %dst.display(), "Copy complete");
        }

        FsOperation::Move => {
            let src = cfg
                .source
                .as_ref()
                .context("'source' is required for the move operation")?;
            let dst = cfg
                .destination
                .as_ref()
                .context("'destination' is required for the move operation")?;

            if src.is_dir() {
                let mut move_opts = fs_extra::dir::CopyOptions::new();
                move_opts.overwrite = cfg.overwrite;
                move_opts.copy_inside = true;
                fs_extra::dir::move_dir(src, dst, &move_opts).with_context(|| {
                    format!("Failed to move dir {} → {}", src.display(), dst.display())
                })?;
            } else {
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file_opts = fs_extra::file::CopyOptions::new();
                file_opts.overwrite = cfg.overwrite;
                fs_extra::file::move_file(src, dst, &file_opts).with_context(|| {
                    format!("Failed to move {} → {}", src.display(), dst.display())
                })?;
            }
            info!(src = %src.display(), dst = %dst.display(), "Move complete");
        }

        FsOperation::Delete => {
            let src = cfg
                .source
                .as_ref()
                .context("'source' is required for the delete operation")?;

            if src.is_dir() {
                std::fs::remove_dir_all(src).with_context(|| {
                    format!("Failed to delete directory: {}", src.display())
                })?;
            } else {
                std::fs::remove_file(src)
                    .with_context(|| format!("Failed to delete file: {}", src.display()))?;
            }
            info!(path = %src.display(), "Delete complete");
        }

        FsOperation::CreateDir => {
            let dst = cfg
                .destination
                .as_ref()
                .context("'destination' is required for the create_dir operation")?;
            std::fs::create_dir_all(dst).with_context(|| {
                format!("Failed to create directory: {}", dst.display())
            })?;
            info!(path = %dst.display(), "Directory created");
        }

        FsOperation::CreateFile => {
            let dst = cfg
                .destination
                .as_ref()
                .context("'destination' is required for the create_file operation")?;
            if let Some(parent) = dst.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if dst.exists() && !cfg.overwrite {
                anyhow::bail!("File already exists (set overwrite: true to replace): {}", dst.display());
            }
            std::fs::File::create(dst)
                .with_context(|| format!("Failed to create file: {}", dst.display()))?;
            info!(path = %dst.display(), "File created");
        }
    }

    Ok(())
}
