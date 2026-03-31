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
        "Http action"
    );

    if ctx.dry_run {
        info!(
            "[dry-run] Would send {} request to: {}",
            method_str(&cfg.method),
            cfg.url
        );
        return Ok(());
    }

    let mut builder = reqwest::Client::builder();
    if cfg.timeout_secs > 0 {
        builder = builder.timeout(Duration::from_secs(cfg.timeout_secs));
    }
    let client = builder.build().context("Failed to build HTTP client")?;

    let method = to_reqwest_method(&cfg.method);
    let mut request = client.request(method, &cfg.url);

    for (key, value) in &cfg.headers {
        request = request.header(key.as_str(), value.as_str());
    }

    if let Some(body) = &cfg.body {
        request = request.body(body.clone());
    }

    let response = request
        .send()
        .await
        .with_context(|| format!("HTTP request to '{}' failed", cfg.url))?;

    let status = response.status();
    info!(status = %status, url = %cfg.url, "HTTP response received");

    if cfg.fail_on_error {
        if cfg.expected_status.is_empty() {
            // No explicit list: accept any 2xx.
            if !status.is_success() {
                anyhow::bail!(
                    "HTTP request to '{}' returned unexpected status: {}",
                    cfg.url,
                    status
                );
            }
        } else if !cfg.expected_status.contains(&status.as_u16()) {
            anyhow::bail!(
                "HTTP request to '{}' returned status {} (expected one of: {:?})",
                cfg.url,
                status,
                cfg.expected_status
            );
        }
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

fn to_reqwest_method(method: &HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
    }
}
