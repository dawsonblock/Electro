//! Web fetch tool — retrieves content from public URLs via HTTP GET.

use crate::network_guard::{
    enforce_host_allowlist, ensure_resolved_host_is_public, host_is_blocked,
    load_domain_allowlist_from_env, validate_public_url,
};
use async_trait::async_trait;
use temm1e_core::types::error::Temm1eError;
use temm1e_core::{Tool, ToolContext, ToolInput, ToolOutput};
use temm1e_core::policy::CapabilityPolicy;


/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Maximum response body size (32 KB — keeps tool output within token budget).
const MAX_RESPONSE_SIZE: usize = 32 * 1024;

pub struct WebFetchTool {
    client: reqwest::Client,
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetchTool {
    pub fn new() -> Self {
        let redirect_allowlist = load_domain_allowlist_from_env(PUBLIC_WEB_ALLOWLIST_ENV);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .user_agent("TEMM1E/0.1")
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                let host = attempt.url().host_str().map(|s| s.to_string());
                match host.as_deref() {
                    Some(h) if host_is_blocked(h) => attempt.error(format!(
                        "Blocked redirect target '{}'. Private, loopback, local, and internal redirects are disabled.",
                        h
                    )),
                    Some(h) => match enforce_host_allowlist(h, &redirect_allowlist, "redirect host") {
                        Ok(()) => attempt.follow(),
                        Err(msg) => attempt.error(msg),
                    },
                    None => attempt.error("Redirect target is missing a host"),
                }
            }))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { client }
    }
}

const PUBLIC_WEB_ALLOWLIST_ENV: &str = "TEMM1E_PUBLIC_WEB_ALLOWLIST";

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch the content of a public web page or API endpoint via HTTP GET. Private, loopback, local, and internal targets are blocked, including blocked redirects, DNS resolutions to private addresses, and optional operator domain allowlists via TEMM1E_PUBLIC_WEB_ALLOWLIST."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The public http:// or https:// URL to fetch"
                },
                "headers": {
                    "type": "object",
                    "description": "Optional HTTP headers as key-value pairs",
                    "additionalProperties": { "type": "string" }
                }
            },
            "required": ["url"]
        })
    }

    fn declarations(&self) -> CapabilityPolicy {
        CapabilityPolicy {
            file_access: Vec::new(),
            network_access: vec!["public-http".to_string()],
            shell_access: temm1e_core::policy::ShellPolicy::Blocked,
browser_access: temm1e_core::policy::BrowserPolicy::Blocked,
        }
    }

    async fn execute(
        &self,
        input: ToolInput,
        _ctx: &ToolContext,
    ) -> Result<ToolOutput, Temm1eError> {
        let raw_url = input
            .arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Temm1eError::Tool("Missing required parameter: url".into()))?;

        let url = match validate_public_url(raw_url) {
            Ok(url) => url,
            Err(msg) => {
                return Ok(ToolOutput {
                    content: msg,
                    is_error: true,
                });
            }
        };

        if let Err(msg) = ensure_resolved_host_is_public(&url).await {
            return Ok(ToolOutput {
                content: msg,
                is_error: true,
            });
        }

        let allowlist = load_domain_allowlist_from_env(PUBLIC_WEB_ALLOWLIST_ENV);
        if let Some(host) = url.host_str() {
            if let Err(msg) = enforce_host_allowlist(host, &allowlist, "web fetch host") {
                return Ok(ToolOutput {
                    content: msg,
                    is_error: true,
                });
            }
        }

        let mut request = self.client.get(url.clone());

        if let Some(headers) = input.arguments.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key.as_str(), val_str);
                }
            }
        }

        tracing::info!(url = %url, "Fetching URL");

        match request.send().await {
            Ok(response) => {
                let status = response.status();
                let status_code = status.as_u16();

                match response.text().await {
                    Ok(mut body) => {
                        if body.len() > MAX_RESPONSE_SIZE {
                            body.truncate(MAX_RESPONSE_SIZE);
                            body.push_str("\n... [response truncated]");
                        }

                        let content = format!(
                            "HTTP {} {}\n\n{}",
                            status_code,
                            status.canonical_reason().unwrap_or(""),
                            body,
                        );

                        Ok(ToolOutput {
                            content,
                            is_error: status.is_client_error() || status.is_server_error(),
                        })
                    }
                    Err(e) => Ok(ToolOutput {
                        content: format!("Failed to read response body: {}", e),
                        is_error: true,
                    }),
                }
            }
            Err(e) => Ok(ToolOutput {
                content: format!("Request failed: {}", e),
                is_error: true,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_filter_blocks_local_targets() {
        assert!(host_is_blocked("127.0.0.1"));
        assert!(host_is_blocked("localhost"));
        assert!(host_is_blocked("10.0.0.8"));
        assert!(!host_is_blocked("example.com"));
    }
}
