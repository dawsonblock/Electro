//! Shared browser launch/runtime policy for local and remote isolated browser backends.

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::handler::Handler;
use temm1e_core::types::error::Temm1eError;

pub const BROWSER_ISOLATION_MODE_ENV: &str = "TEMM1E_BROWSER_ISOLATION_MODE";
pub const BROWSER_REMOTE_URL_ENV: &str = "TEMM1E_BROWSER_REMOTE_URL";
pub const BROWSER_PROXY_REQUIRED_ENV: &str = "TEMM1E_BROWSER_PROXY_REQUIRED";
pub const DEFAULT_REMOTE_BROWSER_URL: &str = "http://127.0.0.1:9223";

pub fn env_flag(name: &str) -> bool {
    matches!(
        std::env::var(name)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub fn browser_isolation_mode() -> String {
    let raw = std::env::var(BROWSER_ISOLATION_MODE_ENV)
        .unwrap_or_else(|_| "remote".to_string());
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() {
        "remote".to_string()
    } else {
        trimmed
    }
}

pub fn browser_uses_remote() -> bool {
    !matches!(browser_isolation_mode().as_str(), "local" | "legacy-local" | "host")
}

pub fn browser_remote_url() -> String {
    let raw = std::env::var(BROWSER_REMOTE_URL_ENV)
        .unwrap_or_else(|_| DEFAULT_REMOTE_BROWSER_URL.to_string());
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        DEFAULT_REMOTE_BROWSER_URL.to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn browser_proxy_required_for_local() -> bool {
    match std::env::var(BROWSER_PROXY_REQUIRED_ENV) {
        Ok(v) => !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"),
        Err(_) => true,
    }
}

pub fn browser_inherit_session_requested() -> bool {
    env_flag("TEMM1E_INHERIT_BROWSER_SESSION")
}

pub fn ensure_browser_runtime_policy(context: &str) -> Result<(), Temm1eError> {
    if browser_uses_remote() {
        if browser_inherit_session_requested() {
            return Err(Temm1eError::Tool(format!(
                "{context} is running in remote browser isolation mode. TEMM1E_INHERIT_BROWSER_SESSION=1 is blocked because live local browser state cannot be copied into the containerized browser boundary."
            )));
        }
        return Ok(());
    }

    if browser_proxy_required_for_local()
        && std::env::var("TEMM1E_BROWSER_PROXY_SERVER")
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return Err(Temm1eError::Tool(format!(
            "{context} local browser launch is blocked. Set {BROWSER_ISOLATION_MODE_ENV}=remote and start the browser sandbox, or explicitly set {BROWSER_ISOLATION_MODE_ENV}=local with TEMM1E_BROWSER_PROXY_SERVER configured."
        )));
    }

    Ok(())
}

pub async fn connect_or_launch_browser(
    config: BrowserConfig,
    context: &str,
) -> Result<(Browser, Handler), Temm1eError> {
    ensure_browser_runtime_policy(context)?;

    if browser_uses_remote() {
        let url = browser_remote_url();
        tracing::info!(context = context, remote = %url, mode = %browser_isolation_mode(), "Connecting to isolated remote browser");

        let http_url = if url.starts_with("ws://") || url.starts_with("wss://") {
            url.replace("ws://", "http://").replace("wss://", "https://")
        } else {
            url.clone()
        };
        let health_url = format!("{}/json/version", http_url.trim_end_matches('/'));
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .map_err(|e| Temm1eError::Tool(format!("Failed to build health check client: {}", e)))?;

        if let Err(e) = client.get(&health_url).send().await {
            return Err(Temm1eError::Tool(format!(
                "CDP Health Check failed: The remote browser at {} is unreachable ({}). Please start the browser sandbox stack with 'docker-compose -f docker-compose.browser-sandbox.yml up -d' or set TEMM1E_BROWSER_ISOLATION_MODE=local explicitly.",
                url, e
            )));
        }

        Browser::connect(url.clone()).await.map_err(|e| {
            Temm1eError::Tool(format!(
                "Failed to connect to isolated remote browser at {url} after health check passed. Error: {e}"
            ))
        })
    } else {
        tracing::warn!(context = context, mode = %browser_isolation_mode(), "Launching local browser process");
        Browser::launch(config).await.map_err(|e| {
            Temm1eError::Tool(format!(
                "Failed to launch local browser process. Error: {e}"
            ))
        })
    }
}
