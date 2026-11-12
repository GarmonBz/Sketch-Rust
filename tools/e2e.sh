#!/usr/bin/env bash
# tools/e2e.sh — the standard acceptance command (ACCEPTANCE.md).
#
# 1. Runs the sketch-app tests, including the headless end-to-end suite
#    (src/e2e.rs, iced_test Emulator: the real program, scripted input,
#    no window, no real mouse).
# 2. Compares every headless screenshot in target/e2e/ with its reference in
#    refs/ui-spec/gallery/ and prints a per-region table.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
export PATH="$HOME/.cargo/bin:$PATH"
# Limit graphics-emulator memory and renderer pressure.
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-2}"

echo "== cargo test -p sketch-app (unit + e2e)"
if ! cargo test -p sketch-app 2>&1 | grep -E "^test |test result|panicked|error"; then
  echo "cargo test produced no output"; exit 1
fi

# state name -> reference image
declare -a PAIRS=(
  "default:ribbon-Home.png"
  "home:ribbon-Home.png"
  "view:ribbon-View.png"
  "tools:ribbon-Tools.png"
  "reference-image:ribbon-Reference_Image.png"
  "help:ribbon-Help.png"
  "file-backstage:file-backstage.png"
  "after-file-escape:after-file-escape.png"
  "after-file-toggle:after-file-toggle.png"
  "dock-areas:dock-Areas.png"
  "dock-text:dock-Text.png"
  "dock-symbols:dock-Symbols.png"
  "dock-calcs:dock-Calcs.png"
  "dock-layers:dock-Layers.png"
  "dock-collapsed:dock-collapsed.png"
  "dock-symbol-selected:dock-symbol-selected.png"
  "dock-text-abc:dock-text-abc.png"
  "dock-pin-toast:dock-pin-toast.png"
  "dock-pin-off:dock-pin-off.png"
  "dock-resized:dock-resized.png"
  "status-default:status-default.png"
  "status-zoom-in:status-zoom-in.png"
  "status-zoom-out:status-zoom-out.png"
  "status-slider-250:status-slider-250.png"
  "status-scale-20:status-scale-20.png"
  "status-grid-off:status-grid-off.png"
  "status-snap-off:status-snap-off.png"
  "status-areasnap-half:status-areasnap-half.png"
  "status-filter:status-filter.png"
  "status-filter-unchecked:status-filter-unchecked.png"
  "status-zoomfence-armed:status-zoomfence-armed.png"
  "status-zoomfence-drag:status-zoomfence-drag.png"
  "status-zoomfence-done:status-zoomfence-done.png"
  "status-north-prompt:status-north-prompt.png"
  "status-north-90:status-north-90.png"
  "status-page-next-confirm:status-page-next-confirm.png"
  "status-page-2of2:status-page-2of2.png"
  "status-page-1of2:status-page-1of2.png"
  "status-underlay-page2:status-underlay-page2.png"
  "status-page-delete-alert:status-page-delete-alert.png"
  "status-page-delete-confirm:status-page-delete-confirm.png"
  "status-page-deleted:status-page-deleted.png"
  "rail-pan:rail-pan.png"
  "rail-draw-exterior:rail-draw-exterior.png"
  "rail-draw-interior:rail-draw-interior.png"
  "rail-measure-toast:rail-measure-toast.png"
  "canvas-pan-drag:canvas-pan-drag.png"
  "canvas-wheel-zoom:canvas-wheel-zoom.png"
  "draw-pen-down:draw-pen-down.png"
  "draw-two-walls:draw-two-walls.png"
  "draw-pen-up:draw-pen-up.png"
  "draw-interior:draw-interior.png"
  "draw-closed-area:draw-closed-area.png"
  "draw-typed-buffer:draw-typed-buffer.png"
  "draw-keyboard-preview:draw-keyboard-preview.png"
  "draw-typed-wall:draw-typed-wall.png"
  "dims-inside:dims-inside.png"
  "dims-hidden:dims-hidden.png"
  "color-dialog-standard:color-dialog-standard.png"
  "color-dialog-custom:color-dialog-custom.png"
  "color-dialog-preset:color-dialog-preset.png"
  "draw-metric-label:draw-metric-label.png"
  "draw-wall-solids:draw-wall-solids.png"
  "draw-bowed-wall:draw-bowed-wall.png"
  "draw-bowed-solid:draw-bowed-solid.png"
  "select-wall:select-wall.png"
  "select-menu:select-menu.png"
  "context-menu:context-menu.png"
  "area-properties:area-properties.png"
  "area-attributes:area-attributes.png"
  "area-codes-editor:area-codes-editor.png"
  "area-definition-editor:area-definition-editor.png"
  "new-area-code:new-area-code.png"
)

echo
printf "%-20s %9s %9s %9s %9s %9s %9s %9s\n" state title ribbon rail canvas dock status overall
for pair in "${PAIRS[@]}"; do
  name=${pair%%:*}; ref=${pair#*:}
  out=$(python tools/compare.py "refs/ui-spec/gallery/$ref" "target/e2e/$name.png" --out "target/e2e/cmp-$name")
  vals=$(echo "$out" | awk '/%$/ {print $NF}' | tr '\n' ' ')
  # shellcheck disable=SC2086
  printf "%-20s %9s %9s %9s %9s %9s %9s %9s\n" "$name" $vals
done
echo
echo "side-by-side + diff images: target/e2e/cmp-<state>/"
