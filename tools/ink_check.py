"""tools/ink_check.py — per-element ink centroid / mass, reference vs candidate.

Usage: python tools/ink_check.py ref.png cand.png SPEC_JSON [STATE_KEY] [--min-y 58 --max-y 129]

For every icon (<i>) and text span in the spec (optionally within a y range),
measures the ink (|pixel - local background| luminance, background = the
box's most common colour in the reference) in a box grown by 3px, and prints
the centroid offset (candidate - reference, px) and the ink-mass ratio.
Flags |d| > 0.5px or mass outside 0.8..1.25.
"""
import argparse
import json
import warnings
from collections import Counter

from PIL import Image

warnings.filterwarnings("ignore", category=DeprecationWarning)


def lum(p):
    return 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2]


def ink(img, box, bg):
    x0, y0, x1, y1 = box
    m = sx = sy = 0.0
    bl = lum(bg)
    for y in range(y0, y1):
        for x in range(x0, x1):
            v = abs(lum(img.getpixel((x, y))) - bl)
            if v > 6:
                m += v
                sx += v * x
                sy += v * y
    if m == 0:
        return 0.0, 0.0, 0.0
    return m, sx / m, sy / m


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("ref")
    ap.add_argument("cand")
    ap.add_argument("spec")
    ap.add_argument("key", nargs="?")
    ap.add_argument("--min-y", type=float, default=0)
    ap.add_argument("--max-y", type=float, default=10000)
    ap.add_argument("--all", action="store_true", help="print unflagged rows too")
    a = ap.parse_args()
    spec = json.load(open(a.spec, encoding="utf-8"))
    elems = spec[a.key] if a.key else spec
    ref = Image.open(a.ref).convert("RGB")
    cand = Image.open(a.cand).convert("RGB")
    flagged = 0
    for e in elems:
        if not (a.min_y <= e["y"] < a.max_y):
            continue
        if e["tag"] not in ("i", "span") or (e["tag"] == "span" and not e.get("text")):
            continue
        box = (int(e["x"]) - 3, int(e["y"]) - 3, int(e["x"] + e["w"]) + 3, int(e["y"] + e["h"]) + 3)
        bg = Counter(ref.crop(box).getdata()).most_common(1)[0][0]
        mr, xr, yr = ink(ref, box, bg)
        mc, xc, yc = ink(cand, box, bg)
        name = e.get("text") or e.get("cls", "")
        if mr == 0:
            print(f"  {name:<34} (no ink in reference)")
            continue
        dx, dy, ratio = xc - xr, yc - yr, mc / mr
        bad = abs(dx) > 0.5 or abs(dy) > 0.5 or not (0.8 <= ratio <= 1.25)
        flagged += bad
        if bad or a.all:
            print(f"{'!' if bad else ' '} {name:<34} at {e['x']:>4},{e['y']:>4}  dx {dx:+.2f}  dy {dy:+.2f}  mass x{ratio:.2f}")
    print(f"{flagged} flagged")


if __name__ == "__main__":
    main()
