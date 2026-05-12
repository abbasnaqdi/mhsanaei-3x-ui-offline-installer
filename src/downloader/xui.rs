use anyhow::{Context, Result};
use console::style;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};

use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::manifest::{
    Manifest, STEP_SERVICE_FILE, STEP_XUI_BINARY, STEP_XUI_SH,
};
use crate::proxy;
use crate::wizard::state::{BuildConfig, TargetOs, XuiVersion};

const GITHUB_API: &str = "https://api.github.com/repos/MHSanaei/3x-ui/releases/latest";
const GITHUB_RELEASE_BASE: &str = "https://github.com/MHSanaei/3x-ui/releases/download";
const XUI_SH_URL: &str = "https://raw.githubusercontent.com/MHSanaei/3x-ui/main/x-ui.sh";

const SERVICE_DEBIAN_URL: &str =
    "https://raw.githubusercontent.com/MHSanaei/3x-ui/main/x-ui.service.debian";
const SERVICE_RHEL_URL: &str =
    "https://raw.githubusercontent.com/MHSanaei/3x-ui/main/x-ui.service.rhel";
const SERVICE_ARCH_URL: &str =
    "https://raw.githubusercontent.com/MHSanaei/3x-ui/main/x-ui.service.arch";
const RC_ALPINE_URL: &str =
    "https://raw.githubusercontent.com/MHSanaei/3x-ui/main/x-ui.rc";

/// Download all x-ui assets: binary tarball, CLI script, service file.
/// Skips files that are already marked Done + valid in the manifest.
pub async fn download(
    config: &BuildConfig,
    out_dir: &str,
    manifest: &mut Manifest,
) -> Result<()> {
    let client = proxy::build_client(&config.proxy)?;

    // ── Resolve version ───────────────────────────────────────────────────────
    let tag = match &config.xui_version {
        XuiVersion::Latest => {
            println!(
                "  {} Fetching latest x-ui version from GitHub...",
                style("→").cyan()
            );
            fetch_latest_tag(&client).await?
        }
        XuiVersion::Specific(t) => t.clone(),
    };

    // Update manifest with resolved version for installer rendering
    if let Some(obj) = manifest.config.as_object_mut() {
        obj.insert("xui_version".to_string(), serde_json::json!(tag));
    }

    println!(
        "  {} Version: {}",
        style("✓").green(),
        style(&tag).yellow().bold()
    );

    // ── Download tarball ──────────────────────────────────────────────────────
    let arch_suffix = config.arch.xui_suffix();
    let tar_name    = format!("x-ui-linux-{}.tar.gz", arch_suffix);
    let tar_dest    = format!("{}/{}", out_dir, tar_name);

    if manifest.step_is_valid(out_dir, STEP_XUI_BINARY) {
        println!("  {} x-ui binary — Already exists, skipping.", style("⏭️").dim());
    } else {
        let tar_url = format!("{}/{}/{}", GITHUB_RELEASE_BASE, tag, tar_name);
        download_with_progress(&client, &tar_url, &tar_dest, &format!("x-ui {} ({})", tag, arch_suffix))
            .await
            .context("Failed to download x-ui binary")?;
        manifest
            .mark_done(out_dir, STEP_XUI_BINARY, vec![tar_name.clone()])
            .context("Failed to save manifest")?;
    }

    // ── Download x-ui.sh (CLI manager) ───────────────────────────────────────
    let xui_sh_dest = format!("{}/x-ui.sh", out_dir);

    if manifest.step_is_valid(out_dir, STEP_XUI_SH) {
        println!("  {} x-ui.sh — Already exists, skipping.", style("⏭️").dim());
    } else {
        download_with_progress(&client, XUI_SH_URL, &xui_sh_dest, "x-ui.sh (CLI manager)")
            .await
            .context("Failed to download x-ui.sh")?;
        manifest
            .mark_done(out_dir, STEP_XUI_SH, vec!["x-ui.sh".to_string()])
            .context("Failed to save manifest")?;
    }

    // ── Download service file ─────────────────────────────────────────────────
    if manifest.step_is_valid(out_dir, STEP_SERVICE_FILE) {
        println!("  {} service file — Already exists, skipping.", style("⏭️").dim());
    } else {
        let (service_url, service_filename) = resolve_service_url(&config.os);
        let service_dest = format!("{}/{}", out_dir, service_filename);
        download_with_progress(&client, service_url, &service_dest, &service_filename)
            .await
            .context("Failed to download service file")?;
        manifest
            .mark_done(out_dir, STEP_SERVICE_FILE, vec![service_filename])
            .context("Failed to save manifest")?;
    }

    Ok(())
}

