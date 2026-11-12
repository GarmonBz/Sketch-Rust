"""tools/compare.py — objective screenshot comparison for ACCEPTANCE.md.

Usage:
    python tools/compare.py <reference.png> <candidate.png> [--out target/compare]

Writes:
    <out>/side-by-side.png  reference | candidate
    <out>/diff.png          candidate dimmed, differing pixels painted red
and prints, per region of the original layout (title bar, ribbon, left rail,
canvas, dock, status bar), the percentage of pixels that differ by more than
a small tolerance. Regions are the original's boxes at 1400x860
(refs/ui-spec/chrome-spec-home.json).
"""
import argparse
import os
import sys

from PIL import Image, ImageChops

TOL = 24  # per-channel tolerance (anti-aliasing / font hinting noise)

REGIONS = {
    "title bar": (0, 0, 1400, 30),
    "ribbon": (0, 30, 1400, 129),
    "left rail": (0, 129, 41, 832),
    "canvas": (41, 129, 1089, 832),
    "dock": (1089, 129, 1400, 832),
    "status bar": (0, 832, 1400, 860),
}


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("reference")
    ap.add_argument("candidate")
    ap.add_argument("--out", default=os.path.join("target", "compare"))
    a = ap.parse_args()

    ref = Image.open(a.reference).convert("RGB")
    cand = Image.open(a.candidate).convert("RGB")
    if ref.size != cand.size:
        print(f"SIZE MISMATCH reference={ref.size} candidate={cand.size}")
        cand = cand.resize(ref.size)

    os.makedirs(a.out, exist_ok=True)
    w, h = ref.size
    sbs = Image.new("RGB", (w * 2 + 8, h), (255, 0, 255))
    sbs.paste(ref, (0, 0))
    sbs.paste(cand, (w + 8, 0))
    sbs.save(os.path.join(a.out, "side-by-side.png"))

    diff = ImageChops.difference(ref, cand)
    mask = diff.convert("L").point(lambda v: 255 if v > TOL else 0)
    overlay = Image.blend(cand, Image.new("RGB", cand.size, (0, 0, 0)), 0.6)
    overlay.paste(Image.new("RGB", cand.size, (255, 40, 40)), (0, 0), mask)
    overlay.save(os.path.join(a.out, "diff.png"))

    total_bad = 0
    print(f"{'region':<12} {'differs':>8}")
    for name, box in REGIONS.items():
        region = mask.crop(box)
        hist = region.histogram()
        bad = hist[255]
        n = region.size[0] * region.size[1]
        total_bad += bad
        print(f"{name:<12} {100.0 * bad / n:7.2f}%")
    print(f"{'overall':<12} {100.0 * total_bad / (w * h):7.2f}%")
    print(f"wrote {a.out}/side-by-side.png and {a.out}/diff.png")
    return 0


if __name__ == "__main__":
    sys.exit(main())
