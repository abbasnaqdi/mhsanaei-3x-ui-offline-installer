use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Proxy configuration provided by the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub kind: ProxyKind,
    /// Full proxy URL, e.g. "socks5://127.0.0.1:1080" or "http://user:pass@proxy:3128"
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyKind {
    None,
    Socks5,
    Http,
}

// ─── Wizard question ──────────────────────────────────────────────────────────

/// Ask the user whether they need a proxy, and if so, which kind and address.
/// Returns None if no proxy is needed.
pub fn ask_proxy() -> Result<Option<ProxyConfig>> {
    let theme = ColorfulTheme::default();

    println!("{}", style("┌─ Proxy Settings (Optional) ─────────────────────────────┐").bold().blue());
    println!();
    println!(
        "  {}",
        style("If this system requires a proxy for downloading, configure it here.").dim()
    );
    println!(
        "  {}",
        style("Guide: SOCKS5 for VPN clients like Clash/V2Ray / HTTP for Squid and similar.").dim()
    );
    println!();

    let needs_proxy = Confirm::with_theme(&theme)
        .with_prompt("Do you need a proxy for downloading?")
        .default(false)
        .interact()?;

    if !needs_proxy {
        println!(
            "  {} No proxy — direct connection",
            style("→").dim()
        );
        println!();
        return Ok(None);
    }

    let kind_items = vec![
        "SOCKS5  (Example: socks5h://127.0.0.1:1080)",
        "HTTP    (Example: http://127.0.0.1:8080)",
    ];
    let kind_sel = Select::with_theme(&theme)
        .with_prompt("Proxy Type")
        .items(&kind_items)
        .default(0)
        .interact()?;

    let kind = if kind_sel == 0 { ProxyKind::Socks5 } else { ProxyKind::Http };
    let default_url = if kind == ProxyKind::Socks5 {
        "socks5h://127.0.0.1:1080"
    } else {
        "http://127.0.0.1:8080"
    };

    let url: String = loop {
        let raw: String = Input::with_theme(&theme)
            .with_prompt("Proxy Address")
            .default(default_url.to_string())
            .interact_text()?;
        let mut raw = raw.trim().to_string();

        // If no protocol is provided, prepend based on kind
        if !raw.contains("://") {
            let prefix = if kind == ProxyKind::Socks5 { "socks5h://" } else { "http://" };
            raw = format!("{}{}", prefix, raw);
            println!("  {} No protocol provided, using: {}", style("ℹ").dim(), style(&raw).cyan());
        }

        // Auto-convert socks5:// to socks5h:// to prevent DNS leaks in restricted networks
        if raw.starts_with("socks5://") {
            raw = raw.replace("socks5://", "socks5h://");
            println!("  {} Auto-converted to {} to prevent DNS issues.", style("ℹ").dim(), style(&raw).cyan());
        }

        // Basic validation
        if raw.starts_with("socks5h://") || raw.starts_with("socks4://")
            || raw.starts_with("http://")  || raw.starts_with("https://")
        {
            break raw;
        }
        println!(
            "  {} Invalid address. Must start with socks5h:// or http://",
            style("✗").red()
        );
    };

    let cfg = ProxyConfig { kind, url };

    // Test the connection
    println!();
    println!(
        "  {} Testing connection through proxy...",
        style("🔗").cyan()
    );

    match test_proxy(&cfg).await_or_run() {
        Ok(ms) => {
            println!(
                "  {} Connection successful! HTTP Round Trip: {}ms",
                style("✓").green().bold(),
                style(ms).yellow()
            );
        }
        Err(e) => {
            println!("  {} Proxy did not respond: {}", style("✗").red(), e);
            println!();

            let choice_items = vec![
                "Retry with the same proxy",
                "Enter a new address",
                "Continue without verification (download might fail)",
                "Continue without proxy",
            ];
            let choice = Select::with_theme(&theme)
                .with_prompt("What would you like to do?")
                .items(&choice_items)
                .default(0)
                .interact()?;

            match choice {
                3 => {
                    println!("  {} Continuing without proxy.", style("→").dim());
                    println!();
                    return Ok(None);
                }
                2 => {
                    println!(
                        "  {} Continuing with proxy without verification.",
                        style("⚠️").yellow()
                    );
                }
                _ => {
                    // Recurse for retry / new address
                    println!();
                    return ask_proxy();
                }
            }
        }
    }

    println!();
    Ok(Some(cfg))
}

// ─── Connection test (sync wrapper for use in the wizard) ────────────────────

struct SyncResult(Result<u64, String>);

/// Synchronously test the proxy by making a GET to a fast 204 endpoint.
fn test_proxy(cfg: &ProxyConfig) -> SyncResult {
    let url = cfg.url.clone();
    SyncResult(std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        rt.block_on(async {
            let client = match build_client_inner(&url, Duration::from_secs(15)) {
                Ok(c) => c,
                Err(e) => return Err(format!("Proxy Config Error: {}", e)),
            };
            
            let start = std::time::Instant::now();
            let res = client
                .get("http://cp.cloudflare.com/generate_204")
                .header("User-Agent", "Mozilla/5.0")
                .send()
                .await;

            match res {
                Ok(resp) => {
                    match resp.error_for_status() {
                        Ok(_) => Ok(start.elapsed().as_millis() as u64),
                        Err(e) => Err(format!("HTTP Error (Status {}): Make sure the proxy is working.", e.status().map(|s| s.as_u16()).unwrap_or(0))),
                    }
                }
                Err(e) => {
                    let mut detailed = format!("Failed: {}", e);
                    if e.is_timeout() {
                        detailed = "Timeout (15s): The proxy server did not respond in time.".to_string();
                    } else if e.is_connect() {
                        detailed = "Connection Refused or Unreachable: Is your proxy app running on this port?".to_string();
                    }
                    Err(detailed)
                }
            }
        })
    })
    .join()
    .unwrap_or_else(|_| Err("Thread panicked during proxy test".to_string())))
}

impl SyncResult {
    fn await_or_run(self) -> Result<u64, String> {
        self.0
    }
}

// ─── Client builder (used by all downloaders) ────────────────────────────────

/// Build a reqwest Client, optionally routing through the given proxy.
pub fn build_client(proxy: &Option<ProxyConfig>) -> Result<reqwest::Client> {
    let proxy_url = proxy.as_ref().map(|p| p.url.as_str());
    build_client_inner(
        proxy_url.unwrap_or(""),
        Duration::from_secs(60),
    )
}

fn build_client_inner(proxy_url: &str, timeout: Duration) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .user_agent("xui-offline-builder/0.1")
        .timeout(timeout);

    if !proxy_url.is_empty() {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| anyhow::anyhow!("Invalid proxy: {}", e))?;
        builder = builder.proxy(proxy);
    }

    Ok(builder.build()?)
}
