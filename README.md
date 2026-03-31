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

Each action has a mandatory `action` field that selects the action variant, and a mandatory `name` field that uniquely identifies it (used for dependency resolution in `wait` actions).

---

### `deploy` — verify a deployable file

```yaml
- name: deploy-myapp          # required – unique identifier
  action: deploy
  file: /path/to/myapp        # required – path to the artefact
  target: local               # optional – local (default) | remote(<host>)
  type: executable            # optional – executable (default) | service | container | script
```

---

### `shell` — run an arbitrary shell command

```yaml
- name: run-hook              # required
  action: shell
  command: echo "hello"       # required – single command OR list of commands
  working_dir: /tmp           # optional – working directory (default: cwd)
  fail_on_error: true         # optional – abort on non-zero exit code (default: true)
```

You can also execute multiple commands in sequence:

```yaml
- name: run-setup
  action: shell
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
  action: filesystem
  operation: create_dir       # required – copy | move | delete | create_dir | create_file
  source: /src/file.txt       # required for: copy, move, delete
  destination: /etc/myapp/    # required for: copy, move, create_dir, create_file
  overwrite: false            # optional – overwrite existing destination (default: false)
```

---

### `wait` — block until listed actions have completed

```yaml
- name: wait-for-deploy       # required
  action: wait
  depends_on:                 # required – list of action names to wait for
    - deploy-myapp
    - run-hook
  timeout_secs: 60            # optional – seconds to wait before failing (0 = unlimited, default: 0)
```

---

### `log` — emit a message to the structured log

```yaml
- name: announce-start        # required
  action: log
  message: "Starting deployment phase 2"   # required – message text
  level: info                 # optional – info (default) | warn | error
```

---

### `sleep` — pause execution for a fixed duration

```yaml
- name: brief-pause           # required
  action: sleep
  secs: 5                     # optional – whole seconds to sleep (default: 0)
  millis: 500                 # optional – additional milliseconds (default: 0)
```

---

### `http` — send an HTTP request

```yaml
- name: health-check          # required
  action: http
  url: "https://api.example.com/health"   # required – target URL
  method: GET                 # optional – GET (default) | POST | PUT | PATCH | DELETE | HEAD
  headers:                    # optional – additional request headers
    Authorization: "Bearer ${API_TOKEN}"
    Content-Type: "application/json"
  body: '{"key": "value"}'    # optional – request body as a plain string
  expected_status: 200        # optional – fail if response status differs (default: fail on non-2xx)
  timeout_secs: 30            # optional – request timeout (0 = none, default: 30)
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
    action: deploy
    file: '${APP_BINARY}'
    type: executable
```

If a variable is missing, parsing fails with a clear error.
