# 3x-ui Offline Installer Builder
[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)]()
[![Rust](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)

**3x-ui Offline Installer Builder** is a high-performance, resilient tool designed to build fully self-contained installation bundles for the 3x-ui panel, specifically for air-gapped and restricted network environments.

🇮🇷 [Read in Persian (فارسی)](README_FA.md) 🇮🇷

---

## 🚀 Introduction
This tool eliminates the need for internet access on target servers by pre-packaging everything—from system dependencies (.deb, .rpm, .apk) to the panel binaries and SSL certificates—into a single, executable shell script. 

### Key Hardening Features:
- **🧠 Smart Update Engine:** Detects existing installations and offers safe updates (preserving DB/users) or clean reinstalls.
- **🌐 Multi-Repo Discovery:** Intelligent scraping of multiple streams (BaseOS/AppStream) for RHEL-based resilience.
- **🔄 Infinite Network Resilience:** Bullet-proof downloader that continuously retries partial downloads instead of failing, ensuring successful bundling even on highly unstable or restricted networks.
- **🔒 Advanced SSL Management:** 
  - Automated **Self-Signed** generation with **Dynamic mode** (defer generation to target server to match its IP automatically).
  - Built-in **Let's Encrypt DNS-01 Challenge** (Generates trusted certs natively utilizing DoH over SOCKS5/HTTP proxy).
  - Custom certificate bundling.

---

## ⚡ Quick Start

### 1. Download
Get the latest pre-built binary for your operating system from the [Releases Page](https://github.com/Fox-Fig/mhsanaei-3x-ui-offline-installer/releases).

### 2. Run the Builder
Execute the builder on a machine with internet access.

**Linux / macOS:**
```bash
chmod +x xui-offline-builder
./xui-offline-builder
```

**Windows:**
Double-click `xui-offline-builder-windows.exe` or run via PowerShell:
```powershell
.\xui-offline-builder-windows.exe
```

### 3. Deploy to Server
Transfer the generated bundle to your target server. The output will be a single `.sh` file (the name depends on the bundle name you chose during the wizard).
```bash
# Run on the target server (No internet required)
bash YOUR_BUNDLE_NAME.sh
```

---

## 📋 Compatibility Matrix

| Distribution | Support | Mode |
| :--- | :--- | :--- |
| **Ubuntu / Debian** | Full (.deb) | 🟢 100% Offline |
| **Rocky / Alma / CentOS** | Base + AppStream | 🟢 100% Offline |
| **Fedora (v39-v44)** | Full (.rpm) | 🟢 100% Offline |
| **Alpine Linux** | Full (.apk) | 🟢 100% Offline |
| **Arch / Manjaro** | Rolling | 🟡 Hybrid |

---

## ❤️ Support & Donate
If you find this project useful, please consider donating to support development and server costs.

| Currency | Address |
| :--- | :--- |
| **Ethereum (ETH)** | `0xb59993FeCace98BF6b89a216f5ca1776028A7047` |
| **Bitcoin (BTC)** | `bc1qx28s2sz3nvhelclpgan24ymflssql8uzcmexn3` |
| **Ripple (XRP)** | `rHoTVZWrPhYWf4uHkHZFicrJsADp57Yq4g` |
| **USDT / TRX (TRC20)** | `TXKnT3drzW4kb7imKrr1DVfwZWkrQWWpJo` |
| **Toncoin (TON)** | `UQBfP7DC-SJZT7aITPIGacrm09H6b_thlSOzc_5zesnBYMBI` |

---

## 📄 License
This project is licensed under the [GPLv3 License](LICENSE).

---

<div align="center">
  Made with ❤️ by <a href="https://t.me/FoxFig">FoxFig Team</a><br>
  Dedicated to all people of Iran 🇮🇷
</div>
