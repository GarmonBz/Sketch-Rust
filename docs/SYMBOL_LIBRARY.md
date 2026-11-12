# Residential symbol library — 0.1.25

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

The checked-in selection manifest and verified source archive are under
`refs/caddillo`. Run `tools/import-caddillo.ps1` to reproduce the library using
the same Rust DXF importer used by the app. Archive SHA256:
`2555528a67338f5e13c173e2f15da8d3db0292a10b20c36b6db07597ad528ed9`.
