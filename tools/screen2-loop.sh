#!/usr/bin/env bash
# tools/screen2-loop.sh — Screen 2 self-check: launch the exe at DPR 1,
# capture Home, then click each ribbon tab (real input) and capture/compare
# it against its gallery image; finally open the File backstage.
# Usage (repo root): bash tools/screen2-loop.sh [state ...]
#   states: Home File View Tools Reference_Image Help   (default: all, in that order —
#   the backstage reference shows the Home body)
set -u
cd "$(dirname "$0")/.."
OUT=target/s2
mkdir -p "$OUT"
states=("$@")
[ ${#states[@]} -eq 0 ] && states=(Home File View Tools Reference_Image Help)

taskkill /IM sketch-app.exe /F >/dev/null 2>&1
sleep 1
(SKETCH_DPR=1 ./target/debug/sketch-app.exe > target/app.log 2>&1 &)
sleep 6
ps1() { powershell -NoProfile -ExecutionPolicy Bypass -File "$@"; }
# Size the window to 1400x860 client first (shot.ps1 does it).
ps1 tools/shot.ps1 -Out "$OUT/warmup.png" >/dev/null

declare -A X=( [Home]=83 [View]=144 [Tools]=203 [Reference_Image]=295 [Help]=385 [File]=25 )
for s in "${states[@]}"; do
  ps1 tools/click-real.ps1 -X "${X[$s]}" -Y 46 >/dev/null
  ps1 tools/shot.ps1 -Out "$OUT/$s.png" >/dev/null
  if [ "$s" = File ]; then ref=refs/ui-spec/gallery/file-backstage.png
  else ref=refs/ui-spec/gallery/ribbon-$s.png; fi
  echo "=== $s vs $ref"
  python tools/compare.py "$ref" "$OUT/$s.png" --out "$OUT/cmp-$s"
  # File toggles: close it again so later states start from a closed menu.
  [ "$s" = File ] && ps1 tools/click-real.ps1 -X 25 -Y 46 >/dev/null
done
