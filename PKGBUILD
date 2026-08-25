# Maintainer: dreamzone-cc <https://github.com/dreamzone-cc>
pkgname=glitch-r5u-git
_pkgname=glitch-r5u
pkgver=1.1.0
pkgrel=1
pkgdesc="Glitch R5U: Native Linux Control Suite for Attack Shark R5 Ultra Gaming Mouse (Rust + Slint UI)"
arch=('x86_64')
url="https://github.com/dreamzone-cc/attackshark-r5-ultra-control"
license=('MIT')
depends=('fontconfig' 'freetype2' 'libxkbcommon' 'wayland')
makedepends=('cargo' 'git')
provides=('glitch-r5u' 'attackshark-r5-ultra-control')
conflicts=('glitch-r5u' 'attackshark-r5-ultra-control')
install="glitch-r5u.install"
source=("git+https://github.com/dreamzone-cc/attackshark-r5-ultra-control.git")
sha256sums=('SKIP')

build() {
    cd "${srcdir}/attackshark-r5-ultra-control"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --release --locked
}

package() {
    cd "${srcdir}/attackshark-r5-ultra-control"
    install -Dm755 "target/release/glitch-r5u" "${pkgdir}/usr/bin/glitch-r5u"
    ln -sf "/usr/bin/glitch-r5u" "${pkgdir}/usr/bin/attackshark-r5-ultra-control"
    install -Dm644 "99-attackshark-r5.rules" "${pkgdir}/usr/lib/udev/rules.d/99-glitch-r5u.rules"
    install -Dm644 "glitch-r5u.service" "${pkgdir}/usr/lib/systemd/user/glitch-r5u.service"
    install -Dm644 "glitch-r5u.desktop" "${pkgdir}/usr/share/applications/glitch-r5u.desktop"
    install -Dm644 "glitch-r5u.desktop" "${pkgdir}/etc/xdg/autostart/glitch-r5u.desktop"
    install -Dm644 "resources/icon.svg" "${pkgdir}/usr/share/icons/hicolor/scalable/apps/glitch-r5u.svg"
    install -Dm644 "resources/icon.png" "${pkgdir}/usr/share/icons/hicolor/256x256/apps/glitch-r5u.png"
}
