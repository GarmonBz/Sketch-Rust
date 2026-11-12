# Sketch Rust — STATUS ledger

Standalone native Windows port of `Appraisal Sketch 1.1`
(`00) sketch standalone.html`, the attached 10.3 MB HTML app).

0.1.29 adds an in-app User Guide on the Help tab (seven topics, including
Appraise-It integration setup, daily use and troubleshooting), with an
Integration Guide button that opens it at that topic. A per-user Windows
installer (crates/sketch-setup, tools/installer.ps1) is built alongside the zip.

0.1.28 fixes a crash when File > Recent Files was opened with at least one
entry: the row's filled file icon was not embedded. A release build now draws
nothing for a missing icon instead of exiting; debug builds and tests still fail.

0.1.27 presents frames without vsync, so the wall being drawn no longer trails
the mouse cursor by a frame or two (ICED_PRESENT_MODE=vsync restores it). It is
also the first numbered build with the CADdillo door pivot fix, Help > About and
the Edit Tools tab styled like the other tabs, which went into late 0.1.26 builds.

0.1.26 is the window and desktop release. Maximize replaces full screen, so the
window stays inside the work area and off the taskbar; the title-bar button
shows Maximize or Restore. Size, position and maximized state are remembered,
with an 800 x 560 minimum, and the frameless window has edge and corner resize
zones. The title bar shows Unsaved/Saved and closing an unsaved document asks
to save. A .sketch file can be dropped on the window or passed on the command
line; opening a file that is already open brings its window forward. The
quick-access caret opens New, Open, Save As and Options. Pinch zooms and
two-finger drag pans. The 16 CADdillo doors pivot on the hinge-side jamb so the
wall cut lies under the symbol; Help > About shows the running version.

0.1.25 removes the Purge/Purged workflow. Delete and Delete category now always
perform permanent library-file deletion, with symbols.json.bak retained. A failed
write leaves the document, preferences and selection intact. Legacy removal IDs
remain excluded, and Hide/Restore only changes visibility.

0.1.24 adds 177 CADdillo residential blocks in ten categories, preserving the
original 462 IDs and a separate public catalog backup. DXF block expansion,
units and opening pivots are shared between manager import and catalog generation.
Door/window scaling now also resizes the hosted opening. Personal preferences
remain intact; the pre-update local catalog/profile backup is in backups/.

0.1.23 fixes category deletion in Manage Symbols. Opening from Fire preserves
that category. A visible, named confirmation precedes deletion; picker/list/
preview state refreshes afterward. Empty categories can also be removed.

0.1.22 adds Save to report / Discard changes / Cancel when closing an integrated
sketch. Save closes after host acknowledgment; failure keeps the editor open.
Title-bar, File Exit and native window close requests share the same flow.

0.1.21 removes the managed compatibility console and moves live host snapshot
rendering off the input thread. Unchanged reads reuse images; late jobs cannot
replace newer saves. High-frequency pointer events redraw locally while HUD
updates are coalesced to frame intervals. Integration remains embedded in Sketch.

Current release: **0.1.29** (2026-11-11). Help > Appraise-It Integration exposes
Enable integration and Restore previous provider without terminal commands.
It uses the embedded registration backend and handles protected-version upgrades.
Print Preview, Print and PDF export
include centered Summary and Breakdown boxes at the bottom, without bottom
branding. Long calculation tables continue on additional pages. The crosshair
remains visible after labeling an area or ending a keyboard-drawn boundary.
Drawing uses the supplied crosshair SVG
with a centered hotspot. Hover updates skip repeated document bookkeeping and
cache the floorplan while keeping live guides and previews responsive.
The current working folder is `dist/0.1.23`; the root `dist` installation is
left at 0.1.17 to preserve the user's running program.
Save writes directly to the current
file; only first Save and Save As show the native file dialog. File binding
changes only after successful writes and clears for New/recovery.
All 13 audited File/Home/View/Tools
control mappings are repaired. Center and Center & Fit now use live
page bounds, fixing both view buttons and C / Shift+C. Curved walls are included.
The 0.1.13 Pro image aspect correction remains verified in actual legacy report
rendering and UAD stored images: squares remain square.
Centered, compact Summary/Breakdown boxes remain, with no bottom branding.
See docs/BUILD_VALIDATION.md for checked fields, image margins and test limits.

The 13 UI wiring gaps found after packaging 0.1.14 are fixed in 0.1.15.
See [the control wiring audit](docs/CONTROL_WIRING_AUDIT.md) for the original
findings and verification of the repaired entry points.

## Decisions (user-confirmed 2026-10-21)

- **Clean-room native repo** — no dependency on the `D:/AIProjects/AppraisalWorkbench Rust`
  crates; everything self-contained here.
