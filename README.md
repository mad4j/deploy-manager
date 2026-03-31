# deploy-manager

Rust implementation of a platform independent deploy manager

## Examples

- `examples/example-deploy.yaml`: basic sample covering the supported action types.
- `examples/example-long-deploy.yaml`: longer Windows-oriented sample for manually testing progress bar redraw and log output. Run it from the repository root with `cargo run -- --file .\\examples\\example-long-deploy.yaml`.
- `examples/example-env-vars.yaml`: demonstrates environment variable interpolation with `${VAR_NAME}` placeholders.

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
