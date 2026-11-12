#!/usr/bin/env bash
# tools/verify-screen2.sh — parent verification of Screen 2 (assumes the exe is
# running with SKETCH_DPR=1). Real clicks on each ribbon tab, PrintWindow capture,
# compare against the reference gallery.
cd "$(dirname "$0")/.." || exit 1
PS=(powershell.exe -NoProfile -ExecutionPolicy Bypass -File)
mkdir -p target/v2
run() {
  local name=$1 x=$2 ref=$3
  "${PS[@]}" 'tools\click-real.ps1' -X "$x" -Y 46
  sleep 1
  "${PS[@]}" 'tools\shot.ps1' -Out "target\\v2\\$name.png"
  echo "## $name"
  python tools/compare.py "refs/ui-spec/gallery/$ref" "target/v2/$name.png" --out "target/v2/cmp-$name" | grep -v wrote
}
run View 144 ribbon-View.png
run Tools 203 ribbon-Tools.png
run RefImage 295 ribbon-Reference_Image.png
run Help 385 ribbon-Help.png
run Home 83 ribbon-Home.png
run File 25 file-backstage.png
