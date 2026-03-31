use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Top-level structure of a deploy YAML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployFile {
    /// Ordered list of actions to execute.
    pub actions: Vec<ActionConfig>,
}

/// Every entry in the `actions` list is one of these variants, discriminated
/// by the `type` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionConfig {
    Deploy(DeployActionConfig),
    Shell(ShellActionConfig),
    Filesystem(FilesystemActionConfig),
    Wait(WaitActionConfig),
}

impl ActionConfig {
    /// Return the human-readable name of this action.
    pub fn name(&self) -> &str {
        match self {
            ActionConfig::Deploy(c) => &c.name,
            ActionConfig::Shell(c) => &c.name,
            ActionConfig::Filesystem(c) => &c.name,
            ActionConfig::Wait(c) => &c.name,
        }
    }
}

// ---------------------------------------------------------------------------
// Deploy action
// ---------------------------------------------------------------------------

/// Processor / architecture target for a deploy action.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeployTarget {
    /// The local machine (default).
    #[default]
    Local,
    /// A remote host identified by hostname or IP.
    Remote(String),
}

/// Executable type for a deploy action.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecType {
    /// A native binary / executable (default).
    #[default]
    Executable,
    /// A system service / daemon.
    Service,
    /// A container image.
    Container,
    /// A script file.
    Script,
}

/// Configuration for a `deploy` action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployActionConfig {
    /// Unique name for this action (used for dependency resolution).
    pub name: String,
    /// Application name / identifier.
    pub app: String,
    /// Path to the binary / artefact that should be deployed.
    pub binary: PathBuf,
    /// Destination directory where the binary will be copied.
    #[serde(default)]
    pub destination: Option<PathBuf>,
    /// Target processor / environment (default: `local`).
    #[serde(default)]
    pub target: DeployTarget,
    /// Type of executable (default: `executable`).
    #[serde(default)]
    pub exec_type: ExecType,
}

// ---------------------------------------------------------------------------
// Shell action
// ---------------------------------------------------------------------------

/// Configuration for a `shell` action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellActionConfig {
    /// Unique name for this action.
    pub name: String,
    /// The shell command(s) to execute. Supports a single string or a list.
    pub command: ShellCommandSpec,
    /// Working directory for the command (default: current directory).
    #[serde(default)]
    pub working_dir: Option<PathBuf>,
    /// Whether a non-zero exit code is treated as an error (default: `true`).
    #[serde(default = "default_true")]
    pub fail_on_error: bool,
}

/// Supported shapes for a `shell.command` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ShellCommandSpec {
    Single(String),
    Multiple(Vec<String>),
}

impl ShellCommandSpec {
    pub fn as_slice(&self) -> Vec<&str> {
        match self {
            ShellCommandSpec::Single(command) => vec![command.as_str()],
            ShellCommandSpec::Multiple(commands) => {
                commands.iter().map(std::string::String::as_str).collect()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Filesystem action
// ---------------------------------------------------------------------------

/// Supported filesystem operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FsOperation {
    Copy,
    Move,
    Delete,
    CreateDir,
    CreateFile,
}

/// Configuration for a `filesystem` action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemActionConfig {
    /// Unique name for this action.
    pub name: String,
    /// Filesystem operation to perform.
    pub operation: FsOperation,
    /// Source path (required for `copy`, `move`, `delete`).
    #[serde(default)]
    pub source: Option<PathBuf>,
    /// Destination path (required for `copy`, `move`, `create_dir`, `create_file`).
    #[serde(default)]
    pub destination: Option<PathBuf>,
    /// Overwrite destination if it already exists (default: `false`).
    #[serde(default)]
    pub overwrite: bool,
}

// ---------------------------------------------------------------------------
// Wait action
// ---------------------------------------------------------------------------

/// Configuration for a `wait` action.
///
/// A wait action blocks until all named actions in `depends_on` have
/// completed successfully.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitActionConfig {
    /// Unique name for this action.
    pub name: String,
    /// Names of the actions that must finish before this one proceeds.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Optional timeout in seconds (0 = unlimited).
    #[serde(default)]
    pub timeout_secs: u64,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn interpolate_env_vars(content: &str) -> anyhow::Result<String> {
    let mut out = String::with_capacity(content.len());
    let mut rest = content;

    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);

        let placeholder = &rest[start + 2..];
        let Some(end) = placeholder.find('}') else {
            anyhow::bail!("Unclosed environment variable placeholder: '${{'")
        };

        let var_name = &placeholder[..end];
        if var_name.is_empty() {
            anyhow::bail!("Empty environment variable placeholder '${{}}' is not allowed");
        }

        let value = std::env::var(var_name)
            .map_err(|_| anyhow::anyhow!("Environment variable '{}' is not set", var_name))?;

        out.push_str(&value);
        rest = &placeholder[end + 1..];
    }