/// Fetch the latest release tag from GitHub API.
async fn fetch_latest_tag(client: &reqwest::Client) -> Result<String> {
    let mut attempts = 0;
    const MAX_RETRIES: u32 = 3;

    loop {
        match client.get(GITHUB_API).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(s) = json["tag_name"].as_str() {
                        return Ok(s.to_string());
                    }
                }
            }
            Err(e) => {
                if attempts < MAX_RETRIES {
                    attempts += 1;
                    println!(
                        "  {} Connection error ({}). Retrying ({}/{})...",
                        style("ℹ").yellow(),
                        if e.is_timeout() { "Timeout" } else { "Network" },
                        attempts,
                        MAX_RETRIES
                    );
                } else {
                    return Err(anyhow::anyhow!("Failed to fetch latest tag after {} retries: {}", MAX_RETRIES, e));
                }
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}

/// Returns (url, filename) for the appropriate service file.
fn resolve_service_url(os: &TargetOs) -> (&'static str, String) {
    match os {
        TargetOs::Ubuntu | TargetOs::Debian => {
            (SERVICE_DEBIAN_URL, "x-ui.service".to_string())
        }
        TargetOs::Arch | TargetOs::Manjaro => {
            (SERVICE_ARCH_URL, "x-ui.service".to_string())
        }
        TargetOs::Alpine => {
            (RC_ALPINE_URL, "x-ui.rc".to_string())
        }
        _ => {
            (SERVICE_RHEL_URL, "x-ui.service".to_string())
        }
    }
}

/// Download a URL to a local file with a progress bar.
pub async fn download_with_progress(
    client: &reqwest::Client,
    url: &str,
    dest: &str,
    label: &str,
) -> Result<()> {
    let mut attempts = 0;
    
    loop {
        match download_with_progress_inner(client, url, dest, label).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                if attempts > 10 { // Give up after 10 full failures for actual files
                     return Err(e);
                }
                
                println!(
                    "  {} Download interrupted ({}). Retrying... ({})",
                    style("ℹ").yellow(),
                    e,
                    attempts
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
}

async fn download_with_progress_inner(
    client: &reqwest::Client,
    url: &str,
    dest: &str,
    label: &str,
) -> Result<()> {
    let mut existing_bytes = 0;
    if let Ok(metadata) = tokio::fs::metadata(dest).await {
        existing_bytes = metadata.len();
    }

    let mut req = client.get(url);
    if existing_bytes > 0 {
        req = req.header("Range", format!("bytes={}-", existing_bytes));
    }

    let resp = req.send().await?.error_for_status()?;
    let status = resp.status();
    
    // Total size is existing bytes + remaining bytes
    let total = existing_bytes + resp.content_length().unwrap_or(0);
    
    let mut file = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        OpenOptions::new().create(true).append(true).open(dest).await?
    } else {
        existing_bytes = 0; // Server didn't support Range, start from scratch
        File::create(dest).await?
    };

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.cyan} [{elapsed_precise}] [{bar:35.cyan/blue}] {bytes}/{total_bytes} {msg}",
        )?
        .progress_chars("█▓░"),
    );
    pb.set_message(label.to_string());
    pb.set_position(existing_bytes);

    let mut stream = resp.bytes_stream();
    let mut downloaded = existing_bytes;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!("✓ {}", label));
    Ok(())
}
