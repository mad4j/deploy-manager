use anyhow::Result;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

use crate::core::actions::{deploy, filesystem, http, log, shell, sleep, wait};
use crate::core::config::{ActionConfig, DeployFile};
use crate::frontend::progress::ProgressTracker;

/// Tracks the completion state of every action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionState {
    Pending,
    Running,
    Success,
    Failed(String),
    #[allow(dead_code)]
    Skipped,
}

/// Shared state table used by the wait action to observe other actions.
pub type StateMap = Arc<Mutex<HashMap<String, ActionState>>>;

/// Execution context passed to every action handler.
pub struct ExecutionContext {
    pub dry_run: bool,
    pub states: StateMap,
}

/// Run all actions in the deploy file sequentially, updating progress.
pub async fn execute(deploy: &DeployFile, dry_run: bool, tracker: &ProgressTracker) -> Result<()> {
    let states: StateMap = Arc::new(Mutex::new(HashMap::new()));

    // Pre-populate state map
    for action in &deploy.actions {
        states
            .lock()
            .unwrap()
            .insert(action.name().to_string(), ActionState::Pending);
    }

    let mut any_failed = false;

    for action in &deploy.actions {
        let name = action.name().to_string();
        info!(action = %name, "Starting action");
        tracker.start_action(&name);

        {
            let mut map = states.lock().unwrap();
            map.insert(name.clone(), ActionState::Running);
        }

        let ctx = ExecutionContext {
            dry_run,
            states: Arc::clone(&states),
        };

        let result = run_action(action, &ctx).await;

        match result {
            Ok(()) => {
                info!(action = %name, "Action succeeded");
                tracker.finish_action(&name, true);
                states
                    .lock()
                    .unwrap()
                    .insert(name.clone(), ActionState::Success);
            }
            Err(ref e) => {
                error!(action = %name, error = %e, "Action failed");
                tracker.finish_action(&name, false);
                states
                    .lock()
                    .unwrap()
                    .insert(name.clone(), ActionState::Failed(e.to_string()));
                any_failed = true;
                // Continue running remaining actions so wait actions can observe
                // the failed state and decide what to do.
                warn!(action = %name, "Continuing after failure");
            }
        }
    }

    if any_failed {
        let _ = tracker.suspend(|| {
            let mut stderr = io::stderr().lock();
            writeln!(stderr)
        });
        anyhow::bail!("One or more actions failed – see log output above.");
    }

    Ok(())
}

async fn run_action(action: &ActionConfig, ctx: &ExecutionContext) -> Result<()> {
    match action {
        ActionConfig::Deploy(cfg) => deploy::run(cfg, ctx).await,
        ActionConfig::Shell(cfg) => shell::run(cfg, ctx).await,
        ActionConfig::Filesystem(cfg) => filesystem::run(cfg, ctx).await,
        ActionConfig::Wait(cfg) => wait::run(cfg, ctx).await,
        ActionConfig::Log(cfg) => log::run(cfg, ctx).await,
        ActionConfig::Sleep(cfg) => sleep::run(cfg, ctx).await,
        ActionConfig::Http(cfg) => http::run(cfg, ctx).await,
    }
}
