pkgname=hyprclip
pkgver=0.1.0            # ggf. anpassen
pkgrel=1
pkgdesc="Clipboard Manager mit GUI und Waybar-Integration"
arch=('x86_64')
url="https://github.com/XsnilzX/hyprclip"
license=('MIT')
depends=('wl-clipboard' 'waybar')
makedepends=('rust' 'cargo')
source=("git+$url")
sha256sums=('SKIP')

build() {
  cd "$pkgname"
  cargo build --release --locked
}

package() {
  cd "$pkgname"
  install -Dm755 target/release/hyprclip "$pkgdir/usr/bin/hyprclip"
  install -Dm644 systemd/hyprclip-watcher.service \
    "$pkgdir/usr/lib/systemd/user/hyprclip-watcher.service"
}
