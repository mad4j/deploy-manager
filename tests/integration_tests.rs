use std::fs;
use tempfile::TempDir;

/// Helpers to invoke the binary under test.
fn binary_path() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    // Strip the test binary path: .../target/debug/deps/integration_tests-<hash>
    // and navigate to .../target/debug/deploy-manager
    path.pop(); // deps
    path.pop(); // debug
    path.push("deploy-manager");
    path
}

fn run(args: &[&str]) -> std::process::Output {
    std::process::Command::new(binary_path())
        .args(args)
        .output()
        .expect("failed to run deploy-manager")
}

fn yaml_single_quoted_str(value: &str) -> String {
    value.replace('\'', "''")
}

fn yaml_single_quoted_path(value: &std::path::Path) -> String {
    // YAML single-quoted scalars keep backslashes literal, ideal for Windows paths.
    yaml_single_quoted_str(&value.display().to_string())
}

fn success_command() -> &'static str {
    if cfg!(windows) {
        "cmd /C echo hello"
    } else {
        "echo hello"
    }
}

fn failing_command() -> &'static str {
    if cfg!(windows) {
        "cmd /C exit 1"
    } else {
        "sh -c 'exit 1'"
    }
}

// ---------------------------------------------------------------------------
// Deploy file parsing / dry-run
// ---------------------------------------------------------------------------

#[test]
fn dry_run_shell_command() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        r#"
actions:
  - name: greet
    action: shell
    command: echo "hello"
"#,
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn dry_run_shell_command_list() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        r#"
actions:
  - name: greet-many
    action: shell
    command:
      - echo "hello"
      - echo "world"
"#,
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn dry_run_filesystem_action() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    let dest = dir.path().join("newdir");
    fs::write(
        &yaml_path,
        format!(
            r#"
actions:
  - name: mkdir
    action: filesystem
    operation: create_dir
    destination: '{}'
"#,
            yaml_single_quoted_path(&dest)
        ),
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn dry_run_deploy_missing_binary() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        r#"
actions:
  - name: deploy-app
    action: deploy
    file: /nonexistent/binary
"#,
    )
    .unwrap();

    // In dry-run mode the binary existence check is skipped.
    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn real_shell_command_success() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        format!(
            r#"
actions:
  - name: say-hello
    action: shell
    command: '{}'
"#,
            yaml_single_quoted_str(success_command())
        ),
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn real_shell_command_failure() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        format!(
            r#"
actions:
  - name: fail-cmd
    action: shell
    command: '{}'
    fail_on_error: true
"#,
            yaml_single_quoted_str(failing_command())
        ),
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap()]);
    assert!(!out.status.success(), "Expected failure but got success");
}

#[test]
fn real_filesystem_create_dir() {
    let dir = TempDir::new().unwrap();
    let new_dir = dir.path().join("subdir").join("nested");
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        format!(
            r#"
actions:
  - name: make-dir
    action: filesystem
    operation: create_dir
    destination: '{}'
"#,
            yaml_single_quoted_path(&new_dir)
        ),
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(new_dir.is_dir(), "Expected directory to be created");
}

#[test]
fn real_filesystem_copy_file() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("source.txt");
    let dst = dir.path().join("dest.txt");
    fs::write(&src, "hello copy").unwrap();

    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        format!(
            r#"
actions:
  - name: copy-file
    action: filesystem
    operation: copy
    source: '{}'
    destination: '{}'
    overwrite: true
"#,
            yaml_single_quoted_path(&src),
            yaml_single_quoted_path(&dst)
        ),
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(fs::read_to_string(&dst).unwrap(), "hello copy");
}

#[test]
fn invalid_yaml_rejected() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("bad.yaml");
    fs::write(&yaml_path, "this: is: not: valid: yaml:::::").unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap()]);
    assert!(!out.status.success());
}

#[test]
fn missing_file_exits_with_error() {
    let tmp = std::env::temp_dir();
    let nonexistent = tmp.join("nonexistent-deploy-manager-file-99999.yaml");
    let out = run(&["run", "--file", nonexistent.to_str().unwrap()]);
    assert!(!out.status.success());
}

#[test]
fn wait_action_dry_run() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        r#"
actions:
  - name: first
    action: shell
    command: echo first
  - name: sync
    action: wait
    depends_on:
      - first
"#,
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn dry_run_supports_env_var_interpolation() {
    let env_var_name = ["USERNAME", "USER"]
        .into_iter()
        .find(|name| std::env::var(name).is_ok())
        .expect("expected USERNAME or USER to be set in test environment");

    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        format!(
            r#"
actions:
  - name: greet
    action: shell
    command: echo ${{{}}}
"#,
            env_var_name
        ),
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn missing_env_var_fails_deploy_file_parsing() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("deploy.yaml");
    fs::write(
        &yaml_path,
        r#"
actions:
  - name: bad-env
    action: shell
    command: "echo ${DEPLOY_MANAGER_TEST_MISSING_ENV_31A03A9B}"
"#,
    )
    .unwrap();

    let out = run(&["run", "--file", yaml_path.to_str().unwrap(), "--dry-run"]);
    assert!(!out.status.success(), "Expected parsing failure but got success");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("is not set"),
        "expected missing env var error, stderr: {}",
        stderr
    );
}
