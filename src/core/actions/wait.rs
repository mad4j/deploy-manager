use anyhow::Result;
use std::time::Duration;
use tracing::{info, warn};

use crate::core::config::WaitActionConfig;
use crate::core::executor::{ActionState, ExecutionContext};

/// Execute a `wait` action.
///
/// Polls the shared state map until every action listed in `depends_on` has
/// reached a terminal state (Success or Failed), or until the optional timeout
/// elapses.
pub async fn run(cfg: &WaitActionConfig, ctx: &ExecutionContext) -> Result<()> {
    info!(depends_on = ?cfg.depends_on, "Wait action");

    if cfg.depends_on.is_empty() {
        info!("No dependencies; wait action is a no-op");
        return Ok(());
    }

    if ctx.dry_run {
        info!("[dry-run] Would wait for: {:?}", cfg.depends_on);
        return Ok(());
    }

    let timeout = if cfg.timeout_secs > 0 {
        Some(Duration::from_secs(cfg.timeout_secs))
    } else {
        None
    };

    let poll_interval = Duration::from_millis(100);
    let start = std::time::Instant::now();

    loop {
        let pending: Vec<String> = cfg
            .depends_on
            .iter()
            .filter(|dep| {
                let map = ctx.states.lock().unwrap();
                match map.get(*dep) {
                    Some(ActionState::Success)
                    | Some(ActionState::Failed(_))
                    | Some(ActionState::Skipped) => false,
                    _ => true,
                }
            })
            .cloned()
            .collect();

        if pending.is_empty() {
            break;
        }

        if let Some(t) = timeout {
            if start.elapsed() >= t {
                anyhow::bail!(
                    "Wait action '{}' timed out after {} seconds waiting for: {:?}",
                    cfg.name,
                    cfg.timeout_secs,
                    pending
                );
            }
        }

        info!(waiting_for = ?pending, "Still waiting…");
        tokio::time::sleep(poll_interval).await;
    }

    // Report whether any dependency failed.
    let mut failed_deps: Vec<String> = Vec::new();
    {
        let map = ctx.states.lock().unwrap();
        for dep in &cfg.depends_on {
            if let Some(ActionState::Failed(msg)) = map.get(dep) {
                warn!(dep = %dep, reason = %msg, "Dependency failed");
                failed_deps.push(dep.clone());
            }
        }
    }

    if !failed_deps.is_empty() {
        anyhow::bail!(
            "Wait action '{}' completed but dependencies failed: {:?}",
            cfg.name,
            failed_deps
        );
    }

    info!(depends_on = ?cfg.depends_on, "All dependencies satisfied");
    Ok(())
}
