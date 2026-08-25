#!/usr/bin/env bash
set -e

echo "Stopping daemon..."
systemctl --user stop glitch-r5u.service attackshark-control.service 2>/dev/null || true
systemctl --user disable glitch-r5u.service attackshark-control.service 2>/dev/null || true

echo "Removing files..."
sudo rm -f /usr/local/bin/glitch-r5u /usr/local/bin/attackshark-r5-ultra-control
sudo rm -f /etc/udev/rules.d/99-glitch-r5u.rules /etc/udev/rules.d/99-attackshark-r5.rules
rm -f "$HOME/.local/bin/glitch-r5u" "$HOME/.local/bin/attackshark-r5-ultra-control"
rm -f "$HOME/.config/systemd/user/glitch-r5u.service" "$HOME/.config/systemd/user/attackshark-control.service"
rm -f "$HOME/.config/autostart/glitch-r5u.desktop" "$HOME/.config/autostart/attackshark-control.desktop"
rm -f "$HOME/.local/share/applications/glitch-r5u.desktop" "$HOME/.local/share/applications/attackshark-control.desktop"

sudo udevadm control --reload-rules || true
sudo udevadm trigger || true
systemctl --user daemon-reload

echo "Uninstalled Glitch R5U successfully."
