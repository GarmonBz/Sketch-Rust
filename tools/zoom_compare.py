"""tools/zoom_compare.py — focused comparison of one box of two screenshots.

Usage: python tools/zoom_compare.py ref.png cand.png x0 y0 x1 y1 out.png [--scale 3]

Prints the % of differing pixels (same tolerance as compare.py) inside the
box and per-column-band (100px) so misplacements can be located, and writes
a stacked zoom: reference / candidate / diff mask.
"""
import argparse

from PIL import Image, ImageChops

TOL = 24


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("ref")
    ap.add_argument("cand")
    ap.add_argument("box", nargs=4, type=int)
    ap.add_argument("out")
    ap.add_argument("--scale", type=int, default=3)
    ap.add_argument("--band", type=int, default=100)
    a = ap.parse_args()
    ref = Image.open(a.ref).convert("RGB").crop(a.box)
    cand = Image.open(a.cand).convert("RGB").crop(a.box)
    mask = ImageChops.difference(ref, cand).convert("L").point(lambda v: 255 if v > TOL else 0)
    w, h = ref.size
    bad = mask.histogram()[255]
    print(f"box {a.box}: {100.0 * bad / (w * h):.2f}% differ")
    for x0 in range(0, w, a.band):
        sub = mask.crop((x0, 0, min(w, x0 + a.band), h))
        n = sub.size[0] * sub.size[1]
        print(f"  x {a.box[0] + x0:5d}-{a.box[0] + x0 + sub.size[0]:5d}: {100.0 * sub.histogram()[255] / n:6.2f}%")
    s = a.scale
    out = Image.new("RGB", (w * s, h * s * 3 + 8), (255, 0, 255))
    out.paste(ref.resize((w * s, h * s), Image.NEAREST), (0, 0))
    out.paste(cand.resize((w * s, h * s), Image.NEAREST), (0, h * s + 4))
    red = Image.new("RGB", (w, h), (0, 0, 0))
    red.paste(Image.new("RGB", (w, h), (255, 40, 40)), (0, 0), mask)
    out.paste(red.resize((w * s, h * s), Image.NEAREST), (0, 2 * h * s + 8))
    out.save(a.out)


if __name__ == "__main__":
    main()
