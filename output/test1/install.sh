#!/bin/bash
# ============================================================
# 3x-ui Offline Installer — Customized by xui-offline-builder
# Target OS      : Ubuntu
# Architecture   : amd64
# ============================================================
set -e

red='\033[0;31m'
green='\033[0;32m'
blue='\033[0;34m'
yellow='\033[0;33m'
plain='\033[0m'

# Bundle path
BUNDLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

xui_folder="/usr/local/x-ui"
xui_service="/etc/systemd/system"

# ── Root Check ──────────────────────────────────────────────
[[ $EUID -ne 0 ]] && echo -e "${red}Error: This script must be run as root.${plain}" && exit 1

echo -e "${green}Starting 3x-ui installation (Offline version)...${plain}"

# ── System Package Installation ─────────────────────────────
install_base_offline() {
    echo "Installing packages from offline bundle..."
    cd "$BUNDLE_DIR"
    dpkg -i ./packages/*.deb 2>/dev/null || true
    apt-get install -f -y -q 2>/dev/null || true
}
install_base_online() {
    echo "Online fallback for missing packages..."
    apt-get update && apt-get install -y -q cron curl tar tzdata socat ca-certificates openssl || true
}

install_base_offline

# ── Stopping Previous Service ───────────────────────────────
systemctl stop x-ui 2>/dev/null || true
rm -rf "$xui_folder" 2>/dev/null || true

# ── Extracting x-ui Binary ──────────────────────────────────
echo -e "${green}Installing x-ui binary...${plain}"
mkdir -p "$(dirname "$xui_folder")"
tar zxf "$BUNDLE_DIR/x-ui-linux-amd64.tar.gz" -C "$(dirname "$xui_folder")"
mv "$(dirname "$xui_folder")/x-ui" "$xui_folder" 2>/dev/null || true
chmod +x "$xui_folder/x-ui"
chmod +x "$xui_folder/x-ui.sh"
chmod +x "$xui_folder/bin/"* 2>/dev/null || true

# ── Installing CLI manager ──────────────────────────────────
cp "$BUNDLE_DIR/x-ui.sh" /usr/bin/x-ui
chmod +x /usr/bin/x-ui
mkdir -p /var/log/x-ui

# ── Panel Configuration ─────────────────────────────────────
echo -e "${green}Configuring panel settings...${plain}"
"$xui_folder/x-ui" setting -username "E70ymP2R" -password "PaRsn109mz" -port "61436" -webBasePath "qzycqKvM4wmI" > /dev/null 2>&1

# ── SSL Configuration ───────────────────────────────────────
setup_ssl() {
    local cert_dest="/root/cert/bundle"
    mkdir -p "$cert_dest"
    cp "$BUNDLE_DIR/ssl/fullchain.pem" "$cert_dest/fullchain.pem"
    cp "$BUNDLE_DIR/ssl/privkey.pem"   "$cert_dest/privkey.pem"
    chmod 644 "$cert_dest/fullchain.pem"
    chmod 600 "$cert_dest/privkey.pem"
    /usr/local/x-ui/x-ui cert \
        -webCert "$cert_dest/fullchain.pem" \
        -webCertKey "$cert_dest/privkey.pem" > /dev/null 2>&1 || true
    echo "  SSL certificate installed"
}

setup_ssl

# ── Service Installation & Activation ───────────────────────
echo -e "${green}Activating x-ui service...${plain}"
cp "$BUNDLE_DIR/x-ui.service" /etc/systemd/system/x-ui.service
chown root:root /etc/systemd/system/x-ui.service
chmod 644 /etc/systemd/system/x-ui.service
systemctl daemon-reload
systemctl enable x-ui
systemctl start x-ui

# etckeeper compatibility
if [ -d "/etc/.git" ]; then
    echo "x-ui/x-ui.db" >> /etc/.gitignore 2>/dev/null || true
fi

echo ""
echo -e "${green}╔════════════════════════════════════════════════════════════╗${plain}"
echo -e "${green}║                3x-ui installed successfully!               ║${plain}"
echo -e "${green}╠════════════════════════════════════════════════════════════╣${plain}"
echo -e "${green}║ Username:      E70ymP2R                                    ║${plain}"
echo -e "${green}║ Password:      PaRsn109mz                                  ║${plain}"
echo -e "${green}║ Port:          61436                                       ║${plain}"
echo -e "${green}║ WebPath:       qzycqKvM4wmI                                ║${plain}"
echo -e "${green}║ Access Link:   https://87.107.109.79:61436/qzycqKvM4wmI    ║${plain}"
echo -e "${green}╚════════════════════════════════════════════════════════════╝${plain}"
echo -e "${yellow}⚠ Keep this information secure!${plain}"
echo ""
echo -e "Management Commands:"
echo -e "  x-ui start / stop / restart / status / log"
