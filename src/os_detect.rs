use crate::wizard::state::{TargetOs, TargetArch};

/// Returns a human-readable package manager name for display.
pub fn package_manager(os: &TargetOs, _version: &Option<String>) -> String {
    match os {
        TargetOs::Ubuntu | TargetOs::Debian => "apt-get".to_string(),
        TargetOs::CentOs => "yum (v7) / dnf (v8+)".to_string(),
        TargetOs::Fedora | TargetOs::AlmaLinux | TargetOs::Rocky | TargetOs::Rhel => "dnf".to_string(),
        TargetOs::Alpine => "apk".to_string(),
        TargetOs::Arch | TargetOs::Manjaro => "pacman".to_string(),
        TargetOs::OpenSuse => "zypper".to_string(),
    }
}

/// Whether offline package download is supported for this OS.
pub fn supports_offline_packages(os: &TargetOs) -> bool {
    matches!(
        os,
        TargetOs::Ubuntu | TargetOs::Debian | TargetOs::AlmaLinux
            | TargetOs::Rocky | TargetOs::Rhel | TargetOs::Fedora
            | TargetOs::Alpine
    )
}

/// Returns the mirror base URL to query for package downloads.
pub struct PkgMirrorInfo {
    pub mirror_base: &'static str,
    pub format: PkgFormat,
}

#[derive(Debug, Clone)]
pub enum PkgFormat {
    Deb,
    Rpm,
    Apk,
}

pub fn mirror_info(os: &TargetOs) -> Option<PkgMirrorInfo> {
    match os {
        TargetOs::Ubuntu => Some(PkgMirrorInfo {
            mirror_base: "http://archive.ubuntu.com/ubuntu/pool/main",
            format: PkgFormat::Deb,
        }),
        TargetOs::Debian => Some(PkgMirrorInfo {
            mirror_base: "http://ftp.debian.org/debian/pool/main",
            format: PkgFormat::Deb,
        }),
        TargetOs::AlmaLinux | TargetOs::Rocky => Some(PkgMirrorInfo {
            mirror_base: "https://dl.rockylinux.org/pub/rocky/9/BaseOS/x86_64/os/Packages",
            format: PkgFormat::Rpm,
        }),
        TargetOs::Fedora => Some(PkgMirrorInfo {
            mirror_base: "https://dl.fedoraproject.org/pub/fedora/linux/releases/39/Everything/x86_64/os/Packages",
            format: PkgFormat::Rpm,
        }),
        TargetOs::Alpine => Some(PkgMirrorInfo {
            mirror_base: "https://dl-cdn.alpinelinux.org/alpine/v3.19/main/x86_64",
            format: PkgFormat::Apk,
        }),
        _ => None,
    }
}

/// Returns the list of packages the original install.sh installs per OS.
pub fn required_packages(os: &TargetOs) -> Vec<&'static str> {
    match os {
        TargetOs::Alpine => vec!["dcron", "curl", "tar", "tzdata", "socat", "ca-certificates", "openssl"],
        TargetOs::Arch | TargetOs::Manjaro => vec!["cronie", "curl", "tar", "tzdata", "socat", "ca-certificates", "openssl"],
        TargetOs::OpenSuse => vec!["cron", "curl", "tar", "timezone", "socat", "ca-certificates", "openssl"],
        TargetOs::AlmaLinux | TargetOs::Rocky | TargetOs::Rhel | TargetOs::Fedora | TargetOs::CentOs => {
            vec!["cronie", "curl", "tar", "tzdata", "socat", "ca-certificates", "openssl"]
        }
        _ => vec!["cron", "curl", "tar", "tzdata", "socat", "ca-certificates", "openssl"],
    }
}

/// Returns the shell snippet for the inline package-install command.
pub fn install_command_online(os: &TargetOs) -> String {
    match os {
        TargetOs::Ubuntu | TargetOs::Debian => {
            "apt-get update && apt-get install -y -q cron curl tar tzdata socat ca-certificates openssl".to_string()
        }
        TargetOs::CentOs => {
            r#"if [[ "${VERSION_ID}" =~ ^7 ]]; then
    yum -y update && yum install -y cronie curl tar tzdata socat ca-certificates openssl
else
    dnf -y update && dnf install -y -q cronie curl tar tzdata socat ca-certificates openssl
fi"#.to_string()
        }
        TargetOs::Fedora | TargetOs::AlmaLinux | TargetOs::Rocky | TargetOs::Rhel => {
            "dnf -y update && dnf install -y -q cronie curl tar tzdata socat ca-certificates openssl".to_string()
        }
        TargetOs::Alpine => {
            "apk update && apk add dcron curl tar tzdata socat ca-certificates openssl".to_string()
        }
        TargetOs::Arch | TargetOs::Manjaro => {
            "pacman -Syu && pacman -Syu --noconfirm cronie curl tar tzdata socat ca-certificates openssl".to_string()
        }
        TargetOs::OpenSuse => {
            "zypper refresh && zypper -q install -y cron curl tar timezone socat ca-certificates openssl".to_string()
        }
    }
}

/// Returns the offline install command (install from ./packages/)
pub fn install_command_offline(_os: &TargetOs, format: &PkgFormat) -> String {
    match format {
        PkgFormat::Deb => {
            r#"dpkg -i ./packages/*.deb 2>/dev/null || true
apt-get install -f -y -q 2>/dev/null || true"#.to_string()
        }
        PkgFormat::Rpm => {
            "rpm -Uvh ./packages/*.rpm 2>/dev/null || true".to_string()
        }
        PkgFormat::Apk => {
            "apk add --no-network --allow-untrusted ./packages/*.apk 2>/dev/null || true".to_string()
        }
    }
}

/// Returns xray binary arch suffix used by x-ui (same as TargetArch::xui_suffix but for the
/// rename logic in the original script).
pub fn xray_arch_rename_needed(arch: &TargetArch) -> bool {
    matches!(arch, TargetArch::Armv7)
}
