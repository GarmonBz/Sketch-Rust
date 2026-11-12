#!/usr/bin/env bash
# Render a subset of headless e2e states and compare them with the gallery.
# Usage: tools/shots.sh <filter> [<filter> ...]
# Each filter is a SKETCH_E2E_ONLY substring; every rendered state whose
# reference exists is compared (status bar / canvas / overall lines).
set -u
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
for f in "$@"; do
  SKETCH_E2E_ONLY="$f" timeout 400 cargo test -q -p sketch-app e2e_screenshots_of_every 2>&1 \
    | grep -E "test result|panicked" -A3
done
for img in target/e2e/*.png; do
  n=$(basename "$img" .png)
  match=0
  for f in "$@"; do case "$n" in *"$f"*) match=1 ;; esac; done
  [ "$match" = 1 ] || continue
  [ -f "refs/ui-spec/gallery/$n.png" ] || continue
  printf '%-24s %s\n' "$n" "$(python tools/compare.py "refs/ui-spec/gallery/$n.png" "$img" --out "target/cmp-$n" \
    | grep -E "^(canvas|status bar|overall)" | tr -s ' ' | tr '\n' ' ')"
done
