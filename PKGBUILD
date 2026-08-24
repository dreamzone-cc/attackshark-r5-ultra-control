# Maintainer: dreamzone-cc <https://github.com/dreamzone-cc>
pkgname=attackshark-r5-ultra-control-git
_pkgname=attackshark-r5-ultra-control
pkgver=1.0.0
pkgrel=1
pkgdesc="Comprehensive Linux Control Center for Attack Shark R5 Ultra Mouse (Rust + Slint UI + KDE System Tray)"
arch=('x86_64')
url="https://github.com/dreamzone-cc/attackshark-r5-ultra-control"
license=('MIT')
depends=('fontconfig' 'freetype2' 'libxkbcommon' 'wayland')
makedepends=('cargo' 'git')
provides=('attackshark-r5-ultra-control')
conflicts=('attackshark-r5-ultra-control')
install="attackshark-control.install"
source=("git+https://github.com/dreamzone-cc/attackshark-r5-ultra-control.git")
sha256sums=('SKIP')

build() {
    cd "${srcdir}/${_pkgname}"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --release --locked
}

package() {
    cd "${srcdir}/${_pkgname}"
    install -Dm755 "target/release/attackshark-r5-ultra-control" "${pkgdir}/usr/bin/attackshark-r5-ultra-control"
    install -Dm644 "99-attackshark-r5.rules" "${pkgdir}/usr/lib/udev/rules.d/99-attackshark-r5.rules"
    install -Dm644 "attackshark-control.service" "${pkgdir}/usr/lib/systemd/user/attackshark-control.service"
    install -Dm644 "attackshark-control.desktop" "${pkgdir}/usr/share/applications/attackshark-control.desktop"
    install -Dm644 "attackshark-control.desktop" "${pkgdir}/etc/xdg/autostart/attackshark-control.desktop"
    install -Dm644 "resources/icon.png" "${pkgdir}/usr/share/icons/hicolor/128x128/apps/attackshark-battery.png"
}
