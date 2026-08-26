#!/usr/bin/env bash
set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}                 Glitch R5U Installer                 ${NC}"
echo -e "${BLUE}${BOLD}  Native Linux • Rust • Slint UI • CachyOS / Arch     ${NC}"
echo -e "${BLUE}${BOLD}======================================================${NC}\n"

if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}[ERROR] Please do not run as root directly. Run as regular user.${NC}"
    exit 1
fi

REPO_URL="https://github.com/dreamzone-cc/attackshark-r5-ultra-control.git"
BUILD_DIR=""

# Check if running from cloned repository or piped from curl
if [ -f "Cargo.toml" ] && [ -f "ui/appwindow.slint" ]; then
    SRC_DIR="$(pwd)"
else
    echo -e "${BLUE}[*] Piped installer detected. Cloning repository...${NC}"
    BUILD_DIR="$(mktemp -d /tmp/glitch-r5u-build-XXXXXX)"
    git clone --depth 1 "$REPO_URL" "$BUILD_DIR"
    SRC_DIR="$BUILD_DIR"
fi

cd "$SRC_DIR"

# 1. Build release binary if not present
if [ ! -f "target/release/glitch-r5u" ]; then
    echo -e "${BLUE}[1/5] Building release binary with Cargo...${NC}"
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}[!] Cargo not found. Attempting to install Rust toolchain...${NC}"
        if command -v pacman &> /dev/null; then
            if sudo -n true 2>/dev/null; then
                sudo pacman -S --needed --noconfirm rust cargo
            else
                echo -e "${YELLOW}[!] Please install rust and cargo via: sudo pacman -S rust cargo${NC}"
            fi
        fi
    fi
    cargo build --release
fi

# 2. Install binary to user path
echo -e "${BLUE}[2/5] Installing binary to ~/.local/bin and /usr/local/bin...${NC}"
mkdir -p "$HOME/.local/bin"
cp -f target/release/glitch-r5u "$HOME/.local/bin/glitch-r5u"
chmod +x "$HOME/.local/bin/glitch-r5u"
ln -sf "$HOME/.local/bin/glitch-r5u" "$HOME/.local/bin/attackshark-r5-ultra-control"

if sudo -n true 2>/dev/null; then
    sudo install -Dm755 target/release/glitch-r5u /usr/local/bin/glitch-r5u 2>/dev/null || true
    sudo ln -sf /usr/local/bin/glitch-r5u /usr/local/bin/attackshark-r5-ultra-control 2>/dev/null || true
fi

# 3. Configure udev rules
echo -e "${BLUE}[3/5] Installing udev rules...${NC}"
if sudo -n true 2>/dev/null; then
    sudo install -Dm644 99-attackshark-r5.rules /etc/udev/rules.d/99-glitch-r5u.rules 2>/dev/null || true
    sudo udevadm control --reload-rules 2>/dev/null || true
    sudo udevadm trigger 2>/dev/null || true
fi

# 4. Install Icons & Desktop Shortcut
echo -e "${BLUE}[4/5] Configuring system icons & desktop autostart...${NC}"
mkdir -p "$HOME/.local/share/icons/hicolor/scalable/apps" "$HOME/.local/share/icons/hicolor/256x256/apps" "$HOME/.local/share/icons/hicolor/128x128/apps" "$HOME/.local/share/icons/hicolor/64x64/apps"
cp -f resources/icon.svg "$HOME/.local/share/icons/hicolor/scalable/apps/glitch-r5u.svg"
cp -f resources/icon.png "$HOME/.local/share/icons/hicolor/256x256/apps/glitch-r5u.png"
cp -f resources/icon_128.png "$HOME/.local/share/icons/hicolor/128x128/apps/glitch-r5u.png"
cp -f resources/icon_64.png "$HOME/.local/share/icons/hicolor/64x64/apps/glitch-r5u.png"

# Also link legacy icon name for safety
cp -f resources/icon.png "$HOME/.local/share/icons/hicolor/256x256/apps/attackshark-battery.png"

mkdir -p "$HOME/.local/share/applications" "$HOME/.config/autostart"
cp -f glitch-r5u.desktop "$HOME/.local/share/applications/glitch-r5u.desktop"
sed "s|^Exec=.*|Exec=$INSTALL_BIN --daemon|" glitch-r5u.desktop > "$HOME/.config/autostart/glitch-r5u.desktop"

# Clean legacy autostart desktop file
rm -f "$HOME/.config/autostart/attackshark-control.desktop"

# 5. Configure & Enable Systemd User Service on Boot
echo -e "${BLUE}[5/5] Enabling background daemon to start automatically on system boot...${NC}"
mkdir -p "$HOME/.config/systemd/user"
cp -f glitch-r5u.service "$HOME/.config/systemd/user/glitch-r5u.service"

# Clean legacy service unit
systemctl --user stop attackshark-control.service 2>/dev/null || true
systemctl --user disable attackshark-control.service 2>/dev/null || true
rm -f "$HOME/.config/systemd/user/attackshark-control.service"

systemctl --user daemon-reload
systemctl --user enable --now glitch-r5u.service

# Clean temporary directory if cloned
if [ -n "$BUILD_DIR" ] && [ -d "$BUILD_DIR" ]; then
    rm -rf "$BUILD_DIR"
fi

echo -e "\n${GREEN}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}                 Glitch R5U Ready! 🚀                 ${NC}"
echo -e "${BLUE}  Autostart on Boot: ENABLED (systemd + XDG autostart)${NC}"
echo -e "${GREEN}${BOLD}======================================================${NC}\n"
