pub mod network;
pub mod xui;
pub mod packages;
pub mod ssl;

use anyhow::Result;
use crate::manifest::{Manifest, STEP_PACKAGES, STEP_SSL};
use crate::wizard::state::{BuildConfig, PackageMode, SslConfig};

/// Download / generate all required files into the output directory.
/// Uses manifest to skip already-completed steps (resume support).
pub async fn download_all(config: &BuildConfig, manifest: &mut Manifest) -> Result<()> {
    let out = &config.output_dir;
    std::fs::create_dir_all(out)?;

    // 1. x-ui binary + CLI + service file (with resume support)
    xui::download(config, out, manifest).await?;

    // 2. System packages (only in offline mode, with resume support)
    if config.package_mode == PackageMode::Offline {
        let pkg_dir = format!("{}/packages", out);
        std::fs::create_dir_all(&pkg_dir)?;
        packages::download(config, &pkg_dir, out, manifest).await?;
    } else {
        // Mark packages as done with empty list (online mode = not needed)
        if !manifest.step_is_done(STEP_PACKAGES) {
            manifest.mark_done(out, STEP_PACKAGES, vec![])?;
        }
    }

    // 3. SSL files (with resume support)
    if manifest.step_is_valid(out, STEP_SSL) {
        println!("  {} SSL — Already exists, skipping.", console::style("⏭️").dim());
    } else {
        match &config.ssl {
            SslConfig::None => {
                // No SSL needed — mark done with empty files
                manifest.mark_done(out, STEP_SSL, vec![])?;
            }
            SslConfig::Custom { fullchain_path, privkey_path } => {
                ssl::copy_custom(fullchain_path, privkey_path, out)?;
                manifest.mark_done(out, STEP_SSL, vec![
                    "ssl/fullchain.pem".to_string(),
                    "ssl/privkey.pem".to_string(),
                ])?;
            }
            SslConfig::SelfSigned { common_name, dynamic } => {
                if *dynamic {
                    println!("  {} SSL — Dynamic generation requested, deferring to target server.", console::style("⏭️").dim());
                    manifest.mark_done(out, STEP_SSL, vec![])?;
                } else {
                    ssl::generate_self_signed(common_name, out)?;
                    manifest.mark_done(out, STEP_SSL, vec![
                        "ssl/fullchain.pem".to_string(),
                        "ssl/privkey.pem".to_string(),
                    ])?;
                }
            }
            SslConfig::LetsEncrypt { domain } => {
                ssl::generate_lets_encrypt(domain, out, &config.proxy).await?;
                manifest.mark_done(out, STEP_SSL, vec![
                    "ssl/fullchain.pem".to_string(),
                    "ssl/privkey.pem".to_string(),
                ])?;
            }
        }
    }

    Ok(())
}