    out.push_str(rest);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

/// Parse a `DeployFile` from a YAML string.
pub fn parse_deploy_file(content: &str) -> anyhow::Result<DeployFile> {
    let interpolated = interpolate_env_vars(content)?;

    let deploy: DeployFile = serde_yaml::from_str(&interpolated)
        .map_err(|e| anyhow::anyhow!("Failed to parse deploy file: {}", e))?;
    validate_deploy_file(&deploy)?;
    Ok(deploy)
}

/// Basic validation: action names must be unique.
fn validate_deploy_file(deploy: &DeployFile) -> anyhow::Result<()> {
    let mut names = std::collections::HashSet::new();
    for action in &deploy.actions {
        let name = action.name();
        if !names.insert(name.to_string()) {
            anyhow::bail!("Duplicate action name: '{}'", name);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r#"
actions:
  - name: copy-binary
    type: deploy
    app: myapp
    binary: /usr/local/bin/myapp
    destination: /opt/apps/
    target: local
    exec_type: executable

  - name: run-setup
    type: shell
    command: echo "Setup complete"
    fail_on_error: true

  - name: create-config-dir
    type: filesystem
    operation: create_dir
    destination: /etc/myapp

  - name: wait-for-setup
    type: wait
    depends_on:
      - run-setup
"#;

    #[test]
    fn parse_sample_yaml() {
        let deploy = parse_deploy_file(SAMPLE_YAML).unwrap();
        assert_eq!(deploy.actions.len(), 4);
    }

    #[test]
    fn duplicate_names_rejected() {
        let yaml = r#"
actions:
  - name: dup
    type: shell
    command: echo 1
  - name: dup
    type: shell
    command: echo 2
"#;
        assert!(parse_deploy_file(yaml).is_err());
    }

    #[test]
    fn defaults_applied() {
        let yaml = r#"
actions:
  - name: deploy-app
    type: deploy
    app: myapp
    binary: /bin/myapp
"#;
        let deploy = parse_deploy_file(yaml).unwrap();
        if let ActionConfig::Deploy(cfg) = &deploy.actions[0] {
            assert_eq!(cfg.target, DeployTarget::Local);
            assert_eq!(cfg.exec_type, ExecType::Executable);
        } else {
            panic!("expected deploy action");
        }
    }

    #[test]
    fn env_var_interpolation_supported() {
        let env_var_name = ["USERNAME", "USER"]
            .into_iter()
            .find(|name| std::env::var(name).is_ok())
            .expect("expected USERNAME or USER to be set in test environment");

        let yaml = format!(
            r#"
actions:
  - name: print-path
    type: shell
    command: echo ${{{}}}
"#,
            env_var_name
        );

        let deploy = parse_deploy_file(&yaml).unwrap();
        if let ActionConfig::Shell(cfg) = &deploy.actions[0] {
            match &cfg.command {
                ShellCommandSpec::Single(command) => {
                    assert!(!command.contains("${"));
                    assert!(command.starts_with("echo "));
                }
                ShellCommandSpec::Multiple(_) => {
                    panic!("expected single command")
                }
            }
        } else {
            panic!("expected shell action");
        }
    }

    #[test]
    fn shell_command_list_supported() {
        let yaml = r#"
actions:
  - name: run-many
    type: shell
    command:
      - echo first
      - echo second
"#;

        let deploy = parse_deploy_file(yaml).unwrap();
        if let ActionConfig::Shell(cfg) = &deploy.actions[0] {
            match &cfg.command {
                ShellCommandSpec::Single(_) => panic!("expected command list"),
                ShellCommandSpec::Multiple(commands) => {
                    assert_eq!(commands.len(), 2);
                    assert_eq!(commands[0], "echo first");
                    assert_eq!(commands[1], "echo second");
                }
            }
        } else {
            panic!("expected shell action");
        }
    }

    #[test]
    fn env_var_missing_is_rejected() {
        let yaml = r#"
actions:
  - name: bad-env
    type: shell
    command: "echo ${DEPLOY_MANAGER_TEST_MISSING_ENV_31A03A9B}"
"#;

        let err = parse_deploy_file(yaml).unwrap_err().to_string();
        assert!(err.contains("is not set"));
    }
}
