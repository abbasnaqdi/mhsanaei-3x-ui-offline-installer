# 🚀 3x-ui Offline Installer Builder (Rust)

A powerful, interactive Rust-based tool designed to build **fully self-contained, offline installation bundles** for the 3x-ui panel. It packages all binaries, system dependencies, and configurations into a single executable shell script, making deployments on network-restricted (air-gapped) servers effortless.

---

## ✨ Key Features

- **📦 Single-File SFX:** Generates a single `.sh` file that contains the entire installation bundle (using Binary Append). No need to transfer multiple folders.
- **🌐 True Offline Mode:** Automatically downloads required system packages (`.deb`, `.rpm`, `.apk`) from official mirrors for 8 different Linux distributions.
- **⚡ Intelligent Resume & Verify:** Automatically detects existing bundles. It verifies file integrity using **SHA256** and allows you to resume failed downloads or edit settings (Port/SSL) without re-downloading heavy assets.
- **🔒 Advanced SSL Management:** Automatically generates Self-Signed certificates based on the target server's IP/Domain or allows the use of custom certificates.
- **🛠️ Fully Configurable:** Customize Port, Username, Password, and WebBasePath during the build phase.
- **🔌 Proxy Support:** Built-in SOCKS5 and HTTP proxy support for environments with restricted internet access during the bundling phase.
- **🖥️ Multi-Platform Builder:** Pre-built binaries available for **Linux, Windows, and macOS**.

---

## 🚀 How to Use

### 1. Get the Tool
You can either download the pre-built binary from the [Releases](https://github.com/MHSanaei/3x-ui-offline-installer/releases) page or build it from source:
```bash
cargo build --release
```

### 2. Generate a Bundle
Run the builder and follow the interactive wizard:
```bash
./target/release/xui-offline-builder
```
At the end of the process, you will have a single installer file (e.g., `xui-bundle.sh`).

### 3. Deploy to Target Server
Transfer the generated file to your destination server and run it:
```bash
# Transfer to server
scp xui-bundle.sh root@YOUR_SERVER_IP:/root/

# Run the installer (No internet required on the server)
bash xui-bundle.sh
```

---

## 📋 Supported Distributions
- **Debian / Ubuntu** (Full Offline Support)
- **CentOS / RHEL / Rocky / AlmaLinux** (Full Offline Support)
- **Alpine Linux** (Full Offline Support)
- **Arch / OpenSUSE** (Online package installation)

---

## 🛠️ CI/CD & Versioning
This project uses GitHub Actions for automated multi-platform builds.
- **Versioning:** Follows `Vx.x.x` format based on Git tags and `Cargo.toml`.
- **Automatic Releases:** Every tag push or manual trigger creates a new release with binaries for Linux, Windows, and macOS, including an automated changelog from the latest commit.

---

## 🛡️ Security
The builder generates random credentials and secure paths by default. It also provides a clear **Access URL** at the end of the installation to ensure you can access your panel immediately.

---
**Developed with ❤️ for the Open Source community.**
