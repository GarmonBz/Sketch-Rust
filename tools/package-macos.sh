#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="${HOME}/.cargo/bin:${PATH}"
export ASDF_RUST_VERSION="${ASDF_RUST_VERSION:-1.97.0}"

# Cargo-bundle resolves [package.metadata.bundle].icon relative to this crate's
# Cargo.toml (crates/sketch-app), hence ../../assets/macos/AppIcon.icns.

SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    -h|--help)
      echo "Usage: $0 [--skip-build]"
      echo "  Produces dist/macos/Sketch Rust.app (Apple Silicon)."
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! command -v cargo-bundle >/dev/null 2>&1; then
  cargo install cargo-bundle --locked --version 0.12.0
fi

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  cargo build --release --locked -p sketch-app --target aarch64-apple-darwin
else
  test -x target/aarch64-apple-darwin/release/sketch-app \
    || { echo "Missing release binary; run without --skip-build" >&2; exit 1; }
fi

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
