# Sketch Rust — STATUS ledger

0.3.22 saves tape measurements with the sketch (user's choice after 0.3.21).
Each page keeps its own in a new `tapeMeasurements` key, written only when there
are some; a line still being taped is saved once it has two points. `.skx` and
JSON carry the whole model, `.sketchy` keeps unmodeled page keys in its level
extras, and `.sketchie` keeps them in its workfile, so (like other app-only data)
they are lost there if another editor changes the file's core geometry. One
unreadable run is dropped alone. Because they are now content: Escape and Delete
in the Measure tool drop only the unfinished line (they cleared everything);
Clear is undoable; a one-point run is discarded; New clears them (its reset did
not touch them before); whole-sketch rotate and flip move them; File > Import
brings them in at the usual offset. Prints, PNG/SVG/DXF/PDF, AXF, RapidSketch
and layer exports leave them out. Moving every wall with a selection does not
move them. Older versions open such a file and keep the key when saving but do
not show it.
Gate (Windows): `cargo test --workspace` 808 passed / 0 failed / 3 ignored, then
a lint fix in the new tests only, after which `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --all --check` and the 12 measurement
tests are clean. Save and reopen was tested in `.sketchy`, `.sketchie`,
`.sketchie.json`, `.skx` and `.json`. NOT verified by hand in the running app.

0.3.21, three user reports.
1. The symbol picker's category buttons wrapped into a 92 px scrolling box
(7 of 14 visible). The user chose a side list (mockup "A"): a Wide window
(>= 1100) gets a 540 px drawer with the categories down the side; narrower ones
keep Favorites and All and use one drop-down. Search now covers every category
(it searched only the open one); choosing a category ends the search. A library
with more categories than fit (the unedited default has 18) scrolls the list.
2. "Pin does not seem to do anything." Picking and placing kept the drawer, but
any tool, a tool fly-out, Escape or another popup's Close shut it. A pinned
drawer now stays until closed or unpinned, Escape goes to the drawing, and the
footer says so. The pin is still not remembered across restarts.
3. Tape measurements could not be touched with Select. Click picks one (it
draws highlighted with each leg's length), an end or corner drags to change it,
a leg drags to move it, arrows nudge, Delete / right-click / Inspector remove
it, all undoable. Leaving the Measure tool now finishes the open measurement.
Unchanged: they are not saved with the sketch, are not in exports, and Escape
while still in the Measure tool clears them all.
Gate (Windows): `cargo test --workspace` 804 passed / 0 failed / 3 ignored, then
a one-line lint fix in a test, after which `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo fmt --all --check` and that test are
clean. Checked with headless pointer input and screenshots; NOT verified with a
real mouse in the running app.

0.3.20 fixes dragging rows in the Areas list (user report: "it's janky"). The
drop line was drawn on every row except the one under the pointer, and taking up
layout space it also shifted those rows. Only a 6 px strip meant "above", and
nothing but a row's name took the drag. Now one line, drawn over the rows, marks
the target; each row takes the drag across its whole face, halves meaning above
and below (a folder keeps its middle for "inside"); either half of an adjacent
row moves past it; and the carried row stays lit. Gate (Windows): `cargo test
--workspace` 798 passed / 0 failed / 3 ignored, then a lint-only rewrite, after
which `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D
warnings` and the drag tests are clean. Checked with headless drags and
screenshots; NOT verified with a real mouse in the running app.

0.3.19 (user reports). Symbols: Corner Fireplace 36, which also replaces the
lopsided corner fireplace that RapidSketch / ACI Sketch files bring in; Sliding
Closet Door 48, 60, 72 and 96; and the 33 in double sink renamed Double Kitchen
Sink 33 and filed under Kitchen (503 symbols). The Areas list now reads
front-first, so the top row draws in front; paint order itself is unchanged, and
canvas clicks now take the area painted on top. A right-click picks a symbol or
text under the pointer, as a left click does, instead of the area beneath, so
Delete removes just that. Gate (Windows): `cargo fmt --all --check` and `cargo
clippy --workspace --all-targets -- -D warnings` clean, `cargo test
--workspace` 795 passed / 0 failed / 3 ignored (one library count assertion was
corrected after the full run and passed on its own). NOT verified: the changes
by eye in the running app, and how ACI Sketch itself draws its corner
fireplace; the swap places it where the walls of the sample show it belongs.

0.3.18 imports interior walls from RapidSketch / ACI Sketch files (user report:
"The interior walls are missing"). They are `FreeformLine` objects on the sketch
itself — two `SnappingHandle` ends, nothing else — which the first sample never
contained, so 0.3.17 left them out. They map to `WallType::Interior` with the
dimension hidden, and straight solid interior walls export as `FreeformLine`.
The reader no longer drops unknown objects inside areas or symbol drawings
silently: one warning names every kind left out. `code_for` also reads "N Car
Attached/Detached/Built-In" as `GAR`. Verified on a second private sample (21
interior walls in, 21 out at identical coordinates, matching member
declarations, and the export loads in .NET Framework's `BinaryFormatter`
against stand-in classes). Gate (Windows): `cargo fmt --all --check` and `cargo
clippy --workspace --all-targets -- -D warnings` clean, `cargo test
--workspace` 789 passed / 0 failed / 3 ignored with that sample supplied, and
the release build succeeds. NOT verified: an export opened in RapidSketch or ACI
Sketch, and the 0.3.18 import has not been checked by eye in the running app.

0.3.17 adds RapidSketch (`.rs`) and ACI Sketch (`.acisketch`) import and
export. The two are one format: an unencrypted .NET `BinaryFormatter` stream
(MS-NRBF) of a `RapidSketch.Sketch`. `sketch_core::nrbf` is our own bounded
reader and writer (no .NET objects are created, so a hostile file can only
fail); `sketch_core::rapidsketch` holds the document, the writer and the
mapping to the save model; `sketch-app/src/rapid_io.rs` the menus and opening.
Layout, mapping and limits are in `docs/RAPIDSKETCH.md`. Verified: the one real
sample (not committed: it is private; `RAPIDSKETCH_SAMPLE=<path> cargo test -p
sketch-core rapidsketch` runs against it) imports with area totals matching its
own labels and survives export and re-import; an export declares the same 28
classes and members as the sample, and loads in .NET Framework's
`BinaryFormatter` against stand-in classes. NOT verified: opening an export in
RapidSketch or ACI Sketch, and any real `.rs` file (the extension is from
public software listings). Gate (Windows): `cargo fmt --all --check` and
`cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo test
--workspace` 788 passed / 0 failed / 3 ignored with the sample supplied, and the
release build succeeds. The imported sample has not been checked by eye in the
running app.

UAD 3.6 F-1 area types (issue #19). The bundled area-code table now carries the
six URAR Appendix F-1 area-breakdown fields. `GLA1`–`GLA10` keep their codes but
describe themselves as "Finished Above Grade"; `BSMT` is "Finished Below Grade"
and `BSMT2` "Unfinished Below Grade" — no "gross living area" wording survives
in the table, and because the codes did not change, existing `.skx` files and
the Appraise-It mapping keep working. Three new built-ins fill the missing
fields: `GLAN` "Finished AG Nonstd" (10.004), `GLAU` "Unfinished AG" (10.005)
and `BSMTN` "Finished BSMT Nonstd" (10.007), all `includeInGLA: false`, so the
standard finished total is unchanged and each gets its own line in the Calcs
tree and the Area Breakdown SUMMARY (both are per-code, so no report text
changed). `geometry::area_codes::f1_slot` is the single code→field mapping
(URAR order), pinned by a test that also proves GBA/garage/porch/UND and
`GLA11` have no field. A stored `area-codes.json` from an older build is
merged with the built-ins it lacks once per `DEFAULT_CODES_REVISION` (marker
file `area-codes.defaults`), so users who edited types still get the new
defaults while a later deletion sticks. `appraise_it` now writes all six
living-unit fields (`FinishedAreaAboveGrade{,Nonstandard}`,
`UnfinishedAreaAboveGrade`, `FinishedAreaBelowGrade{,Nonstandard}`,
`UnfinishedAreaBelowGrade`): `AREA_FIELDS`/`UnitAreaTotals` grew from 3 to 6,
the summary shows each slot, and the read-back-and-rollback path is unchanged.
Field IDs come from the FNMA-URAR-0924 "Area Breakdown" block in the installed
client (`Validation Rules\FNMA-URAR-0924.xml`, and the form definition embedded
in `Bin\Sfrep.AppraiseIt.Client.dll`, which pairs each label with its
`TextField Id`); the three IDs already in use appear there under the same
names, but the three new ones are NOT yet verified against a live report's
living-unit data store — do that before claiming the COM transfer works.
Deferred: the issue's optional item 4 (a per-sketch ANSI/AMS measurement
standard printed with the area summary) — it would change the ported Area
Breakdown text, so it needs its own setting design.

0.3.12 also closes the Pages navigator when a page is picked (user request:
selecting a page left the panel open, forcing a separate close click).
`NavMsg::Go` in `pages_nav_msg` (main.rs) now sets `pages_nav_open = false`,
clears the hover row and re-syncs the canvas, then switches pages — the same
bookkeeping the Close arm performs. Ghost/Delete keep the panel open (they
are not page selections; the topbar's tab dropdown reuses `Go` harmlessly).
RED→GREEN e2e: `picking_a_page_closes_the_pages_navigator` (failed on the
pre-fix tree, passes after). Gate on the final tree: fmt 0, clippy 0, my
test green; 4 pre-existing failures belong to the in-flight 0.3.13
Areas-scoping work below (`drawn_list_shows_only_the_current_pages_areas`,
`areas_and_layers_show_as_one_tree_with_a_folder_button`,
`dragging_a_rows_grip_over_another_row_moves_it_there`,
`p10_touch_input_pad_and_gestures_across_sizes` — each fails solo, in files
this change never touched).

0.3.13 adds Apex AXF import/export to the Menu and scopes the Areas list to
the current page. New in sketch-core/axf.rs: a clean-room ENCODER
(`encode`/`write` + `bow_to_bulge` + `AxfDocument::from_page`) writing the
same APX4WIN layouts the reader documents — area-table entries, boundary
vertices with DXF bulges, boundary markers between areas, Win16-LOGFONT text
records, raw-deflated. No Apex or proprietary code; verified by round-trip
tests (doc → write → read byte-exact; to_model → from_page → write → read)
and an e2e export→reimport flow. Menu wiring: Import › "Apex AXF sketch…"
(pick .axf → `open_axf_path`, the converted-untitled-copy open) and Export ›
"Apex AXF" (save_file_async → `AxfDocument::from_page(current page)` →
`axf::write`), both message-driven dialog continuations; docs/AXF_IMPORT.md
gains the export section with the honest Apex-compat scope note. The Areas
panel fix (the Site Plan screenshot): `drawn_view`'s "area is gone" branch
rendered every dedicated layer whose polygon is not on the current page as an
Other row — it now checks `app.pages` and only keeps layers whose area is
gone NOWHERE (still deletable); cross-page layers appear on their own page
(getAreaLayersForCurrentPage semantics). RED→GREEN e2e:
`drawn_list_shows_only_the_current_pages_areas`. docs/RELEASE_README.txt
updated.

0.3.12 fixes the arrow-key nudge deform reported on the real subject sketch
(9410 Waddell Creek Rd): selecting a wall and pressing an arrow moved it
diagonally (~10 ft sideways) instead of one pitch along the arrow. Root cause:
`nudge_step` used Rust `f64::signum`, which returns 1.0 for +0.0 (JS
`Math.sign(0)` is 0), so the zero axis of `Nudge(0, ±1)` took the
`pitch * 10` Shift branch as a positive direction — every plain arrow carried a
phantom ten-pitch sideways component. Fixed with a JS-faithful sign helper
(main.rs); regression test
`arrow_nudge_with_zero_axis_keeps_that_axis_stationary` (RED→GREEN) plus a
real-file probe (`probe_real_file_nudge_variants`, skips when the workfile is
absent) pin all four directions and the Shift variant, and the real-sketch
geometry after fix: the shared balcony/second-floor wall moves (−1,−52)→
(−1,−53), sides stay vertical, both areas stay rectilinear. Gate on final
tree: fmt/clippy clean, 398 passed; 3 pre-existing failures in parallel
Ledger-tree/row-drag/touch-pad WIP (`areas_and_layers_show_as_one_tree…`,
`dragging_a_rows_grip…`, `p10_touch_input_pad…`) — untouched by this change,
fail in isolation the same way. Remaining `.signum()` sites (area_repair,
chains, sketch-core bow/area signs) flagged in the skill for the same-trap
audit.

0.3.11 stops the file-dialog freeze for good: the interface thread no longer
hand-pumps `PeekMessage`/`DispatchMessage` while a dialog is up (the pump ran
nested inside an unfinished winit dispatch and wedged when Shell dialog init
stalled on a network-share start folder). Dialogs now run on a spawned
`file-dialog` thread awaited by the iced event loop (Task/oneshot), all ~19
call sites are message-driven continuations (`Message::DialogPicked`), and
every dialog still anchors to an explicit local start directory
(`dialog_start_dir`). Pinned by `dialog_tests` source contracts
(`dialogs_never_pump_the_interface_thread`, anchor count on `rfd::FileDialog::
new()` constructions). Verified live: File > Open and Save open real dialogs
and load/save the subject sketch without hanging.

0.3.8 fixes interactive UI hangs and frame freezes: eliminates exponential
backtracking in exterior wall-join healing (`walk_closed_loop`), fixes
T-junction host wall splitting, fast-paths rectangular/convex area label
anchors so `pole_of_inaccessibility` is skipped during drawing and drags,
defers dirty-state hashing during active drags/pans without re-serializing
`.sketchie` backing payloads, keeps the floorplan canvas cache warm across
chrome/overlay hovers, and clamps dotted/hatched area fills to the viewport.

0.3.7 fixes freezes (dialog wait, per-layer redraw cost) and snaps drags and
moves onto nearby endpoints. The MSI registers .sketchie/.skx (and Open with
for .json); the demo is seeded into Recent sketches once; the start screen has
a show-at-launch checkbox (start-screen.json).

0.3.6 re-closes an area opened by deleting a wall from its own record; the
Areas list shows and deletes any layer or area; layer export removed.

0.3.5 layer-row eye, lock and menu always visible; Apply to all labels;
readable label ink on dark canvases; Ctrl+click toggle; angle snap and arrow
nudge follow the snap settings; layers paint in list order.

0.3.4 shows the Open and Save dialogs from a separate thread to stop them
hanging on "Working on it..." (seen in Downloads).

0.3.3 adds a grab handle to each Areas and layers row, lets an explicit Bold
choice override the first-line-bold default on area labels (renderer, inspector
button and exports), removes Sketchie draft 0.1 support (0.2 only, which also
removes the Save As refusal), and ships examples/fallingwater.sketchie with the
installer and portable package.

0.3.2 also opens legacy `.axf` sketches. The file is unpacked (raw deflate),
its areas, arc walls and room labels are mapped into a new untitled sketch
(area codes, names and factors from the file), and Save As writes `.sketchie`.
Each boundary is re-measured and checked against the area the file recorded.
Doors/fixtures, dimension strings, the subject header and metric files are not
imported. See [docs/AXF_IMPORT.md](docs/AXF_IMPORT.md). This replaces the
"AXF files are no longer supported" refusal; no Apex SDK, writer or host
integration comes back with it.

0.3.2 merges the Areas and Layers lists into one tree: area rows with the type
and size on a second line, layer folders, eye and lock on each row, drag to
reorder or move into a folder, and Move to layer in the row menu.

0.3.1 adds the Door Garage 216 symbol (replacing Door - Garage-Dbl), fixes the
offset selection box and handles on symbols whose origin is not at their
centre, and draws symbols inverted on Dark and Blue canvases.

0.3.0 adds a resizable Layers area, a page switcher in the top bar, a
right-click menu that closes on canvas click or Escape, type-following default
area names, a smaller default area-type and symbol set, and Light/Blue/Dark
ordering for both theme selectors.

0.2.9 prevents the floating tool tray from overlapping Zoom, Grid, Snap,
Show, and Dimensions. The shared dock reserves space for all twelve tools
and both control groups; controls move to top rows when the canvas is too
narrow. Sidebar and inspector widths, touch density, popover clearance,
and horizontal tool scrolling are included in placement.

0.2.8 restores Freeform and Erase in the tool tray and command search. Freeform
draws unsnapped interior strokes without dimensions or wall solids. Erase
supports clicks and continuous drags across lines, text, and symbols, respects
hidden/locked layers, and groups each gesture into one Undo step. Mouse and
touch input share the existing save, print, and export pipeline.

0.1.30 removes the Apex-based host integration and AXF support. Sketchy and
SKX remain the editable formats. See the
[Appraise-It Pro integration plan](docs/APPRAISE_IT_PRO_INTEGRATION.md).
Entries below describe earlier builds.

Standalone native Windows floor-plan sketch application.

0.1.29 adds an in-app User Guide on Help (seven topics, including
Appraise-It integration setup, daily use and troubleshooting), with an
Integration Guide button that opens it at that topic. A per-user Windows
installer (installer/SketchRust.wxs, built by tools/installer.ps1 with WiX 5, the
same toolchain as the ArtCraft Launcher's) is built alongside the zip.

0.1.28 fixes a crash when File > Recent Files was opened with at least one
entry: the row's filled file icon was not embedded. A release build now draws
nothing for a missing icon instead of exiting; debug builds and tests still fail.

0.1.27 presents frames without vsync, so the wall being drawn no longer trails
the mouse cursor by a frame or two (ICED_PRESENT_MODE=vsync restores it). It is
also the first numbered build with the CADdillo door pivot fix, Help > About and
uniform Edit Tools styling.

0.1.26 is the window and desktop release. Maximize replaces full screen, so the
window stays inside the work area and off the taskbar; the title-bar button
shows Maximize or Restore. Size, position and maximized state are remembered,
with a persistent minimum window geometry, and edge/corner resize zones. The
title bar shows Unsaved/Saved and closing an unsaved document asks
to save. A .sketch file can be dropped on the window or passed on the command
line; opening a file that is already open brings its window forward. The
quick-access menu opens New, Open, Save As and Options. Pinch zooms and
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

0.1.23 fixes category deletion in Manage Symbols. Opening preserves
that category. A visible, named confirmation precedes deletion; picker/list/
preview state refreshes afterward. Empty categories can also be removed.

0.1.22 adds Save to report / Discard changes / Cancel when closing an integrated
sketch. Save closes after host acknowledgment; failure keeps the editor open.
Title-bar, File Exit and native window close requests share the same flow.

0.1.21 removes the managed compatibility console and moves live host snapshot
rendering off the input thread. Unchanged reads reuse images; late jobs cannot
replace newer saves. High-frequency pointer events redraw locally while HUD
updates are coalesced to frame intervals. Integration remains embedded in Sketch.

Previous release: **0.1.29** (2026-11-11). Help > Appraise-It Integration exposes
Enable integration and Restore previous provider without terminal commands.
It uses the embedded registration backend and handles protected-version upgrades.
Print Preview, Print and PDF export
include centered Summary and Breakdown boxes at the bottom, without bottom
branding. Long calculation tables continue on additional pages. The crosshair
remains visible after labeling an area or ending a keyboard-drawn boundary.
Drawing uses the supplied crosshair SVG
with a centered hotspot. Hover updates skip repeated document bookkeeping and
cache the floorplan while keeping live guides and previews responsive.
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
- **Symbol library is EXTERNAL**: `assets/symbols.json` (462 symbols, 19 categories,
  verified), loaded at runtime next to the exe. Never embed it in the binary.
- **Scope**: core-first (draw/measure/areas + symbols + SKX save/load in v1); everything
  else phased, including Disto laser connect and DXF CAD import/export.
- Sign conventions are load-bearing (see `crates/sketch-core/src/geometry.rs` header).

## Phase plan

- [x] **P0 scaffold** — workspace, `assets/symbols.json` verified (462 symbols / 19
      categories / 43 favorites), cargo check green.
- [x] **P1 geometry kernel** — `sketch-core/src/geometry.rs` with unit tests.
- [x] **P2 file + symbol layers** — `sketch-core/src/skx.rs` (ZIP container, deflate);
      typed `symbols.rs` loader for external `symbols.json` with runtime path resolution.
- [x] **P3 app shell** — Iced canvas, wall drawing (bows/arcs), measure tool, selection,
      pan/zoom, toolbar + panel, status HUD, themes (dark/blue/light).
- [x] **P4 symbols in-app** — placement, wall snap (NONE/SIDE/CENTER + gap widths),
      previews, favorites, context-menu actions, resize/transform handles, geometric wall
      solids.
- [x] **P5 areas** — define-area, area codes + breakdown panel, rollups, basement rules,
      auto-fill subject info.
- [x] **P6 persistence** — SKX save/open, undo/redo history, recent files.
- [x] **Remaining standalone software modules** — layers, pages, dimensions,
      lasso/transform, DXF import/export, PNG/SVG/PDF exports, print preview,
      area breakdown, reference images/geo controls, Options, Subject Info,
      command search and reusable libraries.
- [ ] **Disto physical-device acceptance** — explicitly deferred by the user.
      Native BLE software and simulated measurement tests are implemented.


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
full-suite counts above remain historical. Suite integration and
physical DISTO testing remain deferred.


## 2026-10-29 — 0.1.5 Windows GUI startup

Normal Windows builds now use the GUI executable subsystem and do not allocate
a console window. Test binaries retain console behavior. Package validation
checks the actual PE subsystem (2) and redirected installation-check output.
The area-preservation changes from 0.1.4 remain included.


## 2026-10-29 — 0.1.6 application icon

The app icon SVG (`assets/app-icon.svg`) supplies the native window/taskbar
icon and a multi-resolution executable icon resource. The
`generate_app_icon` example regenerates the 256px PNG and nine-size ICO.
Icons are embedded and need no sidecar. See current package
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
Tools menu lists Apex, Sketch-It and Sketch32; no third-party executable
registration option was found in inspected settings or public docs. This is
a vendor-contract question, not a claim that partner registration is impossible.

No developer documentation is available from the user. The supported desktop
launch/registration and legacy sketch exchange contracts still need SFREP
confirmation before implementing an Apex-equivalent connector. Requirements,
acceptance cases and sources are in docs/APPRAISE_IT_PRO_INTEGRATION.md. No app
connector or report mutation was made; the working 0.1.7 build is unchanged.


## 2026-10-29 — Full Apex replacement, single executable

The user expanded the request to every Apex-capable host and expressly
authorized protocol compatibility/impersonation. No companion programs are
allowed; implementation must be inside sketch-app.exe. The Appraise-It-only
caller gate and delegation have been removed. An embedded CLR component has
passed an actual same-process PID check and an isolated SDK round trip (subject,
two pages, 200 sf, EMF, save/close events). Four native model/export tests and
the ordinary file-menu regression pass. Complete file/API compatibility is
still in development. Current scope, verified capabilities, remaining gaps and
acceptance gates are in the 0.1.29 source history. No global registration has
been activated and no production report has been modified.


## Apex replacement scope and current verification — 2026-10-29

The target is a complete replacement across Apex-compatible hosts, inside the
sketch executable, with no companion runtime program. The Appraise-It-only
caller restriction has been removed. This is still development work, not a
complete replacement release. See the 0.1.29 source history for the gap matrix.

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
configuration and reports remain unchanged. See the 0.1.29 source history for
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
Clippy, optimized build and extracted-package smoke checks passed. Real host
report/registration validation and physical DISTO testing remain open/deferred
as documented in the audit.

Package: dist/SketchRust-0.1.9-native-preview-2026-10-29-windows-x64.zip
Archive SHA-256: caa2c9b9e0bfd4a4b656528b18d9f00387150b4ae08e99d9bcb32b4651b5be43
EXE SHA-256: b5f6a7d8e275e41352ca1023bb9a27499ad5d284aef9074f170f60b33e280526
Ten payload hashes verified; one EXE, no companion DLL payload; Windows GUI
subsystem 2; all nine icons match; 462 symbols/native view/SKX and embedded
same-process compatibility probes pass. Production host configuration and
reports were not changed.
