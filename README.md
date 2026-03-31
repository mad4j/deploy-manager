# deploy-manager

Rust implementation of a platform independent deploy manager

## Examples

- `examples/example-deploy.yaml`: basic sample covering the supported action types.
- `examples/example-long-deploy.yaml`: longer Windows-oriented sample for manually testing progress bar redraw and log output. Run it from the repository root with `cargo run -- --file .\\examples\\example-long-deploy.yaml`.
- `examples/example-env-vars.yaml`: demonstrates environment variable interpolation with `${VAR_NAME}` placeholders.

## Configuration File Format

A deploy file is a YAML document with the following top-level fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `description` | string | no | Human-readable description of the deploy plan |
| `actions` | list | yes | Ordered list of actions to execute |

Each action has a mandatory `type` field that selects the action variant, and a mandatory `name` field that uniquely identifies it (used for dependency resolution in `wait` actions).

---

### `deploy` — copy a binary/artefact to a destination

```yaml
- name: deploy-myapp          # required – unique identifier
  type: deploy
  app: myapp                  # required – application name
  binary: /path/to/myapp      # required – path to the artefact
  destination: /opt/apps/     # optional – target directory
  target: local               # optional – local (default) | remote(<host>)
  exec_type: executable       # optional – executable (default) | service | container | script
```

---

### `shell` — run an arbitrary shell command

```yaml
- name: run-hook              # required
  type: shell
  command: echo "hello"       # required – single command OR list of commands
  working_dir: /tmp           # optional – working directory (default: cwd)
  fail_on_error: true         # optional – abort on non-zero exit code (default: true)
```

You can also execute multiple commands in sequence:

```yaml
- name: run-setup
  type: shell
  command:
    - echo "prepare"
    - echo "migrate"
    - echo "done"
  fail_on_error: true
```

When `fail_on_error` is `true`, execution stops at the first command with non-zero exit code.

---

### `filesystem` — perform a filesystem operation

```yaml
- name: create-config-dir     # required
  type: filesystem
  operation: create_dir       # required – copy | move | delete | create_dir | create_file
  source: /src/file.txt       # required for: copy, move, delete
  destination: /etc/myapp/    # required for: copy, move, create_dir, create_file
  overwrite: false            # optional – overwrite existing destination (default: false)
```

---

### `wait` — block until listed actions have completed

```yaml
- name: wait-for-deploy       # required
  type: wait
  depends_on:                 # required – list of action names to wait for
    - deploy-myapp
    - run-hook
  timeout_secs: 60            # optional – seconds to wait before failing (0 = unlimited, default: 0)
```

---

## Environment Variables In Deploy Files

You can reference environment variables in any YAML string value using `${VAR_NAME}`.
The placeholder is expanded before YAML parsing.
For path-like values, prefer single-quoted YAML strings.

Example:

```yaml
description: 'Deploy ${APP_NAME}'

actions:
	- name: deploy
		type: deploy
		app: '${APP_NAME}'
		binary: '${APP_BINARY}'
		destination: '${DEPLOY_DEST}'
```

If a variable is missing, parsing fails with a clear error.
