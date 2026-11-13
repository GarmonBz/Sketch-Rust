# Opening and saving Sketchy files

Use **File > Open** to open a draft 0.1 or 0.2 `.sketchy.json` workfile or a
SQLite `.sketchy` store. Sketch recognizes the file by its contents. Draft
versions before 1.0 can change incompatibly, so Sketch reports versions and
required extensions it cannot read. [Property Hackers Sketchy draft 0.2](https://github.com/property-hackers/sketchy/blob/3b2e3d15a20d94b60c771ae15e3e0dd19dfd1f3f/SPEC.md)

**Save** writes back to the open Sketchy file. Use **File > Save As** and choose
**Sketchy JSON** (`.sketchy.json`) or **Sketchy Store** (`.sketchy`). New documents
use draft 0.2. Existing files keep their draft version and curve convention.
Saving changes to a store adds a revision and advances the head in a single
transaction. Save As creates a separate document linked to the original revision.

Each building level becomes an editable page. Site geometry has its own page.
Sketch converts the source units and structure placements, keeps circular arcs
editable, and approximates ellipses with line segments. For 0.2, it clips holes,
voids, and subtracting subdivisions to each space and counts overlaps once.
Curved intersections use a polygon approximation. Sketchy areas are not automatically classified
as gross living area. To merge a single Sketchy page into an existing sketch,
use **File > Import...**.

Openings become markers or footprints, including skylights and openings on hole
edges. Features can have custom footprints and stair landings. Virtual edges
stay in area outlines without adding walls. Sketch retains ceiling zones,
per-edge measurement choices, and door swing metadata in the file. The
conversion includes text and outline annotations. Sketch retains existing print
layouts, custom styles, symbol definitions, dimension annotations, reference
images, and unknown metadata in the file. Some of these details are hidden in
the editor. Edited ellipses are saved as polygon outlines.

Files saved by Sketch contain standard Sketchy geometry and an optional extension
for app features such as dimension chains, symbol libraries, and drawing styles.
If another editor changes the standard Sketchy document, Sketch opens the updated
standard geometry.

To build the separate Windows preview, run `tools\package.ps1 -NativePreview`
and pass the resulting `target\package\...` staging folder to
`tools\sketchy-preview-installer.ps1 -StageDirectory`. It uses the same WiX
5.0.2 packages as the regular installer. The preview has its own Start menu entry
and install folder.