- **GUI: Iced 0.14**, split headless `sketch-core` + GUI `sketch-app`.
- **Symbol library is EXTERNAL**: `assets/symbols.json` (extracted verbatim from the
  HTML's `SKETCH_SYMBOL_LIBRARY` — 462 symbols, 19 categories, verified), loaded at
  runtime next to the exe. Never embed it in the binary.
- **Scope**: core-first (draw/measure/areas + symbols + SKX save/load in v1); the rest
  phased. Everything gets ported eventually, including Disto laser connect and DXF
  CAD import/export.
- Porting source of truth: `refs/*.js` extracted from the standalone HTML.
  Behavioral oracle: the original project's `tests/sketch-area-*.test.mjs` (not in this repository).
- Sign conventions are load-bearing (see `crates/sketch-core/src/geometry.rs` header).

## Phase plan

- [x] **P0 scaffold** — workspace, refs extracted, `assets/symbols.json` verified
      (462 symbols / 19 categories / 43 favorites), cargo check green.
- [x] **P1 geometry kernel** — `refs/geometry-engine.js` → `sketch-core/src/geometry.rs`
      method-for-method, with unit tests derived from the browser oracle.
- [x] **P2 file + symbol layers** — `refs/skx-format.js` + `refs/zip-container.js` →
      `sketch-core/src/skx.rs` (ZIP container, deflate); typed `symbols.rs` loader for
      external `symbols.json` with runtime path resolution next to the exe.
- [x] **P3 app shell** — Iced canvas, wall drawing (bows/arcs), measure tool, selection,
      pan/zoom, ribbon + sidebar, status HUD, themes (dark/blue/light from the CSS vars).
- [x] **P4 symbols in-app** — placement, wall snap (NONE/SIDE/CENTER + gap
      widths), previews, favorites, context-menu actions, resize/transform
      handles; plus the closeJoinedExteriorLoops subset and geometric wall
      solids from the P3a gap list.
- [x] **P5 areas** — define-area, area codes + breakdown panel, rollups, basement rules,
      auto-fill subject info.
- [x] **P6 persistence** — SKX save/open, undo/redo history, recent files.
- [x] **Remaining standalone software modules** — layers, pages, dimensions,
      lasso/transform, DXF import/export, PNG/SVG/PDF exports, print preview,
      area breakdown, reference images/geo controls, Options, Subject Info,
      command search and reusable libraries are implemented. See the refreshed
      `refs/ui-spec/PARITY.md` and all-634-row `feature-audit.json` for exact scope,
      native adaptations and explicitly deferred boundaries.
- [ ] **Disto physical-device acceptance** — explicitly deferred by the user.
      Native BLE software and simulated measurement tests are implemented.

## Done

- 2026-10-29: **0.1.0 release audit and portable package (superseded by 0.1.1 above).** Reclassified all 634 feature rows against the current code: 546 implemented, 44 native adaptations, 7 native enhancements, 5 hardware-deferred, 20 expanded-reference-unavailable, 9 source-inactive and 3 suite-only. Retired stale percentage and missing-feature claims. Fixed bay-window mirror, inert Calcs Calculation Details, unavailable command-search file actions, absolute Area Grid Snap and free-label/ordinary snapping, actual symbol/text placement previews, and automatic selection of a placed symbol. Added `--check-installation` for packaged symbol/view/SKX validation and native `tools/package.ps1` to produce a current ZIP, executable folder and SHA-256 provenance manifest. Disto hardware remains deferred; no physical-device claim is made.

- 2026-10-29: **Area-code JSON import/export (S7-011/012).** The editor's file buttons now open independent native JSON pickers. Import stages file text (including BOM handling) without changing the library; Export validates/normalizes the draft and defaults to `sketch-area-codes.json`. Apply persists the validated table so it survives restart. Cancel, I/O errors and picker results from a closed/reopened editor preserve the current draft/library. Four new tests drive the real buttons and cover disk round trips, normalization, Apply/restart, malformed imports, read/write errors, cancellation and stale callbacks. Disto hardware testing remains pending because the device is unavailable.

- 2026-10-29: **Leica DISTO BLE measurement input from the imported `leica-disto-ble` skill.** Installed the shared skill into the user's Codex skills directory. Native transport now discovers Leica BLE devices (Manual lists all devices), discovers the measured E7500i/D510 service and distance characteristic, subscribes to indications/notifications, decodes finite positive float32 LE metres, and feeds the existing keyboard entry path. Standard fills/replaces the length buffer; Continuous commits along the previous heading. Rounding uses the selected sketch units. Connect toggles disconnect; cancellation and session IDs reject late connection/measurement events; hardware drops clear the status chip. The shared keyboard Enter path now starts segments at the committed anchor, preserves the current wall type, and banks undo. Captured bytes and simulated input are tested; the native radio connection remains UNVERIFIED on hardware. Usage and hardware-check procedure: `docs/DISTO.md`.

- 2026-10-28: **Item 3: print/export fidelity.** Print Preview now renders all pages of the exact cached PDF through Windows PDF; its Print action writes the cached bytes. Reports embed installed Verdana regular/bold with actual font advances, clipping/wrapping and Unicode ToUnicode maps. The report's SVG drawing rules now carry symbols, rotated/styled labels, wall dimensions, chains and witnesses, north arrows, fills/patterns, bowed walls, solids and hosted opening cuts into a 300 DPI sketch image. Report headings/tables remain vector text. Fitting includes chains/moved content and reserves general-comment space. PNG now runs the clean canvas passes per visible layer, sharing its text shaping, fills, patterns, wall solids, symbols and chains; drawing aids and selection never export. SVG/DXF now receive actual drawing styles/auto-post options and hosted-wall visible runs. `png_text.rs` and the simplified overlay composition are retired. Validation evidence is recorded in ACCEPTANCE.md.

- 2026-10-28: **Completed the remaining optional area-definition workflow (S7-020..040, 044..049, 063).** New `area_workflow.rs` follows the full `refs/p8-optional-region-classification.js` extract: palette usage badges and collapsible groups; click/Enter/Space to arm/disarm repeated classification; guidance and undefined-outline highlights; Define Areas hover shading/value and the searchable, grouped canvas chooser with custom name, label settings, all original suffix choices and Keep assigning. The chooser traps forward/reverse Tab and blocks drawing shortcuts; Cancel restores selection and never materializes provisional geometry. The type menu assigns/clears selected areas, selects usages, opens New/Edit type and confirms deletion (UND protected). Edit areas now exposes Redefine (one batch undo), Clone, Reopen/continue/reclose, Detect Regions (one undo, no duplicates) and saved label/dimension defaults (including Out/In). Calcs right-click, canvas Edit Definition and area double-click open the same chooser. Locked/hidden targets and stale/page-switched targets refuse changes; tool Escape returns to the previous exterior/interior draw mode. Area drag/drop gains a following type ghost. Keyboard surfaces extend the existing EventTap so native input and headless widget tests share the same path. Uses the expanded reference workflow at the user's explicit request; retired legacy classification-tree controls stay retired. Gate: 473 app + 160 core tests (633 total), all 20 new workflow tests and all 73 reference screenshot pairs checked (worst overall 2.99%); Clippy -D warnings, fmt and diff checks pass; debug exe rebuilt.
- 2026-10-28: **Interior/exterior face measurements now drive area calculations (S7-073).** Ported `WallSolids.offsetRing` and `faceMeasurePolygon`: each edge offsets by half its host wall's thickness (per-wall inches override the type default; a missing host uses exterior thickness), with winding preserved, miter clamping and the original centerline fallback for collapsed/malformed rings or disabled solids. Successful offsets measure bowed edges as chords, matching the HTML. The live calculation, subtraction/GLA rollups, Calcs perimeter, area text/status values, breakdown geometry and all pages' export calculations share the measured values; editable/saved points and edge bows stay on their original centerlines. Recalculation invalidates on measure mode, wall type/thickness/edge association and wall-solid options. Regression coverage: 5 core + 3 app tests for numeric results, both windings, mixed offsets, fallback, bows, subtraction, dialog Apply/undo/redo, option changes, save/open and current/stored-page exports. Original JS numeric oracle independently confirms 90.25 sf interior / 110.25 sf exterior for a 10x10-foot centerline square with 6-inch walls.
- 2026-10-27: **Area-code drag onto the canvas (`defineAreaAt`) and Esc to cancel dock drags.** The Areas-palette tiles join the dock drag (`DragItem::Area { code, name }`; the tile's inner `mouse_area` arms the drag and its release is the click, which now sends `AreaType` from `DockItemRelease`). While one is dragged, `update_area_drop_hint` resolves the point under the pointer and outlines the region in the code's colour (`SketchCanvas::drop_hint` -> `render::draw_drop_hint`); the drop (`drop_area_code`) defines it. New `define_area.rs`: `resolve` (`resolveRegionAt`: the smallest visible area under the point vs. the tightest `find_enclosed_faces` face of the exterior walls, cached until a wall changes; the face wins over a larger area, `polygonRingsEqual` reuses an identical area, a locked smaller area refuses), `materialize` (`regionAreaAt`: an undefined area from the face, then `ownBoundaryLine` for every side that borrows only part of a wall, so a porch against the house wall gets its own wall), `define` (`defineAreaTarget`'s data: code, name, factor >= 1). In `App`: `Define Region` + `Define Area` undo steps (two, as the original banks), the area's generated `Area` / `Area N` layer renamed to its name (`is_generated_layer_name`), toasts `Area defined...` / `Classification cleared.` / `Show and unlock this area before changing its definition.` / `Drop the area definition inside a closed shape.`. The "Undefined · clear" tile drops `UND` / `Undefined Area`. **Escape** now cancels an active dock drag first (any kind), so the release drops nothing and is no click. Tests: 10 e2e (define under the point + undo, layer rename, clear tile, drop inside walls with no area (two undos), porch against a wall gets its own wall, off-canvas / locked drop changes nothing + no hint, Esc cancels area and symbol drags, tile click still picks, real press on a palette tile, hint screenshot) + 2 unit; gate see below.
- 2026-10-27: **Wall measurement labels, symbols and dimension chains in the PNG export.** New `png_overlay.rs`: the canvas's own drawing (`render::draw_export_overlay` = each wall's label through a new shared `draw_line_dimension`, `draw_symbols`, `draw_chains`) is drawn from a `Scene` fitted to the export (`pan` / `zoom` from the core's `PngFit`; new `Scene::text_zoom_floor` = `exportTextZoom`, so `text_zoom()` is `max(zoom, 2 * width / 2048)` as in the original) into an offscreen software iced renderer (`Headless::new(.., Some("tiny-skia"))`, no GPU; a `UserInterface` over a `canvas` the size of the image, `screenshot` onto a transparent background), and that layer is composited over the core raster before `png_text` lays the text on. New direct dep `iced_runtime` (already in the lock) for `UserInterface`. So the label placement (side, pinned position, `sameColorAsLine`), symbol artwork (rotation, flip, scale) and chains (tiers, ticks, overall) match the screen exactly. Gating: Dimensions / Symbols filters, `Show dimensions`, `dimensionHidden`, chain coverage, hidden layers; selection and handles never paint. Left: painting order (walls, then these, then text, not per layer). Checked over white. Tests: 4 e2e (wall label beside the wall's middle + filter + hidden, placed symbol + filter, chain + filter, picture); gate 438 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Text in the PNG export.** The core rasterizer has no fonts, so `export_png::render_png` now hands back the tiny-skia pixmap and its `PngFit` (`export_png_with_backdrop` = that + `encode_png`; the core re-exports `tiny_skia`), and the new app module `png_text.rs` lays the text over it before encoding. Text is set from the installed fonts (new deps `fontdb` + `ttf-parser`, both already in the lock through iced): `fontdb` finds the face for the family / weight / slant (falling back to Arial, then any sans-serif), `ttf-parser` supplies glyph outlines and advances (no kerning), and each string is filled as a path, centred with `textBaseline = 'middle'` and turned about its centre, so rotation and size are exact. Content: `drawLabels` (size `max(9, fontSize * textZoom)`, automatic ink per canvas theme, the Text Background box at its opacity, underline `max(1, size / 14)` at `.43` em) and the auto-post blocks of every defined area through the same `handles::area_blocks` / `block_line_font` / `block_line_step` the canvas uses (per-area font, scale, position, rotation; lines stack in the turned frame), at the export's text zoom `max(fit zoom, 2 * width / 2048)` (`exportTextZoom`); gated by the Areas / Labels filters and layer visibility. Left: wall measurement labels, dimension chains and symbols are still not in the PNG; text goes over the walls instead of per layer. Checked the output over white. Tests: 5 unit (fonts exist, centring, quarter turn, bold / underline, text zoom) + 4 e2e (label placed at its centre, area text + filter, rotation, picture); gate 434 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Drag a symbol or text from the dock onto the canvas** (S8-002 / S9-006). Dock items are now an inner `mouse_area` (press arms `DockDragPress`, release sends `DockItemRelease`) inside the styled button (whose own press is a no-op: the inner area captures it). While armed, the root `EventTap` listens to moves / release (`DockDragMove` / `DockDragRelease`, the dock-resize pattern): >5 px of travel makes it a drag; the canvas shows the placement marker under the pointer for a symbol (`SketchCanvas::symbol_drag`); releasing over the canvas places it (`place_symbol_at`, so wall snap / swing / openings and the `Place Symbol` step apply) or, for text, a plain 12px Arial label (`Place Text`, as the original's drop; the Text Label options are not applied). A click is a release with no drag (`dock_click`, handed from the tap's release to the item's release; a drag's release over its own item is not a click). Also fixed: click-to-place text now banks a `Place Text` undo step (it had none). Checked in the real window (drag with the mouse: marker follows, drop places and enables Undo). Tests: 6 e2e (symbol drop + undo, text drop + undo, release off the canvas, click vs drag, real press on a dock row, click-placed text undo); gate 425 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Edit Tools text group inputs.** The text group now matches the original's order: Font (select), Size, Bold, Italic, Underline, Text Color, Grow, Shrink, Text (`ribbon::text_group(tools)`; new `Item::Select` / `Item::TextInput`, `TextTools`, `TextField`, `TEXT_FONTS`). Values come from `App::text_tools()` (`syncContextualControls`: a selected label wins; else the area whose label / size block is selected, with the auto-post font / size / `#475569` standing in; the size block alone shows `net suffix`). `TextFontPicked` and the typed boxes (`TextFieldInput` buffers in `App::text_edit`, shown while typing, dropped when the selection changes; `TextFieldApply` on Enter) go through `set_selected_text_style` (one `Text Style` undo step; size 6-72, junk = 12; `text` renames an area from its label block only); Text Color opens the Color dialog (`ColorTarget::SelectedText`). Tests: 4 new e2e (values per selection kind, font / size apply + clamp + undo, text value for label / area / size block, colour via dialog) + the group's labels test extended, with a screenshot checked; removed the temporary length probe. Gate 419 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Cut / Copy / Paste cover text labels and symbols** (audit tier 1). `Clipboard` gained `texts` and `symbols`; `copy` returns something when any of walls, areas, labels or symbols are picked; `paste` adds them 2 ft down-right with fresh ids, selected (so one `Paste` step undoes them), shifts a pasted area's pinned label / size blocks, and re-hosts a copied opening on the copy of its wall or detaches it. **Cut data loss fixed**: `Message::Cut` deleted through the walls-and-areas path only, so after Delete learned to remove labels and symbols a mixed cut destroyed them without copying; Cut is now `cutSelected` = copy, then the same `Delete` message (confirm included). A cut with nothing copyable (only an area's text, only a measurement label) does nothing. Tests: 5 e2e (label copy / cut / paste and undo, mixed cut, nothing copyable, symbols with / without their wall, pinned area text); gate 415 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Status-bar totals chip and defined-areas select are live** (audit tier 1). Both were drawn as the boot state forever ("GLA 0.0 sf · 0 areas", "No defined areas 0.0") however many areas were defined. `updateCalcsPanel`'s two jobs are now in the view: the chip is a content-sized container (10 px padding, 6 px between the runs, 18 px tall) reading `GLA <totalGLA> sf` / `<n> area(s)` from the recalculation totals; the select lists every defined area (`statusbar::area_options`: code and name padded to the widest, min 3 and 4, then the gross area to one decimal), is enabled when there are any, and shows the selected area, else the last pick (`App::status_area`), else the last one; choosing an area clears the selection and selects it (`StatusMsg::AreaPick`). The empty state keeps its measured disabled look. Tests: 3 (columns and chip text, live chip / select after defining an area, pick selects); gate 410 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Stored settings** (the rest of the audit's `localStorage` family). New `persist.rs`: the files under `%APPDATA%\SketchRust\`, on only in the real app (`main()` sets `SKETCH_PERSIST`) and, in tests, per thread through `with_temp_store` (so no test can reach the user's profile, and parallel tests never see each other's files); a missing / invalid / BOM-prefixed file reads as "use the bundled value". **`sketch.defaultOptions`** -> `default-options.json`: Save as Default stores the draft, makes it `App::default_options` (what Restore Defaults now applies; it applied the code defaults before) and toasts `Current options saved as defaults.` (`...could not be saved as defaults.` on a failed write; the harness keeps the old session-only text); at start the stored options are applied (`applyOptions(storedDefaultOptions)`). **`sketch.areaCodes`** -> `area-codes.json`: written when a definition is saved or a new code created (the dialogs already said "saved permanently"), read at start into both the App and the dock; **New Sketch and a file carrying no table now keep the table in use** (the original's `resetToBlank` / `applyModel` do; the port reset it to the defaults). **`sketch.textLibraryLabels` / `sketch.symbolFavorites`** -> `text-library-labels.json` / `symbol-favorites.json`: both written after a library edit or Save Favorites (`persistLibraryState`), read at start (unknown symbols drop out; an empty list means the defaults). Checked in the real window with a temp profile (saved defaults hid the grid at start). Tests: 3 unit + 5 e2e (defaults round trip + Restore, session-only without a store, codes + both lists, New / file keep the table, invalid files); gate 407 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Undo covers the page's north angle.** `Snapshot.north_angle` (the original's `snapshotModel` carries `northAngle`); the App stamps the current page's compass when it banks a step and `restore_snapshot` puts it back on the page being edited, for undo, redo and the history list. A rotate or flip of the sketch turns the compass with it, so undoing one used to leave the arrow turned. Setting the north by hand is not an undo step of its own (as in the original) but rides in the next step's snapshot. Test: 1 e2e (rotate, redo, a hand-set bearing, a mirror). Gate: 399 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Dimension (measurement) labels** (audit tier 1). `Selection.dimensions` (`selectedDimensionIds`). `handles::dimension_anchor` / `hit_dimension` are `getDimensionAnchor` / `hitDimensionAt` (the label box = text + 6 px each side, turned with the wall; the hit anchor sits 16 px off the wall, the painter's 10 - the original's own quirk); the hit runs after the 4 px on-the-wall pass (`selection::on_wall`), so a click on the wall still picks the wall. Picked labels paint the select colour: a dashed leader to the wall's middle and a tinted box. **Drag** pins (`move_selection` re-pins from where the label draws + the move, anchored to the wall's middle; `pinned_dimension_point` is `Geometry.pinnedDimensionPoint`). **Delete** hides the label (`dimensionHidden`, pin dropped; the wall stays); **Space tap** with a selection runs `toggleSelectedDimensions` (one `Toggle Dimensions` step). **Select by type > Dimensions** picks every visible label (`Drawing::dimension_visible` = `isDimensionVisible`). **Fence / lasso** (`selection::pick_by_position`) also pick free labels, symbols, area label / size blocks and labels by position. Found and fixed on the way: (1) `getLineDimensionSide`'s polygon rule was never ported - labels of a finished area's walls drew on the placement default (inside for a clockwise draw) instead of outside; (2) a pinned label drew on `dimensionAnchor` instead of the pin carried by the wall's movement; (3) **Delete did nothing to selected text labels or symbols** (`clipboard::delete` only knew walls, areas and chains). Ten e2e selection tests moved their wall click from 8.5 px under the wall (now inside the label's box, as in the original) onto the wall. Tests: 10 new (pick vs wall, drag/pin/undo, pin rides the wall, Delete / Space, select by type, fence, outside labels, delete texts / symbols, highlight pixels); gate 398 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Per-area label fields** (from the audit's tier 1). `Polygon` gained `labelPosition`, `areaSizePosition`, `labelScale`, `areaSizeScale`, `labelFontSize`, `labelFontFamily`, `labelBold`, `labelItalic`, `labelUnderline`, `labelColor` - before, an original file's values were dropped on open and lost on save, and the blocks always sat at the default anchor. `handles::label_position` / `size_position` are `getAreaLabelPosition` / `getAreaSizePosition` (own position, else the anchor, else below the label by 7.5 px a line, scaled); `area_blocks` carries position, scale and typography and both the painter (`draw_area_labels`: colour, family, weight, italic, underline) and the handle boxes read it. **Move:** pressing area text or a free text label picks it and drags the selection; `move_selected` moves a pinned block with its area, a label alone (its figure with it, a separately pinned figure keeps its offset), a size alone, and free labels (the arrow-key nudge no longer moves them itself). **Rotate / flip:** `rotate_selected` turns pinned blocks and picked labels with the area and counter-turns their rotation; `rotate_sketch` / `flip_sketch` now also carry free labels and symbols (position + rotation) and the pinned blocks, and `getAllSketchPoints` counts them for the centre. **Scale:** `scaleSelected` / the wheel take `labelScale` / `areaSizeScale` (0.5-4). **Style:** Edit Tools shows the text group for a picked label / size block; Bold / Italic / Underline toggle (on unless everything picked has it), Grow / Shrink start from the auto-post size - each one undo step like the original's `Text Style` / `Text Size` (labels' were not banked before). **Orient** (H / V) now does what `orientSelected` does - set area blocks, labels and symbols to rotation 0 / 90 - instead of flipping / half-turning symbols only. Clone Area shifts pinned blocks with the copy. Tests: 2 unit + 10 e2e (drag, move, scale, style, orient, rotate, save/open round trip, painted position); gate 389 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Symbol rotation snaps to 15 degrees - and three older symbol bugs the work uncovered.** The drag now follows the original (`start - round(delta / 15) * 15`, Ctrl 1 degree; the first full step banks one `Rotate Symbol` undo and frees the symbol from its wall; an indigo wedge shows the turn) and the corner handles resize in 5% steps (Ctrl 1%, one `Scale` step), via screen-space handle boxes (`handles::symbol_box`). While testing it: (1) **symbols were drawn turned the wrong way** - `draw_symbols` rotated by `-rotation`, the original by `+rotation` (`ctx.rotate`, clockwise on screen) - so a window / door hosted on a diagonal wall ran across the wall instead of along it (it only looked right on horizontal and vertical walls); (2) **the symbol hit-test turned the same wrong way** (`point_in_symbol`), (3) **the rotation handle was only grabbable at the mirror image of where it is drawn** (the world-space handle used the wrong y sign), so the handle could not be picked up where it shows; and (4) **undo did not cover text labels or symbols** (the snapshot held only walls and areas, the original's also holds labels and symbols), so undoing a rotate / resize / delete of those did nothing - `Snapshot` now carries them. Tests: 1 unit + 5 e2e (diagonal-wall alignment via ink direction, 15 degree / Ctrl steps, undo to the wall, 5% resize, turned hit-test); gate 377 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **The remaining tooltips** - `tooltips::tip*` now take an owned title. Added: dock pin ("Keep sidebar open"), collapse / expand sidebar, each layer row's caret ("Expand group" / "Collapse group"), eye ("Hide layer" / "Show layer"), lock ("Unlock layer" / "Lock layer") and name (the layer's own name), the status bar's totals chip ("Calculation summary - click to open the Calcs panel"), snap pitch, Defined areas and underlay selects, the Pages navigator's close, ghost, delete and footer select, and Recent Files' Clear. Checked in the real window (Collapse sidebar, the chip's summary); the layer-row titles are plain state-driven strings. Gate: 372 app + 155 core, clippy 0, fmt clean.
- 2026-10-27: **Rotated text on the canvas.** The earlier note that iced 0.14 draws no rotated text was wrong: `Frame::fill_text` only caches text under a scale/translate transform; under any rotation both the wgpu and tiny-skia frames fall back to `Text::draw_with`, which lays the line out with cosmic-text and fills its glyph outlines as paths. So rotated labels need no outline code of our own - `draw_texts` and `draw_area_labels` push a transform (translate to the centre, rotate by the label's / block's degrees, clockwise on screen like `ctx.rotate`) and draw at the origin; the background pill and underline turn with the text, an area block's lines stack in the turned frame. Free labels (`rotation`) and area label / size blocks (`labelRotation`, `areaSizeRotation`) both paint turned, and the handle boxes now wrap what is drawn. **Also fixed:** selecting a text label crashed the real app - the Edit Tools text group asked for Phosphor icons that were never embedded (`text-b`, `text-italic`, `text-underline`) and `icons::bytes` panics on a missing one; they and the two the original uses for Grow / Shrink (`text-aa`, `text-a-underline`, replacing invented `text-plus` / `text-minus`) are now embedded (the five SVGs fetched from the pinned `@phosphor-icons/core@2.1.1`, the package the other icons came from). The e2e tests never rendered that group, so none caught it. Tests: 2 e2e (ink bounding boxes at 0 / 45 / 90 degrees for a label and an area label); gate 372 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Rotation handles and the text double-click prompt** (S6-071, S9-034, S9-036) - new `handles.rs` (text boxes, area-text layouts, handle hit-tests, drag maths). A selected free text label, area label, area size block and the selected areas draw the original's transform handles (`drawTransformHandles`); pressing one starts `Drag::RotateText` / `Drag::RotateArea`: text turns `start - round(delta / 15) * 15` live (one `Rotate` undo step), areas preview the rotated outline and call `rotateSelected` on release; Ctrl gives 1 degree steps (`CanvasDrag` now carries Ctrl). Area text became selectable: `Selection.area_labels` / `area_sizes`, picked by click (`hitTestAt`'s first pass: the precise box +5 px, else within 22 px of the block), taken by Select All and Select by type > Text, turned by Shift+R. `Polygon` gained `labelRotation` / `areaSizeRotation` (the size falls back to the label's), saved and fed to the SVG / DXF exports (`exporter` hardcoded 0) along with `lineThickness`. Double-clicking a text label opens the `Text:` prompt (`Action::EditText`, one `Edit Text` step). The auto-post painter now shares its layout with the handles (`handles::area_blocks`). **Limit:** iced 0.14 draws no rotated text, so on the canvas only the box / value / exports turn. Tests: 2 unit + 6 e2e (+ screenshot); gate 370 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Reference image 'saved with the sketch' checked against the original - nothing to build.** `buildSaveModel()` / `skx-format.js` carry no image; `saveImageWithSketch` only adds the image to the clean render and to the PNG export (drawing + bounds), which the port already does. Persisting it in the file would be a new format field the original cannot read, so it was left out and PARITY.md corrected.
- 2026-10-26: **Hover tooltips** (the original's `title` attributes) - new `tooltips.rs` wraps a control in iced's `tooltip` (500 ms delay, off-white box with a grey border, Segoe 12). Rail tools show theirs to the right, the quick-access / window / ribbon buttons below, the status bar's above; titles are the original's: all 17 rail tools ("Select/Edit", "Curve Wall", "Measure: chain legs, close for area, Shift locks 45 deg" ...), QAT "Save (F2)" / "Undo (Ctrl+Z)" / "Redo (Ctrl+Y)" / "Undo History" / "Print" / "Command search (Ctrl+K)", "Minimize" / "Toggle Full Screen" / "Close", the ribbon's titled buttons only, and the status bar's Center, Center & Fit, Zoom Fence, Grid, Snap, Area Grid Snap, Filter, Bluetooth, Touch Panel, North, Previous / Next / Add / Delete Page. The emulated e2e harness cannot show a tooltip (its screenshot has no cursor, its text search skips overlays), so the titles are unit-tested and the real window was captured instead (Pan, Chain Dimensions, Undo (Ctrl+Z), Center & Fit all appear). Gate: 362 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Pages navigator** (S10-040..045) - new `pages_nav.rs`. The rail's Pages button toggles the 168 px column between the rail and the canvas (`App::pages_nav_open`; the button highlights while open). Header 'PAGES' + close; one row per page ('Page N', the current one accent-tinted with a 'current' tag, others hover-highlighted with a 'ghost' tag that toggles the underlay and a trash button that appears on hover); clicking a row goes to the page; the trash goes to that page and runs the status bar's Delete Page flow (the 'Delete page N?' confirm, nothing happens on the last page); the footer select sets the underlay ('Underlay: None' / 'Page N'). `canvas_size` subtracts the panel so the pan/zoom canvas shrinks like the original's flex layout, and the Mini-Map snapshot rectangle follows. Tests: 1 unit + 2 e2e (open/close/toggle with canvas width, row switch, ghost toggle, hover-delete confirm, footer select) + screenshot; gate 360 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Edit Tools area buttons** (S7-054..062) - a selected area now shows the original's `contextual-area-tools` group in the Edit Tools ribbon: Fill Pattern / Line Style / Line Thickness (click cycles the select's options), Fill Color / Line Color (open the Color dialog), Area Props, Area Attributes, Auto Subtract and Flip Area. `setSelectedAreaStyle` is `area_props::set_selected_area_style`: the property goes on every selected area and the three outline properties restyle the walls its edges ride (`lineColor` -> the wall's `color`, solid = no style). Polygons gained `lineStyle` / `lineThickness` (saved, and drawn: dash and width of the outline, min 2 px when selected). The area group sits first and the wall / text / symbol group shifts right by its width when both are selected. **Found and fixed on the way:** ribbon buttons carrying their own `action` (Set Length, Curve, Flip / Wider / Narrower opening, Bold / Italic / Underline / Grow / Shrink) were never dispatched - `command()` only consulted the caption table - so they were dead in the real UI; an explicit action now wins. Tests: 2 unit + 4 e2e; gate 357 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Select All takes text labels and symbols** (`selectAll`; Ctrl+A and the ribbon button), respecting layer locks. The hidden `activate-module` / `extended-data` commands have no UI entry in the original and stay out. Gate: 351 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Insert Vertex, Curve Wall and wheel-bowing** - (1) **Insert Vertex** (Insert key or rail tool): a click within 10 px of a wall splits it at the projection (`insertVertexAt`): the second half is a clone with a fresh id, both halves lose any pinned dimension, and every area whose boundary runs along the wall gains the vertex and the two split edges (in path order, either direction); the three notices ("Click directly on a wall...", "That point is already a vertex.", "Straighten the curved wall...") change nothing; one undo step. (2) **Curve Wall tool** (`Mode::Curve`): pressing a wall selects it and starts a `Drag::Curve` that keeps the rise and the pointer's normal distance at grab time; moving sets the bow `4*atan2(2h, chord)` (clamped) live, re-signing every area edge on the wall (`Drawing::set_line_bow` = `commitCurveBow`), and the recalc signature now includes edge bows so GLA / net values follow while dragging; the selected wall paints in the preview ink, with the original's guides (chord, rise, radii, arc length / chord / rise / radius / degrees). (3) **Curve handle**: every selected wall shows the green square on its arc, and pressing it in Select mode starts the same drag. (4) **Select-mode wheel**: selected walls bow by 15 deg (Shift 5, Ctrl 1; height mode 1 ft, Shift/Ctrl 0.1 ft), else selected text / symbols scale x1.08 / x0.92, else zoom. The Shift+wheel bow of the last drawn wall now reads the `D` toggle (it read an unrelated option and so never followed D). Bow changes are not mirrored to other pages' copies of the wall. Tests: 5 unit + 5 e2e (insert splits and undoes, curve-tool drag with live area, handle drag, wheel bow / height / scale / zoom, guide and handle pixels); gate 350 app + 155 core, clippy 0, fmt clean.
- 2026-10-26: **Quick wins from the all-screen audit** - (1) **The canvas never drew
  defined areas.** `drawPolygons` was missing: only a *selected* area painted
  (and only as a flat highlight). `render::draw_areas` now fills each defined
  area (the polygon's own colour / pattern / opacity, else its area code's, else
  `fillGLA`), draws the diagonal / crosshatch / dots patterns clipped to the
  ring (curved edges flattened), the outline in the area's colour, the selection
  highlight, and the auto-post label and size (`getAreaLabelGroups`: code, name,
  net `N sf`, base, factor; stacked centred lines, the size below the label,
  `autoPost*` fonts). `Scene` carries the area codes and the recalculated nets.
  (2) **Filters:** Area fills now gates that pass, Text labels and Symbols gate
  theirs. (3) **Hotkeys:** Shift+R / Ctrl+Shift+R rotate the selection 5 / 1
  degrees (text and symbols turn in place, geometry about its centre), + / -
  scale selected text and symbols (clamped like the original), F / M flip the
  sketch, L / R rotate it (R no longer opens Insert Room Shape - that stays on
  the rail), arrows nudge the selection 1 ft (Shift 0.1) then sweep for closed
  loops. (4) **Select by type** Text and Symbols select those kinds (Dimensions
  still has no set). (5) **Context menu Make Negative and Clone Area** work (a
  negative `factorOverride`; the clone is 2 ft over with its own walls, selected),
  so all 13 entries act. (6) The status-bar Touch Panel button toggles the panel.
  (7) **Undo ordering bug:** Rotate / Flip Area, Auto-Subtract, Area Attributes,
  Area Properties, Orient, Flip Opening and Opening Width banked their undo step
  AFTER the change (so undoing restored the changed state); they now snapshot
  before, as the original's `pushUndo` does. Not done: Select All still skips
  texts and symbols; H / V orient does not touch text labels; the hidden
  `activate-module` / `extended-data` commands.
- 2026-10-26: **Option keys wired (the ones the original really reads)** -
  `alignmentLines` hides the witness guides, `alignmentPoints` hides the vertex
  dots and the snap cross; `sameColorAsLine` paints dimension labels in the
  wall's own colour (the rubber-band preview stays red); `dimensionFont` /
  Bold / Italic / `dimensionFontSize` drive wall and chain dimension labels
  (`theme::font_named` builds a font from family + weight + style); new text
  labels take `textFont`, size, Bold / Italic / Underline and `textColor` (the
  old `#e2e8f0` means automatic, stored empty, drawn in the canvas theme's ink;
  a saved `color: null` now loads), and are drawn centred in their own font
  over `textBackground` at `textBackgroundOpacity` (4px pad, radius 3) with an
  underline; `pointerSize` (32 / 64 / 128, the `-light` variant on the White
  canvas) sizes the drawing pointer; PNG export honours `imageSize` and, with
  `saveImageWithSketch`, draws the visible reference image beneath the sketch
  (decoded with the `image` crate, rotated / flipped / at its opacity, and the
  bounds grown by the original's half-diagonal term) - `export_png_with_backdrop`
  in core; `autoSubtractOverlapped` = false makes new areas `never` (auto-close
  and the closed-loop sweep). Also fixed the reference image's flip (a lone
  Flip Horizontal mirrored both axes). Not rotated: label text (iced has no
  rotated text); the label backing width is estimated (~0.52em a glyph) because
  a canvas draw cannot measure text. Dimension font checked in a headless
  screenshot (Georgia bold italic in the wall colour); the text-label look was
  not seen in the real app.
- 2026-10-26: **Report fidelity pass** - the subject block is now the original's
  grid, not six free-floating strings: a 12pt title band, then six 9.9pt rows
  (7.02pt regular text - the precise CSS sets labels to normal weight - 4.6pt
  side padding, a .35 #777 rule under all but the last) using the `two` (105pt
  | rest), `four` (59 | 103 | 97 | rest), `client` (98 | rest) and `appraiser`
  (136 | rest) column templates, a label then 4pt then its value, each cell
  clipped at its column edge. Every `.report-section-title` (subject, sketch
  heading, summary, the three comment tables) shares one band helper with the
  .45 #555 rule under it; the sketch heading has its side rules. Calculation
  rows and comment lines sit on the CSS baselines. And a real data bug: the
  export model's polygons carried `net: None` and chord-sum perimeters, so the
  report's Net Totals and Net LIVABLE were always 0 in the app; they now carry
  the recalculated net, factor, gross and curve-aware perimeter (this also
  feeds the SVG/PDF labels). Fonts stay Helvetica (no Verdana in a hand-built
  PDF), so column clipping is a little more generous than the browser's.
- 2026-10-26: **Page Layout drives the PDF report (S11-077)** - the original's
  report renderer IS in the captured source (`SketchReport` at the end of
  canvas-engine.js), so this follows it: `export_pdf_report` renders ONE REPORT
  PAGE PER SKETCH PAGE (the export model now carries every page; the current one
  comes from the live drawing) under a `ReportLayout`: the seven captions (blank
  falls back to the default, as `normalizeLayout`'s `||`), `showSketchComments`
  (the general comments as a wrapping 6pt block pinned to the sketch box's
  bottom), `showCommentTables` (off: the calculations take the whole 540pt
  lower box), and `showPageNumbers` ("Page N" per page). `currentForm`,
  `calculationAddendum` and `deedCallsAddendum` are normalised but never read by
  the original's renderer (the form only lands in a `data-current-form`
  attribute), so - faithfully - they have no effect here. The lower section
  was redone to the REPORT_CSS: a grey header band, right-aligned numeric
  columns, the `Net LIVABLE cnt N (rounded) T` line, and the comment tables as
  the original lays them out (table 1 across the top of the 216pt column, 2 and 3
  beneath) - before, they were drawn over the calculations. Text is centred /
  right-aligned with real Helvetica metrics. Also fixed: the sketch geometry was
  anchored to the box's bottom edge and drew below its box. The Print Preview
  toolbar now shows the real page count and the Sketch Page Title.
  Checked by rasterising the output with PyMuPDF.
- 2026-10-26: **Small Screen 11 gaps closed** - Open and Import pickers list
  `.skx .apx .json .sketch` (the core reader already took a full `.apx`);
  Import CAD lists `.dxf`/`.dwg`, a `.dwg` raises the conversion-hint alert, a
  `.dxf` imports through `parse_dxf` and raises the original's `describe`
  alert (lines / shapes / labels, entity count, "Units detected: X (source).",
  skipped list; `sketch_core::cad_import::describe_import`), and a parse failure
  alerts `Could not import <name>: <reason>`; `options.promptOnDeletion` makes
  Delete ask "Delete the selected sketch items?" before banking the undo step
  (nothing selected still returns first).
- 2026-10-26: **Bluetooth (Disto) connect (S12-013/014/083, S4-030, S12-065)** -
  `disto.rs` ports `connectDisto`, which in the original is connection-only
  (`requestDevice({acceptAllDevices})`, then the `#disto-status` chip takes the
  device name and the toast says `Connected to <name>.`; a cancelled chooser is
  silent; other failures toast `Bluetooth connection was not completed.`; no
  Bluetooth toasts the unavailable message, reworded for "this computer").
  A pure state machine (Idle / Scanning / Picking / Connecting, unit-tested
  incl. stale results) drives a chooser card (title, scan status, the device list
  with DISTO-named first, Cancel / Rescan / Connect; Escape cancels). The
  transport is btleplug 0.11 on a worker thread with its own tokio
  current-thread runtime (a 3 s scan, then `peripheral.connect()` with a 10 s
  timeout); the test build swaps in pending futures so no test touches the
  radio. Wired: Tools > Connect, the status bar Bluetooth button, the
  palette's "Connect Bluetooth device" (the last unbuilt palette entry), and
  the Tools ribbon's `Disconnected` chip, which takes the chosen device's name
  (clipped with an ellipsis; the chip stays dimmed like the original's disabled
  button). Run in the real app: the chooser opened, a real scan listed 9
  nearby BLE devices, Cancel closed it silently. NOT verified: connecting to
  any device (none was chosen - they were not mine) and anything against a
  real DISTO; the Disto measurement stream is not in the original and is not
  here. The inventory's S12-030 is the palette's "Save As", not Bluetooth.
- 2026-10-26: **Popup menus + themes: Flip Sketch, Canvas background, Interface
  theme (S2 View, S6 transforms, S12-050/051/061/062)** - `popup_menu.rs` ports
  `showCommandPopup` for the three menus (190px panel, 29px rows, placed at
  `left=min(W-w-8,max(8,anchor.left))`, `top=min(H-h-8,max(8,bottom+2))`, a
  dismissing backdrop, a #63B3ED check on the current theme; no Escape, as the
  original). Home > Flip Sketch (Horizontal / Vertical) runs `flipSketch`
  (undo step, mirror about the centre, bows negate, north mirrors; the
  transform existed unreachable). View > Background sets `state.canvasTheme`
  via `canvas_theme.rs` (White / Dark / Blue: bg, both grid weights, dimension
  labels, witness lines; toast "Drawing background: X"; saved in
  `settings.canvasTheme`, restored on open). View > Interface switches the
  chrome through a runtime palette: the eight `theme.css` custom properties
  became `theme::bg_app()` ... `muted()` (a thread-local `InterfaceTheme`,
  dark/blue/light values from `:root` / `body.blue-theme` / `body.light-theme`)
  and every dialog's local colour consts were folded into them (three measured
  native-dialog greys stay as `muted_alt`/`border_alt`/`text_alt`; light mode
  drops the double text pass). The choice persists to
  `%APPDATA%\SketchRust\interface-theme.txt` (the `sketch.interfaceTheme`
  stand-in, gated on `SKETCH_PERSIST`) and loads at start. The palette's three
  entries open the same menus (anchored at the ribbon button when its tab
  shows, else the hidden element's zero rect), so only Bluetooth is still
  unbuilt there. Driven in the real app: Interface > Light recoloured the
  whole chrome, Background > Dark recoloured the canvas, Flip opened, and Light
  survived a relaunch. Not themed: the wall/area colours (fixed in the
  original too), the cursor images, some per-icon accents.
- 2026-10-26: **Command palette, full (S12-028..070, S12-074) + the QAT buttons** -
  `command_palette.rs` rebuilt from the sidebar-redesign layer: the original's
  42-entry `CATALOG` (labels + F2/Ctrl+P/Ctrl+Z/... hints), the label-substring
  filter, wrapping Up/Down (raw key stream, since the focused input swallows
  them), Enter / a row click runs (closing first), Escape / a backdrop press /
  Ctrl+K again closes, the input focuses on open, "No matching command.", and
  the measured look (520px panel at top 12%, .35 backdrop, active row
  accent-soft). Runs reuse the existing handlers; `save-as`, `new`, `open`,
  `import` and the three exports toast "<id> is not available." exactly as the
  original's `executeCommand` map does; `rotate-sketch` is live now
  (`Message::RotateSketch`: undo step, whole sketch + north turn 90). Flip
  sketch, Canvas background, Interface theme and Bluetooth say they are not
  available in this port yet (their popup menus / transport are unbuilt).
  Driving the real app found that the whole QAT (Save, Undo, Redo, Undo
  history, Print, the magnifier) never worked: each `button` captured the
  press before the wrapping `mouse_area` saw it. The buttons now carry their
  messages (Save is wired too). Checked in the real app: the magnifier opens
  the palette, a row click ran Print preview, Close Preview closed it.
- 2026-10-26: **Mini-Map (S12-012/018/055)** - `minimap.rs` ports `toggleMiniMap`:
  View > Mini-Map (and the palette's new "Mini map" entry) snapshots the canvas
  and shows it in the 260px `.mini-map-overlay` card at the canvas's
  bottom-right (14px edges, 8px padding, 1px border, shadow) with the
  translucent x ("Close Mini-Map"); the button again closes it. The snapshot is
  static, as in the original. The original reads `canvas.toDataURL`; here the
  window screenshot (`window::screenshot`) is cropped to the canvas rectangle
  (`crop_rgba`, unit-tested). A harness without a window gets a blank
  canvas-coloured image, and the headless renderer paints images black, so the
  card's picture was verified in the real app (SKETCH_DPR=1, drew a closed
  rectangle with real clicks, opened View > Mini-Map: the card shows the canvas
  only, and the x closes it). That run caught a bug: cropping by the tracked
  canvas size was wrong, so the crop now derives the canvas rectangle from the
  screenshot's own size (`canvas_rect`).
- 2026-10-26: **Subject Info, full (S12-001..011)** - `subject_info.rs` rewritten:
  the model is the original's object (the 13 parcel fields with `zipCode`,
  `gpsCoordinates`/`gpsLatitude`/`gpsLongitude`, `generalComments`,
  `commentTables` as line lists, `additionalFields` as `{name, value}`; unknown
  suite keys ride through a flatten map; first-port saves with `zip` / string
  extras still load). The dialog edits a draft and commits on OK: the three tabs,
  the Auto-Fill row (shown until auto-filled; the standalone toasts that it
  needs the suite), the read-only metadata strip, the collapsible additional
  fields, the comments and three comment-table editors, Save/Apply Default
  (`%APPDATA%\SketchRust\default-subject-comments.txt`, gated on
  `SKETCH_PERSIST`), the GPS refusal text, and "Subject information updated.".
  No Escape/backdrop close, as the original. The PDF report now prints every
  parcel field (file/parcel no., county, owner, client, appraiser, date).
  Deviations: Inspection Date is a text input (YYYY-MM-DD, validated on OK)
  where the browser has a date picker; sizes come from theme.css, not a pixel
  comparison against a capture (none exists).
- 2026-10-26: **Print + Print Preview (S11-054/055) + the Keyboard Shortcuts
  sheet (S12-015) + Email (S12-026/071)** - `print.rs`: File > Print... and the
  QAT printer write the report PDF to `%TEMP%\SketchRust\<name>.pdf` and open it
  with the OS handler (the browser's print window; the viewer owns the print
  dialog; skipped under `cfg!(test)`), toast "Opening the appraisal report to
  print."; File > Print Preview... opens the overlay (toolbar `Print / Save
  PDF`, `Close Preview`, `N page(s) · Paper portrait`; Escape closes) over a
  paper-proportioned sheet with the title block + the sketch raster.
  `SketchReport.render` is not in the captured source, so the sheet is an
  outline, not the full report (the PDF it opens is the full one). `shortcuts.rs`
  ports `showShortcuts` (F1 / Help button; Escape/F1/Close/backdrop close);
  Ctrl+F1 and File > Email... open the support mailto.
- 2026-10-26: **Layer UI depth (S10-025..038) + underlay ghost (S10-046)** -
- 2026-10-26: **The library managers (S9-008..013 + S8-008 as captured)** -
- 2026-10-26: **Recent Files (S11-049..053) + exit confirm (S11-080) + the file
  keyboard map (Ctrl+S/O/N, F2/F3/F12/Ctrl+F12; File > New wired)** -
- 2026-10-26: **Import merge (S11-044) + the .skx fallback toast (S11-068)** -
  File > Import... picks a sketch (skx or legacy JSON) and merges its FIRST
  page at the (2,2) offset: fresh gen ids for walls/areas/labels/symbols,
  the edges' lineIds and subtractTargetId remapped, each imported area
- 2026-10-26: **Breakdown downloads (S7-070/071) + Calcs Expand/Collapse All
  (S7-016/017)** - `breakdown::to_json` (the areas at the export rounding +
  the totalsByCode/ByGroup/GLA/All rollup) and `to_xml` (the tns:sketch
- 2026-10-26: **The reference image module (S10-007..020, S10-047..049) + the Geo
  menu (S10-021..024)** - `reference_image.rs` models `state.bgImage` (the
  bytes + the natural size decoded from the PNG/JPEG headers, the world
  center/scale/rotation/opacity/flips, the Set Scale and Orient Guide
  modes) with the scale-ratio and orient math unit-tested, and `parseGpsPair`
  (decimal + DMS with the mojibake degree signs). The ribbon's Reference
  Image tab is fully wired: Load/Show/Hide/Remove (confirm), Set Scale
  (two clicks + the known-feet prompt), Smaller/Larger (x0.8/x1.25),
  Rotate Left/Right, Edit Mode (the Toggled look; the drag repositions, the
  wheel rescales x1.05/0.95 instead of zooming), Orient Guide (two clicks +
  the H/V prompt), the flips, and the intensity slider (the invisible iced
  slider over the drawn pill). The canvas renders `drawBgImage` beneath the
  drawing (translate/rotate/flip/draw_image at the intensity opacity) and
  the scale/orient point markers on top. The Geo popup (Load Map Image /
  Auto Locate by Coordinate/Address) saves the georeference onto the
  subject's gps bag (gpsCoordinates/gpsLatitude/gpsLongitude round-trip in
  the save model). The image stays session-only, as captured. Gate: 271
  tests green (5 unit + 2 e2e), clippy 0, fmt clean.
  document: the BOM + CRLF, the subject line, the three comment tables of
  ten from the subject model, per-page areas with UND skipped, the
  description/multiplier from the code table) - both behind the Breakdown
  dialog's Download JSON/XML buttons (rfd save, the original's names); the
  report and the downloads share `breakdown_results()`. The Calcs panel's
  Expand All / Collapse All now drive the dock's collapsed-group set. Gate:
  264 tests green (2 core export tests), clippy 0, fmt clean.
  assigned its dedicated layer, `pushUndo("Import")` and the
  `Imported {name}.` toast; a geometryless file toasts the error. The save
  now falls back to the legacy JSON with the
  "Could not write .skx - saved as JSON instead." toast (S11-068). Gate: 264
  tests green (2 import unit tests), clippy 0, fmt clean.
  `recent.rs` models the entries (`{name, lastOpened, data}`, deduped,
  newest first, 10 x 1.5MB caps, per-entry-tolerant parse), an ISO
  timestamp via the civil-date math and the `toLocaleString`-style display;
  the File backstage's Recent Files item toggles the 300px submenu (header +
  Clear + the file rows / the two-line empty note); a row reopens the model
  (apply_model + re-record); records land on save + open; the sidecar mirror
  (`%APPDATA%\SketchRust\recent-files.json`) stands in for localStorage,
  gated on `SKETCH_PERSIST` so the e2e stays deterministic. Exit asks
  "Close this sketch? Save your work first if needed." (title bar + backstage
  Exit). Gate: 262 tests green (3 new e2e + 3 unit), clippy 0, fmt clean.
  `text_manager.rs` ports `showTextLibraryManager` (the dock's Edit button):
  the list (size-14 select, range-filtered, localeCompare-sorted) + label
  input + the alpha rail + Add/Remove/Replace with the original's four
  toasts, per-save sort/dedupe, `textLibraryLabels` persisting. 
  `symbol_organizer.rs` ports `showSymbolOrganizer` (the "Organize..."
  button - the captured build has NO Manage Symbols layer): the checkbox
  list of every symbol (label-sorted, deduped, filter input), Restore
  Defaults / Save Favorites, `state.symbolFavorites` now round-trips
  (None falls back to the library defaults) and the Favorites category
  lists the saved set. Toasts: "Favorite symbols updated." Both dialogs
  use the sidebar-manager overlay (rgba .46 backdrop, 430px card, backdrop
  press closes); the input focuses on open. Gate: 256 tests green (2 e2e +
  6 unit), clippy 0, fmt clean.
  `layers.rs` becomes the layer tree model: `areaPolyId` (numeric-legacy
  serde), `flattenLayerOrder`/`layerAncestry`/`descendantLayerIds`,
  `uniqueLayerName`, `createLayerRaw`, branch-removal `remove`,
  `visible_rows` (collapsed-group fold) and `moveLayer` (before/after/inside
  with the can-move guards). `dock.rs` replaces the placeholder panel with
  the real tree (S10-025): per-row caret/folder/shapes icon/name/eye/lock,
  depth indent, active outline, hover/hidden/locked/dragging fades, the
  empty note, drop zones (group 6/13/6, leaf split at the original ratio;
  inside-on-leaf resolves to after) with drop markers, inline rename (Enter
  commits, Esc cancels via the raw-event subscription + K::Escape, blank
  refused) and the right-press context menu
  (Rename/Hide-Show/Lock-Unlock/Export; routed through the EventTap because
  iced right presses carry no cursor for mouse_area). `main.rs` ports the
  layer system: `assignAreaToDedicatedLayer` + `removeEmptyAreaLayer` + the
  lazy per-update sync (`getAreaLayersForCurrentPage`'s half),
  `exportLayer` (rfd save, Sketch-Layer JSON, toast), `autoAssignLayer` +
  `reorderSelectionWithinLayers` wired to the canvas menu's Send to Back /
  Bring to Front (S10-037/038, now is_wired; toast
  "Moved selection to layer X."), `setLayerVisible`/`setLayerLocked` undo
  labels + branch selection clears, and undo snapshots now carry
  `state.layers` + `activeLayerId`. The canvas honors layers:
  `applyLayerOrder`'s stable paint-order sort runs with the per-update sync,
  hidden layers drop out of every render pass (walls, solids, bowed,
  symbols, texts) and hidden/locked entities refuse selection
  (`LayerGates` through hit_test/select_at/select_all/fence/lasso/
  select_by_type). 9 new layer e2e tests + the Send/Bring e2e; 7 unit
  tests in layers.rs. Gate: 248 tests green, clippy 0, fmt clean.
- 2026-10-26: **Chain dimensions (S9-024/035/040/045) + Page Layout dialog
  (S11-033..037)** - `chains.rs` (records `{id, seedLineId, side, offset,
  showOverall, hiddenSpans}`, tier solver, 8-px straight-wall seeding,
  hit/coverage), the Rail tool wired (`handleChainClick` order: pushUndo then
  record; miss toast), Select-mode chain-first hit with Ctrl toggle +
  `chain-offset` drag (`Move Dimension Chain` banked), `deleteSelected` chain
  removal, undo snapshots + per-page storage (`dimensionChains`) + save/load
  round-trip (test extended with a chain on page 0), `drawChains` port
  (witness stubs, ticks, hiddenSpans/minimum-gated span labels, overall tier,
  coverage-gated per-wall labels). `page_layout.rs` ports `state.pageLayout`
  (paperSize / currentForm composition / addenda / captions with
  normalizePageLayout clamps in the serde), the showPageLayout card (paper
  radios, paper-driven form picker, five switches, seven caption fields,
  Restore Defaults / Apply / Cancel), File > Page Layout wired,
  `settings.pageLayout` written + parsed (the passthrough stop removed), PDF
  export reads `layout.paperSize`. Gate: 231 tests green, clippy 0, fmt
  clean; the dock double-click e2e flake passed in isolation and in the full
  gate.
- 2026-10-26: **Screen 8 complete (Symbol Placement + foundational features)** -
  wall-snap, real rendering, selection/move/resize/rotate handles, follow-along,
  flip/opening-width, command palette, touch panel, file I/O, room shapes,
  text labels + styling, measure tool, text library dock, layer model, subject
  info dialog. 221 tests passing, clippy clean.
- 2026-10-25: **7k complete (Breakdown Details dialog)** — `breakdown_dialog.rs`
  ports the measured 1080×720 overlay (S7-003): report `<pre>` in 12px Consolas
  `#ccc`, the 647px `#1E1E1E` preview pane with `previewSvg` viewBox math, and
  the four measured buttons (Copy, Feet & inches toggle, Download JSON/XML —
  stubbed to native-save-dialog toasts). Ribbon wired (`Area\nBreakdown`). 5 e2e
  tests. Two `breakdown.rs` fixes landed: unsigned underflow in `normalize`'s
  winding-reversal bow lookup (latent on clockwise polygons), and a `<3` point
  guard mirroring the original's early return. Commit `dab7173`.
- 2026-10-24: **7j built (recalcAutoSubtract core)** — sketch-core
  area_calc.rs ports AreaCalc.recalculate (containment tree, subtraction
  roll-up, factors, rollup totals) with no boolean ops; verified live that UND
  deducts (null level = wildcard). Wired behind a signature guard; calcs tree
  reads Subtracted/Net Area from the pass. 8 core unit tests. Gaps: totals
  chip static, Area Breakdown report, GLA event.
- 2026-10-24: **Screen 7i built (live area palette)** — dock::paint_palette
  ports the original's real renderer paintPalette (short group labels
  'Living space' etc., UND excluded from groups and rendered as the trailing
  "Undefined · clear" tile, first-seen group order, codeColor chain, tile
  click arms the type). Static GROUPS transcription deleted. Reference-checked
  against dock-Areas: overall 0.89%, dock 1.74%. Named gaps: usage badge, tile
  drag, collapse state. 6 unit tests.
- 2026-10-24: **Screen 7h built (area-definition editor)** — new def_editor.rs
  renders showAreaDefinitionEditor (500x358 at (450,251), five rows, 474x62
  preview, Cancel/Save Permanently), opened from the Areas dock's "Edit areas…"
  link. Save rewrites the entry, renames the code's polygons and repaints their
  edge lines, then reports "X settings saved permanently.". Pixel-verified:
  card box exact, every band within 2px, overall 2.05% (the lowest of screen 7).
  Swatches static, preview pattern not drawn (named). 4 unit tests.
- 2026-10-24: **Screen 7g built (New Area Code dialog)** — new
  new_code_dialog.rs renders showAddAreaCode (500x540 at (450,160), ten 39px-pitch
  rows, 474x62 preview, Cancel/Create Code), opened from the Areas dock's
  "New area" link. Create logic matches the live page exactly for create / no
  code / no name / duplicate / junk multiplier (Math.max(0, Number(x)||1)).
  Pixel-verified: card box exact, all seven labels within 1px, overall 2.38%.
  Swatches are static and the preview doesn't draw the fill pattern (named).
  4 unit tests.
- 2026-10-24: **Screen 7f built (area-code table editor)** — new
  codes_editor.rs renders showAreaCodeEditor (720x554.8 at (340,152.59) over a
  rgba(0,0,0,.62) overlay, monospace textarea on #111111, four #3E3E42 buttons),
  with iced's text_editor for the JSON. Apply runs the ported AreaCodes::from_json
  and shows "Area code table updated." or the catalogue's verbatim error;
  Restore defaults rewrites the 32-code catalogue. app.area_codes is real state
  now. Import/Export need a native file dialog (deferred, named). 4 unit tests.
  Pixel-verified: card box exact (153/706 x 340/1059), every band within 1px,
  overall 2.92%; the comparison forced the title/hint offsets, the textarea's
  first line and an explicit 16.5px line height.
- 2026-10-24: **Screen 7e built (Calcs tree widget)** — the dock's Calcs panel
  renders the model: group headers (11px semibold, monospace #63B3ED total,
  carets), area rows ("name : net"), and detail rows (Base/Factor/Total/
  Subtracted/Net/Perimeter) from theme.css's .calcs-* rules; groups start
  expanded and areas collapsed, an area row selects its polygon, and the summary
  comes from calcs::summary. 1 e2e covering the whole flow. NOT pixel-compared:
  no reference exists for the populated tree yet.
- 2026-10-24: **Screen 7d verified (View Calcs numbers)** — new calcs.rs is
  updateCalcsPanel's computation: areas grouped by code in first-seen order with
  group totals, and per-area Base Area / Factor / Total / Net Area / Perimeter
  (perimeter via the existing geometry engine); UND counted, not grouped.
  Verified against the live original (GLA1 1250.00 / GAR 400.00, rows
  "First Floor : 1000.00" etc., Perimeter 40.00). The summary is the original's
  four rows (Defined / Undefined / Total Gross Living / Total Gross Building),
  with the areaTotals fallback the unported recalculate() implies. View › View
  Calcs toggles the dock's Calcs tab. 5 unit tests. Still open: recalcAutoSubtract
  and the tree widget.
- 2026-10-24: **Screen 7c verified (Area Attributes dialog)** — new
  attr_dialog.rs renders showSelectedAreaAttributes (520x261 at (440,299.5),
  Area/Attribute/Value header, four rows on a 30px pitch, Cancel/OK), read-only
  from the Home ribbon and editable from Edit Tools; OK merges the four values
  and banks undo. Pixel-verified: card box exact, every content band exact
  (319/355/398/428/458/488), buttons within 1px. Also corrected the 7b
  dialog's buttons to the measured #3E3E42 (no accent primary in this family).
  3 unit tests.
- 2026-10-24: **Screen 7b built (Area Properties dialog)** — new area_dialog.rs
  renders the measured card (520x456 at (440,202), 3x2 auto-post grid, Suffix,
  Cancel/Apply) with pick_list stand-ins for the original's three native
  selects; opened from Edit Tools › Area Props and the context menu's
  Properties…, Apply runs apply_properties and banks undo, and a no-area
  selection shows the original's notice. 3 unit tests on its state machine.
  Pixel-verified against a live reference: the card's border box is exact
  (202/657 x 440/959) and all thirteen content bands match within 1-3px;
  overall 2.86%. The comparison forced three fixes: no container padding plus a
  PIN_Y=2 calibration, the two missing group captions ("Calculation
  Adjustments", "Automatic labels") with their rules, and an Edit Tools panic
  when an area (not a wall) was selected.
- 2026-10-24: **Screen 7a verified (area properties model)** — the area-code
  catalogue already existed in geometry.rs (my duplicate area_codes.rs was
  deleted before commit). Polygon gained factor / measure_to / auto_post /
  attributes (additive, serde-defaulted). New area_props.rs transcribes
  updateSelectedAreaProperties / updateSelectedAreaAttributes /
  toggleSelectedAreaAutoSubtract, checked against the live original; the
  Number(null)==0 empty-Factor quirk is reproduced and pinned. The dialog's
  full control geometry is measured into refs/ui-spec/area-properties-spec.json
  for the widget step. 6 unit tests.
- 2026-10-24: **Screen 6i verified (drag-move + endpoint handles)** — Screen 6
  COMPLETE. transform::move_selected (both regimes: a connected wall stretches
  when lines move; only an area's own boundary moves with the area, plus
  neighbours glued by a shared wall) and move_line_endpoint (a shared vertex
  moves everywhere, area corners follow, a bowed wall keeps its angle via
  chord/2*tan(bow*pi/720)). All four cases exact against the live original
  (arc height 1.316525 for a 20 ft chord at 30 deg). Drags: endpoint handle
  (9px tolerance) grabs the vertex, a hit starts a selection move, a miss
  starts the marquee; moves snap to whole grid steps and bank one undo step;
  the release closes joined loops. ownBoundaryLine detachment not ported.
  4 unit + 2 e2e.
- 2026-10-24: **Screen 6h verified (marquee, fence, lasso)** — fence_select and
  lasso_select use the original's containment rules (both endpoints inside /
  all area points inside; additive keeps the earlier pick; a lasso under three
  points selects nothing), checked against the live original with identical
  fixtures. The pointer is the marquee tool in Select mode: a press that hits
  nothing starts the drag, a press on a wall keeps the click path, and
  FenceOutcome routes Zoom / Select / Lasso from one release handler. 3 unit +
  2 e2e.
- 2026-10-24: **Screen 6g verified (rotate and flip)** — new transform.rs
  transcribes rotateSelected / flipSelectedAreas / rotateSketch / flipSketch:
  selection transforms use a point-key set so shared corners stay welded, and
  every reflection negates the signed bows. All six cases checked against the
  live original on the 10x10 square (north 30 -> 120 after a sketch rotate,
  330 / 150 after the two flips; rotating one wall also moves the walls and
  area corners sharing its endpoints). Context menu Rotate 90 / Flip Horz /
  Flip Vert now act, leaving five of the nine entries live. rotateSketch /
  flipSketch are verified but unreachable (their buttons are hidden in the
  original's build) and marked allow(dead_code) with that reason. 7 unit +
  2 e2e.
- 2026-10-24: **Screen 6f verified (canvas context menu)** — new
  context_menu.rs renders showCanvasContextMenu's 190x434 panel (13 rows,
  5 separators, 5px padding, 1px border, 2px under the cursor, clamped 8px
  inside the viewport). Pixel-exact: all seven border/rule rows match the
  reference (302/341/466/533/629/696/735) and the button text bands match
  within 1px; reference overall 1.44%, canvas 0.70%. Right-click on the
  canvas ends the pen, selects an unselected hit, then opens the menu; the
  rail draw-tool right-click still wins. Delete/Cut/Copy/Paste wired; the
  other nine entries are deferred with reasons (transforms, layers, areas,
  properties). Six Phosphor glyphs added. 4 unit tests.
- 2026-10-23: **Screen 6e verified (select by type)** — new select_menu.rs +
  selection::select_by_type: Edit Tools › Select opens the six-entry command
  popup (All/Areas/Text/Dimensions/Symbols/Freeform) anchored under the
  button; rules per the original (All = everything, Areas = polygons,
  Freeform = interior walls; text/dimensions/symbols clear because those
  selection sets do not exist yet). ALSO fixed the shared command-popup row
  spec measured against the live original (29px rows, 13.33px labels, 13.34px
  icons, borders inside the box, ROW_Y0 = 12 calibration) — this corrected
  5d-2 and 6c-2 too: wall-shape menu overall 1.05% -> 0.97%, select-menu
  1.36%. 6 unit + 1 e2e.
- 2026-10-23: **Screen 6d verified (clipboard + delete)** — new clipboard.rs
  transcribes copySelected/paste/deleteSelected: copy takes an area's boundary
  walls; paste offsets 2 ft, remints ids, remaps edges onto the copied walls
  (rebuilding when a wall is missing) and selects the copies; delete applies
  the original's rules (a deleted boundary wall reopens its area; areas along
  a deleted wall go; walls a surviving area uses are kept; a deleted area's
  coincident outline never takes a neighbour with it). promptOnDeletion is
  false by default so Delete does not prompt. Live-original parity checked on
  all five cases. Edit Tools Cut/Copy/Paste/Select All/Delete wired; Ctrl+C/X/V,
  Delete, Backspace bound. 5 unit + 2 e2e.
- 2026-10-23: **Screen 6c-2 verified (undo history popup)** — new
  undo_history.rs renders showUndoHistory's command popup (same component as
  the wall-shape menu, anchored 2px under the QAT history button, newest
  first, "N. label", arrow-u-up-left icon, 11.5px rows); choosing a row
  rewinds via UndoStack::undo_to (the original's while-length>target loop).
  The QAT's six buttons are live: Undo/Redo follow the stacks and dim when
  empty, the history button opens the popup and reports "Nothing to undo
  yet." on an empty stack. 3 unit + 1 e2e.
- 2026-10-23: **Screen 6c verified (undo / redo)** — new undo.rs transcribes
  snapshotModel/pushUndo/undo/redo with labelled snapshots (lines, polygons,
  cursor, live chain), redo cleared on push, and the undone action's label
  carried into redo; Drawing::snapshot/restore keep restoreDrawingChain's
  rule (the chain is only restored while still drawing). Undo/redo clear the
  selection; wall commits, bowing, Set Length and appearance changes bank a
  step. Home/Edit Tools Undo/Redo enable from the stacks; Ctrl+Z,
  Ctrl+Shift+Z, Ctrl+Y bound; empty-stack history shows the original's
  "Nothing to undo yet.". 4 unit + 2 e2e tests. Next: the history popup,
  clipboard, transforms, marquee/fence/lasso, context menu, select-by-type.
- 2026-10-23: **Screen 6b-2 verified (the Edit Tools panel acts)** —
  set_selected_line_length moves the active handle along the wall's direction
  (live-original parity: start -> (-21,3), end -> (9,3) for a 10 ft wall set
  to 20, invalid values rejected) and follows area vertices; appearance
  applies lineStyle/color to every selected wall; the panel shows live values
  (length 2dp, "Length · Start|End handle", style, colour) with a real length
  field (Enter applies), Set Length, and Curve -> curve tool. Line Style
  cycles and Line Color opens the Color dialog (ColorTarget) as documented
  stand-ins for native select/color popups. select-wall holds at ribbon
  3.35%, canvas 0.08%.
- 2026-10-23: **Screen 6b verified (contextual Edit Tools ribbon)** — the Edit
  Tools tab (#F8FAFC, x 413.92, w 83.81) appears with a selection and takes
  over the ribbon; its body (8 base buttons + divider + Length/Set
  Length/Line Style/Line Color/Curve, all measured from the live DOM) is
  laid out. Reference select-wall: ribbon 7.68% -> 3.35%, canvas 0.08%,
  overall 1.06%. The fields are visual only for now (6b-2 wires them), and
  the fetched `intersect` glyph (Phosphor, MIT) was added to the icon table.
- 2026-10-23: **Screen 6a verified (selection model, hit testing, click select)**
  — new selection.rs with Selection, hit_test_at (4px on-wall pass, 9px wall
  pass, then polygon interiors), select_at (additive toggle + active
  endpoint), select_all, distance_to_wall (chord or flattened arc). Live
  original parity on the hit cases, the selected id and the active endpoint;
  tolerances 0.2/0.45 ft pinned in tests. Selected walls draw #C05AFA at
  max(5, base+1); selected areas fill + outline. Ctrl+A and Escape wired.
  Reference select-wall: canvas 0.20% (its ribbon reads 7.68% because the
  original's Edit Tools panel, S6-030..034, is not built yet). Next: 6b the
  Edit Tools panel, then undo/clipboard/transforms.
- 2026-10-23: **Screen 5e-3 verified (areas from healed loops) — S5-048 COMPLETE**
  — joins.rs closes the loop: heal, find_enclosed_faces over the exterior
  walls, materialise faces (bow signed by direction), skip non-whole/claimed/
  intact-covered faces, drop composites and larger rings that reuse a smaller
  boundary, repair a repairable polygon with equal rings or push a new area
  (UND, or the Define First type/name). Live-original parity: four separate
  walls -> created 1, UND "Undefined Area", area 100, edges [1,2,3,4]; a
  second run creates 0. Not ported (inert): splitWallsAtJunctions (off by
  default) and dedicated layers (later screen). Screen 5 is now complete.
- 2026-10-23: **Screen 5e-2 verified (wall joins: the heal)** — joins.rs now
  assembles heal_exterior_wall_joins (welds to endpoints and wall bodies,
  crossing splits, dangling-piece pruning that respects intact-area vertices,
  closed-loop search, endpoint commit + area-vertex follow) with the
  getScreenToleranceWorld(16) = 16/(20*zoom) tolerance, wired into the click
  path after a wall commits. Parity on the live original: a 0.1 ft gap welds
  to exactly (0,0), a 3 ft gap is untouched. STILL OPEN: the
  closeJoinedExteriorLoops polygon materialisation (new areas from healed
  loops via find_enclosed_faces, repairable polygons, splitWallsAtJunctions).
- 2026-10-23: **Screen 5e-1 (wall-join primitives) — FOUNDATION** — new
  `sketch-core/src/joins.rs` with segment_cross_point (corner and
  extrapolated crossings rejected), point_key, line_passes_through /
  exit_key_from, walk_closed_loop and MIN_CLOSED_AREA, each verified against
  the live original's own functions. 10 unit tests. S5-048 itself (the heal
  assembly: welds, crossing splits, dangling-piece pruning, endpoint commits,
  and closeJoinedExteriorLoops' polygon materialisation) is still OPEN — the
  weld step is intentionally not shipped alone because the original only
  commits a weld that completes a closed loop.
- 2026-10-23: **Screen 5d-8 verified (Define First)** — options.defineFirst blocks
  exterior drawing until an area type is chosen (pen stays up, the original's
  exact notice), and a closed loop then takes the chosen type/name instead of
  UND. Message::DefineFirst / Message::AreaType set them; the dock's area
  panel (screen 7) will drive them. Confirmed against the live original by
  dispatching the same click. Screen 5 left: wall joins (S5-048) — a ~200-line
  graph algorithm (welds, crossing splits, dangling-piece pruning, loop walk),
  scoped as its own step.
- 2026-10-23: **Screen 5d-7 verified (curved wall solids)** — closes the S5-054
  gap: `render::curve_metrics` (getCurveMetrics in world space) and
  `draw_curved_solid` (the annular sector between radius ± t/2, swept through
  the apex, max(8, ceil(|sweep|/(π/24))) steps). Before this a bowed wall with
  solids on drew NOTHING (the stroke was skipped and no fill replaced it);
  the live original's getWallSolids returns an empty straight list for a bowed
  wall, so the sector is the only path. Reference draw-bowed-solid: canvas
  0.01%, overall 0.91%. curve_tests pin the circle. Screen 5 still open: wall
  joins (S5-048), Define First (S5-056).
- 2026-10-23: **Screen 5d-6 verified (Shift+wheel bowing)** — `Drawing::bow_last_wall`
  ports CanvasEngine's wheel branch: degree mode (+-15, Ctrl 1) with the arc
  height derived, height mode (+-1 ft, Ctrl 0.1) with the angle derived, both
  clamped at 359. The test numbers are the live original's own responses to
  dispatched shift+wheel events. iced's wheel carries no modifiers, so the
  canvas widget tracks ModifiersChanged and publishes shift/ctrl with
  Message::Wheel. Reference draw-bowed-wall: canvas 0.05%, overall 0.93%.
  Still open in screen 5: curved-wall solids, wall joins (S5-048), Define
  First (S5-056).
- 2026-10-23: **Screen 5d-5 verified (wall solids)** — `sketch-core/src/solids.rs`
  ports computeWallSolids with Free/T/Inline/Corner junction classification,
  hosted-opening gaps and ±half side faces; the piece lists are asserted
  verbatim against the live original's getWallSolids (including the original's
  asymmetric end-extension clamp). `options.rs` adds wallSolids + the 6in/4in
  thicknesses and `Drawing::wall_thickness_of`; the renderer fills the quads
  (interior then exterior) and skips the stroke for solid lines while a
  live-chain line keeps its preview. Reference draw-wall-solids: canvas 0.02%,
  overall 0.91%. NOT DONE: bowed-wall annular sectors, offsetRing (screen 7).
  Next: Shift+wheel bow (S5-040), wall joins (S5-048), Define First (S5-056).
- 2026-10-23: **Screen 5d-4 verified (units / precision formatting)** — new
  `sketch-app/src/format.rs` transcribing formatLength / formatFeetInches /
  formatLengthDual / formatArea over the Options, with 11 tests whose
  expectations were read off the live original (ties, feet-inches carry,
  stripping, metric, grouping). Dimension and rise/run labels go through it;
  `Message::Units` sets the options for screen 12. The HUD pill stays raw, as
  the original's updateHUD does. Next: wall solids (S5-054), Shift+wheel bow
  (S5-040), wall joins (S5-048), Define First (S5-056), then the area UI.
- 2026-10-23: **Screen 5d-3 verified (Draw Color + the Color dialog)** — new
  `sketch-app/src/color_dialog.rs`: the Standard tab's hexagon hue/sat wheel
  (200x200 pixels generated in Rust, radius 98, v=1) with a live cursor and 48
  presets, the Custom tab's SV square + 24x200 hue strip + RGB inputs, the
  original's exact hsvToRgb/rgbToHsv/rgbToHex math, and OK/Cancel/×/backdrop
  semantics with the toast "Drawing color updated.". Wired through
  `Message::Color` / `Message::OpenColorDialog` and the ribbon's `draw-color`
  command; `Drawing::draw_color` is the `lineStyle.color || state.drawColor`
  fallback the original uses (the Options colours keep their defaults).
  References color-dialog-standard/custom/preset: canvas 0.10/0.19/0.10%,
  overall 0.43/0.49/0.43% (gate 6%). 12 unit + 5 e2e tests. iced lessons:
  borders draw inside the box, `image()` letterboxes unless
  `content_fit(Fill)`, and `text_input`'s intrinsic height excludes its
  border. Next: wall solids, units formatting, Shift+wheel bow, wall joins,
  Define First (the rest of screen 5), then the area UI (screen 7).
- 2026-10-22: **Screen 5b-1 verified (drawing walls with the mouse)** — new
  `sketch-core/src/model.rs` (the persisted wall shape, the original's JSON
  field names), `sketch-app/src/drawing.rs` (pen down/up, snapPoint /
  structuralSnap / getDrawingTarget, commits, auto-close, the HUD strings,
  toggleLastDimension), `render.rs` (walls, rotated dimensions, preview line
  with length / rise-run / live angle / return guide, alignment lines,
  vertices, snap cross), canvas click / double-click / right-press wiring,
  Escape + mode-change pen handling, the live status readout pill and the
  original's sticky 10px status scrollbar. Evidence in ACCEPTANCE.md.
- 2026-10-22: **Screen 5b-2 verified (auto-close into an area)** — the
  persisted `Polygon` model + `build_edges`/`vertex_keys` in
  `sketch-core/src/model.rs` (string layer ids), `auto_close` creates the UND
  area from an exterior loop with the chain's edges and the computed area,
  closed-area vertices stop attracting new walls, a `draw-closed-area`
  reference + e2e state + test. Deferred: the dedicated "Area" layer (8/9)
  and the Calcs area rows (7).
- 2026-10-22: **Screen 5c verified (typed wall entry + the arrow-key cursor)**
  — `drawing.rs` gained the typed buffer (digits/./x/l/r/</, with the
  original's rules), parseBuffer, Enter (pen down / commit / pen up), arrow
  moves with options.stepUnit + diagonals + Ctrl alignment snaps, J, A,
  cleanCoord and a faithful parseFloat; `input.rs` maps them in
  InputController's precedence order; `render.rs` draws the dashed keyboard
  preview and the drawing pointer; the app carries options.stepUnit. Evidence
  in ACCEPTANCE.md.
- 2026-10-22: **Screen 5d-1 verified (cycle dimensions, placement, visibility)**
  — `View › Apply (Dims)` cycles outside/inside/hidden with the original's
  toasts (view_state::cycle_dimensions + DimensionPlacement, wired through
  StatusMsg::CycleDimensions and the ribbon's Apply button), the placement
  flips the label side in render.rs, and dimension visibility follows
  isDimensionVisible. Two states + a test, all regions inside the gate
  (canvas 0.06-0.08%, ribbon 2.35%, overall 0.94-0.95%). Next: 5d-2 (wall shapes, Draw
  Color, wall solids, units formatting) and the area UI (screen 7).
- 2026-10-22: **Screen 5a verified (modes + navigation)** — Mode enum for all
  rail tools (`view_state.rs`), `rail.rs` keyed by tool id with mode-driven
  highlight, canvas pan (Pan tool / middle / Space+left) + wheel zoom +
  cursor policy + painted drawing pointer (`sketch_canvas.rs`, cursors in
  assets/cursors), `input.rs` key map (E/I/F5/Enter/Escape/G/D/P/S/T/F4/C,
  Space down/up), EventTap `on_key_up`, toast height fix, `Harness::send`
  for unscriptable input. Evidence in ACCEPTANCE.md. Next: 5b mouse drawing.
- 2026-10-22: **Screen 4 ACCEPTED (4b: north, pages, underlay, dialogs)** —
  `pages.rs` (addPage / deletePage / goToPage / setUnderlay / per-page
  north, parseFloat), `dialog.rs` (alert / confirm / prompt modal, focus +
  select-all, Escape / Enter / backdrop), status-bar right block as a
  natural-width flex row, View-tab page commands, disabled areas select,
  `snap::nudge` (iced floor vs Chrome round for text rows). Harness: per-step
  15ms drain for late widget-operation tasks; `SKETCH_E2E_ONLY` filter.
  Evidence in ACCEPTANCE.md. Next: Screen 5 (canvas drawing).
- 2026-10-22: **Screen 4a verified (status-bar view controls)** — new
  `view_state.rs` (pan/zoom/1:N/grid/snap/pitch/mode/fence/filters, ported
  formula-for-formula from SketchApp), live `statusbar.rs` (natural-width
  centre block between equal spacers = the original's re-centring; invisible
  iced slider / pick_list over the pixel-tuned drawings), `filter.rs`
  (popover + Chrome dark checkbox), zoom fence in `sketch_canvas.rs`
  (capture-style drag, dashed fill/stroke), rail Select highlight driven by
  mode, ribbon View › Grid / Filter commands, EventTap `on_key` (shortcuts
  only for keys no widget captured), `snap.rs` (pixel-snapped draw for the
  fractional centre row). Evidence in ACCEPTANCE.md. Next: Screen 4b.
- 2026-10-22: **Screen 3 ACCEPTED (dock tabs, pin/collapse, resize)** — built
  and verified by the parent (delegations kept dying at kickoff). Evidence in
  ACCEPTANCE.md. New: `symlib.rs` (external `symbols.json` index, loaded once
  per process via `sketch_core::symbols::resolve`; preview = ported
  renderSymbolPreview/renderSymbolGeometry/traceSymbolPolyline, light canvas
  palette #334155/#FFFFFF), `dock.rs` rewritten (`DockState`/`DockMsg`; Areas,
  Text, Symbols, Calcs, Layers panels; measured tab edges per active tab +
  flex growth; 30px collapsed rail; 7px resize handle overlay; clamp
  320-400 / canvas >= 220), `toast.rs` (`.app-toast`, 3.2s, generation-guarded
  timer), `EventTap` gained `on_move`/`on_release` (pointer capture for the
  drag). 14 Phosphor icons added (core 2.1.1). Fixed a regression found by
  the pixel check: pin/collapse buttons had moved 1px right of Screen 1.
  Feature inventory Screen 3: S3-001..007, 009..011, 014..019 built; S3-012 and
  S3-020 are hidden legacy pieces (state kept in `DockState.pinned`, nothing
  visible); S3-008 waits for placement (Screens 8-9); S3-013 for File save
  (Screen 11).
- 2026-10-22: **Screen 2 ACCEPTED** (commit 0605c78; evidence in ACCEPTANCE.md).
- 2026-10-22: **Screen 2 candidate (ribbon tab bodies + File backstage) —
  built by a subagent, NOT yet accepted** (the parent verifies; the
  ACCEPTANCE.md box stays unchecked).
  - `ribbon.rs` reworked into a data model (`Btn`/`Glyph`/`Caption`/`Item`,
    looks Ribbon / Plain / Toggled, disabled = CSS opacity .38) with tab state
    (`TabId` incl. File, `Body`); Home kept item-for-item from Screen 1.
    `ribbon_tabs.rs` (View / Tools / Reference Image / Help) is GENERATED by
    `tools/gen_ribbon.py` from `refs/ui-spec/chrome-spec-states.json` (re-run
    it, never hand-edit). Disabled per the spec: Home Undo/Redo, View
    Previous, Tools Disconnected. View Grid = toggled style (rgba(0,122,204,.2)
    + border; its painted box is 50px, snapped like Chrome). Reference Image
    intensity slider = Chrome's default range look sampled from the gallery.
  - `backstage.rs`: `#file-menu` (0,54 200 wide, to the window bottom) with all
    15 items, 4 dividers, badge icons (Save As pencil, Print Preview eye,
    Recent caret), footer Options/Exit `.file-btn`s; hovers per theme.css.
    Behavior per ui-manager.js: File toggles it and becomes `.active` (bg-panel
    + accent stripe; `.file-tab:hover` #006bb3 still wins) while the previous
    body stays; another tab closes it and switches; a pointer press outside it
    (not on the File tab) or Escape closes it (File stays active); the press
    still reaches its target. Items visual-only except Exit (= Close).
  - 7 Phosphor SVGs added from `@phosphor-icons/core@2.1.1` (regular layout;
    fill bluetooth, eye-slash, arrow-counter-clockwise, arrow-clockwise,
    envelope-simple, printer).
  - **Measured** (`SKETCH_DPR=1`, `tools/screen2-loop.sh` = real-input tab
    clicks via new `tools/click-real.ps1` + `tools/shot.ps1` + `compare.py`):
    View: ribbon 2.09% / overall 0.87% · Tools 1.22% / 0.77% · Reference
    Image 2.75% / 0.94% · Help 0.71% / 0.71% · File backstage (vs
    file-backstage.png): title 2.14 · ribbon 2.08 · rail 2.31 · canvas 0.26 ·
    dock 1.74 · status 4.52 · overall 0.99% · Home unchanged from Screen 1
    (2.14/2.22/3.97/0.00/1.74/4.32, overall 0.88%). Every other region equals
    the Screen 1 baseline. Because a wrong body scores only ~4-7% on the
    ribbon region, also checked per element: `tools/ink_check.py` (ink
    centroid/mass per spec icon/span) — every element within ~1px, mass
    0.85-1.16x (same spread as accepted Home); `tools/zoom_compare.py`
    banded zooms show AA edges only.
  - **Behavior driven with real input** (`tools/screen2-interact.sh`, pixel
    probes): File opens; press inside menu keeps it open; canvas press
    closes (File stays active); Escape closes; File twice closes; open + View
    tab closes it AND shows View; ribbon-btn hover = #3e3e42 + #3f3f46 border;
    File-tab hover with menu open = #006bb3; Exit ends the process.
  - Fix found by that drive: switching the root widget (column ↔ stack)
    when the menu closed on press reset the tab button's state, so the
    release never clicked the tab — the root is now always a 2-layer stack.
  - Not done / deliberate: Recent Files submenu, Edit Tools contextual tab
    (screen 6), ribbon/menu actions (later screens). Gates: fmt clean,
    clippy 0 warnings, `cargo test --workspace` 109 passed (102 core + 7 app).

- 2026-10-22: **Restart / Screen 1 candidate (static default-state chrome) —
  built by a subagent, NOT yet accepted** (ACCEPTANCE.md requires the
  parent's own side-by-side check). `crates/sketch-app/src/**` replaced
  from scratch (attempt-1 code deleted, not carried over): `main.rs`
  (borderless 1400x860 window, DPR handling, window tasks), `theme.rs`
  (CSS palette, fonts, `at()` pin helper, glyph ink passes), `icons.rs`
  (embedded Phosphor SVGs + verbatim QAT logo), `titlebar.rs`, `ribbon.rs`,
  `rail.rs`, `dock.rs`, `statusbar.rs` — every element placed from
  `refs/ui-spec/chrome-spec-home.json`, colors from the spec/`theme.css`
  and pixel sampling of `ribbon-Home.png`. Deps trimmed to iced +
  sketch-core (rfd/image/btleplug/futures return with their screens).
  Added 8 missing Phosphor SVGs (list-bullets, minus, plus, caret-left,
  x-bold, fill: push-pin, arrow-up, pencil-simple) from
  `@phosphor-icons/core@2.1.1`.
  - **Measured** (`SKETCH_DPR=1`, `tools/shot.ps1` → `tools/compare.py` vs
    `ribbon-Home.png`; identical vs `dock-Areas.png`): title bar 2.14% ·
    ribbon 2.22% · left rail 3.97% · canvas 0.00% · dock 1.74% · status bar
    4.32% · overall 0.88% (attempt 1: 90.11%). ~75% of the remaining
    differing pixels are ClearType colour fringes in the reference (Chrome
    renders text AND icon-font glyphs with subpixel AA; iced only does
    grayscale); a strict low-tolerance flat-area diff finds no structural
    differences.
  - **Window controls verified with real mouse input** (target/ctl.ps1):
    Minimize → iconic; Toggle Full Screen → 3456x2160 and back to
    1400x860; title-bar drag moved the window 120,60 px; Close → process
    exits.
  - **Display scaling**: by default the app follows the system DPI like the
    browser does (this host is 250% → the 1400x860 CSS-px window doesn't
    fit, so it opens maximized to the work area; verified crisp, no edge
    clipping). `SKETCH_DPR=1` forces 1 CSS px = 1 physical px — REQUIRED
    for `tools/shot.ps1`/`compare.py`, whose reference is DPR 1. If another
    app holds the foreground lock, shot.ps1's SetForegroundWindow fails and
    it captures whatever covers the window (canvas ≠ 0% is the tell) — pin
    the window topmost first.
  - Not matched / deliberate: Pages navigator button renders empty exactly
    like the original (its `ph-files` glyph is missing from the original's
    icon-font subset); only window controls are live — all other controls
    are visual with theme.css hover states. Gates: fmt clean, clippy 0
    warnings, `cargo test --workspace` 104 passed (102 core + 2 app).
- 2026-10-22: **P10 — Disto X-series connect (the last app feature). BLE
  transport + measurement feed, with the mandatory honest limit: the BLE
  transport and notification parsing are UNVERIFIED against real DISTO
  hardware — no device was available in the build environment.**
  - **Browser-fidelity finding (verified in the standalone HTML before
    porting)**: the browser's `connectDisto` (refs/sketch-app.js ~5449) is
    CONNECTION-ONLY — `navigator.bluetooth.requestDevice({
    acceptAllDevices: true })`, the `#disto-status` chip flip (name +
    `.active`), and `Connected to <name>.`; user cancel (NotFoundError) is
    silently swallowed, other failures notify
    `Bluetooth connection was not completed.` There is NO GATT
    subscription anywhere in the HTML and the `distoDiscovery` /
    `distoComPort` / `distoRounding` / `distoDrawingMode` options
    (defaults in refs/state-init.js, Options dialog "Hardware" panel in
    refs/ui-manager.js) are never consumed by any measurement path. The
    port therefore ships the connection flow as the browser does it and
    implements the measurement feed as a documented COMPLETION of those
    unwired settings — not transcribed browser behavior.
  - **`sketch-core/src/disto.rs` (pure, headless)**: `DistoRounding` /
    `DistoDrawingMode` / `DistoDiscovery` enums parsing the browser's exact
    option strings with tolerant fallback to the browser defaults;
    `round_distance` (nearest-multiple, half away from zero; none =
    identity); `bearing_from_cursor` / `endpoint_from_bearing_distance`
    (world feet, y-down, [0,360)); `apply_measurement` (round → bearing →
    endpoint); `DistoSettings` with camelCase serde carrying the browser
    strings byte-compatibly.
  - **`sketch-app/src/disto.rs` (transport + state machine)**:
    `DistoManager` — Disconnected → Discovering → Connecting → Connected →
    Measuring with the browser's error paths (silent user-cancel vs
    notified failure carrying "Bluetooth connection was not completed" +
    transport detail onto the status line); devices sorted DISTO-named
    first over `acceptAllDevices` semantics. Real transport: a long-lived
    iced subscription worker (`Subscription::run` + `stream::channel`) over
    btleplug 0.11 winrtble (`platform::{Manager, Adapter, Peripheral}`) —
    3 s discovery scan, connect, discover-services, subscribe to the first
    NOTIFY characteristic, notifications pumped into the state machine,
    disconnect via command; every transport failure lands in
    `DistoConnectFailed` so the status line shows it. Notification parser
    is defensive and UNVERIFIED: ASCII decimal meters, then f32 LE meters
    at offset 0/3, then u16 LE millimeters (text-looking payloads never
    decode as binary); undecodable payloads are simply not fed. Mock
    transport (scripted `DistoEvent`s) drives all state-machine tests.
  - **Measurement integration** (`state.rs`): `disto_apply_measurement` —
    standard mode commits the leg immediately (draw: `draw_line` from the
    pen cursor; measure: the run's anchor, auto-anchoring the run at the
    cursor on the first measurement); continuous mode stages
    `disto_preview` (canvas crosshair + dashed leg from the anchor) that
    the next canvas/measure click commits. Bearing comes from the run
    anchor toward the live pointer world position. No run → refused with a
    guidance notice.
  - **UI**: Tools tab Connect button runs the flow and the status chip
    mirrors `#disto-status` (label = device name, accent when connected;
    a second click disconnects); the status bar's Bluetooth entry wired;
    device picker overlay (the requestDevice chooser equivalent) with
    Cancel; Options popup on the Tools tab reproducing the browser
    Options dialog's Hardware panel (Discovery Automatic/Manual radios,
    COM Port field, Rounding + Drawing Mode pick lists) with an on-panel
    UNVERIFIED note.
  - **SKX persistence**: `settings.options.distoDiscovery` /
    `distoComPort` / `distoRounding` / `distoDrawingMode` emitted and
    restored (browser keys, round-trip test; legacy files keep defaults).
  - Evidence: `cargo fmt --all --check` clean; `cargo test --workspace`
    219 passed / 0 failed (24 new: 7 core disto tests — parsing
    round-trips, defaults vs refs/state-init.js, all five rounding rules,
    bearing/endpoint round-trips, camelCase serde; 7 app state-machine
    tests — happy path + chip text, measuring phase, silent cancel vs
    notified failure, disconnect reset, DISTO-first sort, parser
    ASCII/binary/garbage, settings+options feed contract; 6 state feed
    tests — draw commit + rounding + continuous stage/commit-on-click +
    measure anchor/extend + refusal guidance + settings parse; 1 SKX
    round-trip; 3 pre-existing reshuffles); `cargo clippy --workspace
    --all-targets` 0 warnings; `cargo build -p sketch-app` links
    `target/debug/sketch-app.exe`.
  - Honest limits for the parent: the BLE transport (scan/connect/notify)
    and the notification payload parser are UNVERIFIED against real
    hardware — the measurement-characteristic choice ("first NOTIFY
    characteristic") and the payload encodings are best-effort and a real
    DISTO may need the Leica measurement service/characteristic and frame
    format; failures surface on the status line rather than silently
    breaking. The measurement feed itself (rounding/bearing/drawing-mode)
    is the port's documented completion of the browser's unwired options —
    the browser never implemented it. Manual/COM-port discovery is stored
    but has no transport (the browser's field is equally inert); the
    parser treats decoded values as meters (SI) — feet conversion is ours.

- 2026-10-21: **P9 — fidelity polish: real Phosphor icons, command popups,
  dock resize drag, hover states, window-title fix.** `sketch-app` only;
  `sketch-core` untouched.
  - **Ribbon/rail/status icons**: `assets/icons/{regular,fill}/*.svg` — the
    MIT-licensed `@phosphor-icons/core@2.1.1` SVGs for exactly the
    `ph`/`ph-fill` set ui-manager.js + the tool rail markup use (85 files,
    incl. `plus-circle`/`minus-circle` and the fill-weight menu carets) —
    embedded at compile time via `include_str!` in `icons.rs` (no network at
    runtime). `icons::icon(name, fill, color)` extracts the `<path>` data,
    rebuilds `viewBox="0 0 256 256"` SVG bytes with the browser's inline
    icon colors baked in (`#63B3ED/#F6AD55/#FC8181/#5DAE84/#A0AEC0/#F6E05E`),
    and caches a `svg::Handle` per (icon, color) so per-frame rendering only
    clones handles. Every ribbon button, tool-rail button, status button and
    popup item now renders its real Phosphor glyph (iced `svg` widget, the
    `svg` feature added to the app crate); the caret-▾ labels use
    the 8 px filled caret.
  - **Command popups** (`showCommandPopup` → native overlays):
    `PopupKind::{Select,Flip,WallShapeExterior,WallShapeInterior,Geo,
    CanvasBackground,InterfaceTheme}` rendered through `iced::widget::stack`
    — a full-window transparent backdrop (outside click dismisses, Escape
    closes first) plus a `.command-popup`-styled panel (bg-panel, border,
    190 px, 7×10 items, hover bg-hover, ` ✓` on the active theme item)
    anchored at the button's layout position. Items: Select▾ 6 entries
    (→ `selectByType`), Flip 2 (→ `flipSketch`), Wall Shape 4 (Line →
    `setMode`; Rectangle/Circle/Polygon → the `promptDialog` Width → Height
    → sides chain as a bottom prompt bar → `addWallShape`), Geo 3 (map
    image / Auto Locate by Coordinate / by Address with the browser's
    prefill, GPS validation message and notify texts via
    `showGeoReferenceDialog`), plus Background and Interface theme menus.
    The rail draw tools carry the browser's contextmenu: right-click opens
    the Wall Shape menu (`mouse_area.on_right_press`; left-press still sets
    the mode).
  - **Dock resize drag**: the `SIDEBAR RESIZE LAYER` port — a 7 px strip on
    the dock's left edge (`#sidebar-resize-handle`): col-resize cursor
    (`mouse_area.interaction(ResizingHorizontally)`), accent tint at 0.35
    while hovered/dragging, live drag via the strip's own
    press/move/release (the strip follows the pointer as the width tracks
    it, so no global mouse subscription is needed), `MIN 320 / MAX 400 /
    CANVAS_MIN 220` clamp with `avail = max(MIN, windowW - 220)`, and
    double-click reset to 312 (`sidebar_width` feeds the dock width;
    collapsed docks hide the strip).
  - **Sketch transforms + typed selection + wall shapes** (state): ports of
    `flipSketch` (bow negation incl. `arcHeight`, `transformSketch` over
    lines/polygons/symbols, bearing `360-a` / `180-a` with north
    normalization), `rotateSketch` (bbox-center rotation, north + d),
    `selectByType` (areas/text/symbols/freeform/dimensions — honest port
    mapping: text → defined areas, dimensions → chains, the port has no
    free-label or per-wall dimension ids), and `addWallShape`
    (rectangle/circle/polygon centered on the view, circle's min 24 sides,
    polygon 3..24, exterior closes into an area with Define-First stamping,
    dedicated-layer assignment and recalc).
  - **Small chrome**: hover states per theme.css on ribbon buttons
    (bg-hover + border), tabs, tool rail, sidebar tabs, action buttons
    (`#4e4e52`), library rows, area buttons, status buttons; `.tab.active`
    gains the 2 px accent top bar; the status-right totals chip uses the
    browser's `·` separators (GLA · N areas · Total · zoom) with the accent
    GLA; window title now `Appraisal Sketch - {file}` (the browser's
    `updateTitle` pattern — the stray "1.1" is gone); Rotate Sketch and
    Apply (cycle-dimensions) buttons wired.
  - Evidence: `cargo fmt --all --check` clean; `cargo test --workspace`
    195 passed / 0 failed (10 new: flip mirrors points/bows/north + undo,
    rotate keeps area + north + undo, select-by-type families,
    add-wall-shape rect/circle/hex + interior-no-area + undo, icons module
    loads + path extraction + svg document color baking); `cargo clippy
    --workspace --all-targets` 0 warnings; `cargo build -p sketch-app`
    links `target/debug/sketch-app.exe`.
  - Known gaps for the parent: dimension/chain text rotation remains
    impossible on iced 0.14 — the canvas `Text` primitive carries only
    position/color/size/align (verified in the iced_core 0.14 source, no
    rotation/transform field, and `Frame` applies transforms to geometry
    only), so labels stay horizontal with the browser's
    normal-offset/leader placement (the P7 fallback, now confirmed as a
    platform gap rather than an omission). Popup anchors are computed from
    the ribbon's layout arithmetic (content padding 12, 74 px button
    stride, 25 px divider) rather than measured widget geometry — they sit
    at the right spot for the default window but drift if the anchor's tab
    layout changes. The dock drag ends when the pointer escapes the 7 px
    strip (no pointer capture in iced; the strip follows the pointer so
    escapes are rare). The View-tab Background/Interface popups both land
    on the port's single palette (the browser keeps canvas-background and
    interface themes separate; this port has one `ThemeId`). The File tab
    still renders its dropdown as a button group, and the command palette
    (Ctrl+K), undo-history popup, and canvas context menu are still
    unported.

- 2026-10-21: **P8 — underlay/reference image, scale calibration, keyboard
  pointer, region classification.**
  - **Underlay / reference image** (`app.bgImage`, refs extracted to
    `refs/p8-*.js`): `model::BgImage` carries the browser's exact field names
    (`visible/x/y/scale/rotation/opacity/flipH/flipV` + a port-added `path`);
    the interaction fields (`scaleMode/scaleP1/scaleP2/editMode/orientMode/
    orientP1/P2`) are session-only (`#[serde(skip)]`). Methods ported
    method-for-method on `SketchState`: `loadBgImage` (auto-scale
    `60*PPF / img.width`, opacity 0.5), `showBgImage/hideBgImage/
    removeBgImage/rotateBgImage ((r+d) % 360)/flipBgImageH/V/scaleBgImageBy/
    setBgIntensity` plus the Edit-Mode drag (`dragState 'photo'` →
    `Drag::BgImageMove`, center follows the world delta) and the wheel resize
    (1.05 / 0.95, checked before every other wheel branch). Rendering:
    `canvas.rs draw_bg_image` paints the decoded image under the grid
    (`Frame::draw_image` with center bounds + `Radians` rotation + opacity);
    flips are baked into the RGBA (`apply_flips`, the port of
    `ctx.scale(sx, sy)` — iced images have no flip) into a rebuilt
    `Handle::from_rgba`. Loading: rfd file pick + `image`-crate decode once
    at open (`main.rs load_bg_image_dialog`); `reload_bg_image_from_path`
    re-loads on file open with the persisted placement restored.
  - **Scale points**: `startSetScale` (Select mode forced + the browser's
    alert text as a notice), `handleScaleClick` two-point flow with
    `drawScalePoints` markers (amber crosshair + circle + Point 1/2 labels +
    dashed connector), and the `promptDialog` continuation as a bottom bar:
    `scaleFactor = knownFeet / worldDist` applied to the image scale; a
    blank/invalid answer cancels without touching the scale. Orient Guide
    (`startPhotoOrient/handleOrientClick/apply_orientation`: `v…` →
    `rotation += 90 - angle`, else `rotation -= angle`) with its
    Horizontal/Vertical prompt bar; Escape cancels both flows.
  - **Keyboard pointer mode**: `state.keyboard_pointer` +
    `keyboard_pointer_anchor`; `enterKeyboardPointerMode` parks the pointer
    at the keyboard cursor while arrows drive it (RiseRun arrow branch
    included), `updateKeyboardHUD` (raw atan2 degrees, not the apex
    conversion), `CanvasEngine.pointermove`'s exit (> 3 px from the anchor
    restores the pointer), and `drawKeyboardPointer` paints a marker at the
    cursor scaled by `options.pointerSize` (32/64/128 else 128; the option
    now round-trips with `imageSize`/`imageDepth` in the save model).
  - **Region classification (optional, honestly marked)**: the browser's
    optional workflow script (`refs/p8-optional-region-classification.js`)
    is the region-classification layer — its detection
    (`detectRegions`, `findEnclosedFaces`) already landed in P5; this round
    adds `resolveRegionAt` (read-only: smallest visible polygon under the
    point beats the tightest exterior-wall face unless the polygon is
    smaller-then-face/locked; same-ring polygons win; face cache keyed on
    the exterior walls' geometry like `app.regionFaceCache`), the
    define-flow hover preview (`track_region_hover` — with a code armed in
    Select mode, the canvas shades the region the next click defines with
    its measured area at the label anchor), and the workflow footer's
    `Find undefined` toggle (`Find UND ✓` in the Define panel, dashed
    indigo rings over UND areas on canvas). The browser marks the whole
    workflow "Optional region classification. No drawing completion event
    opens this UI." — the port keeps that status: nothing auto-opens it.
  - **SKX persistence**: `settings.bgImage` carries `path` + placement
    (`visible/x/y/scale/rotation/opacity/flipH/flipV` + decoded
    width/height); the browser never saved bgImage, the unknown key is
    ignored by the original app on open. Session interaction state is
    never persisted. Round-trip tests pin the JSON both directions plus a
    browser-shaped file without the key.
  - Evidence: `cargo fmt --all --check` clean; `cargo test --workspace`
    189 passed / 0 failed (16 new: 12 state tests — underlay defaults,
    transforms, flip math, scale calibration, orient guide, edit-mode
    move, keyboard pointer enter/exit/HUD, resolveRegionAt face/poly/
    cache-invalidation/hover-gating — 1 core `along_line_t` formula test,
    3 SKX round-trips incl. the no-underlay legacy file);
    `cargo clippy --workspace --all-targets` 0 warnings;
    `cargo build -p sketch-app` links `target/debug/sketch-app.exe`.
  - Known gaps for the parent: the keyboard pointer draws an arrow marker
    instead of the browser's `cursor-<size>.png` assets (no cursor assets
    in the repo; `pointerSize` is honored); remove-image uses a native
    Yes/No box rather than an in-canvas confirm; the orient prompt is a
    Horizontal/Vertical button bar (the browser's prompt is a text field
    accepting 'v'-prefixed answers); the optional workflow's palette
    drag-and-drop assignment (`startTileDrag`) and its SVG overlay are
    replaced by the armed-code hover/click flow (no drag ghost); underlay
    export bounds include the image only via `saveImageWithSketch`
    semantics not yet wired (export paths ignore the underlay).
- 2026-10-21: **P7 — chain dimensions, pages, layers.** `sketch-app`
  only; `sketch-core` untouched (its existing
  `GeometryEngine::compute_dimension_chain` / `bearing_degrees` /
  `pinned_dimension_point` are consumed as-is; `split_walls_at_junctions`
  added to the app `Options` and threaded into the chain call).
  - **Chain dimensions**: `Mode::ChainDim` + `handle_chain_click`
    (`handleChainClick` verbatim — nearest straight wall within 8 px,
    cross-product side, undo point, `DimensionChainRec` with the browser's
    exact camelCase fields `seedLineId/side/offset/showOverall/hiddenSpans`;
    offset = `options.dimensionOption` 8.5 ft const). Spans always recompute
    from the live walls (`computeChain` → core), so T-junction splits (with
    `options.splitWallsAtJunctions`, the options checkbox default off) and
    hosted-opening gap edges (SIDE hinge+gap rule) honor edits automatically.
    `cycleDimensions` (off → out → in → off) and `isDimensionVisible` ported;
    chain-covered walls suppress their per-wall labels (`getChainCoverage`).
    Canvas `drawChains`: witness lines, shared dimension line with
    architectural ticks, per-span labels, overall dimension on the outer
    tier, selected highlight; Select-mode `getChainHit` picks chains
    (8 px, both tiers) and a `Drag::ChainOffset` slides the line along the
    run's normal (side from the projection sign, offset ≥ 0.5, undo point).
    Delete removes selected chains; undo/redo restores records.
  - **Pages**: `pages: Vec<PageData>` + `current_page` with the browser's
    page model (lines/polygons/symbols/northAngle/dimensionChains per page;
    labels not modeled in this port). `get_page_data`/`load_page_data`/
    `save_current_page`/`go_to_page`/`add_page`/`delete_page` ported — page
    switch clears the undo stacks (`loadPageData`), the last page refuses to
    delete with the browser's message, north angles normalize to [0, 360).
    Ribbon File tab gains a Pages group (◀ n/N ▶ / Add / Delete).
  - **Layers**: full Photoshop-style stack in `state.rs` — `ensureLayers`,
    `flattenLayerOrder`, `defaultLayerId`, `getEntityLayerId` (missing layer
    → default), `layerAncestry` visibility/lock, `descendantLayerIds`,
    `addLayer`/`createLayerRaw` (string ids `layer-<genId>` exactly as the
    browser), `uniqueLayerName`, `assignAreaToDedicatedLayer` (idempotent
    reuse, re-close candidate reuse, empty-source-layer cleanup, walls move
    with the area; runs after auto-close, Ctrl+Click define, detect regions,
    clone, and on open), `ensureSymbolsLayer`, `removeEmptyAreaLayer`,
    `removeLayer`, `renameLayer`, `setLayerVisible/Locked` (undo points +
    selection clearing), `setActiveLayer`, `moveLayer` (before/after/inside,
    self-nesting guard), `moveSelectionToLayer`, `duplicateLayer`
    (entity copies with id remaps incl. edges/container/subtract ids, clone
    becomes active), `applyLayerOrder` (entity arrays stable-sorted into
    paint order so draw order matches the stack), `bringToFront`/
    `sendToBack`/`autoAssignLayer` (reused dedicated layer, top/bottom),
    `reorderSelected` (within-layer segments). Entity `layerId` fields are
    `Option<String>` now (the browser's layer ids are strings — required for
    round-trip); new walls/symbols land on the active layer; locked layers
    never pick (`isEntityLocked` in `hit_test_at`); hidden layers don't
    render. Dock Layers tab is a real tree (topmost first, group indent,
    activate/eye/lock per row, +/duplicate/delete, move-selection,
    to-front/to-back).
  - **SKX persistence**: `buildSaveModel`/`write_skx` now take the live
    state (`saveCurrentPage` semantics baked in) and emit typed `layers` +
    `activeLayerId` at the top level and `dimensionChains` + `northAngle`
    per page; `applySaveModel` normalizes every page (`ensurePageModel`:
    bow clamp + arc-height re-derivation, factor/subtractMode defaults) and
    returns `pages`/`layers`/`activeLayerId`/`dimensionsEnabled`/
    `dimensionPlacement`; next_id covers all pages' entities + chain ids.
    Round-trip tests pin the browser JSON (`pages[n].dimensionChains[0].
    seedLineId`, `layers[n].id/name/areaPolyId`, `pages[0].lines[0].layerId`
    as a string) both directions, including a browser-shaped file.
  - Evidence: `cargo fmt --all --check` clean; `cargo test --workspace`
    173 passed / 0 failed (18 new: 13 layer/chain/page state-machine tests
    incl. T-junction split + door-gap spans + run tracking + page
    separation + undo, 5 SKX round-trips incl. multi-page, browser-layer
    file, ensurePageModel arc re-derivation); `cargo clippy --workspace
    --all-targets` 0 warnings; `cargo build -p sketch-app` links
    `target/debug/sketch-app.exe`.
  - Known gaps for the parent: chain span labels stay horizontal — iced
    canvas text has no rotation (the browser rotates labels along the run
    axis); the layout, ticks and witness geometry follow the JS otherwise.
    Layer rename input, tree drag-reorder (`moveLayer` is ported + tested,
    no drag UI), per-layer symbol-library grouping, layer export/import
    files, and `wallGapWidth`/`wallSideFlip` library fallback in chains
    (placed symbols always carry their snap) are not built; the panel
    exposes activate/visible/lock/add/duplicate/delete/move/z-order.
    `pageLayout`/`underlayPageIndex` (ghost floors) and the page-numbered
    window title are later-phase; pages persist but exports/print still use
    the live page only.
- 2026-10-21: **P6 — exporters & CAD: DXF export + import, SVG export, PNG
  export, PDF export with the print layout, ribbon wiring.** Core modules
  headless in `sketch-core` (additive; no existing pub API touched), app
  glue in `sketch-app/src/export.rs` (new).
  - **Export model** (`export_model.rs`): neutral `ExportPage`/`ExportLine`/
    `ExportPolygon`/`ExportLabel`/`ExportSymbol`/`ExportSolid` — the
    headless stand-in for the browser's `buildExportModel()` page — with
    `curveApex`/`curveCircle` (rise = chord/2·tan(bow·π/720) three-point
    circle), `flattenRing(..., 0.02)`, `getExportBounds` (line endpoints +
    apexes + flattened rings + labels + symbols + label positions, empty
    default {0,10,0,10}), `buildAreaLineColors` (area Line-color override
    keyed by edge lineId), and `autoPostLines` (name + Math.round(net) +
    suffix).
  - **SVG export** (`export_svg.rs`): `refs/exporter.js` exportSVG
    transcribed — 1 unit = 1 screen pixel (20/foot), 40-unit margin, y-down
    transform, wall dash `10 6`/`2 5`, bowed walls as SVG arc paths
    (sweep-flag picked so the arc passes through the apex), straight walls
    broken through their opening runs, solid pieces filled first when
    geometric thickness is enabled, area labels/labels/symbols with
    escapeXml, symbol entities drawn from the external library geometry
    (y negated, bulge radius = chord(1+b²)/4|b|).
  - **DXF export** (`export_dxf.rs`): R12 ASCII — `dxfPair` two-space pair
    lines, HEADER $ACADVER AC1009 + $EXTMIN/$EXTMAX to the export bounds;
    straight walls LINE (or closed solid POLYLINE/VERTEX/SEQEND when wall
    solids enabled), bowed walls ARC (CCW 50→51 passing through the apex),
    area polygons closed POLYLINE on `AREA-<CODE>`/`AREA-UNDEFINED`
    (flattened), auto-post labels TEXT on AREA-TEXT (height
    max(0.5, fontPx/20)), free labels TEXT on TEXT, control chars stripped.
  - **DXF import** (`cad_import.rs`): `refs/cad-import.js` CadImport
    transcribed — pair stream (code/value with the JS `i += 2` pairing),
    sectionRanges, headerInsunits, the explicit vertex state machine
    (legacy POLYLINE VERTEX/SEQEND vs LWPOLYLINE, bulge group 42), extentOf,
    pickScale ($INSUNITS 1→1/12 in, 2→ft, 3→5280 mi, 4→mm, 5→cm, 6→m;
    unitless inference: mm when span>1000 & plausible, else inches/feet/m;
    default inches), bulgeToArcPoints (bulge=tan(sweep/4), CCW side by
    sign), circlePoints (32), arcPoints (CCW unwrap so 350°→10° sweeps
    20°), entitiesToSketch (LINE degenerate-skip, closed-flag polylines →
    polygons, CIRCLE → closed polygon, ARC → segment lines, TEXT/MTEXT
    label codes stripped per the JS regex, POINT → empty labels, INSERT →
    skipped count). App insertion: undo point, lines as exterior walls,
    closed polygons, labels counted + reported (no free-label layer yet),
    notice via the browser's `describe` shape. DWG rejected with the
    conversion hint.
  - **Print/PDF layout** (`print_layout.rs`): the standalone HTML's
    SketchReport constants transcribed and verified in its own source —
    PAGE_SIZES legal 612×1008 / letter 612×792 / a4 595.28×841.89 (css
    'legal/letter/A4 portrait'), SKETCH_WIDTH 540, SKETCH_PAD 20,
    SKETCH_HEIGHTS legal 608/letter 392/a4 442, LOWER_MARGIN 37;
    `get_bounds` fit variant from line endpoints + flattened polygon rings
    ONLY (labels/symbols excluded; solids included when present; degenerate
    spans widen to 1 ft; empty default {0,40,0,28}); `make_transform`
    min-fit scale centered in both axes, y-down
    y_screen = offset + (maxY − world.y)·scale; `calculation_rows` grouped
    code/totals.
  - **PDF export** (`export_pdf.rs`): hand-rolled minimal PDF 1.4 (no deps;
    the browser's output is print-CSS HTML, so the layout constants are
    transcribed and content drawn as PDF operators) — one page, MediaBox
    from the paper size, title SKETCH/AREA TABLE ADDENDUM (top 16.4pt),
    SUBJECT INFO box (top 38.2pt h 72.2pt, section rows), SKETCH heading
    (top 110.4pt h 12pt), sketch body (top 122.4pt w 540pt) with the fitted
    geometry — print wall widths exterior max(1.3, thickness||2) / interior
    max(0.65, thickness||1), dashed `8 5` / dotted `1.5 3.5` dash operators,
    bowed walls flattened — AREA CALCULATIONS SUMMARY table (324pt column,
    header h 12, rows h 10.74), COMMENT TABLE 1-3 ruled boxes (10 lines ×
    10.45), page number (height−21.5). Helvetica/Helvetica-Bold WinAnsi.
  - **PNG export** (`export_png.rs`): tiny-skia 0.11 (default-features =
    false, features std + png-format — png-format is required for PNG
    encoding) — `SketchApp.exportPNG` transcribed: target from
    `options.imageSize` (default 2048×1536), margin 60, min-fit zoom clamped
    0.1..10, centered, TRANSPARENT background (`renderTo(canvas,
    { transparent: true, clean: true })`); polygon fills (fill-opacity) +
    outlines, wall solids when enabled, wall strokes with dash runs
    (`8 5`/`1.5 3.5`), bowed walls flattened through the ring flattener.
  - **Ribbon wiring**: File tab Export group now live — Export PNG/DXF/
    SVG/PDF + Print (→PDF) + Import…/Import CAD with rfd save/open dialogs;
    `Message::{ExportSvg,ExportPng,ExportPdf,ExportDxf,ImportCad}` dispatch
    arms; notices ("SVG exported." / import `describe` summary with units +
    skipped counts).
  - Honored-by-omission: `exportLayer` (layers are a later phase) and
    `exportPurged` (symbol-library purge list, not an exporter) are not
    ported yet.
  - Evidence: `cargo fmt --all --check` clean; `cargo test --workspace`
    155 passed / 0 failed (46 new: DXF fixture round-trip through the real
    parser, SVG well-formedness/viewbox/dash/arc/escape, PNG IHDR
    dimensions + transparent corner + fit clamps, PDF MediaBox paper
    constants + layout sections + escape + print wall widths, fit-scale
    math on a known rectangle, fit-bounds label/symbol exclusion,
    calculation rows, pairs reader, insunits factors, unitless inference,
    bulge/arc flattening, legacy-POLYLINE vertex machine, app export-page
    mapping + solids + insert + notice); `cargo clippy --workspace
    --all-targets` 0 warnings; `cargo build -p sketch-app` links
    `target/debug/sketch-app.exe`.
  - Known gaps for the parent: PNG rasterizes geometry only (labels/symbol
    text not rastered — no font dependency in the core; the browser PNG
    includes them); PDF text positioning is layout-constant-based without
    per-string text metrics (single-line text, no wrapping); per-line
    `lineStyle`/label styling fields don't exist in the port's line/polygon
    models yet so exports use the option defaults; PNG export resolution
    always uses the default 2048×1536 (`options.imageSize` has no settings
    UI yet); PDF paper size fixed to Legal (`options.pageLayout` has no
    counterpart yet).
- 2026-10-21: **P5 — areas pipeline: define-area workflow, AreaBreakdown
  rollup, subject-info autofill, area editing, SKX persistence.**
  `sketch-app` only; `sketch-core` untouched (its `DEFAULT_AREA_CODES`,
  `validate_area_codes`, `basement_rollup_from_totals`, and
  `area_label_anchor` are consumed as-is).
  - **AreaCalc pipeline** (`areas.rs`, new): `refs/area-breakdown.js`'s
    embedded AreaCalc module transcribed method-for-method —
    `areaLevel`/`sameLevel` (BSMT*→0, GLA/GBA$n→n, other defined→1, UND any),
    `buildContainmentTree` (tightest same-level container via
    `polygon_contains`), `cycleMembers`/`resolveSubtraction` (auto→container,
    always→validated explicit target with the browser's warning strings,
    cycles broken with one warning each), `applySubtractions`,
    `applyFactors` (factorOverride wins; `positive:false` flips the sign),
    and the live-totals `rollup` (empty/UND never reach totals; unknown
    codes group to OTH and never count toward GLA; a deducted area still
    counts as itself unless net ≤ 0). `SketchState::recalculate()`
    (=`recalcAutoSubtract`) runs it after every close/apply/edit/delete and
    writes containerId/subtractedFromId/subtracted/effectiveMultiplier/sign/
    net back onto the stored polygons; `areaCalcRevision` bumps per commit.
  - **AreaBreakdown** (`areas.rs`): the class transcribed — CCW normalize
    with reversed-edge bow negation, signed circular segments (opposing the
    bow sign), rectilinearPartition + mergeBands, squareOff with triangle
    deducts, ear-clip triangulate, triangleDims (axis-aligned preferred),
    the winding-flip rule when a major segment outweighs the chord polygon,
    the balancing `Polygon` piece, and `toText` (detail + SUMMARY).
    Pinned against a Node oracle run over the ref slices: 1339+171=1510
    BSMT/BSMT2 basement rollup, 40×30 GLA minus 10×8 garage (net 1120,
    garage still 80), PORCH ×0.5 with negative STAIR inside (net 42),
    unknown-code→OTH, factorOverride-wins, bowed 20×10 gross
    173.26965010386067 / perimeter 60.52344305954062 with piece table,
    L-shape 360+96, triangle 0.5×20×15, and the exact toText SUMMARY block.
  - **Define-area workflow**: Define First gates the first exterior click
    (`handleCanvasClick`) with the browser's notice; closed rings stamp the
    armed classification (`autoClose` defineFirst branch). Apply stamps the
    pending code+name onto the selection or the last closed polygon
    (`btn-apply-area` fallback) via `defineAreaTarget`; Ctrl+Click on the
    canvas with a code armed runs `defineAreaAt` — region-under-point
    including `regionAreaAt`'s face materialization (enclosed exterior faces,
    `ownBoundaryLine` for edges that borrow part of a wall) with the
    browser's "Drop the area definition inside a closed shape." refusal.
    Redefine / Clone (walls offset +2') / Reopen / Detect Regions /
    Edit buttons wired in the Define panel; `assertAreaInvariants` ported and
    run under `debugAreaCalculations` after each recalculate.
  - **Calcs panel** (`ui.rs`): `updateCalcsPanel` ported — per-code groups
    from the polygons in first-encounter order with each area's net + factor
    (nested areas indented), summary block with Defined/Undefined counts,
    Total GLA, Total GBA, the BSMT+BSMT2 basement split (core's
    `basement_rollup_from_totals`), OTH, and Total All Areas; plus the Area
    Breakdown dialog's detail text via `AreaBreakdown::to_text`.
  - **Automatic labels**: global `autoPost` toggles (Code/Name/Calcs/Base/
    Factor) in the Define panel; the canvas label block sits on
    `area_label_anchor` and the size block below it (`getAreaSizePosition`'s
    `separationPixels / PIXELS_PER_FOOT` rule), honoring per-area
    `poly.autoPost` overrides merged over the globals.
  - **Area editing**: dialog-equivalent editor zone (name, factor override,
    subtraction mode pick list, deduction target) applying through
    `editSelectedAreas` with the browser's validation messages ("Factor must
    be zero or greater.", "Subtraction mode must be auto, always, or never.",
    "Deduction target must be an area id."); `updateSelectedAreaProperties`
    ported as `setSelectedAreaProperties` (type/name/factor/auto-subtract/
    per-area autoPost with the `dims`→`dimensionHidden` mirror).
  - **Subject info**: typed `SubjectInfo` model (browser camelCase fields,
    legacy string → generalComments on open); `mergeSubjectInfo` ported
    exactly (blank never overwrites, comments/tables/additionalFields
    preserved, GPS pair follows lat/lon) with `formatGpsPair`/`parseGpsPair`
    (decimal + DMS); the docked Subject Info panel edits all 13 parcel
    fields + comments with the browser's GPS validation message and Apply.
  - **SKX persistence**: `buildSaveModel` now emits the typed `subjectInfo`,
    `subjectInfoAutoFilled`, `settings.defineFirst`, `settings.autoPost`,
    the live `areaCodes` table (`includeInGLA` camelCase from core), and
    `options.defineFirst`/`options.autoSubtractOverlapped`; polygons persist
    the browser's computed fields (containerId, subtractedFromId,
    subtracted, effectiveMultiplier, sign, net, factor, autoPost,
    perimeter). Legacy files derive `factorOverride` from a plain `factor`
    on open (`ensurePageModel` rule).
  - Evidence: `cargo fmt --all --check` clean; `cargo test --workspace`
    109 passed / 0 failed (22 new: 10 areas-pipeline oracle pins, 8 state
    workflow tests, 2 SKX persistence tests, GPS + invariant tests);
    `cargo clippy --workspace --all-targets` 0 warnings; `cargo build -p
    sketch-app` links `target/debug/sketch-app.exe`.
  - Known fidelity gaps for the parent: the browser drops area definitions
    onto the canvas by drag-and-drop; the native shell defines via Ctrl+
    Click with an armed code instead. `assignAreaToDedicatedLayer` (every
    area on its own layer) is not ported — the Layers tree remains a stub
    and polygons keep `layerId: null`. Label/size positions are computed
    (no per-area drag pins: `labelPosition`/`areaSizePosition`). The
    Subject panel has no comment-tables/additional-fields editors and its
    Auto-Fill button is disabled (the suite-host bridge
    `__collectSketchSubjectInfo` has no native counterpart; the merge
    function itself is ported and tested). `measureTo` face-measure rings
    (interior/exterior offsets) are not ported — the editor omits the
    field. `toJSON`/`toXml` breakdown exports and autoPost font styling are
    later-phase work. Area attributes table (planRefNumber/floorCode/
    yearBuilt) not ported; the Edit button opens the property editor.
- 2026-10-21: **P4 — symbol placement depth (+ closeJoinedExteriorLoops subset,
  geometric wall solids).** `sketch-app` only; `sketch-core` untouched.
  - **Placement + wall snap** (`state.rs`): `resolveSymbolPlacement` ported
    verbatim — SIDE/CENTER/NONE with the `max(1.5, getScreenToleranceWorld(28))`
    tolerance, straight-walls-only nearest-wall projection (t in [0,1]),
    `rotation = -angle·180/π`, and the exact side-flip rule (`offset < 0.05`
    keeps the re-snap swing; otherwise drop-side decides; exterior walls flip
    via `exteriorSwingFlip`, signed-area winding + path-follow sense
    transcribed). `wallGapWidth` stored in FEET (definition inches / 12).
    `placeSymbolAt` commits with undo, selects, returns to Select; placement
    mode (`Mode::PlaceSymbol`) previews the snapped symbol at the cursor
    (canvas `drawPlacementPreview`, accent stroke, resolved rotation).
  - **Transforms + handles** (`symbol_render.rs`): `getSymbolTransform` port
    (bounds→screen rect, rotation about the anchor, corners, top anchor,
    rotation handle 30 px above), corner resize hit (7 px), rotation handle
    hit (11 px). Canvas draws symbols at `pixelScale = PPF·zoom·scale/12`
    with rotation + `flipH` mirror (drawSymbols' translate→rotate→scale(-1,1)
    order), selected accent stroke, and the labeled fallback box for
    unresolvable types. Drag-resize scales 0.25..8 in 5% steps (Ctrl 1%) with
    a live % chip; drag-rotate steps 15°/1°-Ctrl and detaches `wallId`;
    wheel/+/- scale the selection; H/V orient flattens and detaches;
    symbols move/drag with the selection (symbols-only drags move freely),
    `resnapSelectedSymbols` on drag release, `refreshHostedSymbols` follows
    non-selected hosted openings when walls move. Delete/clone (Ctrl+D)
    remove/detach-and-copy symbols; `pointInSymbol` hit test (rotated local
    bounds, 6 px pad) picks symbols before walls.
  - **Dock library management** (`ui.rs`): per-row ★ favorite toggle and ✕
    hide ("Delete from library" is a HIDE — the definition is parked so
    placed instances keep rendering via `symbol_definition`'s park fallback),
    "Restore hidden..." / "Purge hidden..." (purge also drops placed
    instances and records the type), favorites list now reads
    `state.symbol_favorites`. Edit Tools tab gains Flip Swing / Opening
    Wider / Opening Narrower when a hosted opening is selected (the browser's
    canvas context-menu actions); View tab gains a Wall Solids toggle.
  - **SKX persistence** (`model.rs`/`fileio.rs`): `SketchSymbol` with the
    browser's exact camelCase fields (`symbolType`, `wallId`, `wallSnap`,
    `wallGapWidth`, `wallFraction`, `wallSideFlip`, `wallSideExplicit`,
    `flipH`, `rotation`, `scale`); `pages[0].symbols` typed and round-tripped
    (tolerant defaults, legacy files fine); `settings.symbolFavorites`
    saved/restored; next_id covers symbol ids.
  - **P3a gaps fixed**: geometric wall solids — new `wall_solids.rs` ports
    `WallSolids.computeWallSolids` + `classifyJunction` + `gapSpan` +
    `wallGaps` + `openingRuns` (L/T/X/inline joins, corner extension over the
    mates' half-thickness, T-stops at the through-wall face, hosted-opening
    gaps cut through the full thickness); canvas fills solids (bowed walls as
    annular sectors) and the legacy stroke ALWAYS breaks at openings via
    `openingRuns`. `closeJoinedExteriorLoops` (subset): face walk through core
    `find_enclosed_faces`, materialized-edge check, signature dedup,
    claimed-line and containment filters, minimum area 0.01, repair-or-create
    UND polygons; runs after every committed wall (`finishCommittedWall`, pen
    ends when the committed wall joins the new ring) and after selection
    drags (`finalizeSelectionMove` toasts).
  Evidence: `cargo fmt --all --check` clean; `cargo test --workspace` 87
  passed / 0 failed (23 new: side/center snap + swing flip, drag re-snap,
  hosted follow, flip/opening width, clone/scale/delete, hide/purge parking,
  joined-loop close + no-duplicate + interior-exclusion, wall-solid corner/T
  joins + gap cuts + opening runs, symbol SKX round-trip + browser field
  names, resize/rotate transform math); `cargo clippy --workspace
  --all-targets` 0 warnings; `cargo build -p sketch-app` links
  `target/debug/sketch-app.exe` with `symbols.json` next to it.
  Known gaps for the parent: `healExteriorWallJoins` (welding free ends onto
  wall bodies / trimming overshoots) and `splitWallsAtJunctions` are NOT
  ported — only already-exactly-joined wall loops auto-close; library
  mutations (favorites/hidden/purged) are session-state only (the browser
  persists them in localStorage; a native sidecar is later-phase), SKX
  carries favorites only; dock drag-from-library (HTML5 drag) not ported —
  click-to-arm covers placement; symbol TEXT entities follow the transform's
  position but glyphs do not rotate (iced canvas text); per-symbol wall-solid
  thickness overrides (`line.thickness`) not modeled; per-row Rename and the
  full symbol-manager dialog not ported (manager basics live in the dock
  panel).
- 2026-10-21: **P3b — UI fidelity pass (window now looks like Appraisal Sketch 1.1).**
  New `ui.rs` + `symbol_render.rs` in `sketch-app`; `main.rs`/`state.rs`/`canvas.rs`
  extended. Ported from the real markup (`refs/ui-manager.js`, ribbon/sidebar
  layers, standalone-HTML tool rail) with `refs/theme.css` values — no eyeballing:
  - **Ribbon**: tab strip (File blue #007acc / Home / View / Tools / Reference
    Image / Help + contextual **Edit Tools** appearing with a selection) above a
    tool row of `ribbon-btn` groups (min-width 60, 4×6 pad, 11 px labels) with
    1×40 `.ribbon-divider`s. Home groups: clipboard (Cut/Copy/Paste/Select
    All/Select▾/Delete/Undo/Redo), Rotate/Flip Sketch, Draw Color, Subject
    Info/Area Attributes — verbatim order/labels. New/Open/Save live on the File
    tab (its dropdown menu is a popup gap, so actions render as a group); Grid
    toggle + Interface theme cycle on View.
  - **Left tool rail**: the real 40 px `.toolbar-left` — 17 `tool-btn`s (32×32)
    in the original order with group separators above items 5/9/12/15 and real
    tooltips; select/fence/pan/ext/int/measure wired to modes, Define/Text/
    Symbols rail buttons open their sidebar panels.
  - **Sidebar dock**: 312 px right dock, horizontal tab strip (Define/Text/
    Symbols/Calcs/Layers + collapse chevron; collapses to the 30 px rail).
    Define panel = three-zone workbench (Selection readout, Classification
    search + grouped tree from `sketch_core` `DEFAULT_AREA_CODES` with group
    names/`CODE - Name`/single-code roots, name input, pending code, Apply
    stamping the code onto the selected polygon with undo, Labels/Area-tools
    rows). Symbol Library = category pick list (Favorites + 19 categories from
    the EXTERNAL `assets/symbols.json`), search, list, click-to-arm placement
    onto the canvas (drawn via the library geometry, `max(12, 24·zoom)` box),
    92×68 preview canvas stroking the symbol's LINE/POLYLINE(+bulge)/ARC/
    ELLIPSE/TEXT entities per `renderSymbolPreview`/`renderSymbolGeometry`
    (y-up mapping, `fillColor === 0` fill rule), wall-snap caption, Organize…
    disabled. Calcs/Layers are styled stubs (trees are P5+).
  - **Widget theming**: every button/input/pick-list/panel/container styled
    from the `theme.rs` palettes (dark/blue/light) via per-widget style
    closures — ribbon/panel/canvas/status zones, `.action-btn`, `.area-btn`,
    `.lib-symbol-item`, `.sidebar-tab`, inputs. Search/name inputs suppress
    mode keys + the drawing buffer while focused (`text_focus` gate).
  - **Status bar**: 28 px bg-ribbon strip with top border, 11 px text — same
    P3a readouts plus the real bar's Center/Center&Fit/Zoom Fence/Grid/Snap
    buttons and the armed-symbol/notice/toast slot.
  - Window title `"Appraisal Sketch 1.1 - Untitled"` (hyphen, matching the
    real window-title pattern); 1280×800 default kept.
  Evidence: `cargo fmt --all --check` clean; `cargo test --workspace` 74 passed
  / 0 failed (6 new: preview transform fit, y-up flip, bulge sampling + JS
  bulge-radius formula, wall-snap caption, placed-symbol ids/snap/reset);
  `cargo clippy --workspace --all-targets` 0 warnings; `cargo build -p
  sketch-app` links `target/debug/sketch-app.exe`; `symbols.json` copied next
  to the exe so the launched window resolves the external library.
  Known fidelity gaps for the parent: Phosphor icons approximated with text
  glyphs; dock width fixed (no resize handle); popup menus, command palette,
  File dropdown, and wall-shape carets not rendered; sidebar tabs horizontal
  (shipped redesign layout) rather than vertical; Text Library label catalogue
  not ported (list honestly empty); Calcs/Layers trees stubbed; placed symbols
  not persisted to `.skx` (P4/P6); define-area full draw→classify flow is P5.
- 2026-10-21: **P3 partially complete (P3a — first working window, core-first).**
  `sketch-app` grew from a stub to a full Iced 0.14 shell: `state.rs` (headless
  app logic: mode machines, `snapPoint`/`structuralSnap`/`getDrawingTarget`
  wall-chain commit, keyboard buffer `parseBuffer` + Enter commit chain,
  measure click-out runs, hit-test/fence/move/delete selection, undo/redo
  snapshots with `restoreDrawingChain`, `zoomTo`/`centerAndFit`), `canvas.rs`
  (paint port: grid, walls, polygons+labels, selection handles, snap preview,
  measure chips; curved edges flattened through `GeometryEngine::flatten_ring`
  — 1 px sagitta, no re-derived bezier; Program::update publishes
  Press/Move/Release/Wheel), `fileio.rs` (`buildSaveModel`-shaped JSON,
  camelCase serde, `.skx` via `sketch_core::skx`, legacy-JSON open, tolerant
  read), `main.rs` (toolbar, status HUD, full keyboard via one
  `event::listen_with` subscription, rfd New/Open/Save). Modes live end to
  end: select+marquee, draw-exterior/interior (Shift 45° ortho, endpoint
  join, click-close auto area), measure, space/middle pan, wheel zoom to
  cursor, Shift+wheel bow. Evidence: `cargo fmt --all --check` clean;
  `cargo test --workspace` 68 passed / 0 failed (15 new sketch-app tests:
  buffer parse, snap join, 45° constrain, auto-close ring area, measure
  runs, wheel bow, sign-convention ring area, fence/delete, zoom-to-cursor,
  skx round-trip incl. bows, legacy JSON, corrupt bytes); `cargo clippy
  --workspace --all-targets` 0 warnings; `cargo build -p sketch-app` links
  `target/debug/sketch-app.exe`. Known simplifications for the parent: no
  `closeJoinedExteriorLoops` (walls committed by drag-join don't auto-create
  areas), no wall-solid geometry option yet (single-line render), dimension
  labels horizontal (iced canvas text doesn't rotate), placement preview /
  symbols / chains / underlay deferred to P4+.
- 2026-10-21: P2 skx codec + symbol loader complete: `skx.rs` (ZIP container port —
  deflate-raw manifest/summary/part, stored mimetype, EOCD scan, JS sha256 fallback,
  corrupt-input returns error not panic) and `symbols.rs` (typed tolerant loader,
  resolve order exe dir → exe assets → cwd; 462/19/43 pinned). Verified on the FINAL
  tree by the parent: `cargo test --workspace` 53 passed / 0 failed,
  `cargo clippy --workspace --all-targets` 0 warnings — commit 542dae0.
- 2026-10-21: P1 geometry kernel complete; all 39 `GeometryEngine` methods plus the
  file's `AreaCodes` module ported to `sketch-core/src/geometry.rs` (free
  functions/structs, `GeometryEngine` methods, serde on persisted shapes); 34
  oracle-derived tests green; `cargo test -p sketch-core` and
  `cargo clippy -p sketch-core --all-targets` clean. Oracle values captured by
  running `refs/geometry-engine.js` in Node (the ref file is truncated
  mid-`AreaBreakdown`; the class body and AreaCodes IIFE extract cleanly).
- 2026-10-21: P0 scaffold complete; `cargo check` green; symbol library extracted and
  verified; refs/ contains geometry-engine, area-breakdown, input-controller,
  canvas-engine, exporter, ui-manager, sketch-app, zip-container, skx-format, cad-import.
  Note: `lib.rs` declared `skx`/`symbols` modules whose files were never committed;
  placeholder module files created so the crate compiles (P2 replaces them).


## 2026-10-29 — 0.1.4 area properties survive boundary repair

Deleting a boundary wall now retains a page-local record of the original area.
Reclosing its outline restores its identity, name, code, attributes, factor and
subtraction settings, styling, label settings and layer/group. Geometry and
calculated area follow the replacement walls. Unfinished areas stay out of totals.
Matching uses surviving walls and their interior side; ambiguous matches retain
the record rather than transferring properties to a different room. Explicit
whole-area deletion does not create a repair record.

Repair records participate in Undo/Redo and survive SKX, autosave recovery and
layer export/import with remapped IDs. Twelve new regressions cover these paths,
changed geometry, curved boundaries, sequential deletions, complete boundary
replacement, malformed optional history and neighboring-room isolation.

Current validation and package hashes are in docs/BUILD_VALIDATION.md. Earlier
full-suite and screenshot counts above remain historical. Suite integration and
physical DISTO testing remain deferred.


## 2026-10-29 — 0.1.5 Windows GUI startup

Normal Windows builds now use the GUI executable subsystem and do not allocate
a console window. Test binaries retain console behavior. Package validation
checks the actual PE subsystem (2) and redirected installation-check output.
The area-preservation changes from 0.1.4 remain included.


## 2026-10-29 — 0.1.6 original HTML application icon

The qat-logo SVG from refs/ui-markup.html now supplies the native window/taskbar
icon and a multi-resolution executable icon resource. The artwork is unchanged.
assets/app-icon.svg is the source; the generate_app_icon example regenerates
the 256px PNG and nine-size ICO. Icons are embedded and need no sidecar. See current package
validation in docs/BUILD_VALIDATION.md.


## 2026-10-29 — 0.1.7 area-label spacing

Automatic area measurements sit an additional six pixels below the name/code
block at normal zoom. The gap follows label/measurement scaling and the shared
position is used by drawing, hit testing and report sketches. Manually pinned
measurement positions remain unchanged. SVG and fallback PDF label spacing
are also increased. Existing placement/typography checks are updated to the
new spacing; current validation is recorded in docs/BUILD_VALIDATION.md.


## 2026-10-29 — Appraise-It Pro desktop integration requested

The user requested Apex 7-style launch/edit/return integration with the native
Appraise-It Pro application, covering both legacy forms and UAD 3.6, and
explicitly excluding the HTML portal. Native Pro 3.8.8 and Apex v7 are installed.
Public COM/.NET documentation and read-only reflection confirm a desktop API;
TransferSketchData is documented for dynamic/UAD 3.6 reports only. Pro's native
Tools ribbon lists Apex, Sketch-It and Sketch32; no third-party executable
registration option was found in inspected settings or public docs. This is
a vendor-contract question, not a claim that partner registration is impossible.

No developer documentation is available from the user. The supported desktop
launch/registration and legacy sketch exchange contracts still need SFREP
confirmation before implementing an Apex-equivalent connector. Requirements,
acceptance cases and sources are in docs/APPRAISE_IT_PRO_INTEGRATION.md. An
unsent SDK request is in docs/SFREP_DESKTOP_INTEGRATION_REQUEST.txt. No app
connector or report mutation was made; the working 0.1.7 build is unchanged.


## 2026-10-29 — Full Apex replacement, single executable

The user expanded the request to every Apex-capable host and expressly
authorized protocol compatibility/impersonation. No companion programs are
allowed; implementation must be inside sketch-app.exe. The Appraise-It-only
caller gate and delegation have been removed. An embedded CLR component has
passed an actual same-process PID check and an isolated SDK round trip (subject,
two pages, 200 sf, EMF, save/close events). Four native model/export tests and
the ordinary backstage regression pass. Complete file/API compatibility is
still in development. Current scope, verified capabilities, remaining gaps and
acceptance gates are in docs/APEX_REPLACEMENT.md. No global registration has
been activated and no production report has been modified.


## Apex replacement scope and current verification — 2026-10-29

The target is a complete replacement across Apex-compatible hosts, inside the
sketch executable, with no companion runtime program. The Appraise-It-only
caller restriction has been removed. This is still development work, not a
complete replacement release. See docs/APEX_REPLACEMENT.md for the gap matrix.

Current focused checks: six Rust model tests, 48 embedded-component contract
assertions, Clippy with warnings denied, formatting, and a rebuilt-executable
SDK IPC round trip pass. Custom tag collection operations, snapshot proxy
identity, original polygon IDs, page-change events, five raster image formats,
legacy image members and physical-size-correct EMF export are covered. Native
Open/Save/Base64 requests reject unsupported operations explicitly rather than
opening a blank document or misrepresenting SKX as an Apex file. Corrupt linked
recovery state is rejected before GUI startup. Genuine AX file editing, live
model mutation, remaining SDK surfaces and non-SDK host mechanisms remain open.

No shared launch configuration or production report has changed. No 0.1.8
replacement package has been produced; the last standalone package is 0.1.7.

Additional setup validation: all nine focused Rust tests now pass, including
creating and reverting missing configuration, exact original restoration, and
preserving externally changed settings. The SDK running-process marker is held
inside the sketch process. No production configuration has been activated.


## Native Apex development implementation — 0.1.8

Independent AXF codec, layouts 1–38, now passes reference-reader acceptance
with populated files. Genuine-writer fixtures are edited/reopened, and all 462
symbol-library entries export as native blocks. Native Save/Save As/Base64/Load,
live fields/tags/areas, deletion, zoom, breakdown, callbacks and georeference
operations are implemented inside the sketch EXE. The host transport is embedded;
legacy file and TOTAL/WinTOTAL XML/COM modes use the same EXE. Production Apex
configuration and reports remain unchanged. See docs/APEX_REPLACEMENT.md for
explicit limitations and the real-host verification gates. This must not be
called a completed universal replacement. Current release work is a development
compatibility preview with package-level smoke checks.


## 0.1.8 compatibility preview — final validation

Native layouts 1–38 and genuine-writer edited fixtures pass the installed Apex
reader; all 462 symbol definitions round-trip. The expanded real-model SDK
transport test passes native Save/Save As/Base64/Load, public and subject tags,
georeferences, breakdown, deletion/zoom, full-form EMF physical size, UI Save
notification, repeated New and host Close. Native file-mode edit/return passes
in the same sketch PID without loading the SDK transport. Linked process
shutdown now avoids the observed CLR/PDF DLL-detach deadlock after document and
session cleanup. Clearing public area tags preserves native description/height.

The core suite passed 163 tests. The full app graphics runner reported all 536
assertions passing but lingered during Windows DLL shutdown and needed cleanup;
that is not recorded as a clean full-suite process exit. Focused final regression
and release/package validation are recorded in docs/BUILD_VALIDATION.md.

This remains a development compatibility preview. Metric/advanced CAD cases,
unfinished native drafts, additional calculation semantics, area-code URLs,
multiple-file startup and actual legacy/UAD 3.6 host verification remain open.
No production host configuration or report has been changed.

Packaged and smoke-checked: dist/SketchRust-0.1.8-native-preview-2026-10-29-windows-x64.zip.
Archive SHA-256: 32f24b692e9f6c2f98dd16d7e54505c439b6aede7d151cf2de9ba5cc58786a4c.
EXE SHA-256: 2d5955b462a5548a766644e0390634fd3d7747c408943f50d13e1cb752a4c3cc.
All ten manifest payloads verified; GUI subsystem 2; nine original icon sizes
match; 462 symbols/native view/SKX and embedded same-process host probe pass.
Final focused count: 13 compatibility/setup tests. Final Clippy and optimized
build pass. DISTO hardware testing remains deferred.


## 2026-10-29 — 0.1.9 requested native gaps implemented and packaged

Added metric import/save and native-unit API returns; nested, reflected and
nonuniform CAD transforms; real point preservation; drawable native symbols
with original topology retained in unused CAD blocks. Native area calculations
use signed factors and recursively deduct child net values, including correct
negative-child-net totals. Face measurement exports real measured boundaries
and restores matching wall centers/types/thickness settings.

Open native drafts retain repair history, IDs, names/codes/factors/attributes and
layer/group metadata inside AXF. HTTP/HTTPS/file code-table loading runs inside
the EXE; multiple native inputs get independent windows and save destinations.
Native JSON pages are typed-validated before binding so invalid records cannot
silently open as blank pages. Native angles, dimension references, styled-block
opacity and zero-scale insert conventions were checked against the real engine.

Validation: 164 core tests; 287 app tests with 253 e2e cases filtered; 54 component
contracts; 121 reference-reader files; six actual drawing-engine measurement
cases; full live SDK/model checks; loopback HTTP and file URLs; two independent
file sessions; repair and closed-file reopen in layouts 1/23/38. Formatting,
Clippy, optimized build and extracted-package smoke checks passed. No new GPU
screenshot/full-suite claim is made. Real host report/registration validation
and physical DISTO testing remain open/deferred as documented in the audit.

Package: dist/SketchRust-0.1.9-native-preview-2026-10-29-windows-x64.zip
Archive SHA-256: caa2c9b9e0bfd4a4b656528b18d9f00387150b4ae08e99d9bcb32b4651b5be43
EXE SHA-256: b5f6a7d8e275e41352ca1023bb9a27499ad5d284aef9074f170f60b33e280526
Ten payload hashes verified; one EXE, no companion DLL payload; Windows GUI
subsystem 2; all nine embedded icons match the HTML artwork; 462 symbols/native
view/SKX and embedded same-process compatibility probes pass. Production host
configuration and reports were not changed.
