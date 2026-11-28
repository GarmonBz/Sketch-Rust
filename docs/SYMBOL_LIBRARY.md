# Residential symbol library

## Smaller default library

The default `symbols.json` no longer carries the older Arrows, Doors,
Windows and Miscellaneous categories or the 73 symbols in them. The
Doors & windows category (the CADdillo doors and windows) stays, and so do all
other categories. `symbols.json` now holds 503 symbols. They can be added back
to the library later if they are needed.

Doors & windows has two garage doors: **Door Garage 108** (9 ft, CADdillo) and
**Door Garage 216**, a double door 18 ft wide (216 in). Door Garage 216 is the
CADdillo 108-inch door with its width doubled (id `custom/door-garage-216`), so
it is not one of CADdillo's own blocks. It replaces the older Favorites symbol
Door - Garage-Dbl; sketches that used that symbol open with Door Garage 216 in
its place.

Fireplaces & structure has **Corner Fireplace 36** (id
`custom/fireplace-corner-36`): 48 in along each wall, with the firebox and
hearth of the CADdillo Fireplace 36 turned onto the slanted face, so it is not
one of CADdillo's own blocks. It is drawn for the top left corner of a room;
rotate it in 90-degree steps for the other corners. The older **Corner** symbol,
a 6 ft triangle, is still there.

A RapidSketch or ACI Sketch file brings its own corner fireplace, stored
stretched out of square. Opening one, or a sketch imported from one, swaps it for
Corner Fireplace 36, facing the same way, in its wall corner.

Doors & windows has **Sliding Closet Door** in 48, 60, 72 and 96 in. The 72 is
CADdillo's Door Bypass 72; the other three are resized from it, with the same
jambs. Kitchen has **Double Kitchen Sink 33**, CADdillo's 33 x 22 in sink, moved
there from Bath & plumbing. Both kept their ids, so saved sketches are unchanged.

## Plain category names and fewer duplicates (0.2.2)

The CADdillo categories now have plain names. Six of them merged into the
older category of the same kind, with the CADdillo symbols listed first:

| Category | Holds |
|---|---|
| Kitchen | CADdillo kitchen plus the older kitchen symbols |
| Furniture | CADdillo furniture plus the older furniture |
| Bath & plumbing | CADdillo bath and plumbing plus the older bath symbols |
| Stairs & accessibility | CADdillo stairs, ramps and rails plus the older stairs |
| Fireplaces & structure | CADdillo fireplaces, chimney and columns plus the older fireplaces |
| Utilities | CADdillo mechanical items plus the older utility symbols |

Doors & windows, Laundry, Electrical and Outdoor keep their own categories
under plain names. Bathroom, Bedroom and Landscaping are now in sentence case.

67 older symbols that a CADdillo symbol replaces are retired: for example the
residential toilet (now Toilet), the standard range (Range 30), the 36-inch
refrigerator (Fridge 36), the 90-degree interior and exterior doors (Door
Swing 30 and 36), and the matching copies in the old built-in Favorites set.
Older symbols with no CADdillo equivalent stay, such as corner and oval tubs,
45-degree doors, double garage doors, sliding doors, ranges with grills and the
fire-protection symbols.

Retired symbols are deleted: `symbols.json` holds the 572 symbols you can place,
and no copy of the old shapes is kept. Each retired id is listed under `aliases`
in `symbols.json` with its replacement. When a sketch that uses one opens (or a
layer is imported), Sketch Rust puts the replacement in its place, at the same
position, rotation, scale and wall; the sketch saves the new id from then on.
The replacement is the newer drawing of the same fixture, so it can differ a
little in size or detail. The full list is in `tools/symbol-curation.json`; run
`python tools/apply-symbol-curation.py` to apply it to a catalog. Saved
preferences for categories that kept their key still apply.

## Residential symbol library — 0.1.25

177 CADdillo residential DXF blocks are available in the CADdillo categories:
doors/windows, laundry, utilities, furniture, kitchen, bath/plumbing,
stairs/accessibility, electrical, outdoor and fireplaces/structure. Common
fixtures are the defaults in Favorites for a new profile. Existing personal
favorites and symbol-manager changes are preserved.

The original 462 definitions and IDs remain available so existing sketches
continue to render. A separate, unchanged copy ships as `symbols.previous.json`.
To restore the original catalog, close Sketch, rename `symbols.json` to keep
your current copy, then copy `symbols.previous.json` to `symbols.json`.
This does not reset personal favorites or hidden/deleted category preferences.
The original local profile and installed catalog were also backed up separately
before this update; private profile data is not included in the portable ZIP.

Symbols use inches internally and feet in drawings: a 36-inch opening is 3 feet,
a 27-inch washer is 2.25 feet and the 60-by-30-inch tub is 5 by 2.5 feet.
Nominal door width measures the opening; overall preview bounds also include
the jambs and swing. Doors and windows insert at the opening midpoint and snap
to straight walls.
Doors face the pointer's side of the wall; furniture and fixtures place freely.
Scaling a placed door/window also scales its hosted wall opening. The source's
optional promotional text is excluded from symbol geometry, preview and output.

Use Manage symbols > Add DXF / JSON to import more blocks. DXF imports now resolve
BLOCK/INSERT geometry, including nested transforms, units, arcs, ellipses and
solid arrowheads. A missing or cyclic block reference reports an error instead
of importing an empty symbol. Curves are sampled at import for rendering.

Delete and Delete category permanently remove definitions from the active catalog
and placed instances from every page and history. The previous file is kept as
`symbols.json.bak`. There is no Purge action, Purged category, or in-app restore
for deleted symbols. Hide and Restore affect visibility only. A write failure
leaves definitions, instances, history and the selection intact so Delete can
be retried after the file becomes writable. Legacy removal preferences remain
excluded and cannot be restored through the manager.

Source: [CADdillo block library](https://caddillo.com/blocks/), downloaded
2026-10-31 from `https://caddillo.com/downloads/caddillo-blocks-library.zip`.
The actual downloaded archive has 413 DXFs; 177 residential blocks were selected.
CADdillo releases its CAD block geometry under **CC0 1.0**, permitting copying,
modification and redistribution without attribution. The software and CADdillo
brand are not included in this permission. See the publisher's
[license](https://caddillo.com/license/) and the
[CC0 dedication](https://creativecommons.org/publicdomain/zero/1.0/).
No vendor logo or copyright notice is added to sketches or report/print images.

The selection manifest and verified source archive were kept under
`refs/caddillo`; that folder was removed from the working tree with the rest of
`refs/` in the interface cutover (commit 81ddd69) and is in git history before it. Run `tools/import-caddillo.ps1` to reproduce the library using
the same Rust DXF importer used by the app. Archive SHA256:
`2555528a67338f5e13c173e2f15da8d3db0292a10b20c36b6db07597ad528ed9`.
