#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:-$ROOT/assets/macos/layers/master-1024.png}"
SET="$ROOT/assets/macos/AppIcon.iconset"
ICNS="$ROOT/assets/macos/AppIcon.icns"
rm -rf "$SET"
mkdir -p "$SET"
for sz in 16 32 128 256 512; do
  sips -z "$sz" "$sz" "$SRC" --out "$SET/icon_${sz}x${sz}.png" >/dev/null
  sips -z $((sz * 2)) $((sz * 2)) "$SRC" --out "$SET/icon_${sz}x${sz}@2x.png" >/dev/null
done
iconutil -c icns "$SET" -o "$ICNS"
ls -la "$ICNS"