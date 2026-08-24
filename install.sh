#!/usr/bin/env bash
set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}  Attack Shark R5 Ultra Control Center Installer      ${NC}"
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
    BUILD_DIR="$(mktemp -d /tmp/attackshark-build-XXXXXX)"
    git clone --depth 1 "$REPO_URL" "$BUILD_DIR"
    SRC_DIR="$BUILD_DIR"
fi

cd "$SRC_DIR"

# 1. Build release binary if not present
if [ ! -f "target/release/attackshark-r5-ultra-control" ]; then
    echo -e "${BLUE}[1/5] Building release binary with Cargo...${NC}"
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}[!] Cargo not found. Attempting to install Rust toolchain...${NC}"
        if command -v pacman &> /dev/null; then
            sudo pacman -S --needed --noconfirm rust cargo
        else
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
        fi
    fi
    cargo build --release
fi

# 2. Install binary
echo -e "${BLUE}[2/5] Installing binary to /usr/local/bin and ~/.local/bin...${NC}"
mkdir -p "$HOME/.local/bin"
cp -f target/release/attackshark-r5-ultra-control "$HOME/.local/bin/attackshark-r5-ultra-control"
chmod +x "$HOME/.local/bin/attackshark-r5-ultra-control"

if command -v sudo &> /dev/null; then
    sudo install -Dm755 target/release/attackshark-r5-ultra-control /usr/local/bin/attackshark-r5-ultra-control
fi

# 3. Configure udev rules
echo -e "${BLUE}[3/5] Installing non-root udev rules...${NC}"
if command -v sudo &> /dev/null; then
    sudo install -Dm644 99-attackshark-r5.rules /etc/udev/rules.d/99-attackshark-r5.rules
    sudo udevadm control --reload-rules || true
    sudo udevadm trigger || true
fi

# 4. Install Desktop Shortcut & App Icon
echo -e "${BLUE}[4/5] Installing Desktop Shortcut & Icon...${NC}"
if command -v sudo &> /dev/null; then
    sudo install -Dm644 resources/icon.png /usr/share/icons/hicolor/128x128/apps/attackshark-battery.png
fi
mkdir -p "$HOME/.local/share/applications" "$HOME/.config/autostart"
cp -f attackshark-control.desktop "$HOME/.local/share/applications/attackshark-control.desktop"
cp -f attackshark-control.desktop "$HOME/.config/autostart/attackshark-control.desktop"

# 5. Configure & Start Systemd User Service
echo -e "${BLUE}[5/5] Registering and starting background service...${NC}"
mkdir -p "$HOME/.config/systemd/user"
cp -f attackshark-control.service "$HOME/.config/systemd/user/attackshark-control.service"
systemctl --user daemon-reload
systemctl --user enable --now attackshark-control.service

# Clean temporary directory if cloned
if [ -n "$BUILD_DIR" ] && [ -d "$BUILD_DIR" ]; then
    rm -rf "$BUILD_DIR"
fi

echo -e "\n${GREEN}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}  Attack Shark R5 Ultra Control Center Ready! 🚀       ${NC}"
echo -e "${BLUE}  Status: $(systemctl --user is-active attackshark-control.service)${NC}"
echo -e "${GREEN}${BOLD}======================================================${NC}\n"
