# Sketch Rust — STATUS ledger

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
