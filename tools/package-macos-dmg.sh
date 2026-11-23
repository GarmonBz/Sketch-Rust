#!/usr/bin/env bash
# Build Sketch Rust.app (via package-macos.sh) and wrap it in a UDZO .dmg under dist/.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-build) SKIP_BUILD=1; shift ;;
    -h|--help)
      echo "Usage: $0 [--skip-build]"
      echo "  Produces dist/SketchRust-<version>-macos-arm64.dmg (+ .sha256)."
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)"
if [[ -z "${VERSION}" ]]; then
  echo "version not found in Cargo.toml" >&2
  exit 1
fi

APP="dist/macos/Sketch Rust.app"
if [[ "$SKIP_BUILD" -eq 1 ]]; then
  ./tools/package-macos.sh --skip-build
else
  ./tools/package-macos.sh
fi

test -d "$APP"

STAGE="$(mktemp -d "${TMPDIR:-/tmp}/sketch-macos-dmg.XXXXXX")"
cleanup() { rm -rf "$STAGE"; }
trap cleanup EXIT

cp -R "$APP" "$STAGE/"

mkdir -p dist
DMG_NAME="SketchRust-${VERSION}-macos-arm64.dmg"
DMG_PATH="dist/${DMG_NAME}"
rm -f "$DMG_PATH"

# Compressed read-only image; double-click → drag .app to Applications (or run in place).
hdiutil create \
  -volname "Sketch Rust" \
  -srcfolder "$STAGE" \
  -ov \
  -format UDZO \
  "$DMG_PATH"

shasum -a 256 "$DMG_PATH" | awk '{print $1 "  '"$DMG_NAME"'"}' >"${DMG_PATH}.sha256"

echo "Packaged: $DMG_PATH"
echo "Checksum: ${DMG_PATH}.sha256"
file "$DMG_PATH"
