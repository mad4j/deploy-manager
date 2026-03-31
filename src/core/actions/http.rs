use anyhow::{Context, Result};
use std::time::Duration;
use tracing::info;

use crate::core::config::{HttpActionConfig, HttpMethod};
use crate::core::executor::ExecutionContext;

/// Execute an `http` action.
pub async fn run(cfg: &HttpActionConfig, ctx: &ExecutionContext) -> Result<()> {
    info!(
        url = %cfg.url,
        method = ?cfg.method,
        "HTTP action"
    );

    if ctx.dry_run {
        info!(
            "[dry-run] Would send {} {} (timeout={}s)",
            method_str(&cfg.method),
            cfg.url,
            cfg.timeout_secs,
        );
        return Ok(());
    }

    let timeout = if cfg.timeout_secs > 0 {
        Duration::from_secs(cfg.timeout_secs)
    } else {
        Duration::from_secs(u64::MAX / 1_000_000_000)
    };

    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .context("Failed to build HTTP client")?;

    let mut request = match cfg.method {
        HttpMethod::Get => client.get(&cfg.url),
        HttpMethod::Post => client.post(&cfg.url),
        HttpMethod::Put => client.put(&cfg.url),
        HttpMethod::Patch => client.patch(&cfg.url),
        HttpMethod::Delete => client.delete(&cfg.url),
        HttpMethod::Head => client.head(&cfg.url),
    };

    for (key, value) in &cfg.headers {
        request = request.header(key.as_str(), value.as_str());
    }

    if let Some(body) = &cfg.body {
        request = request.body(body.clone());
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("HTTP request failed: {} {}", method_str(&cfg.method), cfg.url))?;

    let status = response.status();

    info!(
        status = status.as_u16(),
        url = %cfg.url,
        "HTTP response received"
    );

    if let Some(expected) = cfg.expected_status {
        anyhow::ensure!(
            status.as_u16() == expected,
            "HTTP action got status {} but expected {} for {}",
            status.as_u16(),
            expected,
            cfg.url
        );
    } else {
        anyhow::ensure!(
            status.is_success(),
            "HTTP action got non-success status {} for {}",
            status.as_u16(),
            cfg.url
        );
    }

    Ok(())
}

fn method_str(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Head => "HEAD",
    }
}
