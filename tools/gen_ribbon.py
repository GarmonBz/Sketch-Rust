"""tools/gen_ribbon.py — generate crates/sketch-app/src/ribbon_tabs.rs.

Reads the measured element boxes of every ribbon tab body from
refs/ui-spec/chrome-spec-states.json (captured from the running original at
1400x860) and emits the Rust item tables that ribbon.rs renders. Run from the
repo root:  python tools/gen_ribbon.py
"""
import json
import os
import re
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPEC = os.path.join(ROOT, "refs", "ui-spec", "chrome-spec-states.json")
OUT = os.path.join(ROOT, "crates", "sketch-app", "src", "ribbon_tabs.rs")

TABS = [
    ("ribbon:View", "view"),
    ("ribbon:Tools", "tools"),
    ("ribbon:Reference Image", "reference_image"),
    ("ribbon:Help", "help"),
]

# Gap between the icon and the text of the inline (non .ribbon-btn) buttons,
# from their inline style in the original markup.
INLINE_GAP = {"btn-photo": 6.0}
DEFAULT_INLINE_GAP = 4.0


def rgb(s):
    m = re.match(r"rgba?\((\d+), (\d+), (\d+)", s)
    return "hex(0x%02x%02x%02x)" % tuple(int(v) for v in m.groups())


def color_const(s):
    c = rgb(s)
    return {"hex(0xcccccc)": "text_main()", "hex(0x858585)": "muted()"}.get(c, c)


def f(v):
    return "%.1f" % float(v)


def font_px(e):
    return float(e["font"].split("px")[0])


def inside(e, b):
    return (
        b["x"] <= e["x"] < b["x"] + b["w"] and b["y"] <= e["y"] < b["y"] + b["h"]
    )


def glyph(e):
    cls = e["cls"].split()
    fill = cls[0] == "ph-fill"
    name = cls[1][3:]
    return 'g("%s", %s, %s, %s, %s, %s)' % (
        name,
        "true" if fill else "false",
        f(font_px(e)),
        color_const(e["color"]),
        f(e["x"]),
        f(e["y"]),
    )


def caption(text, x, y, w, h, size, font, color):
    return 'cap("%s", %s, %s, %s, %s, %s, %s, %s)' % (
        text,
        f(x),
        f(y),
        f(w),
        f(h),
        f(size),
        font,
        color,
    )


def gen_tab(elems):
    body = [e for e in elems if 58 <= e["y"] < 129 and e.get("tag") != "div" or (
        58 <= e["y"] < 129 and "ribbon-divider" in e.get("cls", ""))]
    items = []
    i = 0
    while i < len(body):
        e = body[i]
        cls = e.get("cls", "")
        if "ribbon-divider" in cls:
            items.append("Item::Divider(%s)" % f(e["x"]))
            i += 1
            continue
        if e["tag"] == "button":
            kids = []
            j = i + 1
            while j < len(body) and body[j]["tag"] != "button" and inside(body[j], e):
                kids.append(body[j])
                j += 1
            look = "Look::Ribbon" if "ribbon-btn" in cls else "Look::Plain"
            h = e["h"]
            if e.get("bg"):
                look = "Look::Toggled"
                # Its painted box is visible: Chrome lays the 49.2px button
                # out at y=68.4 and snaps it to rows 68..117 (50px) — sampled
                # in ribbon-View.png; the spec rounds it to 49.
                h += 1
            glyphs, caps = [], []
            for k in kids:
                if k["tag"] == "i":
                    glyphs.append(glyph(k))
                elif k["tag"] == "span" and k.get("text"):
                    txt = k["text"]
                    if k["h"] >= 26:
                        txt = txt.replace(" ", "\\n", 1)
                    line = 13.0 if k["h"] >= 26 else k["h"]
                    caps.append(caption(txt, k["x"], k["y"], k["w"], line,
                                        font_px(k), "ARIAL", "text_main()"))
            if look == "Look::Plain":
                ic = kids[0]
                gap = DEFAULT_INLINE_GAP
                for pre, v in INLINE_GAP.items():
                    if e.get("id", "").startswith(pre):
                        gap = v
                tx = ic["x"] + ic["w"] + gap
                caps.append(caption(e["text"], tx, e["y"], e["x"] + e["w"] - tx,
                                    e["h"], font_px(e), "ARIAL", "text_main()"))
            items.append(
                "Item::Btn(Btn {\n            x: %s,\n            y: %s,\n            w: %s,\n"
                "            h: %s,\n            look: %s,\n            enabled: %s,\n"
                "            glyphs: vec![%s],\n            captions: vec![%s],\n        })"
                % (
                    f(e["x"]), f(e["y"]), f(e["w"]), f(h), look,
                    "false" if e.get("disabled") else "true",
                    ", ".join(glyphs), ", ".join(caps),
                )
            )
            i = j
            continue
        if e["tag"] == "span" and e.get("text"):
            items.append("Item::Caption(%s)" % caption(
                e["text"], e["x"], e["y"], e["w"], e["h"], font_px(e), "SEGOE",
                color_const(e["color"])))
        elif e["tag"] == "input":
            items.append("Item::Slider { x: %s, y: %s, w: %s, h: %s }" % (
                f(e["x"]), f(e["y"]), f(e["w"]), f(e["h"])))
        elif e["tag"] == "i":
            items.append("Item::Glyph(%s)" % glyph(e))
        i += 1
    return items


def main():
    spec = json.load(open(SPEC, encoding="utf-8"))
    out = [
        "//! GENERATED by `tools/gen_ribbon.py` from",
        "//! `refs/ui-spec/chrome-spec-states.json` — do not edit by hand; edit the",
        "//! generator (or the spec) and re-run `python tools/gen_ribbon.py`.",
        "//!",
        "//! The View / Tools / Reference Image / Help ribbon bodies: every element at",
        "//! its measured page position (the original at 1400x860 CSS px).",
        "",
        "use crate::ribbon::{cap, g, Btn, Item, Look};",
        "use crate::theme::*;",
        "",
    ]
    for key, name in TABS:
        items = gen_tab(spec[key])
        out.append("/// `%s` body." % key.split(":")[1])
        out.append("pub fn %s() -> Vec<Item> {" % name)
        out.append("    vec![")
        for it in items:
            out.append("        %s," % it)
        out.append("    ]")
        out.append("}")
        out.append("")
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(out))
    # Keep `cargo fmt --all --check` clean.
    subprocess.run(["rustfmt", "--edition", "2021", OUT], check=False)
    print("wrote", OUT)


if __name__ == "__main__":
    main()
