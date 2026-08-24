#!/usr/bin/env bash
set -e

echo "Stopping daemon..."
systemctl --user stop attackshark-control.service 2>/dev/null || true
systemctl --user disable attackshark-control.service 2>/dev/null || true

echo "Removing files..."
sudo rm -f /usr/local/bin/attackshark-r5-ultra-control
sudo rm -f /etc/udev/rules.d/99-attackshark-r5.rules
rm -f "$HOME/.config/systemd/user/attackshark-control.service"
rm -f "$HOME/.config/autostart/attackshark-control.desktop"
rm -f "$HOME/.local/share/applications/attackshark-control.desktop"

sudo udevadm control --reload-rules
sudo udevadm trigger
systemctl --user daemon-reload

echo "Uninstalled successfully."
