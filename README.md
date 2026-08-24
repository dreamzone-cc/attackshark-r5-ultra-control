# Attack Shark R5 Ultra Control Center (Linux) 🖱️⚡

[![CachyOS Tested](https://img.shields.io/badge/CachyOS-Validated-00d1b2?style=for-the-badge&logo=archlinux)](https://cachyos.org)
[![Arch Linux](https://img.shields.io/badge/Arch_Linux-Compatible-1793d1?style=for-the-badge&logo=archlinux)](https://archlinux.org)
[![Rust Native](https://img.shields.io/badge/Rust-Native_Performance-dea584?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Slint UI](https://img.shields.io/badge/Slint_UI-Hardware_Accelerated-27ae60?style=for-the-badge)](https://slint.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)

A standalone, high-performance, native Linux driver and GUI Control Center for the **Attack Shark R5 Ultra Gaming Mouse** (PAW3395 Sensor + Nordic MCU). Built from scratch in **Rust** and **Slint UI**, featuring direct non-root HID kernel communication, live hardware parameter manipulation, KDE Plasma 6 Wayland System Tray integration, and onboard EEPROM profile synchronization.

---

## ⚡ One-Line Quick Install (CachyOS / Arch Linux)

Install, compile, configure udev rules, and launch the background daemon with a single command:

```bash
curl -sSL https://raw.githubusercontent.com/dreamzone-cc/attackshark-r5-ultra-control/main/install.sh | bash
```

---

## 🌟 Key Features

- **🎯 6-Stage DPI Performance**:
  - Full configuration from 50 to 26,000 DPI.
  - Per-stage active selector with real-time mouse MCU sync.
  - Distinct per-stage color identifiers.
- **⚡ Polling Rate Selection**:
  - Instant hardware switching: `125Hz`, `250Hz`, `500Hz`, `1000Hz`, `2000Hz`, `4000Hz`, `8000Hz` (sub-millisecond latency).
- **🔬 Optical Sensor Tuning**:
  - **Lift-Off Distance (LOD)**: `1.0 mm (Low)` vs `2.0 mm (High)`.
  - **Debounce Delay**: `0 ms` to `30 ms` click response tuning.
  - **Motion Sync & Ripple Control**: Hardware-level trajectory smoothing toggles.
- **🌈 RGB Lighting & Color Engine**:
  - Multiple effect modes: `Off (Dark)`, `Static Color`, `Breathing`, `Neon Spectrum`, `Color Wave`.
  - Live interactive RGB color palette (Red, Green, Blue, Cyan, Yellow, Purple, White, Orange).
  - Real-time `Brightness (0–100%)` and `Effect Speed (1x–10x)` sliders.
- **🔋 Live Power & Battery Monitor**:
  - Live hardware register battery percentage read via HID feature reports.
  - Charging state detection (Fast Charging Type-C vs 2.4G Wireless Discharging).
  - Auto-Sleep delay slider (`1` to `30` minutes).
- **🕒 KDE Plasma 6 Wayland System Tray Integration**:
  - Native `StatusNotifierItem` applet next to your digital clock.
  - Live color-coded battery status dot.
  - Quick context menu: `Open Control Center`, `Refresh Status`, `Quick DPI Stage Switcher (Stages 1–6)`, `Quit Daemon`.
- **⚙️ Profile & Onboard Memory Persistence**:
  - Direct sync to mouse onboard EEPROM memory.
  - Local JSON configuration persistence at `~/.config/attackshark-control/config.json`.
  - Systemd user daemon for instant background start on boot.

---

## 🖥️ Graphical Interface Overview

The Control Center provides a modern, docked dark UI categorized into 5 functional areas:

1. **Dashboard**: Live hero battery card, power management slider, and instant performance badges.
2. **DPI & Sensor**: 6 DPI stage cards, 7 polling rate pills, LOD switcher, and debounce delay slider.
3. **RGB Lighting**: Mode selection, interactive color presets, and live brightness/speed controls.
4. **Button Remap**: Overview of all 6 programmable mouse buttons.
5. **Profiles & Settings**: Autostart toggle, onboard profile selector, and factory reset.

---

## 📦 Manual Installation & Building

### 1. Prerequisites (CachyOS / Arch Linux)

```bash
sudo pacman -S --needed base-devel rust cargo git fontconfig freetype2 libxkbcommon wayland
```

### 2. Clone and Build

```bash
git clone https://github.com/dreamzone-cc/attackshark-r5-ultra-control.git
cd attackshark-r5-ultra-control
cargo build --release
```

### 3. Run One-Click Installer

```bash
./install.sh
```

### 4. Build Arch / CachyOS Package via PKGBUILD

```bash
makepkg -si
```

---

## 🛠️ Hardware Protocol & Architecture

The Attack Shark R5 Ultra uses a Nordic MCU paired with a PixArt PAW3395 optical sensor. Communication occurs over USB Interface 2 via 65-byte HID feature reports (`Report ID 0x00`):

```text
Host -> Device (HIDIOCSFEATURE_65 / ioctl 0xC0414806)
[0x00, 0xA1, 0x00, 0x02, Func, 0x00, Opcode, Payload...]
```

### Opcode Summary Map

| Opcode | Function | Description / Payload |
| :--- | :--- | :--- |
| **`0x83`** | Battery Query | Reads live battery percentage (`byte[8]`) and charging state (`byte[9]`). |
| **`0x81`** | Firmware Version | Reads MCU firmware version (`byte[7..10]`). |
| **`0x01`** | Polling Rate | `0x00` (125Hz), `0x01` (250Hz), `0x02` (500Hz), `0x03` (1000Hz)... |
| **`0x08`** | Lift-Off Distance | `0x00` (1.0 mm Low), `0x01` (2.0 mm High). |
| **`0x0B`** | Debounce Delay | `0` to `30` ms delay. |
| **`0x1A`** | DPI Stage Config | Sets stage index (`0..5`), DPI High/Low bytes, and stage RGB colors. |
| **`0x13`** | RGB Lighting | Mode (`byte[7]`), Speed (`byte[8]`), Brightness (`byte[9]`), RGB (`byte[10..12]`). |

---

## 🔧 Managing Background Service

The control daemon runs as a lightweight user systemd service:

```bash
# Check service status and live logs
systemctl --user status attackshark-control.service

# Restart daemon
systemctl --user restart attackshark-control.service

# View real-time logs
journalctl --user -u attackshark-control.service -f
```

---

## 🗑️ Uninstallation

To cleanly remove all binaries, systemd units, and udev rules:

```bash
cd attackshark-r5-ultra-control
./uninstall.sh
```

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

Developed with ❤️ for the Linux & CachyOS Gaming Community.
