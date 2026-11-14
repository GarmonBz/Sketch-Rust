# Opening and saving Sketchie files

Use **File > Open** to open a Sketchie 0.2 JSON workfile (`.sketchie.json`) or a
SQLite revision store (`.sketchie`). Sketch Rust also opens earlier Sketchy
0.1 and 0.2 workfiles and stores (`.sketchy.json` and `.sketchy`). It recognizes
both formats by their contents and reports unsupported versions and required
extensions. [Property Hackers Sketchie draft 0.2](https://github.com/property-hackers/sketchie/blob/a33c0f4dcdfbeb19df743820e2c6b4be72aa13c4/SPEC.md)

**Save** writes back to the open file using its original format marker and draft
version. Use **File > Save As** and choose **Sketchie JSON** or **Sketchie Store**
for new files or to save a separate Sketchie copy of a Sketchy 0.2 file. Draft 0.1
files use the legacy Save As options and retain their curve convention.
Saving changes to a store adds a revision and updates the head and format version
in a single transaction. Save As creates a separate document linked to the
original revision.

Each building level becomes an editable page. Site geometry has its own page.
Sketch converts the source units and structure placements, keeps circular arcs
editable, and approximates ellipses with line segments. For 0.2, it clips holes,
voids, and subtracting subdivisions to each space and counts overlaps once.
Curved intersections use a polygon approximation. Gross living area
classification requires review in Sketch Rust. To merge a single Sketchie page into an existing sketch,
use **File > Import...**.

Openings become markers or footprints, including skylights and openings on hole
edges. Features can have custom footprints and stair landings. Virtual edges
stay in area outlines without adding walls. Sketch retains ceiling zones,
per-edge measurement choices, and door swing metadata in the file. The
conversion includes text and outline annotations. Sketch retains existing print
layouts, custom styles, symbol definitions, dimension annotations, reference
images, and unknown metadata in the file. Some of these details are hidden in
the editor. Edited ellipses are saved as polygon outlines.

Files saved by Sketch contain standard Sketchie geometry and an optional extension
for app features such as dimension chains, symbol libraries, and drawing styles.
If another editor changes the standard document, Sketch opens the updated
standard geometry.

To build the separate Windows preview, run `tools\package.ps1 -NativePreview`
and pass the resulting `target\package\...` staging folder to
`tools\sketchy-preview-installer.ps1 -StageDirectory`. It uses the same WiX
5.0.2 packages as the regular installer. The preview has its own Start menu entry
and install folder.
