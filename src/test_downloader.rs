use anyhow::Result;
use console::style;
use crate::wizard::state::{BuildConfig, TargetOs, TargetArch, PackageMode, XuiVersion, OutputKind, SslConfig};
use crate::downloader::packages;
use crate::proxy;

pub async fn test_all_mirrors() -> Result<()> {
    println!("{}", style("━".repeat(54)).cyan());
    println!("{}", style("  🔍  Testing Package Mirrors and Parsers...").cyan().bold());
    println!("{}", style("━".repeat(54)).cyan());
    println!();

    let oss = vec![
        TargetOs::Ubuntu,
        TargetOs::Debian,
        TargetOs::AlmaLinux,
        TargetOs::Rocky,
        TargetOs::Fedora,
        TargetOs::Alpine,
    ];

    let client = proxy::build_client(&None)?;

    for os in oss {
        let display = os.display_name();
        print!("  {} Testing {:<20} ", style("→").dim(), style(display).bold());
        
        let config = BuildConfig {
            os: os.clone(),
            arch: TargetArch::Amd64,
            os_version: None,
            package_mode: PackageMode::Offline,
            server_host: "1.1.1.1".to_string(),
            xui_version: XuiVersion::Latest,
            panel_port: 54321,
            panel_username: "admin".to_string(),
            panel_password: "admin".to_string(),
            panel_web_base_path: "/".to_string(),
            ssl: SslConfig::None,
            proxy: None,
            output_dir: "/tmp/test".to_string(),
            output_kind: OutputKind::Folder,
        };

        // We only test the first package to verify URL extraction
        let packages = crate::os_detect::required_packages(&os);
        let pkg = packages[0];

        let mirror = crate::os_detect::mirror_info(&os).unwrap();
        
        use crate::os_detect::PkgFormat;
        let res = match mirror.format {
            PkgFormat::Deb => packages::download_deb_test(&client, pkg, config.arch.deb_arch()).await,
            PkgFormat::Rpm => packages::download_rpm_test(&client, pkg, mirror.mirror_base).await,
            PkgFormat::Apk => packages::download_apk_test(&client, pkg, mirror.mirror_base).await,
        };

        match res {
            Ok(url) => {
                println!("{} URL: {}", style("✓").green(), style(url).dim());
            }
            Err(e) => {
                println!("{} Failed: {}", style("✗").red(), style(e).red());
            }
        }
    }

    println!();
    println!("{}", style("  ✅  Mirror testing complete.").green().bold());
    Ok(())
}
