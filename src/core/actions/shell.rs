use anyhow::{Context, Result};
use tracing::info;

use crate::core::config::ShellActionConfig;
use crate::core::executor::ExecutionContext;

/// Execute a `shell` action.
pub async fn run(cfg: &ShellActionConfig, ctx: &ExecutionContext) -> Result<()> {
    let commands = cfg.command.as_slice();
    anyhow::ensure!(!commands.is_empty(), "Shell action requires at least one command");
    anyhow::ensure!(
        commands.iter().all(|command| !command.trim().is_empty()),
        "Shell action contains an empty command"
    );

    info!(count = commands.len(), "Shell action");

    if ctx.dry_run {
        for command in &commands {
            info!("[dry-run] Would execute: {}", command);
        }
        return Ok(());
    }

    for (index, command) in commands.iter().enumerate() {
        info!(
            command = %command,
            position = index + 1,
            total = commands.len(),
            "Executing shell command"
        );

        // Split the command into program + args using a simple shell-style split.
        let parts = shell_split(command);
        anyhow::ensure!(!parts.is_empty(), "Empty command");

        let (program, args) = parts.split_first().unwrap();

        let mut cmd = tokio::process::Command::new(program);
        cmd.args(args);

        if let Some(dir) = &cfg.working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .await
            .with_context(|| format!("Failed to spawn command: {}", command))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.trim().is_empty() {
            info!(stdout = %stdout.trim(), "Command output");
        }
        if !stderr.trim().is_empty() {
            info!(stderr = %stderr.trim(), "Command stderr");
        }

        if cfg.fail_on_error && !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            anyhow::bail!("Command '{}' exited with code {}", command, code);
        }
    }

    Ok(())
}

/// Minimalist shell-style word splitter (handles quoted strings).
fn shell_split(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;

    for ch in s.chars() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_split_simple() {
        assert_eq!(
            shell_split("echo hello world"),
            vec!["echo", "hello", "world"]
        );
    }

    #[test]
    fn shell_split_quoted() {
        assert_eq!(
            shell_split(r#"echo "hello world""#),
            vec!["echo", "hello world"]
        );
    }

    #[test]
    fn shell_split_single_quoted() {
        assert_eq!(shell_split("echo 'foo bar'"), vec!["echo", "foo bar"]);
    }
}
