# Opening and saving Sketchy files

Use **File > Open** to open a `.sketchy.json` file from the [Property Hackers
Sketchy draft](https://github.com/property-hackers/sketchy/blob/4941b6a3abfd1e540a2db44f3e887f3784366eb6/SPEC.md).
Sketch checks the JSON `format` marker and supports draft versions with major
version `0`. It reports unsupported major versions and required extensions.

**Save** writes back to the open Sketchy file. Use **File > Save As** and choose
**Sketchy** (`.sketchy.json`) to create a Sketchy workfile. Saving an existing
Sketchy workfile to a different path creates a new document linked to the
original revision.

Each building level becomes an editable page. Site geometry has its own page.
Sketch converts the source units and structure placements, keeps circular arcs
editable, and approximates ellipses with line segments. It uses the geometry to
calculate areas, applies area factors and direct cutouts, and excludes room
subdivisions from level totals. Sketchy areas are not automatically classified
as gross living area. To merge a single Sketchy page into an existing sketch,
use **File > Import...**.

Openings become markers and fixed features become labeled footprints. The
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
