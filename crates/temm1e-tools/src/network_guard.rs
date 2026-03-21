//! Shared network boundary helpers for public-web tools.
//!
//! These helpers enforce the repo's public-web policy:
//! - only http/https URLs
//! - no loopback/private/link-local/internal/local targets
//! - optional operator allowlist via environment variable
//! - DNS resolution must stay on public addresses

use url::Url;

/// Split and normalize a comma-separated allowlist of hostnames.
pub fn load_domain_allowlist_from_env(var_name: &str) -> Vec<String> {
    std::env::var(var_name)
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().trim_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn host_matches_allow_entry(host: &str, entry: &str) -> bool {
    let host = host.trim().trim_matches('.').to_ascii_lowercase();
    let entry = entry.trim().trim_matches('.').to_ascii_lowercase();
    if host.is_empty() || entry.is_empty() {
        return false;
    }
    host == entry || host.ends_with(&format!(".{entry}"))
}

pub fn enforce_host_allowlist(host: &str, allowlist: &[String], label: &str) -> Result<(), String> {
    if allowlist.is_empty() {
        return Ok(());
    }

    if allowlist
        .iter()
        .any(|entry| host_matches_allow_entry(host, entry))
    {
        Ok(())
    } else {
        Err(format!(
            "Blocked {} '{}'. TEMM1E_PUBLIC_WEB_ALLOWLIST is set and this host is not permitted.",
            label, host
        ))
    }
}

pub fn host_is_blocked(host: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    if host.is_empty() {
        return true;
    }

    if matches!(host.as_str(), "localhost" | "localhost.localdomain")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host.ends_with(".localhost")
    {
        return true;
    }

    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(ip) => {
                ip.is_private()
                    || ip.is_loopback()
                    || ip.is_link_local()
                    || ip.is_multicast()
                    || ip.is_unspecified()
            }
            std::net::IpAddr::V6(ip) => {
                let seg0 = ip.segments()[0];
                ip.is_loopback()
                    || ip.is_unspecified()
                    || (seg0 & 0xfe00) == 0xfc00
                    || (seg0 & 0xffc0) == 0xfe80
            }
        };
    }

    false
}

pub fn validate_public_url(raw: &str) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|e| format!("Invalid URL: {}", e))?;
    match url.scheme() {
        "http" | "https" => {}
        other => {
            return Err(format!(
                "Unsupported URL scheme '{}'. Only http and https are allowed.",
                other
            ));
        }
    }

    let host = url
        .host_str()
        .ok_or_else(|| "URL must include a host".to_string())?;
    if host_is_blocked(host) {
        return Err(format!(
            "Blocked URL host '{}'. Private, loopback, local, and internal targets are disabled.",
            host
        ));
    }

    Ok(url)
}

fn resolved_port(url: &Url) -> u16 {
    url.port_or_known_default()
        .unwrap_or(if url.scheme() == "https" { 443 } else { 80 })
}

pub async fn ensure_resolved_host_is_public(url: &Url) -> Result<(), String> {
    let host = url
        .host_str()
        .ok_or_else(|| "URL must include a host".to_string())?;
    let port = resolved_port(url);
    let resolved = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| format!("Failed to resolve host '{}': {}", host, e))?;

    let mut found = false;
    for addr in resolved {
        found = true;
        let ip = addr.ip();
        if host_is_blocked(&ip.to_string()) {
            return Err(format!(
                "Blocked URL host '{}'. DNS resolved to non-public address {}.",
                host, ip
            ));
        }
    }

    if !found {
        return Err(format!("Failed to resolve host '{}': no addresses returned.", host));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_allow_entry_matches_exact_and_subdomain() {
        assert!(host_matches_allow_entry("docs.github.com", "github.com"));
        assert!(host_matches_allow_entry("github.com", "github.com"));
        assert!(!host_matches_allow_entry("evilgithub.com", "github.com"));
    }

    #[test]
    fn load_allowlist_normalizes() {
        std::env::set_var("TEMM1E_TEST_ALLOW", " GitHub.com, docs.rs ,,EXAMPLE.ORG. ");
        let values = load_domain_allowlist_from_env("TEMM1E_TEST_ALLOW");
        assert_eq!(values, vec!["github.com", "docs.rs", "example.org"]);
        std::env::remove_var("TEMM1E_TEST_ALLOW");
    }
}
