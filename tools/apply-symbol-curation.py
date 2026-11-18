"""Apply tools/symbol-curation.json to assets/symbols.json.

Merges categories (moving symbols and their category fields), renames
categories, and retires symbols that a newer one replaces. A retired symbol is
deleted and its id goes into the catalog's `aliases`, so an old sketch that
placed it opens with the replacement in its place. Safe to run more than once.

    python tools/apply-symbol-curation.py [catalog]   (default assets/symbols.json)
"""
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
catalog = Path(sys.argv[1]) if len(sys.argv) > 1 else root / "assets" / "symbols.json"
curation = json.loads((root / "tools" / "symbol-curation.json").read_text(encoding="utf-8"))
lib = json.loads(catalog.read_text(encoding="utf-8"))
symbols = lib["symbols"]
cats = {c["key"]: c for c in lib["categories"]}

# Merge: the source's symbols go first (they replace older ones), then the
# target's own; the source category is removed.
for src, dst in curation["mergeCategories"].items():
    if src not in cats:
        continue  # already merged
    moved = cats[src]["symbolIds"]
    cats[dst]["symbolIds"] = moved + [i for i in cats[dst]["symbolIds"] if i not in moved]
    for sid in moved:
        if sid in symbols:
            symbols[sid]["category"] = dst
    del cats[src]

for key, name in curation["renameCategories"].items():
    if key in cats:
        cats[key]["name"] = name
for sym in symbols.values():
    if sym.get("category") in cats:
        sym["categoryName"] = cats[sym["category"]]["name"]

# The original catalog used 0.5 as a midpoint marker. The editor now
# records offsets in inches from the anchor implied by the snap mode.
for sid, offset in curation["legacyGapOffsets"].items():
    symbol = symbols.get(sid)
    if symbol and symbol.get("wallGapAlignment") == 0.5:
        symbol["wallGapAlignment"] = offset

retire = curation["retire"]
aliases = lib.setdefault("aliases", {})
for sid, replacement in retire.items():
    if replacement not in symbols:
        sys.exit(f"unknown replacement in curation: {sid} -> {replacement}")
    symbols.pop(sid, None)
    aliases[sid] = replacement
# An older alias that led to a retired id now leads to its replacement.
for sid, to in list(aliases.items()):
    aliases[sid] = retire.get(to, to)
lib["aliases"] = dict(sorted(aliases.items()))
for c in cats.values():
    c["symbolIds"] = [i for i in c["symbolIds"] if i not in retire]
lib["defaultFavorites"] = [i for i in lib["defaultFavorites"] if i not in retire]

lib["categories"] = [c for c in lib["categories"] if c["key"] in cats]
for c in lib["categories"]:
    c.update(cats[c["key"]])
lib["generatedFrom"] = (
    "CADdillo residential selection (CC0 1.0), plus preserved original library; "
    "curated 2026-11-17 (tools/symbol-curation.json)"
)

lib["symbolCount"] = len(symbols)
catalog.write_text(json.dumps(lib, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
# Retired shapes are no longer kept anywhere (owner decision 2026-11-17).
catalog.with_name("symbols.retired.json").unlink(missing_ok=True)
listed = {i for c in lib["categories"] for i in c["symbolIds"]}
print(
    f"{catalog.name}: {len(symbols)} symbols, {len(lib['categories'])} categories, "
    f"{len(listed)} listed, {len(lib['aliases'])} aliases"
)
