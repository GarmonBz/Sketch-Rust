#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="${HOME}/.cargo/bin:${PATH}"
export ASDF_RUST_VERSION="${ASDF_RUST_VERSION:-1.97.0}"

# cargo-bundle resolves [package.metadata.bundle].icon relative to this crate's
# Cargo.toml (crates/sketch-app), hence ../../assets/macos/AppIcon.icns.

if ! command -v cargo-bundle >/dev/null 2>&1; then
  cargo install cargo-bundle --locked --version 0.12.0
fi

cargo build --release --locked -p sketch-app --target aarch64-apple-darwin

(
  cd crates/sketch-app
  cargo bundle --release --target aarch64-apple-darwin
)

APP_SRC=$(find target/aarch64-apple-darwin/release/bundle -name '*.app' | head -1)
mkdir -p dist/macos
rm -rf "dist/macos/Sketch Rust.app"
cp -R "$APP_SRC" "dist/macos/Sketch Rust.app"

MACOS_DIR="dist/macos/Sketch Rust.app/Contents/MacOS"
cp -f assets/symbols.json "$MACOS_DIR/symbols.json"
cp -f assets/symbols.json "$MACOS_DIR/symbols.original.json"
chmod a-w "$MACOS_DIR/symbols.original.json"
cp -f assets/symbols.previous.json "$MACOS_DIR/symbols.previous.json" 2>/dev/null || true

file "$MACOS_DIR/sketch-app"
test -f "$MACOS_DIR/symbols.json"
# CoreBluetooth kills a bundled app that lacks this key when DISTO scans.
plutil -extract NSBluetoothAlwaysUsageDescription raw \
  "dist/macos/Sketch Rust.app/Contents/Info.plist" >/dev/null
echo "Packaged: dist/macos/Sketch Rust.app"