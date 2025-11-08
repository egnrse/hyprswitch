# Maintainer: Elia Nitsche <nitscheelia at gmail dot com>
# Contributor: H3rmt

# This is a fork of the original hyprswitch as it got renamed to hyprshell!

pkgname=hyprswitch
# x-release-please-start-version
pkgver=4.0.0
# x-release-please-end
pkgrel=1
pkgdesc="A CLI/GUI that allows switching between windows in Hyprland"
arch=('any')
url="https://github.com/egnrse/hyprswitch/"
license=("MIT")
makedepends=('cargo')
depends=('hyprland' 'gtk4-layer-shell' 'gtk4')
source=("$pkgname-$pkgver.tar.gz::https://github.com/egnrse/hyprswitch/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('2133feabc060b2e58608927ac8126b98bedbb311f764444e6a336582d0a90aa0')

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cd "$pkgname-$pkgver"
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cd "$pkgname-$pkgver"
    cargo build --frozen --release
}

package() {
    install -Dm0755 -t "$pkgdir/usr/bin/" "$pkgname-$pkgver/target/release/$pkgname"
}

post_install() {
    echo "Please restart the hyprswitch daemon"
}
