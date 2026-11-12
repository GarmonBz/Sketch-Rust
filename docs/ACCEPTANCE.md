# ACCEPTANCE — native rebuild, screen by screen

The first native attempt (through commit 57f9b29, tag `attempt1-rejected`) was
rejected by the user: "most of the features are missing and what is there is
broken." Its unit-tested kernel (`crates/sketch-core`) stays; the app layer is
rebuilt here one screen at a time.

## The rule

A screen is DONE only when the parent (not a subagent) has:
1. run `tools/e2e.sh` — the headless end-to-end suite (`crates/sketch-app/src/e2e.rs`,
   `iced_test` 0.14 Emulator running the REAL program from `program()`, scripted
   mouse/keyboard input through the real widget tree, no window, no real mouse)
   plus a per-region pixel comparison of every headless screenshot against its
   `refs/ui-spec/gallery/*.png` reference — and found no visible difference in
   layout, sizes, colors, text, icons, or states,
2. confirmed the screen's interactions as e2e tests that drive the same input
   the original was driven with in the browser and reach the same results,
3. recorded the evidence (the e2e.sh table + what was driven) in this file.

Headless screenshots measured IDENTICAL to live PrintWindow captures of the exe
(every region of every Screen 2 state, to 0.01%), so they are a faithful proxy.
Real-input driving of the live window (`tools/click-real.ps1`) is a last resort:
the user works on this PC, clicks sometimes never reach the app, and it moves
the user's mouse. Behavior that must be testable goes through the widget tree
(the emulator does not deliver events to subscriptions).

Unit tests are necessary but are NOT acceptance. A subagent's report is NOT
acceptance.

## Reference material

- `refs/ui-spec/chrome-spec-home.json` — every visible element of the default
  state (Home tab, Areas dock): tag, class, text, tooltip, x/y/w/h, background,
  text color, font, border, radius, padding. Region boxes at 1400x860:
  title bar 0,0 1400x30 (#1e1e1e) · ribbon 0,30 1400x99 (#2d2d30) ·
  workspace 0,129 1400x703 · status bar 0,832 1400x28 (#2d2d30).
- `refs/ui-spec/chrome-spec-states.json` — the same for every ribbon tab
  (Home/View/Tools/Reference Image/Help) and every dock tab
  (Areas/Text/Symbols/Calcs/Layers).
- `refs/ui-spec/gallery/*.png` — screenshots of each of those states.
- `refs/*.js` — behavior source; `refs/theme.css` — styling source.
- `assets/icons/` — the original's Phosphor icon set (MIT, regular + fill).

## Screens, in order

| # | Screen | Acceptance evidence |
|---|---|---|
| 1 | Static chrome, default state: borderless window, own title bar (logo, QAT, save state, title, min/full-screen/close, drag to move), tab strip, Home ribbon, 17-button left rail, white canvas, right dock on Areas, status bar | screenshot matches `ribbon-Home.png` / `dock-Areas.png` |
| 2 | Ribbon tabs: View, Tools, Reference Image, Help bodies + File backstage; contextual Edit Tools tab on selection | each tab matches its gallery image |
| 3 | Dock tabs: Text, Symbols (external library), Calcs, Layers; pin/collapse; resize | each tab matches its gallery image |
| 4 | Status bar controls live: grid/snap toggles, grid pitch, zoom slider + scale, segment readout, north, pages, underlay, filter | interactions match the original |
| 5 | Canvas: grid, pan/zoom, draw exterior/interior walls with live dimensions, ortho, snap, keyboard entry | same drawing produces the same picture in both |
| 6 | Selection/edit: select, fence, lasso, move, delete, insert vertex, curve wall, undo/redo + history | same edits, same results |
| 7 | Areas: define areas, Areas dock, attributes, Calcs totals | same areas, same numbers |
| 8 | Symbols: place from library, wall snap, flip, resize | same placements |
| 9 | Text library, chain dimensions, room shapes, measure | match |
| 10 | Pages, layers, reference image/underlay, geo-referencing | match |
| 11 | File: new/open/save .skx, print, PDF/PNG/SVG/DXF export, CAD import, options dialog | files round-trip with the original |
| 12 | Touch panel, Bluetooth (Disto), command search, remaining dialogs | match (Disto: hardware-unverifiable, stated) |

## Status

### Current standalone release audit and package (2026-10-29)

- Replaced the stale missing-feature lists and percentage estimates with the
  explicit 634-row ledger in `refs/ui-spec/feature-audit.json`, reproduced by
  `tools/audit_features.py`. Classification: 546 implemented, 44 native
  adaptations, 7 native enhancements, 5 Disto hardware-deferred, 20 expanded
  reference-unavailable, 9 source-inactive and 3 suite-only. Grouped regression
  evidence is not claimed as independent pixel acceptance of every row.
- Closed confirmed software gaps: B bay-window mirror (matching the original
  reflected endpoint/run state, with undo/redo), the Calcs Calculation Details
  button, seven command-search file actions, absolute Area Grid Snap including
  off-grid starts, relative ordinary geometry snapping, free label-only moves,
  and real symbol/text placement previews. A newly placed symbol is selected
  and opens the contextual editor. The resulting hosted-opening view exposed
  a misspelled icon name; it now uses the embedded `arrows-left-right` asset.
- Six new regressions cover mirror geometry/undo/redo, Calcs button dispatch,
  new-symbol contextual selection, free-label/area-grid dragging and the move
  threshold/reference behavior. All catalog commands have native handler tests.
- The first unconstrained parallel graphics run, concurrent with an optimized
  build, ended with an access violation; it is not
  acceptance evidence. The standard `tools/e2e.sh` now defaults to two test
  workers and uses pipefail, avoiding its previous duplicate test-suite run.
- The native `--check-installation` path checks the external library, constructs
  the view and writes/reads SKX in memory without user-profile persistence.
  `tools/package.ps1` builds the optimized executable and packages it with the
  external symbol JSON, usage notes, audit and a SHA-256 provenance manifest.
- Physical Disto testing remains deferred at the user's explicit request.
- Final software gate: **487 app tests + 163 core tests pass (650 total)**.
  The app suite ran with `--test-threads=2`, including the complete headless
  reference-state screenshot test. All 73 configured reference pairs were
  compared with the established regions/tolerance; maximum overall difference
  remains 2.99%. Per-region evidence: `target/e2e/release-comparison.txt`.
- Final Clippy `--workspace --all-targets -- -D warnings`, formatting and diff
  checks pass. The final optimized Windows x64 build completed successfully.
- Packaged ZIP extracted and every manifest file hash verified. Its executable
  matches `target/release/sketch-app.exe` (SHA-256
  `E356962C100749CAE92A0015A4BE79E4198340E217832BEF9010CEBFD2594EB0`).
  From an empty working directory, `--check-installation` reports 462 external
  symbols, native view construction and a successful in-memory SKX round trip.
  A separate copy without symbols.json exits 1 with the expected error.
  Evidence: `target/package-smoke/result.json` and its stdout/stderr files.
- Computer Use launched `dist/sketch-app.exe` and inspected the returned unique
  process-backed window: initial native chrome, drawing canvas, populated Areas
  palette and status bar rendered without an error overlay. No mouse/keyboard
  drawing automation was used on the desktop. The app is left open for review.

### Area-code JSON file actions (2026-10-29, S7-011/012)

- `refs/ui-manager.js::showAreaCodeEditor` is the behavioral reference. Import
  loads a selected JSON file into the textarea without parsing or applying it.
  Export parses/validates and pretty-prints the draft before the native save
  picker, using `sketch-area-codes.json`. Both actions keep the editor open.
- Apply remains the only action that changes the live table, and now also
  persists it through the existing code-library sidecar. The shared update path
  refreshes the palette and calculations. Import strips a UTF-8 BOM like the
  browser's text reader. Picker cancellation is silent; file errors report the
  reason and preserve the draft/library. Closing/reopening invalidates pending
  native picker results.
- Four new tests cover real-widget button routing, file import/export and
  normalized values, Apply/restart persistence, malformed text remaining
  editable, read/write failures, cancellation and stale callbacks. Test builds
  inject native picker results; no file chooser opens on the user's desktop.
- Disto native hardware acceptance is still pending: the user reported that
  the instrument is unavailable for this session.
- Validation: all 481 app tests pass, including the four new file-action tests
  and the existing Disto regressions. The reference-state screenshot test was
  scoped to `area-codes-editor` for this change; its overall difference remains
  2.99%, with comparison images in `target/e2e/cmp-codes-json/`. Clippy with
  `-D warnings`, formatting and whitespace checks pass. Debug executable rebuilt.

### Leica DISTO BLE measurement path (2026-10-29)

The user requested importing and applying the `leica-disto-ble` skill, kept
outside this repository. Its E7500i/D510 hardware
observations supplement the older standalone HTML's connection-only behavior.

- Native `disto_transport.rs` discovers Leica devices, connects, discovers the
  Leica service and distance characteristic, and subscribes to INDICATE/NOTIFY.
  Only that characteristic feeds positive finite float32 LE metres; no unit,
  tilt or temperature characteristic is interpreted as distance.
- Standard replaces the ordinary keyboard length buffer and waits for direction
  and Enter. Continuous commits along the previous segment's heading. Rounding
  follows the selected sketch units before converting to internal feet.
- Connect toggles disconnect. Cancel/Escape invalidates pending connections;
  scoped scan/connect/measurement events reject previous sessions. Unsubscribe,
  disconnect cleanup and physical-link checks run on the Bluetooth worker.
- Regression tests use the captured `cf f7 db 3f` sample and injected radio
  events. They cover rounding, invalid payloads, stale events, cancellation,
  modal/inactive guards, metric conversion, Standard/Continuous drawing,
  interior wall type and undo/redo. They revealed and fixed missing anchor
  restoration and undo banking in the shared keyboard Enter path, matching
  `refs/input-controller.js` and `refs/sketch-app.js::drawLine`.
- No new dialog layout or visual acceptance claim. **Native hardware connection
  remains UNVERIFIED.** The skill's browser hardware result is not acceptance
  evidence for the native btleplug implementation. See `docs/DISTO.md` for the
  real-device acceptance procedure.
- Validation: 477 app tests and 163 core tests pass (640 total), including the
  complete headless reference-state screenshot test. Clippy with `-D warnings`,
  formatting and whitespace checks pass; the debug executable was rebuilt.
  All 73 configured screenshot pairs were compared at the existing tolerance;
  maximum overall difference remains 2.99%. Per-region results are recorded in
  `target/e2e/disto-comparison.txt` (existing visual differences are not claimed
  resolved by this work).

### Remaining area-definition workflow (2026-10-28)

The user requested completion of the workflow described by the inventory,
including the optional layer absent from the older gallery captures. The
behavioral reference is `refs/p8-optional-region-classification.js` (original
HTML lines 22488-23059), plus the existing `defineAreaTarget`, `detectRegions`
and `reopenSelectedArea` implementations in `refs/sketch-app.js`.

- Implemented guidance, collapsible palette groups, current-page usage badges,
  click/Enter/Space arming, repeated canvas assignment, type drag ghost/drop,
  undefined highlights and the corrected New area / Edit areas links.
- Implemented the region chooser: search across groups/descriptions, group
  buttons, classification, custom name, label settings, original suffix list,
  repeat selection, Save/Cancel, focus wrapping and suppression of drawing
  shortcuts. Existing custom classifications remain editable. A provisional
  region is materialized only on Submit; cancellation preserves drawing and
  history. Page changes, deleted/hidden/locked targets and resize dismiss it.
- Implemented the type context menu and its assignment/select/New/Edit/Delete
  actions, with confirmation and UND protection; the Edit areas sheet's batch
  Redefine, Clone, Reopen, Detect Regions and saved label/dimension defaults.
  Calcs right-click, canvas Edit Definition and area double-click share the
  chooser. Escape closes the sheet or returns to the previous drawing mode.
- Twenty new app tests cover real palette/group/menu input, keyboard Space and
  Enter activation, chooser keyboard editing and forward/reverse focus wrap,
  repeated assignment, cancellation/materialization/undo, labels and custom
  names, unknown codes/search, type deletion confirmation, locked/hidden/stale
  targets, region detection/reopening/reclosing, batch undo, defaults save/open,
  canvas/Calcs context entry, page changes and four workflow screenshots.
- New screenshots: `target/e2e/workflow-hover.png`, `workflow-chooser.png`,
  `workflow-editor.png`, `workflow-type-menu.png`. These supplement the existing
  references; they do not establish pixel identity with an uncaptured chooser.
- Final gate: **473 app + 160 core tests pass (633 total)**, including the
  complete reference-state screenshot test and all 20 new workflow tests.
  Clippy with `-D warnings`, fmt and `git diff --check` pass. All 73 existing
  reference pairs compared using the standard regions/tolerance; worst overall
  difference remains 2.99% (area-code editor), with per-region results in
  `target/e2e/area-workflow-comparison.txt`. New workflow screenshots inspected
  for layout and readable controls. Debug executable rebuilt successfully.

### S7-073 follow-up — interior/exterior wall-face calculations (2026-10-28)

- `offsetRing` / `faceMeasurePolygon` ported from `refs/area-breakdown.js`
  and `refs/sketch-app.js`. Geometry remains on centerlines for editing and
  persistence; measured copies feed containment, subtraction, GLA, Calcs,
  breakdowns and export/report totals for both live and stored pages.
- Five core regressions verify a 10x10-foot square with 6-inch walls
  (centerline 100 sf / 40 ft, interior 90.25 sf / 38 ft, exterior 110.25 sf /
  42 ft), winding/variable offsets, disabled/invalid/collapsed fallback,
  bowed-edge chord measurements and a measured child's subtraction.
  Running the original JS offset function independently confirms the areas
  and the collapsed-ring fallback.
- Three app regressions exercise Area Properties Measure/Apply, undo/redo,
  live Calcs/breakdown values, option/type/thickness cache invalidation,
  save/open and export of current/stored pages with different thicknesses.
  Targeted gates: all 8 pass; Clippy with `-D warnings` and fmt pass.
- Full gate: 453 app + 160 core tests pass (613 total), including the full
  headless reference-state screenshot test. All 73 pairs configured in
  `tools/e2e.sh` compared using `tools/compare.py`'s regions and tolerance;
  maximum overall difference 2.99% (area-code editor). Per-region evidence:
  `target/e2e/face-measure-comparison.txt`. Existing visual differences are
  not claimed resolved by this calculation change. Debug executable rebuilt.
- No dialog layout changes; this closes the calculation gap, not the other
  outstanding Screen 7 workflow or visual acceptance items.

- [x] Screen 1 — accepted by the parent 2026-10-22. Evidence: `tools/shot.ps1`
      (PrintWindow capture, SKETCH_DPR=1) vs `refs/ui-spec/gallery/ribbon-Home.png`
      via `tools/compare.py`: title bar 2.14% · ribbon 2.22% · left rail 3.97% ·
      canvas 0.00% · dock 1.74% · status bar 4.32% · overall 0.88% (attempt 1:
      90.11%). Diff image shows text/icon anti-aliasing edges only, no missing or
      misplaced element; 3x zoom strip comparison of the whole status bar
      identical item for item. Window controls driven with real input
      (`tools/window-controls.ps1`): Minimize → iconic, Full Screen → 3456x2160
      and back, title-bar drag moved the window 120,60, Close ended the process.
      Real 250% DPI launch (maximized) reviewed: crisp, layout adapts (canvas
      grows, rail/dock fixed). Buttons other than window controls are visual-only
      until their screens.
- [x] Screen 2 — accepted by the parent 2026-10-22. `tools/e2e.sh`: 16/16 tests
      (9 e2e) pass; headless screenshots vs gallery (title/ribbon/rail/canvas/
      dock/status/overall): default 0.88% · View 0.87% · Tools 0.77% · Reference
      Image 0.94% · Help 0.71% · File backstage 0.99% · after File→Escape 0.97% ·
      after File→File 0.97% — every region ≤ 4.52%, diffs are text/icon
      anti-aliasing only. Enlarged ribbon strips identical per tab; View's
      dimmed Previous confirmed by pixel brightness (101 vs Next 204, both
      builds). Interactions driven through the emulator, matching the original's
      behavior measured in the browser: File toggles the backstage; Escape,
      a canvas press and another tab close it; a press inside it keeps it open;
      File reopens after an outside close; every tab shows its body. The close
      rules moved from a subscription to a root `EventTap` widget so the real
      window and the emulator share one event path. Gate: fmt clean, 118 tests,
      clippy 0.
- [x] Screen 3 — accepted by the parent 2026-10-22. Seven new references
      captured from the live original (dock collapsed, symbol selected +
      preview, Text "ABC", pin off, pin toast, dock dragged to 400px) with
      their measured spec (`refs/ui-spec/chrome-spec-screen3.json`).
      `tools/e2e.sh`: 27/27 tests (16 e2e); headless vs gallery, dock region /
      overall: Areas 1.73/0.88% · Text 2.65/1.05% · Symbols 2.85/1.08% ·
      Calcs 0.93/0.74% · Layers 0.49/0.66% · collapsed 0.30/0.62% · symbol
      selected 3.38/1.19% · Text ABC 0.60/0.67% · pin toast 1.73/0.96% · pin
      off 1.73/0.88% · resized 0.70/0.90%; Screens 1-2 rows unchanged.
      Tab/button borders pixel-exact in every tab state at 312 and 400px;
      labels within ±1px (sub-pixel AA). Live exe PrintWindow capture ==
      headless screenshot (0.00% every region). Interactions driven in the
      emulator: each tab shows its body + Areas round trip; pin toggles with
      the toast expiring after 3.2s; collapse/expand keeps the panel; resize
      drag follows the pointer (1040 → 360px), clamps 320-400, fixes on
      release, double-click resets to 312; text range + search + pick; symbol
      preview, search, category select. External library: real exe loads
      `symbols.json` from the cwd and from beside the exe (462 symbols, 18
      categories, 43 favorites), shows an error state (no crash) when absent;
      nothing embedded. Gate: fmt clean, 129 tests, clippy 0.
      Known, not claimable here: (a) hover styling cannot appear in headless
      screenshots (`iced_test` screenshot() draws with Cursor::Unavailable) —
      the selected-symbol row and the collapsed expand button show hover in
      the references only; hover colors are set from the CSS values; (b) the
      rail's pointer highlight clears in the original when a symbol is picked
      (placement mode) — belongs to Screen 8; (c) S3-008 auto-hide after
      placement (Screens 8-9), S3-013 saving dock state in the sketch file
      (Screen 11), tooltips (Screen 12), and the Edit / Manage symbols /
      Calcs buttons' dialogs (Screens 7-9) are not yet wired.
## Reference-capture correction (2026-10-22): the canvas grid

Most gallery images were captured from a hidden browser tab, where the
original's requestAnimationFrame never fires — so its canvas showed blank
white instead of the grid. All 19 references now carry the original's own
forced canvas render (`refs/ui-spec/canvas/`, see its README) wherever the
canvas shows through. The native canvas (`sketch_canvas.rs`, port of
CanvasEngine.drawGrid, light theme, pan = startup canvas center, never
recentered) matches it at 0.00% in every state, live and headless; Screens
1-3 were re-verified against the corrected references with no other change.

- [x] Screen 4 — accepted by the parent 2026-10-22 (4a commit 3e20e78, 4b
      this commit).
      4a evidence: 13 new references captured from the live original
      (`refs/ui-spec/gallery/status-*.png`, spec
      `refs/ui-spec/chrome-spec-screen4.json`; the capture helper reproduces the
      corrected Home reference at 0.00%). `tools/e2e.sh`: 39/39 app tests
      (23 e2e); headless vs gallery, status / overall: default 4.27/0.88% ·
      zoom in 4.25/0.88 · zoom out 4.29/0.88 · slider 250% 4.26/0.88 · 1:20
      4.34/0.89 · grid off 4.33/0.88 (canvas 0.00) · snap off 4.34/0.88 ·
      area snap + 1/2' 4.30/0.88 · filter popover 4.27/1.11 · one filter
      unchecked 4.27/1.12 · zoom fence armed 4.27/0.88 · mid-drag 4.27/0.88 ·
      after fence 4.23/0.88. Canvas 0.00% at every zoom (120/83/250/429%) and
      for the dashed fence. Screen 1 HEAD build on the same reference: status
      4.32% — no regression (the scale-input and area-select text sit 1px
      right in both builds: pre-existing sub-pixel AA). Live exe == headless
      (0.00%). Interactions in the emulator, matching numbers taken from the
      original: +/- (120/83%), slider, 1:N entry (1:20 keeps 100%, junk
      ignored), Grid/Snap/Area-snap, pitch menu, Filter toggle from the status
      bar and View › Filter with remembered checks, View › Grid, totals chip
      opens Calcs even when collapsed, zoom fence (drag (300,300)→(500,450) →
      429%, pan 1228.657/803.743, back to Select, rail highlight follows the
      mode), Center / Center & Fit / C / Shift+C no-ops on an empty sketch
      (as the original), C typed into a search box stays text. Gate: fmt
      clean, 141 tests, clippy 0.
      Known, not claimable here: hover styling (headless); Chrome's native
      select popup is OS-drawn and uncapturable — the pitch menu uses the app
      palette; the 1:N input applies on Enter (iced has no blur event), the
      original also on blur; filters have nothing to hide until Screens 5-9.
      4b evidence: 9 more references from the live original (north prompt,
      north 90°, Next-page confirm, 2 of 2, 1 of 2, Underlay: Page 2, delete
      alert, delete confirm, after delete), captured with the canvas shown
      as an <img> overlay so modal backdrops/shadows composite over the real
      grid (method reproduces an accepted reference at 0.00%). `tools/e2e.sh`:
      48/48 app tests; status / overall: north prompt 2.23/0.45% · north 90°
      4.30/0.88 · next confirm 2.23/0.43 · 2 of 2 4.04/0.87 · 1 of 2
      4.04/0.87 · underlay 4.04/0.87 · delete alert 2.23/0.42 · delete
      confirm 1.66/0.39 · deleted 4.01/0.87; every earlier state unchanged
      (max region 4.51%, max overall 1.19%). Dialogs: box edges exact; title,
      message, input value, Cancel/OK caption rows exact in all four (iced
      rasterizes text rows at floor(line top), Chrome at round — corrected
      with measured draw nudges, `snap::nudge`); residual = glyph AA
      (DirectWrite vs iced's rasterizer). Right block laid out as Chrome's
      flex row: underlay select 107 → 115px with "Underlay: Page N", centre
      block re-centres; borders land on the reference's pixels. North arrow
      rotation and degree label match. Disabled defined-areas select (after
      any page load) matches the measured 0.7-opacity look. Live exe ==
      headless (0.00%). Driven in the emulator: north prompt (pre-selected,
      typed value replaces it, 450 → 90°, Cancel and junk keep it, Enter =
      OK), north kept per page, Next on the last page confirms then adds,
      Next to an existing page doesn't ask, Previous (inert on page 1),
      Add, Delete (alert on the last page, confirm otherwise), Escape /
      backdrop dismiss, a press inside keeps the dialog, underlay pick, View
      › Add / First Page / Last Page / Previous (enabled after page 1) /
      Delete / Next. Gate: fmt clean, 150 tests, clippy 0.
      Deferred with their screens: the segment HUD (S4-022) goes live with
      drawing (Screen 5); defined-area rows in the select (S4-021) with
      areas (Screen 7); underlay ghost drawing needs page content (Screen
      5+); Bluetooth (S4-030) and Touch Panel (S4-031) with their panels
      (Screen 12). Not claimable: hover; the prompt's selection shows
      Chrome's inactive-window grey (the only state a hidden tab can
      capture).
- [ ] Screen 5 — in progress. **5a (modes + navigation) verified 2026-10-22.**
      5b (mouse drawing, dimensions, snapping, closing into areas, HUD), 5c
      (keyboard entry), 5d (wall shapes, Draw Color + colour dialog, cycle
      dimensions, wall solids, units formatting) to follow.
      5a evidence: 6 references from the live original (Pan / Draw Exterior /
      Draw Interior active, Measure + its toast, Pan-tool drag, wheel zoom).
      Headless vs gallery, rail / canvas / overall: Pan 3.95/0.00/0.88% ·
      Draw Ext 3.92/0.00/0.88 · Draw Int 3.91/0.00/0.88 · Measure toast
      3.98/0.16/0.98 · pan drag 3.95/0.00/0.88 · wheel zoom 3.97/0.00/0.88
      (grid exact at zoom 1.089). Toast box fixed to the original's 41px
      (iced draws borders inside padding) — the screen-3 pin toast improved
      0.15 → 0.07% canvas. All 48 states within prior maxima (region ≤4.51%,
      overall ≤1.19%); live exe == headless (0.00%). Numbers from the
      original as unit tests: Pan drag (400,400)→(500,450) = pan (623.5,
      401.5); wheel up, up @ (300,300), down @ (600,400) = zoom 1.089, pan
      (573.3915, 366.0835), "+ 109%". Driven in the emulator: every rail
      mode tool sets its mode + highlight (accent border/fill, inherited
      icons white), Measure/Curve/Chain toasts, Text/Symbols/Define Area open
      their panels (even with the dock collapsed), Pan-tool drag (left drag
      in Select does not pan), keys E / I / F5 / Enter / Escape / G / D (curve
      mode toast) / P / S / T / F4, C only when not drawing, keys typed in a
      text box ignored, a dialog swallows keys. Not scriptable in iced_test
      (no wheel, middle button or held Space): their event → message mapping
      is unit-tested on the canvas program and the messages are driven
      through the app (`Harness::send`). Cursor policy (grab / grabbing /
      crosshair / hidden + painted 128px drawing pointer, extracted to
      assets/cursors) unit-tested; the painted pointer cannot appear in a
      headless screenshot (no cursor) — live-only. Gate: fmt clean, 160
      tests, clippy 0.
- [ ] Screen 5 — in progress. **5a (modes + navigation) accepted 2026-10-22.**
      **5b-1 (drawing walls with the mouse) verified 2026-10-22**; 5b-2 (closing
      a loop into an area, wall-join healing, the area rows/calcs) to follow,
      then 5c (keyboard entry), 5d (wall shapes, Draw Color, cycle dimensions,
      wall solids, units formatting).
      5b-1 evidence: 4 references from the live original (pen down, two walls,
      pen up after Escape, an interior chain), driven with in-page canvas
      pointer/click events. Headless vs gallery, canvas / status / overall:
      pen down 0.03/5.61/0.94% · two walls 0.05/5.23/0.96 · pen up 0.02/4.30/
      0.91 · interior 0.05/5.24/0.96. The wall geometry, colours and readout
      are pinned as unit tests from the original's own numbers: clicks
      (300,300) (500,300) (500,500) give walls (-11,3)->(-1,3)->(-1,-7) as
      exterior #2B6CB0 width 4, the HUD reads "10.0' / 95° / DOWN" then
      "8.0' / 285°", Escape lifts the pen but keeps the mode, a duplicate
      click adds nothing, a click within 16px of the start closes the chain,
      and Space toggles the last wall's dimension (or arms the next).
      Drawn: walls (round caps/joins, dash styles, bowed arcs), rotated
      dimension labels, the red preview line with its length, rise/run and
      live-angle guides, the dashed return guide, alignment lines through
      aligned endpoints, vertex dots, and the green snap cross.
      Status bar: the readout pill became live (length / angle / UP-DOWN with
      DOWN in green) and the original's `overflow-x: auto` behaviour is
      ported — once the readout wraps, a 10px native scrollbar appears and
      STAYS until the content fits the narrower bar (hysteresis, matched to
      live rects). The zoom slider click was re-probed for the new layout
      (439.95 = 250%; iced's hit geometry differs from a native range input by
      a fraction of a pixel). New state: `draw-*` x4 in e2e.sh; new e2e test
      `e2e_status_scrollbar_follows_the_drawing_readout`. All pre-existing
      states within their prior maxima (status bar <= 4.51%, overall <= 1.19%);
      the new drawing states are <= 5.61% (region gate 6%).
      **5b-2 (auto-close into an area) verified 2026-10-22** — the model and
      the behaviour, not yet the area's own UI:
      * Reference `draw-closed-area` (a 10x10 ft exterior loop closed at the
        start). Headless vs gallery: canvas 0.10% (wall edges differ by 1/255
        of antialiasing only — the same wall lands on the same columns in both),
        status 4.45%, overall 0.97%.
      * The live original after the close: 4 walls, one polygon
        `{type:'UND', name:'Undefined Area', area:100, 4 edges carrying the
        chain's line ids in order}`, pen up, mode still `draw-exterior`, HUD
        back to `0.0' | 0.0° | UP`, no toast, the areas select still
        "No defined areas", the Calcs panel still "No defined areas yet".
        Ours matches, asserted by `e2e_closing_a_loop_creates_an_undefined_area`
        and the unit test `an_exterior_loop_becomes_an_undefined_area`.
      * An UND area draws NOTHING in the original (drawPolygons returns before
        fill/outline for `type === 'UND'`), so the walls stay plain linework and
        auto-close needed no renderer change.
      * `sketch-core/src/model.rs` gained the persisted `Polygon` shape
        (`build_edges`, `vertex_keys`) with the original's JSON field names;
        a closed area's vertices stop attracting new walls
        (`closedAreaVertexKeys`), which is asserted.
      * Layer ids are STRINGS in the original ("layer-1", "layer-6"), not
        numbers — `SketchLine.layerId` / `Polygon.layerId` are now
        `Option<String>` and round-trip in tests.
      * DEFERRED to the Layers screen (8/9): auto-close also creates and
        activates a dedicated "Area" layer (observed live: layers
        `[["layer-1","Layer 1"],["layer-6","Area"]]`, active `layer-6`, and the
        area's layerId `layer-6`). Ours records the area with no layer yet.
        DEFERRED to screen 7: the area rows / totals in the Calcs panel.
      **5c (typed wall entry + the arrow-key cursor) verified 2026-10-22** —
      S5-016, S5-018, S5-024 to S5-038, S5-050, S5-057, S5-060, S5-061.
      3 references from the live original, driven with real KeyboardEvents
      (draw-typed-buffer, draw-keyboard-preview, draw-typed-wall). Headless vs
      gallery, canvas / status / overall: typed buffer 0.00/5.00/0.90% ·
      keyboard preview 0.32/5.63/1.12 · typed wall 0.10/4.97/0.98.
      * Typed entry, pinned to the original's own numbers: clicking (300,300)
        drops the first point at (-11,3); typing "20" echoes `20'` into the
        readout (angle untouched); Enter commits a 20 ft wall to (-11,23) as
        exterior #2B6CB0 width 4 and leaves "0.0' | 90° | DOWN" (drawLine sets
        the angle; the mouse path deliberately clears it); a second Enter with
        an empty buffer lifts the pen. "10<45" draws at an absolute angle,
        "10x5" is RISE x RUN (the first number is the rise), "10r"/"10l" turn
        relative to the last heading (the first wall points up), and "10" then
        two ArrowUps move +10 then +1 (the buffer drives the first step) to
        (-11,14) with the HUD reading "11.0' | 90°" — the keyboard path shows
        the MATH angle where the mouse path shows a compass bearing, an
        inconsistency in the original that is reproduced deliberately.
      * Buffer rules from InputController: digits and '.', 'x' only after a
        digit, l/r only when the buffer is all numeric, '<' or ',' only after
        a digit, Tab inserts '.', Backspace removes one. Arrow steps follow
        options.stepUnit (whole 1 / half 0.5 / tenths 0.1, Shift 0.1);
        PgUp/Home/End/PgDn walk 45-degree diagonals at distance/sqrt(2);
        Ctrl and Ctrl+Shift arrows snap to alignment and intersecting
        alignment; J jumps to the nearest endpoint; A / Ctrl+F3 auto-close;
        cleanCoord rounds to 1e-4. `parseFloat` is ported (longest numeric
        prefix) so "10." and "1.2.3" behave.
      * Two bugs found by these tests: the turn direction was inverted ("10r"
        turned left), and I had assumed "10x5" was run x rise.
      * The keyboard pointer (drawKeyboardPointer) and the dashed preview are
        implemented; the 128px pointer image is NOT painted by the headless
        renderer from a canvas `draw_image`, so it is absent from our
        screenshots (all of the keyboard-preview canvas diff is that image:
        267 black px over a 128x128 box). Same class of limit as the mouse
        pointer; live-only.
      * Still open in 5c: the keyboard pointer's live check, and 5d (wall
        shapes, Draw Color, cycle dimensions, wall solids, units formatting).
      **5d-1 (cycle dimensions / placement / visibility) verified 2026-10-22** —
      S5-002, S5-063, S5-064.
      * `View › Apply (Dims)` cycles outside -> inside -> hidden -> outside,
        toasting "Dimensions: Outside" / "Inside" / "Hidden" (no trailing
        period) — transcribed from the live original, whose state reads
        `[true,'out'] -> [true,'in'] -> [false,'in'] -> [true,'out']` and whose
        toast text matches character for character. Asserted by
        `e2e_dimension_cycle_and_placement` (the toast through `expect`, the
        state through `into_state`), and hiding keeps the placement, as the
        original does.
      * `dimensionPlacement: 'in'` flips every label to the path's inside:
        `path_dimension_side` mirrors its result and the no-polygon fallback
        becomes -1, with unit tests on both sides.
      * Dimension visibility now follows `isDimensionVisible` (hidden flag,
        the Dimensions filter, `dimensionsEnabled` unless the wall is live,
        distance >= 0.1; the minimum-dimension option belongs to screen 12).
      * Two new states (`dims-inside`, `dims-hidden`) are in the matrix with
        their references, driven through the message the ribbon button sends:
        the references were captured with the original's HOME tab still
        showing (its tab strip is not hit-testable from the page — a zero-size
        `#tab-view`, and `elementFromPoint` returns BODY), so the ribbon is
        left alone and the states compare cleanly. Headless vs gallery,
        canvas / ribbon / status / overall: inside 0.08/2.35/4.30/0.95% ·
        hidden 0.06/2.35/4.30/0.94 (every region inside the 6% gate).
      **5d-2 (wall shapes) verified 2026-10-22** — S5-011 to S5-015.
      * The caret inside Draw Exterior / Interior (page 15..24, 305..314)
        opens `showCommandPopup`'s menu: 190x128 at (8, 317) for the exterior
        button (anchor bottom + 2), 5px padding, 1px border, shadow
        `0 8px 22px rgba(0,0,0,.42)`, 29.5px rows with a 16px icon and the
        label 9px after it, hover `--bg-hover`. Rows Line / Rectangle /
        Circle / Polygon; Line just sets the mode (the caret does NOT switch
        tools), and a press outside dismisses the menu. New reference states
        `wall-shape-menu`, `wall-shape-width-prompt`, `wall-shape-rectangle`;
        headless vs gallery, canvas / rail / status / overall: menu
        0.24/4.73/4.30/1.05% · prompt 0.10/2.23/2.26/0.42 · rectangle
        0.04/-/4.45/0.93 (all regions inside the gate).
      * The prompt chain matches `showWallShapeMenu`: Rectangle asks "Width in
        feet" (20) then "Height in feet" (20) and adds; Circle asks "Diameter
        in feet" once (its height is the diameter); Polygon asks Width ->
        Height -> "Number of sides" (8). A falsy or unparsable entry abandons
        the shape, as `if (!width) return` does. Unit-tested in `shapes.rs`
        and driven end to end (`e2e_wall_shape_menu_and_prompt_chain`).
      * `addWallShape` (ported earlier as `Drawing::add_wall_shape`) then adds
        the shape centred on the canvas: the live original's Rectangle at the
        default 20x20 gives exactly the four walls
        (-10,-10)->(10,-10)->(10,10)->(-10,10)->(-10,-10) plus an UND area of
        400 sf, which is what ours produces.
      * Two bugs found by the tests: the chain asked for sides after a
        RECTANGLE (only a polygon does), and the caret's mouse area wrapped a
        pinned child — a pin's bounds are the whole parent, so it swallowed
        every press on the rail button underneath.
      * The RIGHT-CLICK trigger is done too: a right press anywhere is
        reported by the root EventTap and hit-tested against the two draw
        buttons, so `contextmenu` opens the same menu anchored under that
        button (the exterior menu at (8, 317), the interior at (8, 351)).
        Asserted for both buttons. NOTE: iced reports a right press with an
        UNAVAILABLE cursor, so the tap keeps the last move position and falls
        back to it; the e2e must `move` before `press right` for the same
        reason.
      * NOT DONE: the menus for the rail's other tools (Shapes / Geo / Pages,
        screens 9 / 10).
      **5d-3 (Draw Color and the Color dialog) verified 2026-10-23** — S5-001,
      S5-004 to S5-007.
      * `Home > Draw Color` (`draw-color` -> `toggleDrawColorPalette`) opens
        the dialog on `state.drawColor`; OK makes that colour `state.drawColor`
        and shows the original's toast "Drawing color updated.". The wall
        colour is `lineStyle.color || state.drawColor` (sketch-app.js:2717)
        and the Options colours default to `#2B6CB0` / `#63B3ED`
        (state-init.js:102), so Draw Color is a FALLBACK that shows through
        once an option colour is blanked — implemented that way
        (`Drawing::line_style`), not as a replacement.
      * Geometry, measured from the live DOM: card 340 wide, 493 tall on
        Standard / 441 on Custom, centred; header 43 with `Color` (Segoe UI
        16/600) and a 22.52x22 x at x 302.48; tabs 169x32 (Arial 12, active
        `--bg-hover` + 600); wheel canvas 202x202 at (30, 91) — a 200x200
        bitmap inside a 1px border, hexagon radius 98, v=1; side column 66
        wide with "New" (Segoe UI 10 muted) over a 52x32 preview at (266,
        109); 48 presets at (15, 303) as 8 columns of 35.25x18, gap 4;
        Custom's SV square 200x200 at (15, 91) and hue strip 24x200 at
        (227, 91); RGB inputs 60x22 at x 52.11, pitch 28; footer border at
        card y 445, Cancel 71.36x26 (transparent) and OK 51.34x26
        (`--bg-active` + white) at y 456.
      * Three iced findings, each proved by the pixel diff: a container's
        border is drawn INSIDE its box, so the surfaces had to BE the border
        boxes (image inset by 1) and the original's coordinates measure from
        that border box; `image()` letterboxes by default
        (`ContentFit::Contain`) and rendered the hue strip 183px tall in its
        198px box, so `content_fit(Fill)` (2716 -> 120 differing px); and
        `text_input`'s intrinsic height excludes its border, while the
        half-pixel card put every text run 1px high — each was pinned to the
        reference's measured line box.
      * References `color-dialog-standard`, `color-dialog-custom`,
        `color-dialog-preset`: canvas 0.10% / 0.19% / 0.10%, overall 0.43% /
        0.49% / 0.43% (gate 6%). What remains inside the dialog is glyph
        antialiasing and the 10px cursor ring's AA.
      * Tests: 12 unit (the original's colour math and rounding, the presets
        table, open/OK/Cancel, tab heights, wheel/SV/hue picking, drag gating,
        RGB clamping, the wheel and hue pixel buffers) and 5 e2e (opens on the
        draw colour; tabs switch and the card resizes; preset + OK sets the
        colour and toasts; Cancel and a backdrop press change nothing; a wheel
        press sets hue/sat with v=1).
      **5d-4 (units / precision formatting) verified 2026-10-23** — S5-062.
      * New `sketch-app/src/format.rs`: `formatLength`, `formatFeetInches`,
        `formatLengthDual` and `formatArea` (sketch-app.js:541-576) over the
        Options (metric/imperial, `decimalPrecision`, `stripTrailingZeroes`,
        `areaLabelSuffix`), defaulting to the original's `state-init.js:89`
        values. Every expected string in its 11 tests was read off the live
        original by calling `app.formatLength(..)` etc., so the tests encode
        observed behaviour, including `toFixed` ties (precision 1 renders
        12.5 as "13"), `formatFeetInches(0.04)` = "0'" against 0.05 = `0' 1"`,
        and the metric/stripped variants.
      * The dimension and rise/run labels now go through it
        (`Scene::drawing.units`), and `Message::Units` sets the options (the
        Options dialog on screen 12 will drive the same message).
      * The HUD pill deliberately does NOT: `UIManager.updateHUD` takes a raw
        number string and appends "'" itself (ui-manager.js:2866), so the live
        readout ignores the unit system — confirmed on the live original,
        which still shows "10.0'" with metric options. The quirk is preserved.
      * A `draw-metric-label` reference was captured (same two walls, metric
        options, live original): canvas region 0.28% against our metric
        render. NOTE its STATUS region reads 6.24%, which is a capture
        artifact, not a defect: the reference was recaptured through an
        emulated 1400x860 viewport after the browser daemon restarted, and a
        fresh IMPERIAL capture of the unchanged state shows the same 2.43%
        status drift against the accepted reference. The status bar's text is
        provably unit-independent (`.status-center` innerText is identical in
        both unit modes), so the metric state's evidence is its canvas.
      **5d-5 (wall solids) verified 2026-10-23** — S5-054.
      * `sketch-core/src/solids.rs` ports `computeWallSolids`
        (area-breakdown.js:858) with the junction classification behind it:
        `Free` (no extension), `T` (the endpoint lands inside another wall's
        span, stopping at its near face by `-(host/2)/sin(angle)` clamped at
        0.25), `Inline` (collinear continuation) and `Corner` (L/X, extending
        over the largest mate half thickness). Hosted openings cut gaps
        (CENTER spans the middle, SIDE runs from the hinge and mirrors with
        Flip Swing), and each wall also yields its two ±half side faces.
      * PARITY: the piece lists were read off the live original's
        `app.getWallSolids()` for the free / corner / T / inline cases and are
        asserted verbatim in `matches_the_live_originals_solids_exactly`.
        One asymmetry is reproduced deliberately: the original computes
        `solidEnd = length + Math.max(0, endExtension)`, so a T shortens a
        wall at its START but not at its END — the test pins both directions.
      * `wallThicknessOf`: a per-line `thickness` in inches wins over
        `options.exteriorThickness` / `interiorThickness` (6in / 4in by
        default); `wallSolidsEnabled` needs the option plus a positive
        thickness. `Options` now carries the units, the toggle and both
        thicknesses (`Message::WallSolids`, `Message::Units` set them).
      * Rendering: filled quads per piece, interior pass then exterior (the
        original's order), then the stroke pass skips any line with a solid —
        but a line still in the live chain keeps its stroke preview, exactly
        as `drawWallSolids` does with `penDown && currentLineIds`.
      * Reference `draw-wall-solids` (two walls, 6in exterior, pen up):
        canvas 0.02%, overall 0.91% — the best canvas figure in the suite.
      * NOT DONE (deferred, stated): bowed walls still stroke instead of
        filling — `drawCurvedWallSolid`'s annular sector is not ported yet —
        and `offsetRing` (the measureTo interior/exterior face ring) belongs
        to the area work on screen 7.
      **5d-6 (Shift+wheel bowing) verified 2026-10-23** — S5-040.
      * `Drawing::bow_last_wall(delta, height_mode)` reproduces
        CanvasEngine's wheel handler: in 'degree' mode the bow advances by
        the step and the arc height follows
        (`chord/2 * tan(bow*PI/720)`); in 'height' mode the rise advances and
        the angle follows (`4*atan2(2h, chord)`). Both clamp at +-359.
      * Steps: 15 degrees, Ctrl 1; in height mode 1 ft, Ctrl 0.1 ft. The
        numbers in `bow_tests` are the live original's own response to
        dispatched `wheel` events with `shiftKey` on a 10 ft chord
        (15 / 30 / 15 / 16 degrees with arcs 0.327717 / 0.658262 / 0.327717 /
        0.349634, and height mode 45.23973 / 49.629674 / 4.583051 degrees).
      * Only while drawing and only with a wall to bow; a plain wheel still
        zooms. iced's wheel event carries no modifiers, so the canvas widget
        remembers the last `ModifiersChanged` and publishes `shift`/`ctrl`
        with `Message::Wheel` — the same state the browser gets from
        `e.shiftKey` / `e.ctrlKey`.
      * Reference `draw-bowed-wall` (one wall, two Shift+wheel ups = 30
        degrees, pen up): canvas 0.05%, overall 0.93%.
      **5d-7 (curved wall solids) verified 2026-10-23** — closes the S5-054
      gap noted in 5d-5.
      * `render::curve_metrics` is `getCurveMetrics` in world coordinates
        (radius `(chord/2)/sin(theta/2)`, rise `chord/2*tan(theta/4)`, apex
        and centre offset along the chord normal), and `draw_curved_solid` is
        `drawCurvedWallSolid`: the annular sector between `radius -+ t/2`,
        swept from the start to the end angle the way that passes the apex,
        with `max(8, ceil(|sweep|/(PI/24)))` steps. Hosted openings are not
        cut in bowed walls — the original's stated v1 limitation.
      * The live original confirms the branch this fills: with wall solids on
        and a 30-degree bow, `getWallSolids().solids` is EMPTY (the straight
        list skips bowed walls) while `wallThicknessOf` is 0.5 ft, so the
        sector is the only thing that draws the wall. Ours had been skipping
        the stroke without filling anything, i.e. drawing no wall at all.
      * Reference `draw-bowed-solid` (one wall bowed 30 degrees, 6in
        exterior, pen up): canvas 0.01%, overall 0.91% — the closest match in
        the suite.
      * `curve_tests` pins the circle: radius 19.318516, apex 3.658270 and
        centre -15.660254 for a 10 ft chord at +30 degrees, mirrored for -30,
        and no metrics for a straight wall.
      **5d-8 (Define First) verified 2026-10-23** — S5-056.
      * `options.defineFirst` blocks exterior drawing while no area type is
        chosen: the click does not put the pen down and the original's notice
        appears verbatim — "Define First is enabled. Choose an area type
        before drawing." (sketch-app.js:2688). Interior drawing is not gated.
      * A closed loop then takes `state.selectedAreaType` as its `type` and
        `selectedAreaName || selectedAreaType` as its name instead of
        UND / "Undefined Area" (sketch-app.js:2879). `Message::DefineFirst`
        and `Message::AreaType(kind, name)` set both; the dock's area panel
        (screen 7) will be their real driver.
      * Verified on the live original by dispatching the same canvas click:
        penDown false, 0 lines, the toast visible with that exact text, and
        penDown true once `selectedAreaType = 'GLA'`. Ours behaves the same
        (`e2e_define_first_blocks_exterior_drawing_until_a_type_is_chosen`,
        `e2e_define_first_names_the_closed_area`).
      **5e-1 (wall-join graph primitives) — FOUNDATION, not S5-048 complete**
      * `sketch-core/src/joins.rs` ports the four primitives
        `healExteriorWallJoins` and `closeJoinedExteriorLoops` were missing:
        `segment_cross_point` (a proper crossing — a shared corner and an
        extrapolated intersection are both rejected, which the original's
        comment says prevents "phantom walls / truncates real ones"),
        `point_key` (5-decimal identity), `line_passes_through` /
        `exit_key_from`, and `walk_closed_loop` (the recursive graph walk that
        prefers the straightest continuation and may pass straight through a
        T-junction on a wall's body), plus `MIN_CLOSED_AREA = 0.01`.
      * Verified against the live original's own functions: `segmentCrossPoint`
        returns [5,0] for a proper crossing and null for the corner,
        extrapolated and parallel cases; `pointKey(-11,3)` is
        "-11.00000,3.00000"; `minimumClosedArea()` is 0.01. Ten unit tests
        cover those plus the walk finding a closed square (4 edges, real ids)
        and returning nothing for an open chain.
      * NOT YET DONE — S5-048 remains open: assembling `heal_exterior_wall_joins`
        itself (welds to endpoints and wall bodies, crossing splits, dangling
        sub-edge pruning that never trims an existing area's vertex, then
        committing the endpoint moves) and the `closeJoinedExteriorLoops`
        polygon materialisation on top of `find_enclosed_faces`. The weld step
        is deliberately NOT shipped alone: the original only commits a weld
        when it completes a closed loop, so a standalone weld would change
        geometry the original leaves alone.
      **5e-2 (wall joins: the heal itself) verified 2026-10-23** — S5-048's
      healing half.
      * `joins::heal_exterior_wall_joins` transcribes `healExteriorWallJoins`:
        welds (a free end onto a nearby endpoint, canonicalized onto the older
        wall, or onto a wall body via `projectPointToSegment`), the
        `alreadyJoined` guard for ends that already cross another wall,
        crossing splits on both walls, the virtual node/sub-edge graph, the
        dangling-piece prune (which never trims a vertex protected by an
        intact area), the closed-loop search through `walk_closed_loop`, and
        the commit that moves endpoints and follows any area vertex sitting on
        them. Weld tolerance is `getScreenToleranceWorld(16)` = 16/(20*zoom)
        feet: 0.8 ft at 100%, 0.4 ft at 200%.
      * PARITY on the live original: with a square whose fourth wall ends
        0.1 ft short, `app.healExteriorWallJoins()` moves that end to exactly
        (0,0) and ours does the same; with a 3 ft gap the original leaves it
        at (0,3) and so do we. Its own tolerance at zoom 1 reads 0.8 ft,
        which is what `16/(PIXELS_PER_FOOT*zoom)` gives.
      * Wired into the click path (`finishCommittedWall` ->
        `closeJoinedExteriorLoops`), after the wall commits. All 116 app tests
        still pass, so the existing drawing references are undisturbed.
      * STILL OPEN in S5-048: `closeJoinedExteriorLoops`' polygon
        materialisation — creating a NEW area from a healed loop via
        `find_enclosed_faces`, the repairable-polygon pass and
        `splitWallsAtJunctions`. Auto-close of the drawn chain (5b-2) still
        covers the common case.
      **5e-3 (wall joins: areas from healed loops) verified 2026-10-23** —
      S5-048 complete.
      * `joins::close_joined_exterior_loops` transcribes the rest of
        `closeJoinedExteriorLoops`: heal first, then `find_enclosed_faces`
        over the exterior walls, materialise each face's edges (with the bow
        angle signed by whether the face follows or reverses the wall),
        discard faces that are not whole live walls, skip signatures an intact
        area already covers or whose walls are all claimed, drop composites
        that wrap or subdivide an intact area and larger rings that reuse a
        smaller candidate's boundary, then either REPAIR a repairable polygon
        with equal rings (preserving its identity) or push a new area
        (`UND`/"Undefined Area", or the Define First type/name) with the
        original's `subtractMode`.
      * PARITY on the live original: four separate walls (not a chain) give
        `created: 1`, one area `type UND`, name "Undefined Area", area 100,
        edges [1,2,3,4]; a second run creates 0 and the count stays 1. Ours
        produces exactly that (`four_separate_walls_close_into_one_undefined_area`,
        `an_existing_area_is_not_duplicated`, `define_first_names_the_materialised_area`,
        `an_open_shape_creates_no_area`).
      * Wired into the click path in place of the heal-only call. A chain
        closed by auto-close is not duplicated: its polygon is intact, so its
        signature is already in `existing`.
      * NOT ported, both inert: `splitWallsAtJunctions` (undefined in the
        original's defaults, i.e. off) and the dedicated-layer assignment
        (layers are a later screen).

## Screen 6 — selection

      **6a (selection model, hit testing, click select) verified 2026-10-23**
      — S6-035, S6-043, S6-047, S6-065 (partly), S6-079's foundation.
      * New `sketch-app/src/selection.rs`: `Selection` (selectedLineIds,
        selectedPolyIds, activeLineEndpoint), `hit_test_at` in the original's
        order — a 4 px "on the wall" pass that wins over everything, then the
        9 px wall pass, then polygon interiors — `select_at` (clear unless
        additive, toggle when additive, nearest end becomes the active
        handle), `select_all`, and `distance_to_wall` (chord for a straight
        wall, the flattened arc for a bowed one).
      * Verified against the live original: its `hitTestAt` at (-6,3) on a
        wall, (-6,3.1) just off it, (-6,5) far away and at the shared corner
        return `line:1`, `line:1`, `null`, `line:1`; `selectLineAt` gives
        `selectedLineIds = [1]` with `activeLineEndpoint.endpoint = 'start'`
        (the click is equidistant, and the original's `<=` picks start); its
        tolerances are 0.2 ft (4 px) and 0.45 ft (9 px) at 20 px/ft, zoom 1 —
        the numbers `selection::tests` pin.
      * Rendering: a selected wall takes `colors.selectLine` (#C05AFA) at
        `max(5, base + 1)` px; a selected area fills `rgba(43,108,176,.08)`
        with a #2B6CB0 outline (an UND area is invisible until selected).
        Reference `select-wall`: canvas 0.20%.
      * Keys: Ctrl+A selects all; Escape now lifts the pen, else clears the
        selection, else picks the Select tool (S6-047).
      * KNOWN GAP shown by that reference: its RIBBON region reads 7.68%
        because the original swaps in the Edit Tools panel (Length / Set
        Length / Line Style / Line Color / Curve) while a line is selected —
        S6-030 to S6-034, not built yet. The canvas is the evidence here.
      **6b (the contextual Edit Tools ribbon) verified 2026-10-23** — S6-005's
      tab, and the layout for S6-001..003, 013..015, 030..034.
      * The Edit Tools tab appears at the end of the tab strip while anything
        is selected and is `.active` — measured `#F8FAFC` with `#171717` text
        at x 413.92, 83.81 wide. Selecting swaps the ribbon to it; clearing
        the selection (Escape) hands the previous tab back.
      * Body measured on the live original: 8 base buttons at y 68.41 (h
        49.19, 22px icons at y 73.41, 11px captions at y 99.41) — Cut, Copy,
        Paste, Select All, Select, Delete, Undo, Redo — the divider at
        x 410.25, then the line group at x 418.25: the Length field (label
        "Length · Start handle" at y 74.5, input 54x23 at y 90.5),
        Set Length, the Line Style select (92x25 at y 89.5), the Line Color
        swatch (32x24 at y 72) and Curve.
      * `reference select-wall`: ribbon 7.68% -> **3.35%**, canvas 0.08%,
        overall 1.06% — the ribbon is now inside the gate and in line with the
        Home tab's own 2.14-2.35%.
      * `intersect` was the one Phosphor glyph missing from `assets/icons`;
        it was fetched from `@phosphor-icons/core` (MIT, the set the original
        renders through its `ph` font) and registered in the icon table.
      * NOT DONE in this step: the fields are visual only — the Length input,
        Set Length, Line Style, Line Color and Curve do not yet act on the
      * NOT DONE: the Edit Tools tab does not switch away if the user clicks
        another tab while the selection persists.

      **6b-2 (the Edit Tools panel acts) verified 2026-10-23** — S6-030 to
      S6-034.
      * `Drawing::set_selected_line_length` is `setSelectedLineLength`: the
        ACTIVE handle (the one the click picked, else the end) moves along the
        wall's own direction so the wall becomes the entered length, and any
        area vertex sitting on the moved endpoint follows. Verified against
        the live original on a 10 ft wall from (-11,3) to (-1,3): with the
        start handle active, length 20 moves the start to (-21,3); with the
        end handle active it moves the end to (9,3); 0, -5 and junk are
        rejected (false). `edit_tools_tests` pins all of it.
      * `Drawing::set_selected_line_appearance` applies `lineStyle` / `color`
        to every selected wall (the original's `setSelectedLineAppearance`),
        with "solid" stored as the absence of a style.
      * The panel shows live values (`updateAreaActionStates`): the length to
        2dp, the label naming the active handle ("Length · Start handle" /
        "Length · End handle"), the style and the wall's colour. The length is
        a real text field whose Enter applies it; Set Length applies the field;
        Curve switches to the curve tool.
      * Line Style cycles solid -> dashed -> dotted on click and Line Color
        opens the Color dialog for the selected walls (`ColorTarget`), because
        the original uses native `<select>`/`<input type=color>` popups this
        port has no equivalent for — a stated stand-in, not a silent one.
      * `reference select-wall` holds at ribbon 3.35%, canvas 0.08%, overall
        1.06% with the live values in place.
      **6c (undo / redo) verified 2026-10-23** — S6-014, S6-015, S6-022,
      S6-040 to S6-042, and S6-021's empty-stack notice.
      * New `sketch-app/src/undo.rs` transcribes `snapshotModel` / `pushUndo`
        / `undo` / `redo`: a labelled snapshot of lines, polygons, the cursor
        and the live drawing chain; `pushUndo` clears the redo history; the
        redo entry carries the label of the action being undone, which is what
        the history list shows. `Drawing::snapshot` / `restore` implement
        `restoreDrawingChain`'s deliberate conservatism — the chain is only
        restored while it is still being drawn, because restoring a finished
        chain's pen-down state used to leave an invisible start point behind.
      * Undo/redo drop the selection, as the original's `clearSelection` after
        every step does. Banked before: wall commits ("Draw"), bowing ("Bow"),
        Set Length, and wall appearance changes.
      * Home and Edit Tools › Undo/Redo now enable from the stacks
        (`updateUndoRedoControls`) instead of being permanently disabled, and
        run `undo`/`redo`; Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y are bound.
        `undo-history` with an empty stack shows the original's notice
        "Nothing to undo yet."; with entries it names them in a toast until
        the popup list lands.
      * Tests: 4 unit (stack walk, push clearing redo, labels newest-first,
        the redo label) and 2 e2e (draw two walls, Undo leaves one, Redo
        restores two; the empty-stack notice).

      **6c-2 (the undo history popup) verified 2026-10-23** — S6-021.
      * New `sketch-app/src/undo_history.rs`: `showUndoHistory`'s command
        popup — the same component the wall-shape menu uses, anchored 2px
        under the QAT's history button, entries newest first and numbered
        "N. <label>" with the `arrow-u-up-left` icon at the popup's own 11.5px
        label size (`.command-popup.undo-history-popup`). Choosing row i
        rewinds to stack index `len-1-i` via `UndoStack::undo_to`, which
        transcribes `undoTo`'s `while (undoStack.length > target) undo()`
        loop; the popup closes on choice.
      * The QAT's Save / Undo / Redo / Undo History / Print / Zoom buttons are
        live now instead of drawn-only: Undo and Redo follow the stacks
        (`updateUndoRedoControls`, dimmed at `over(TEXT, 0.38)` when empty),
        and the history button opens the popup — always enabled, because an
        empty stack reports "Nothing to undo yet." itself.
      * Noted while testing: three drawing clicks bank `[empty, empty, one
        wall]`, because `pushUndo('Draw')` runs on every click and the first
        click only puts the pen down. That is the original's own behaviour,
        not an off-by-one; the e2e pins it.
      * Tests: 3 unit (closed popup, empty stack, row-to-index mapping and
        numbering) and 1 e2e (open with three steps, rewind to index 2 -> one
        wall and two entries left, popup closed).

      **6d (clipboard and delete) verified 2026-10-23** — S6-030 to S6-034.
      * New `sketch-app/src/clipboard.rs` transcribes `copySelected` /
      `paste` / `deleteSelected`:
        - Copy takes the selected walls and areas *plus the boundary walls of
          any selected area*, so a copied area keeps its outline.
        - Paste offsets by 2 ft (the original's `const offset = 2`), mints new
          ids, remaps each area's edges onto the copied walls, and rebuilds
          them via `build_edges` when a wall was not copied. The copies come
          back selected. `copySelected` does not bank undo; paste and delete
          do ("Paste" / "Delete").
        - Delete follows the original's rules: an explicitly selected wall
          always goes (deleting a boundary wall reopens its area); an area
          goes if selected, if one of its own walls is deleted, or if it runs
          along a deleted wall; a wall a surviving area still uses is kept;
          and an area is never removed merely because a *deleted area's*
          outline coincides with it — the "second floor drawn over the first
          floor" bug the original's comment records.
        - `promptOnDeletion` defaults to false (refs/state-init.js:88), so
          Delete never asks first; the confirm dialog belongs to Options
          (screen 12).
      * Verified against the live original with the same fixtures: copy ->
      {lines 4, polys 1}; paste -> 8 lines, 2 polys, first point (2,2), new
      edges pointing at the copied walls (9-12), 4 lines + 1 poly selected;
      delete wall 1 -> 3 lines, 0 polys; delete area 100 with area 101 sharing
      wall 1 -> polys [101], lines [1]; delete with nothing selected ->
      unchanged.
      * Edit Tools' Cut / Copy / Paste / Select All / Delete now run the
      commands, and Ctrl+C, Ctrl+X, Ctrl+V, Delete and Backspace are bound
      (Ctrl+A already was).
      * Tests: 5 unit (copy takes the outline; paste offset/ids/selection;
      delete a boundary wall; delete an area sharing a wall; delete nothing)
      and 2 e2e (copy+paste through the panel; delete then undo).

      **6e (select by type) verified 2026-10-23** — S6-035.
      * New `sketch-app/src/select_menu.rs` + `selection::select_by_type`:
        Edit Tools › Select opens the original's six-entry command popup
        (Select All / Areas / Text / Dimensions / Symbols / Freeform Lines,
        in that order, with `selection-all`, `polygon`, `text-t`, `ruler`,
        `couch`, `line-segment` icons), anchored 2px under the button at
        (213.25, 68.41) 46x49.19.
      * `selectByType` clears the selection first, then: All → everything
        (the original's `else` fallback), Areas → polygons, Freeform →
        `type === 'interior'` walls. Text, Dimensions and Symbols select
        nothing because this port has no text, symbol or dimension-selection
        set yet — the original's `selectedAreaLabelIds`/`selectedDimensionIds`
        have no counterpart here, and the menu entry clears rather than
        pretends.
      * **Command-popup row spec corrected (applies to 5d-2, 6c-2 and 6e).**
        Measured against the live original: rows are 29px (not 29.5) with
        13.33px Arial labels `#ccc` and 13.34px icons (not 12px/16px), 7px/10px
        padding and a 9px gap. The panel is 186 tall for six rows — iced draws
        the 1px border INSIDE the box, so the CSS height needs the borders
        added. A `pin`ed row stack also renders 7px above its container's
        content box, so `shapes::ROW_Y0 = 12` calibrates the first row onto the
        reference pixels. Verified: the select menu's text bands now sit at
        134/163/192/221/250/279 with borders at 120/305 (reference
        134/162/193/220/251/279, borders 120/305), and the wall-shape menu's
        at 332/361/389/418 with borders 317/444 (reference identical ±1px).
        The wall-shape menu's own numbers improved as a result: overall
        1.05% → 0.97%, left rail 4.52% → 4.06%, canvas 0.25% → 0.13%.
      * `reference select-menu`: title 2.29%, ribbon 3.32%, rail 3.97%,
        canvas 0.57%, dock 1.73%, status 4.30%, overall **1.36%** — inside the
        6% region gate.
      * Tests: 3 unit (the six labels in order and the 186px height; closed
        menu; open menu), 3 unit for the selection rules, 1 e2e (menu opens
        from the button, a row closes it, Areas clears with no areas, Freeform
        selects exactly the interior wall).

      **6f (canvas context menu) verified 2026-10-24** — S6-036, S6-037.
      * New `sketch-app/src/context_menu.rs`: `showCanvasContextMenu`'s
        190x434 panel — 13 rows of 29px and five 1px `#3F3F46` separators
        carrying 4px margins, 5px padding, 1px border, opened 2px under the
        cursor and clamped 8px inside the viewport (the original's
        `Math.min(innerWidth - w - 8, Math.max(8, left))`).
      * Verified pixel-exact against the reference captured from the live
        original with the panel at y=302: every border and rule row matches
        (302, 341, 466, 533, 629, 696, 735) and every button's text band
        matches within 1px. `reference context-menu`: canvas 0.70%, overall
        **1.44%**.
      * Right-click on the canvas now does what the original's `contextmenu`
        handler does: ends the pen (`penUp`), hit-tests the point and selects
        the item only when it is not already selected (`isHitSelected`), then
        opens the menu. The rail's draw-tool right-click still opens the
        wall-shape menu first; a measure run in progress is left alone.
      * Wired: Delete, Cut, Copy, Paste (the four clipboard actions, which
        run the same handlers as the Edit Tools buttons). Deferred with
        reason: Make Negative and Auto-Subtract need the area model, Send to
        Back / Bring to Front are `autoAssignLayer` and need layers, Flip
        Horz / Vert and Rotate 90° need the transform screen, Clone Area needs
        `cloneSelected`'s symbol handling, Properties… needs the area
        properties dialog. `Action::is_wired` marks the four that act.
      * A 1px pinned rule lands 6px below a row pinned at the same `y`, so
        `SEP_ORIGIN_DELTA` converts the separators out of the row space — the
        same class of iced pin-origin quirk `ROW_Y0` documents.
      * Six missing Phosphor glyphs fetched from `@phosphor-icons/core`
        (trash, arrow-line-down, arrow-line-up, copy-simple, sliders) and
        embedded.
      * Tests: 4 unit (the 190x434 panel; every button and separator row
        against the measured offsets; the thirteen labels and the four wired
        actions; the clamped popup).

      **6g (rotate and flip) verified 2026-10-24** — S6-038, S6-039.
      * New `sketch-app/src/transform.rs` transcribes `rotateSelected`,
        `flipSelectedAreas`, `rotateSketch` and `flipSketch`. Two ideas carry
        it: selection transforms work on a **point-key set** (five decimal
        places) rather than ids, so any point in the sketch sharing a selected
        endpoint moves with it and shared corners stay welded; and a
        reflection negates every signed bow angle (`bowAngle`, `arcHeight`,
        each area edge's `bowAngle`), because otherwise a curved wall's arc
        jumps to the wrong side of its chord.
      * Verified against the live original on the 10x10 square with north at
        30 degrees, all six cases:
        - `rotateSketch(90)` → lines (10,0→10,10), (10,10→0,10), (0,10→0,0),
          (0,0→10,0), north **120**.
        - `flipSketch('horizontal')` → the x-mirror, north **330**
          (`360 - a`).
        - `flipSketch('vertical')` → the y-mirror, north **150** (`180 - a`).
        - `rotateSelected(90)` on the area → the same geometry, north
          untouched.
        - `flipSelectedAreas('horizontal')` → the same as the sketch flip.
        - `rotateSelected(90)` with only wall 1 selected → that wall becomes
          (5,-5)→(5,5) **and walls 2 and 4 and the area's first two corners
          move too**, because they share its endpoints. The corners that share
          no key stay put. `transform::tests` pins all of it.
      * Rotate 90°, Flip Horz and Flip Vert in the canvas context menu now act
        (banking "Rotate" / "Flip Area"); that leaves five of the nine
        previously-dead entries live.
      * `rotateSketch` / `flipSketch` are implemented and verified but
        unreachable: their ribbon buttons are hidden in the original's own
        build (zero-sized rects in the live DOM), so they carry
        `#[allow(dead_code)]` with that reason recorded rather than being
        deleted.
      * Rotations leave a `-3e-16` residue exactly as the original's JS does,
        so the tests compare to 1e-9 rather than bit-for-bit.
      * Tests: 7 unit (the six live-parity cases above plus the bow reversal)
        and 2 e2e (Rotate 90° turns the selected wall vertical and banks undo;
        Flip Horz preserves its length).

      **6h (marquee, fence and lasso selection) verified 2026-10-24** — S6-040,
      S6-041.
      * `selection::fence_select` is `fenceSelect`: **containment, not
        intersection** — a wall needs both endpoints inside the rectangle, an
        area all of its points; Ctrl (`additive`) keeps the earlier pick
        instead of clearing.
      * `selection::lasso_select` is `lassoSelect`: the same containment rules
        against a closed freehand path, and fewer than three points selects
        nothing.
      * Verified against the live original with identical fixtures: the fence
        over (-1,-1)-(10,10) selects lines `[1,3]` and area `100` (the wall
        sticking out and the area with a corner outside are left alone);
        additive keeps a pre-selected line 2 (`[1,2,3]`); the lasso triangle
        selects `[1]` + `[100]`; two lasso points select nothing; the bigger
        lasso square selects `[1]` + `[100]` (line 3 at y=9 and area 101 at
        y=30 stay out).
      * The pointer is the marquee tool in Select mode: a press that hits
        nothing starts the drag (`marquee_start`), a press on a wall keeps the
        existing click path, and the release runs the fence. `FenceOutcome`
        reports what a finished drag should do (None / Zoom / Select / Lasso),
        so the zoom-fence keeps working through the same path.
      * The lasso and fence modes already existed on the rail; they now finish
        a selection instead of only the zoom fence doing anything.
      * Tests: 3 unit (whole-entity containment, additive, lasso rules) and 2
        e2e (a drag enclosing both walls selects both and releases the fence;
        a box covering one endpoint only selects nothing).

      **6i (drag-move and endpoint handles) verified 2026-10-24** — S6-042,
      S6-043.
      * `transform::move_selected` is `moveSelected`, with both of its regimes:
        with **lines** selected a connected wall stretches (only the endpoint it
        shares with the selection follows, its far end stays anchored); with an
        **area** selected only that area's own boundary walls move and a
        neighbour area glued by a genuinely shared wall comes along, while a
        merely coincident vertex is left alone.
      * `transform::move_line_endpoint` is `moveLineEndpoint`: a wall endpoint is
        a shared vertex, so it moves everywhere it occurs (adjoining walls
        stretch, their opposite ends stay put), the area corners on it follow,
        and a bowed wall keeps its angle — its arc height is recomputed as
        `chord/2 * tan(bowAngle * pi/720)`.
      * Verified against the live original, all four cases exact: moving wall 1
        of a 0..10/10..20 pair by (5,0) gives walls `[5,0,15,0]` and
        `[15,0,20,0]` with the area's two points following; moving the area by
        (0,3) leaves wall 2 at `[10,0,20,0]`; dragging wall 1's end to (12,4)
        moves wall 1's end, wall 2's start and the area corner together; and a
        20 ft chord at 30 degrees gives an arc height of **1.316525**.
      * The drag itself: a press on a selected wall's endpoint handle grabs that
        vertex (`type: 'line-endpoint'`, a 9px screen tolerance), any other press
        on a hit starts a selection move (`type: 'selection'`), and a press that
        misses still starts the marquee. Moves are applied as **whole grid steps**
        from the grab point when snapping is on (so an on-grid sketch stays on
        the grid and lengths stay exact) and the undo step is banked once, on the
        first actual movement. The release closes joined exterior loops, as
        `finalizeSelectionMove` does.
      * Not ported: `ownBoundaryLine`'s detachment of a wall that a moving area
        shares with a staying area — that needs `edgeBorrowsPartOfWall`.
      * `CanvasState` press handling is unchanged: an earlier attempt to fall
        back to the last `CursorMoved` position made every press use a stale
        point and broke real presses, so it was reverted. iced_test reports a
        press at the position from before its own instruction's move, so the
        endpoint-handle test drives `CanvasPress`/`CanvasDrag`/`CanvasDragEnd`
        directly with explicit canvas coordinates, while the selection-move test
        uses a scripted drag.
      * Tests: 4 unit (the two move regimes, the shared-vertex endpoint drag, the
        bowed-wall arc height) and 2 e2e (a scripted drag moves the wall by whole
        grid steps and banks undo; the end handle is grabbed and only that vertex
        moves — world y grows upwards, so a downward drag is a negative delta).

      **7a (area properties model + the catalogue) verified 2026-10-24** —
      S7-005, S7-006, and the area-code catalogue S7-009/010 depend on.
      * **The catalogue already existed** in `sketch-core/src/geometry.rs`
        (`AreaCodes`: 32 defaults, `normalize_area_code`, `validate_area_codes`,
        `from_json`/`to_json`, `as_map`, `basement_rollup_from_totals`, with the
        `includeInGLA` serde rename). I started a duplicate
        `sketch-core/src/area_codes.rs` before checking, found the original, and
        **deleted the duplicate** rather than commit a second copy — the lesson
        is to grep the core for a module before porting it.
      * `Polygon` gained `factor` (default 1), `measure_to` (default
        `center`), `auto_post` (`AutoPost`: code/name/dims/calcs/base/factor +
        suffix "sf", defaults Name/Dims/Calcs on) and an `attributes` bag — all
        additive with serde defaults, so saved workfiles still load.
      * New `sketch-app/src/area_props.rs` transcribes
        `updateSelectedAreaProperties`, `updateSelectedAreaAttributes` and
        `toggleSelectedAreaAutoSubtract`: an empty code or name leaves the old
        one; a non-finite factor clears `factorOverride` while a set one
        applies; `autoSubtract: false` is `subtractMode: 'never'`; an unknown
        `measureTo` falls back to `center`; `autoPost.dims === false` hides
        every wall on the selected areas' edges; attributes merge rather than
        replace; the auto-subtract toggle flips the whole selection and toasts
        "Auto-Subtract enabled/disabled for selected area(s)."
      * Verified against the live original: the full payload reproduces
        type/name/factorOverride 0.5/factor 0.5/never/interior/autoPost and all
        four walls hidden; the minimal payload leaves the code and name; an
        unknown `measureTo` gives `center`; the toggle flips `never` -> `auto`.
      * **One quirk found and reproduced, not corrected:** the original does
        `Number.isFinite(Number(properties.factor))`, so `factor: null` and an
        empty field both coerce to **0** (`Number(null)` and `Number('')` are
        0) — an empty Factor sets factor 0. `AreaProperties::factor` is
        `Option<f64>` where `None` is the original's `undefined` (clears the
        override) and the dialog passes `Some(0.0)` for an empty field. Both
        branches are pinned in `area_props` tests.
      * The dialog is measured into `refs/ui-spec/area-properties-spec.json`
        with every control's rect.

      **7b (the Area Properties dialog) verified 2026-10-24** — S7-005.
      * New `sketch-app/src/area_dialog.rs` renders the measured card: 520x456
        at (440,202), `--bg-panel`, 1px `--border`, radius 5,
        `0 16px 46px rgba(0,0,0,.5)`, 12px padding, a `rgba(0,0,0,.46)`
        backdrop, the close × at (935.3,215), the Area/Name/Factor/
        Auto-Subtract/Measure To rows at their measured x and widths, the 3x2
        auto-post grid at x 464/623.7/783.3 y 520/543, Suffix, and
        Cancel/Apply at y 618.
      * The three selects are `pick_list`s styled to the original's boxes: the
        original uses native `<select>` elements, which iced has no equivalent
        for — the same documented stand-in the Edit Tools' Line Style uses.
      * Wired: Edit Tools › Area Props and the canvas context menu's
        Properties… both open it for the first selected area; with no area
        selected the original's notice "Use Edit Tools to change the selected
        item." appears instead. Apply runs `apply_properties` and banks "Area
        Properties"; Cancel, the × and a press on the backdrop close it.
      * The dialog's state machine is unit-tested (opens with the area's
        values, a whole factor prints without decimals, an empty Factor field
        is 0 while junk clears the override, every control reaches the payload,
        and the Positive/Negative radio is display-only because the original's
        `updateSelectedAreaProperties` never reads it).
      * **Pixel-verified 2026-10-24** against `reference area-properties`
        (captured from the live original with a 10x10 UND area selected, the
        same state the headless state builds by drawing four walls, closing the
        loop and clicking inside):
        - The card's border box is **exact**: rows 202 and 657, columns 440 and
          959 in both images.
        - All thirteen content bands now match within 1-3px: 220/258/298/333/
          365/390/422/453/500/519/542/577/621 against the reference's
          221/258/298/332/362/389/422/453/499/520/543/577/626.
        - `reference area-properties`: title 1.16%, ribbon 4.59%, rail 2.23%,
          canvas 3.38%, dock 0.50%, status 2.26%, overall **2.86%**.
      * Three corrections the comparison forced, none of which the measurements
        alone revealed:
        - The card must have **no container padding**: every child is placed at
          its measured absolute position, and a padded container shifted the
          whole pinned stack down (the same pin-origin quirk the command popup
          documents). A `PIN_Y = 2` calibration then landed the rows on the
          reference pixels.
        - Two elements were **missing entirely**: the original's group captions
          "Calculation Adjustments" (above Factor) and "Automatic labels" (above
          the checkbox grid), each a caption plus a 1px rule running to the
          card's right edge. They are the reference's bands at 332 and 499; the
          comparison exposed them as unexplained bands, and reading the reference
          region identified them.
        - Selecting an **area** (rather than a wall) crashed the Edit Tools
          ribbon: `items(Body::EditTools, None)` panicked with "Edit Tools needs
          its values" because the panel assumed a selected line. The original
          hides its `contextual-line-tools` group for an area-only selection, so
          `edit_tools` now takes `Option<&EditToolsData>` and appends the line
          group only when a wall is selected.
      * The Cancel/Apply buttons are both `#3E3E42` with `#ccc` text: measuring
        the attributes dialog (below) showed that this dialog family has **no
        accent primary**, so the earlier blue Apply was corrected.

      **7c (the Area Attributes dialog) verified 2026-10-24** — S7-006.
      * New `sketch-app/src/attr_dialog.rs` renders `showSelectedAreaAttributes`
        measured at (440,299.5) 520x261: the title at (453,313.5), the close ×,
        the Area/Attribute/Value header at y 345.5, four rows on a 30px pitch
        (Plan Ref Number, Floor Code, FloorCodeText, Year Built) with 11px
        labels at x 636.8 and 148.6-wide inputs at x 793.4, and Cancel (59.5)
        plus OK (37.3) at y 519.5, both `#3E3E42`.
      * `editable` mirrors the original: the Home ribbon opens it read-only
        (`editAreaAttributes` -> `showSelectedAreaAttributes(false)`) and Edit
        Tools opens it editable; OK merges all four values through
        `apply_attributes` and banks "Area Attributes"; Cancel, the × and a
        backdrop press close it. With no area selected it shows "Select an area
        before editing its attributes."
      * **Pixel-verified** against `reference area-attributes` (the same 10x10
        UND area, attributes dialog open): the card's border box is exact
        (300/560 x 440/959) and every content band matches exactly —
        319/355/398/428/458/488 against the reference's 319/355/398/428/458/488,
        with the buttons 1px off before a final nudge.
      * Calibration found by the comparison: the reference's row text sits 6px
        below the measured row box (a label centred in its 29px input), so the
        row base moved from 386.5 to 392.5; the title and header moved +2.
      * Tests: 3 unit (opens with the area's attributes, missing ones are empty,
        the payload carries all four keys in order, edits land, and the card and
        row pitch are the measured ones).

      **7d (View Calcs — the numbers) verified 2026-10-24** — S7-002 (model).
      * New `sketch-app/src/calcs.rs` is `updateCalcsPanel`'s computation:
        areas grouped by code in first-seen order (the original's `Map`), each
        group carrying the sum of its net areas, and each area a row of Base
        Area / Factor / Total / Net Area / Perimeter — the perimeter through the
        existing `GeometryEngine::calculate_perimeter`. `UND` areas are excluded
        from the groups and counted for the summary.
      * Verified against the live original with the same four areas: groups
        "GLA1 1250.00" and "GAR 400.00"; area rows "First Floor : 1000.00",
        "Wing : 250.00", "Garage : 400.00"; detail rows Base Area 1000.00,
        Factor 1.00, Total 1000.00, Net Area 1000.00, Perimeter 40.00 — all
        reproduced exactly.
      * The summary is the original's four rows, not the two I first invented:
        **Defined Areas**, **Undefined Areas** (only when any, in `#E53E3E`),
        **Total Gross Living Area** and **Total Gross Building Area**, the last
        two to one decimal place with " sf". Their values come from
        `app.areaTotals`, which the unported `recalculate()` produces, so
        `summary` takes an `Option<AreaTotals>` and falls back to 0.0 exactly as
        the original's `this.app.areaTotals || { totalGLA: 0 }` does — the live
        capture prints "0.0 sf" for the same reason.
      * `View › View Calcs` now runs `toggleCalcsPanel`: it closes the dock when
        Calcs is already showing and otherwise opens it on the Calcs tab.
      * Not yet done: `recalcAutoSubtract`, so an area's net equals its total and
        `Subtracted`/`containerId` child ordering are absent.

      **7e (the Calcs tree) verified 2026-10-24** — S7-002 (widget).
      * The dock's Calcs panel now renders the model instead of the static
        empty state: group headers (11px semibold with a monospace `#63B3ED`
        total, 6px/8px padding, `caret-down`/`caret-right`), area rows (10px
        with a 10px caret, "name : net"), and each area's detail rows (Base
        Area / Factor / Total / (Subtracted in `#FC8181`) / Net Area /
        Perimeter, muted label against a monospace value) — all from
        `theme.css`'s `.calcs-*` rules.
      * Groups open expanded and areas closed, as the original's markup does;
        a group header toggles it, and an area row toggles its details **and
        selects that area** (`clearSelection` then `selectedPolyIds.add`).
      * The summary is driven by `calcs::summary`, so it shows the original's
        rows rather than the three hardcoded ones the panel shipped with.
      * The model is built once per `view()` from the polygons the canvas is
        drawing, which is where the original calls `updateCalcsPanel` after
        every change.
      * Tests: 1 e2e (a drawn loop becomes an area; a code applied through the
        properties dialog puts it in a group; View Calcs opens the dock on the
        Calcs tab; the area row expands and selects, then collapses; the group
        header toggles).
      * Not pixel-compared: the tree's own rendering has no reference capture
        yet (the screen-3 `dock-calcs` reference is the empty state), so this is
        structurally faithful but not visually verified.

      **7f (the area-code table editor) built 2026-10-24** — S7-004, S7-010.
      * New `sketch-app/src/codes_editor.rs` renders `showAreaCodeEditor`
        measured at (340,152.59) 720x554.8 over a full-screen
        `rgba(0,0,0,.62)` overlay: a 44px title row ("Area code table"), the
        12px hint at (355,207.6), a 690x412.8 monospace (Consolas) textarea at
        (355,233.59) on `#111111`, and four `#3E3E42` buttons at y 667.4 —
        Apply (59.4 wide), Import JSON (103), Export JSON (103.8), Restore
        defaults (123.1). The card has NO radius and no padding, as measured.
      * The textarea is iced's `text_editor`; the JSON it opens with is the
        catalogue's own `to_json`, so a round-trip through Apply is lossless.
        Apply runs the already-ported `AreaCodes::from_json` and shows either
        "Area code table updated." or the catalogue's verbatim error (e.g.
        "Area code 1 has an invalid code."), then closes on success.
        Restore defaults rewrites the textarea with the 32-code catalogue.
      * `app.area_codes` is now real state (initialised from
        `default_area_codes`) rather than re-derived per view, which is what
        `setAreaCodes` needs.
      * **Deferred with the reason:** Import JSON and Export JSON need a native
        file dialog (the original uses a hidden `<input type=file>` and a
        download anchor); both currently run Apply's validation path and are
        named here rather than faked. The New Area Code dialog (S7-009) and the
        area-definition editor (S7-007) are separate rows, still open.
      * **Pixel-verified** against `reference area-codes-editor`: the card's
        border box is exact (rows 153/706, columns 340/1059) and every content
        band matches within 1px — title 171/212/249/266/282/298/314/332 against
        the reference's 170/212/249/265/282/298/315/333. `reference
        area-codes-editor`: title 0.52%, ribbon 0.42%, rail 1.17%, canvas 4.50%,
        dock 0.09%, status 1.57%, overall **2.92%**.
      * Three corrections the comparison forced: the title sat 7px high and the
        hint 2px high; the textarea's first line started 3px high; and iced's
        default line height gave a 15.5px pitch against the reference's 16.5, so
        the editor needed an explicit `LineHeight::Absolute(Pixels(16.5))`.
        The title is Segoe UI 16 (the dialog family's font) and its close × is a
        16px glyph, not the 20px one first used — the oversized glyph was
        merging into the title's ink band and made the band read 4px high.
      * Tests: 4 unit (opens with the table's JSON, restore rewrites the text
        and re-parses to the 32 defaults, Apply's validation surfaces the
        original's message for a bad table and parses a good one, and the card
        and button coordinates are the measured ones).

      **7g (the New Area Code dialog) built and pixel-verified 2026-10-24** —
      S7-009.
      * New `sketch-app/src/new_code_dialog.rs` renders `showAddAreaCode`
        measured at (450,160) 500x540 (radius 5, 12px padding): ten rows with
        the label left and a 336-wide control at x 601 — Code 206, Name 245,
        Description 284, Group 323, Multiplier 362 (39px pitch), Include in GLA
        401, Line color 427, Fill color 467, Fill style 507, Fill opacity 548 —
        then a 474x62 preview at (463,588) and Cancel (59.5) / Create Code
        (93.6) at y 660. Opened from the Areas dock footer's "New area" link,
        which previously pressed `Noop`.
      * The create logic is the original's submit, checked line by line against
        the live page: code trimmed and upper-cased, then "Code is required.",
        "Name is required.", `Area code "X" already exists.`; description
        defaults to the name, an empty group becomes `OTH`, and the multiplier
        follows `Math.max(0, Number(x) || 1)` — so junk, empty and 0 all become
        1 and a negative clamps to 0. Success shows `Area code "X" created.`
      * Verified live for five submissions (create, no code, no name, duplicate,
        junk multiplier) — the notices and the normalised entries match exactly.
      * **Pixel-verified** against `reference new-area-code`: the card's border
        box is exact (rows 160/699, columns 450/949) and all seven labels land
        within 1px (218/257/296/335/374/438/478 against the reference's
        217/256/295/334/373/438/478). `reference new-area-code`: title 1.10%,
        ribbon 0.92%, rail 2.23%, canvas 3.30%, dock 0.50%, status 2.26%,
        overall **2.38%**.
      * Two corrections the comparison forced: the row labels sat 1-3px low
        (`place` already adds `PIN_Y`), and the colour swatches needed to sit
        5px ABOVE their labels — the reference's 428 band is the swatch, not
        the label, which a band-start comparison alone reads as a label error.
      * **Not wired:** the two colour swatches are static (the original opens a
        colour picker on each), and the preview shows the fill at its opacity
        rather than drawing the fill *style* pattern. Both are named here.
      * Tests: 4 unit (normalisation, the three notices verbatim, the multiplier
        coercion table, and an empty group becoming OTH).

      **7h (the area-definition editor) built and pixel-verified 2026-10-24** —
      S7-007.
      * New `sketch-app/src/def_editor.rs` renders `showAreaDefinitionEditor`
        measured at (450,251) 500x358 (radius 5, 12px padding): five rows with
        the label left and a 336-wide control at x 601 — Area name 297, Line
        color 336, Fill color 376, Fill style 416, Fill opacity 457 (a 286px
        range plus an `<output>` at x 895 reading "10%") — then a 474x62
        preview at (463,497) and Cancel (59.5) / Save Permanently (126.9) at y
        569. The swatch geometry reuses the 7g lesson (5px above its label).
      * Saving does more than rewrite the catalogue, exactly as the original's
        submit does: the entry is rewritten (name trimmed, color, fillColor,
        fillStyle, fillOpacity), every polygon of that code is renamed, and the
        lines of its edges are repainted to the line colour. It reports
        `X settings saved permanently.`, and "Select an area definition to
        edit." when no definition is selected. Opened from the Areas dock
        footer's "Edit areas…" link, which previously pressed `Noop`.
      * **Pixel-verified** against `reference area-definition-editor`: the
        card's border box is exact (rows 251/608) and every band is within 2px
        (title -1, swatches +1, style 0, opacity -5 at the slider's thumb,
        preview +2, buttons +2). `reference area-definition-editor`: title
        1.10%, ribbon 0.92%, rail 2.23%, canvas 2.75%, dock 0.50%, status
        2.26%, overall **2.05%** — the lowest of the screen-7 dialogs.
      * **Not wired:** the swatches are static (the original opens the colour
        dialog on each), the preview doesn't draw the fill pattern, and the
        `<output>` reads the slider's rounded value only while the slider is
        dragged (the original binds on `input` for every control).
      * Tests: 4 unit (opens on the entry's own values, rewrites only that
        code, renames the code's polygons and repaints their edge lines, and
        the empty-string fallbacks to `#718096` / `solid`).

      **7i (the live area palette) built and reference-checked 2026-10-24** —
      S7-008.
      * The Areas dock's palette is now driven by the live `app.areaCodes`
        table instead of a static transcription: `dock::paint_palette` ports
        the original's real renderer `paintPalette`
        (`p8-optional-region-classification.js` ~22730), which we had to
        re-locate — `populateAreaCodes` targets a `.area-tree` DOM that does
        not exist in the original, so its `groupNames` map ("Gross Living
        Area") is NOT the palette's labels. `paintPalette` uses the short
        `labels` map: GLA→'Living space', GBA→'Building area', GAR→'Garage /
        carport', BSMT→'Basement', P/P→'Outdoor', OTH→'Storage', LAND→'Land',
        SITE→'Site', NCA→'Non-calculated' (unknown groups fall back to the
        code).
      * Model rules ported: UND is excluded from the groups by CODE and
        rendered as the trailing "Undefined · clear" tile (UND entry's colour
        or the `#E53E3E` fallback when the table lacks it); groups iterate in
        first-seen order EXCLUDING group UND; empty groups are skipped;
        `codeColor` is `entry.color || entry.fillColor || '#858585'`; tiles
        show `entry.name || entry.code`; clicking a tile arms that type
        (`setArmed` → `selectedAreaType`/`selectedAreaName`).
      * The static GROUPS/UNDEFINED transcription is deleted; the palette is
        the single source now.
      * **Reference-checked** against `dock-Areas`: overall **0.89%**, dock
        **1.74%** (the earlier wrong-label iteration measured 1.02%/2.47%).
        Residual row diffs are 1-2px text positioning and glyph antialiasing,
        not content.
      * **Named gaps:** the per-tile usage badge (`.area-tile-used`, the count
        of polygons carrying that code, shown only when > 0) is not rendered —
        it needs the polygon counts threaded into the dock view, and the
        reference state has no polygons so the default view is unaffected;
        tile drag-to-classify (startTileDrag) is not ported; group
        collapse/expand state (collapsedGroups) is not yet wired to the
        header click.
      * Tests: 6 unit (short labels, first-seen order without UND, the
        trailing clear tile, the missing-UND fallback, the codeColor chain,
        and empty-group/label-fallback behaviour).

      **7j (`recalcAutoSubtract` — the recalculation core) built
      2026-10-24** — the pipeline the Calcs summary and the Area Breakdown
      both read.
      * `window.AreaCalc` IS the area-breakdown module: the app's
        `recalcAutoSubtract()` → `recalculate()` calls
        `AreaCalc.recalculate(polygons, areaCodes)` directly. Ported as
        `sketch-core/src/area_calc.rs` from `area-breakdown.js` lines
        555-717: `recomputeAreas` (gross/area/perimeter via the engine),
        `areaLevel`/`sameLevel` (BSMT*→0, GLA|GBA+n→n, other defined codes→1,
        UND→null — and a null level pairs with ANY level), the containment
        tree (smallest same-level container, gross then id tiebreak, engine
        `polygon_contains` at the JS defaults tolerance 1e-4 / maxSagitta
        0.01), `resolveSubtraction` (`auto`/`always`/`never` with the two
        verbatim warnings), `applySubtractions` (a per-container roll-up of
        the children's gross — no polygon boolean operations anywhere),
        `applyFactors` (net = (gross - subtracted) × multiplier × sign) and
        `rollup` (totalsByCode/Group, totalGLA, totalAllAreas; a deducted
        area still counts as what it is, only a negative net stays out).
      * **Verified against the live page** for the tricky pair: a 40x40 GLA1
        house with a 10x10 UND square inside nets 1500 — `sameLevel` treats a
        null level as a wildcard, so an undefined outline DOES deduct from
        its container while UND itself stays out of the totals (its
        multiplier is 0). One test asserted the opposite before the check;
        the page won, as always.
      * Wired: `App::recalculate_if_needed` runs at the end of every update
        behind a hash signature of the polygons and the code table (the
        original's `paletteSignature` guard), so pointer movement and panning
        cannot trigger a pass. It fills `calcs_totals` (TotalGLA and
        `totalsByGroup.GBA`) and keeps the per-polygon computed values for
        the Calcs tree, which now reads Subtracted and Net Area from the pass
        (`calcs::build` takes a `computed` map; before the first pass every
        area's net equals its total, exactly as the original behaves).
      * The totals chip ("GLA 0.0 sf · N areas") still reads static text —
        named as a gap, as are the Area Breakdown report (S7-003) and the
        GLA event the suite listens for.
      * Tests: 8 core unit (levels and same-level pairing, nested deduction,
        cross-level non-deduction, cycle warning, missing-target warning,
        factor override, negative sign, UND deduction verified live).

      **7k (the Area Breakdown's computation and report) built 2026-10-24** —
      S7-003's core, `sketch-core/src/breakdown.rs`.
      * The decomposition engine is fully ported: `signedArea`, `normalize`
        (winding fix with mirrored bows), `segmentArea`/`edgeLength` (the
        circular-segment geometry of a bowed edge), `isRectilinear`,
        `rectilinearPartition` (horizontal bands merged into columns via the
        string-keyed `mergeBands`), `squareOff` (each diagonal gains the
        corner that grows the area least; the corner triangles deduct; more
        than 4 diagonals or a non-reconciling result falls back to
        triangles), ear-clipping `triangulate` with the 10000-iteration guard
        and `triangleDims` (axis-aligned base preferred).
      * `computeForPolygon` rules ported: the arc segments of every bowed
        edge; the winding flip when a major segment outweighs the chord
        polygon (all signed components flip together); a `balance` piece when
        the pieces miss the stored gross by more than `minPieceArea`;
        `reconciled` within 0.01.
      * `toText` ported verbatim: `AREA CALCULATIONS`, per-code groups in
        first-seen order with two-space nesting for contained areas, the
        piece lines (`w x h`, `0.5 x base x height`, `arc r=r angle°`,
        `balance`/`polygon`), Gross / Subtracted (only when non-zero) /
        Multiplier (only when ≠ 1) / Net / Perimeter with the JS padStart/
        padEnd widths, then `SUMMARY` with the codes in first-seen order (JS
        object key order — a BTreeMap sort was wrong and the verbatim test
        caught it), Total Gross Living Area, Total All Areas.
      * **Verified against the live page** for three fixtures — a 30x30 GLA1
        rectangle, a chamfered GAR pentagon (square-off: one 150.00 rectangle
        minus two -12.50 corner triangles), and a PORCH with a 45° bow (arc
        r=13.1 45.0° = -6.68) — and the port's `to_text` output matches the
        original's **character for character**, including the padding. Two
        test constants were first fabricated from the rounded display values
        (r=13.0699… vs the true 13.0656…); the debug print corrected them.
      * `rollup` (with the deducted-area rule) and `fmtDim` (decimal and
        feet-inches, including the 12-inch carry) are ported; `toJSON` /
        `toXml` / the preview SVG and the dialog UI are still open.
      * Tests: 5 unit (the rectangle, the square-off, the arc, the verbatim
        report, and the feet-inches + Subtracted/Multiplier lines).
      * Tests: 5 unit (grouping order and totals, the row's five numbers, an
        unnamed area falling back to its code, the empty note, and the four
        summary rows including the totals fallback and the absent Undefined row).
      * **Dialog built 2026-10-25** — `breakdown_dialog.rs` ports the measured
        1080×720 Breakdown Details overlay (S7-003): report `<pre>` in 12px
        Consolas `#ccc`, the 647px `#1E1E1E` preview pane with `previewSvg`'s
        viewBox math, the four measured buttons (Copy, Feet & inches style
        toggle, Download JSON, Download XML — the last two stubbed to native-
        save-dialog toasts), and the empty-input guard. Wired into the Tools
        ribbon (`Area\nBreakdown`). 5 e2e tests; two `breakdown.rs` fixes
        surfaced: unsigned underflow in `normalize`'s winding-reversal bow
        lookup (latent on clockwise polygons), and a `<3` point guard.

## Screen 9 - chain dimensions (S9-024/035/040/045) - built 2026-10-26, pending parent acceptance

      * New `chains.rs`: the `state.dimensionChains` record
        `{ id, seedLineId, side, offset, showOverall, hiddenSpans }` (serde
        camelCase, riding pages verbatim), the `ChainTiers` tier solver
        (signed offset = `max(0.5, offset) * (side || 1)`; the overall tier's
        extra = `sign(signed) * max(1.5, |signed|*0.5)`), `seed_wall` (the
        8-px straight-wall pass; `|bowAngle| >= 0.5` skipped), and the
        `Drawing` impls `compute_chain`/`chain_tiers`/`chain_hit`/
        `chain_coverage` (coverage = the union of every chain's `lineIds`,
        suppresses the per-wall labels while a chain owns them -
        canvas-engine.js:1660's gate, render-time only, never mutated).
      * Behavior ported end to end: the rail's Chain Dimensions tool
        (`handleChainClick`: pushUndo('Dimension Chain') + the record with the
        click's cross-sign as the side, `Number(options.dimensionOption) || 3`
        offset; the miss toasts `Chain dimensions attach to straight walls.`);
        Select-mode hit first (Ctrl toggles membership, a joined click arms
        the `chain-offset` drag, `Move Dimension Chain` pushed once via the
        drag bank); `deleteSelected` removes selected chains (and selected
        chains alone do not touch the walls under them); undo snapshots carry
        the records; per-page storage (`getPageData`/`loadPageData`) and the
        save model pages round-trip `pages[0]["dimensionChains"]`
        (round-trip test extended with a chain + page switch).
      * Rendering ported from `drawChains` (canvas-engine.js:1968): witness
        stubs + the shared dimension line with architectural ticks at each
        ref, per-span labels via `units.formatLength` with the
        `hiddenSpans`/`minimum` gates, the overall tier on the second, outer
        line, `SELECT_LINE` vs `dimLabel` colors by selection, and the offset
        grab handle.
      * Cross-sign note: the original's
        `dx*(wy-sy) - dy*(wx-sx) >= 0 => +1` in world y-up means clicking
        BELOW a wall seeds side -1; the tests pin both.
      * Gate: 227 tests green (full suite, 298 s), clippy 0, fmt clean. The
        dock double-click e2e flaked once in an earlier gate and passed in
        isolation plus the final full gate.

## Screen 11 - Options dialog + Page Layout dialog - built 2026-10-25/26, pending parent acceptance

      * Options (S11-001..032): flat `options.rs` with the original's exact
        camelCase keys (the `Units` quartet including
        `unitSystem/decimalPrecision/stripTrailingZeroes/areaLabelSuffix`),
        the six-tab dialog card, applyOptions' clamps/mirrors, render.rs pull
        of the configurable colors. Save model writes `settings.options`
        flat.
      * Page Layout (S11-033..037): new `page_layout.rs` carries
        `state.pageLayout` - `paperSize` (`legal|letter|a4`), `currentForm`
        (`<form>-<paper>`; the prefix torn off exactly like
        `normalizePageLayout`: split('-') minus last, unknown lands on
        standard), the five addenda switches (true³ + two false defaults) and
        the seven `captions` - with manual flat serde so an unrecognised
        paper/form clamps to the defaults on load, as `applySaveModel` does.
        The dialog card ports `showPageLayout` (paper radios, the form picker
        whose labels follow the paper like `refreshForms`, the five
        checkboxes, the seven caption fields, Restore Defaults / Apply /
        Cancel). File > Page Layout wired; Apply stores the layout + the
        `Page layout applied: Legal.` toast; the PDF export reads
        `layout.paperSize` instead of the Options sticker; the save model
        writes `settings.pageLayout` (the key's passthrough stop removed) and
        the round-trip test pins a Letter/Full-Page layout with a caption
        edit through the save -> apply -> settings parse.
      * Gate: 4 page_layout unit tests + round-trip green; clippy 0; fmt
        clean; full suite green at 231 tests.

## Screen 10 - layer system (S10-025..038) + underlay ghost (S10-046) - built 2026-10-26, pending parent acceptance

- **Panel (S10-025..034).** The Layers tab renders the current page's area
  layers only (getAreaLayersForCurrentPage's filter: areaPolyId names a live
  polygon that points back), with depth indent, active outline, hover bg,
  the hidden/locked/dragging fades, the collapsed-group fold and the empty
  note. Row click activates; eye/lock toggle with the original's undo labels
  ("Layer Visibility" / "Layer Lock") and selection clears; the caret folds
  groups; the context menu (right-press) offers Rename / Hide|Show /
  Lock|Unlock / Export; rename commits on Enter, refuses blanks, and Esc
  cancels. Drag reorder (S10-031) arms on a strip press, shows the
  drop-before/after/inside markers, and commits through moveLayer's guards
  with the "Move Layer" undo push.
- **Layer mechanics.** Every closed area gets a dedicated layer
  (assignAreaToDedicatedLayer: dedicated reuse, re-closed reuse, create via
  the shared genId counter, drained source removal) synced lazily each
  update, exactly like the original's panel/page-sync entry points.
  flattenLayerOrder/layerAncestry/descendantLayerIds/moveLayer are faithful
  ports; the branch removal is a superset of the original's dangling-parent
  bug (commented in remove_empty_area_layer).
- **Canvas effects.** applyLayerOrder's stable paint-order sort runs with
  the sync; hidden layers drop out of every render pass (walls, solids,
  bowed walls, symbols, texts); hidden/locked entities refuse selection via
  LayerGates in hit_test/select_at/select_all/fence/lasso/select_by_type.
- **Menu ops + export.** Send to Back / Bring to Front (S10-037/038) port
  autoAssignLayer + reorderSelectionWithinLayers (toast "Moved selection to
  layer X.", undo "Auto-Assign Layer"); Export (S10-035) writes the
  Sketch-Layer JSON (version/layers/lines/polygons/labels/symbols) via the
  save dialog with the "Exported layer" toast. S10-036 (Import) has no UI
  entry in the original either.
- **Snapshot fidelity.** Undo snapshots now carry state.layers +
  activeLayerId (the original's snapshotModel), so layer moves/toggles undo
  as a whole.
- **Approximations (documented gaps).** Hidden-layer names use the muted
  fade without strike-through (iced 0.14 text has no strike); the context
  menu anchors at the row instead of the right-click point; leaf rows split
  their drop strip 6/13/6 (the middle resolves to "after", matching
  moveLayer's behavior); rename has no blur-commit (no on_blur in iced).
- **Gate.** 248 tests green (9 layer e2e + Send/Bring e2e + 7 layers.rs unit
  tests), clippy 0 warnings, fmt clean.

## The library managers (S9-008..013 + S8-008 as captured) - built 2026-10-26, pending parent acceptance

- **Manage Text Library** (the dock's Edit button): the list + label input +
  alpha range rail + Add/Remove/Replace, ported from showTextLibraryManager
  with the original's four validation toasts, per-save sort/dedupe, and the
  focused input on open.
- **Organize Favorite Symbols** (the Symbols dock's Organize... button):
  every library symbol as a checkbox row (label-sorted, deduped, filter
  input), Restore Defaults (checkboxes only) and Save Favorites (persists
  state.symbolFavorites, refreshes the Favorites category, toasts).
  Note: the feature inventory's S8-008..021 "Manage Symbols" family is NOT
  in the captured build - the live original ships this organizer, so the
  PARITY ledger now records those rows as out-of-scope while the captured
  app is the source of truth.
- **Persistence.** settings.symbolFavorites round-trips (the save writes
  the list or null; the load filters to known types; the settings_extra
  passthrough stop removed).
- **Gate.** 256 tests green (2 manager e2e + 6 unit), clippy 0 warnings,
  fmt clean.

## Recent Files + exit confirm + keyboard maps (S11-049..053/058/080, file keys) - built 2026-10-26, pending parent acceptance

- **Recent Files.** The File backstage's Recent Files item toggles the
  submenu (300px at the menu's edge): the header + Clear, a file row per
  entry (the #F6E05E fill file icon, the name, the toLocaleString-style
  date) and the two-line empty note. Entries model the original's
  localStorage list - {name, lastOpened, data} with the model JSON
  embedded, deduped by name, newest first, capped at 10 with the ~1.5MB
  skip - recorded on save and open, reopened through apply_model and
  re-recorded. The browser's localStorage becomes a sidecar JSON at
  %APPDATA%\SketchRust\recent-files.json, written/read only when
  SKETCH_PERSIST is set (main() sets it; the e2e never does, so tests
  stay deterministic).
- **Exit confirm (S11-080).** The title bar's Close and the backstage Exit
  ask "Close this sketch? Save your work first if needed." first; OK runs
  iced::exit, Cancel keeps the app.
- **File > New + the keyboard map.** File > New and Ctrl+N open the
  "Start a new sketch? Unsaved changes will be lost." confirm; OK ports
  resetToBlank (entities/undo/pen/cursor/nextId/layers/pages/name/mode -
  the options survive). Ctrl+S / F2 save, Ctrl+O / F3 open, F12 /
  Ctrl+F12 save-as - input.rs maps them in the original's order. F1 and
  Ctrl+F1 wait for the Shortcuts dialog and Email (unbuilt).
- **Gate.** 262 tests green (3 new e2e + 3 recent.rs unit tests), clippy 0
  warnings, fmt clean.

## Import merge + the save fallback (S11-044, S11-068) - built 2026-10-26, pending parent acceptance

- **Import...** (File backstage): the picker reads skx or legacy JSON and
  merges the FIRST page at the (2,2) offset - fresh gen ids everywhere, the
  edges' lineIds and the imported subtract targets remapped, each imported
  area assigned its dedicated layer (assignAreaToDedicatedLayer), the
  "Import" undo bank, and the "Imported {name}." toast; a file without
  geometry toasts the error instead.
- **Save fallback (S11-068).** A failed .skx container write falls back to
  the pretty JSON with the "Could not write .skx - saved as JSON instead."
  toast and a .json file name (the recent-files record follows the real
  extension).
- **Gate.** 264 tests green (2 import unit tests), clippy 0 warnings, fmt
  clean.

## Breakdown downloads + Calcs Expand/Collapse (S7-070/071, S7-016/017) - built 2026-10-26, pending parent acceptance

- **Download JSON / XML** (the Breakdown dialog): reakdown::to_json ports
  toJSON (the areas at the 2/3-place export rounding + the rollup object)
  and 	o_xml ports toXml (the tns:sketch namespace document, the BOM +
  CRLF join, the subject value, the three 10-comment tables from the
  subject model, per-area code/index/name/uniqueId with UND skipped, the
  description and multiplier from the code table, the one-decimal sizes).
  The dialog's buttons write rea-breakdown.json / .xml through the
  save dialog; the report and the downloads read one reakdown_results().
  Note: the port's breakdown numbers cover the current page (its recalc
  design) - the original normalizes the whole sketch; recorded in the XML
  doc comment.
- **Expand All / Collapse All** (the Calcs panel): the buttons flood or
  clear the dock's collapsed-group set (S7-016/017).
- **Gate.** 264 tests green (2 core export tests), clippy 0 warnings, fmt
  clean.

## The reference image module + the Geo menu (S10-007..024, S10-047..049) - built 2026-10-26, pending parent acceptance

- **The model.** state.bgImage ported: the image bytes (PNG/JPEG; the
  natural size decoded from the file headers), the world center, the scale
  (auto-fit to ~60 feet on load), the rotation, the opacity (the intensity
  slider's 0-100 at the 50 default), the flips, and the Set Scale / Orient
  Guide / Edit Mode states. SESSION-ONLY, as captured (buildSaveModel
  never carries it; saveImageWithSketch only widens the exported PNG's
  bounds - that export-bounds behavior is not ported yet).
- **The canvas.** drawBgImage beneath the drawing (translate/rotate/
  flip/draw_image at the intensity opacity); the Set Scale / Orient Guide
  markers (crosshair + circle + label, #F6AD55) and the guide line on
  top; the mode clicks intercept the canvas before the regular logic;
  Edit Mode's drag repositions and the wheel rescales (x1.05/0.95)
  instead of zooming.
- **The ribbon.** All fourteen buttons wired (Load/Show/Hide/Remove with
  its confirm/Set Scale/Smaller/Larger/Rotate Left/Right/Edit Mode with
  the Toggled look/Orient Guide/Flip Horizontal/Vertical), the intensity
  slider interactive (an invisible iced slider over the drawn pill), the
  Geo popup (Load Map Image/Auto Locate by Coordinate/Address).
- **The Geo prompts.** parseGpsPair ports the decimal and DMS forms (with
  the capture's mojibake degree signs); the coordinate flow validates,
  saves the subject's gpsCoordinates/gpsLatitude/gpsLongitude and toasts;
  the address flow saves the address. The subject fields round-trip in
  the save model (camelCase, empty-string-skipping).
- **Gate.** 271 tests green (5 reference_image unit tests + 2 e2e), clippy
  0 warnings, fmt clean.


## 2026-10-29 — 0.1.1 symbol management, layer JSON and recovery autosave

Requested native enhancements now replace the earlier expanded-reference exclusions.
The Symbols dock opens Manage Symbols and exposes right-click actions. The manager
provides category/search/Hidden filters, selection, per-row viewing, focused previews,
enlarging, rename, favorites, hide/restore, category deletion, permanent purge and
DXF/symbol-JSON additions. User changes persist separately from the bundled library.
Hide retains definitions for existing placements; purge confirms twice and removes
instances from all pages, undo/redo and clipboard, and rejects their return through
layer import. Custom definitions travel in SKX and layer files. DWG needs conversion
to DXF; this build does not claim a native binary-DWG decoder.

File > Import Layer accepts validated Sketch-Layer JSON with original coordinates,
group hierarchy, layer flags, remapped entity IDs, area membership/subtraction,
hosted openings and dimension chains. The operation is undoable. Export now includes
text, symbols, chains and custom definitions with named layer IDs. Named wall/area
layer IDs also survive ordinary SKX save/open.

Auto-Save writes changed complete-document SKX recovery files at the configured
1–120 minute interval, with atomic replacement and no picker. Startup and File >
Recover Autosave offer the newest valid snapshot. Unchanged documents are skipped;
corrupt candidates are skipped; failed writes preserve the preceding valid snapshot.
Later, New and Open retain earlier recovery files. Saving a different sketch cannot
remove a deferred recovery. Successful manual save cleans only the current/recovered
snapshot. Reference images remain session-only.

Validation:

- Full app suite: **501 passed, 0 failed**, including real widget interactions,
  reference screenshots and the full report/paper/preview matrix (617.43 seconds).
- Final focused run after the purge-import, deferred-recovery and row-view refinements:
  **18 passed, 0 failed** (12 management/model regressions plus 6 symbol/recovery UI
  regressions). These repeat tests in the 501, not additional unique test counts.
- Core suite: **163 passed, 0 failed**. **664 unique tests** passed across the runs.
- Clippy workspace/all-targets with warnings denied and formatting pass.
- All **73 reference screenshot pairs** compared at the established tolerance;
  maximum overall difference **2.99%**. New menu/dock actions intentionally extend
  the reference. No universal pixel-parity claim is made.
- New UI evidence inspected: target/e2e/symbol-manager-native.png and
  target/e2e/autosave-recovered-native.png. Region results:
  target/e2e/comparison-results.json.

Windows harness note: after reporting all 501 passes, the graphics test process
retained its executable during shutdown. The completed runner was interrupted and
its generated executable moved aside before relinking. The final focused app run
and separate core run both exited successfully. No user app process was terminated.

Suite integration and physical DISTO acceptance remain deferred at user request.
Release/package provenance and installation checks are recorded in
[BUILD_VALIDATION.md](BUILD_VALIDATION.md).


## 2026-10-29 — 0.1.2 symbol-manager fit and live-preview correction

The user supplied the original two-pane Manage Symbols screenshot. The previous
native vertical layout could overflow its controls and placed the preview below
the list. It is replaced by a bounded modal with independently scrolling symbol
list and preview metadata, a continuously visible preview canvas, and an action
footer that wraps inside the available width. Only the list scrolls through the
library; scrolling it does not move the preview or footer away.

Clicking a symbol name updates the preview immediately without toggling bulk
selection. The focused row is highlighted; changing filters selects an available
preview. Checkboxes remain the bulk-action selection. Row View and Enlarge open a
separate Symbol Viewer. Escape returns to the manager before closing the manager.
Actions with unmet selection/category prerequisites are disabled.

- 20 focused app tests pass (the 18 existing management/symbol/recovery regressions
  plus new live-preview/viewer and small-window regressions).
- 163 core tests pass. Clippy workspace/all-targets with warnings denied and
  formatting pass.
- The real widget tree is exercised: click a name, verify updated type metadata,
  open Enlarge, Escape back to the manager, retain independent checkbox selection,
  and use the footer Close at 960×640 and 800×600.
- Inspected target/e2e/symbol-manager-live-preview.png (1400×860),
  symbol-manager-960x640.png, symbol-manager-800x600.png, and symbol-viewer-native.png.
  Footer actions and row View buttons fit; viewer metadata is readable.
- The earlier full 501-app/163-core run and 73 reference comparisons are historical
  0.1.1 evidence, not a new full-suite or pixel-parity claim for 0.1.2.

Suite integration and physical DISTO testing remain deferred. See
[BUILD_VALIDATION.md](BUILD_VALIDATION.md) for current package provenance.


## 2026-10-29 — 0.1.3 deleted-boundary repair

The user's delete/redraw failure was reproduced before the fix: the real mouse
sequence recreated the area, but the same replacement committed by Enter left
three old walls plus the replacement with no polygon. Enter called enter_key
without the finishCommittedWall/closeJoinedExteriorLoops path used by mouse input.

Mouse, typed Enter, touch and bay mirror now use a shared completion adapter.
Exterior commits detect closed faces; the pen and keyboard pointer end only when
the new wall belongs to an intact closed ring. Detecting an older unrelated loop
keeps the current run active. Interior illustration lines do not create appraisal
areas. Undo/redo keeps the replacement and its area closure in one edit. Recreated
areas follow the original app: UND unless Define First supplies a selected type.

The existing drawing/core ID adapter also mixed next-unused and last-allocated
counter conventions. Translating that boundary prevents the next generated wall
from reusing the newly created polygon's ID.

- 69 affected app regressions pass (drawing, clipboard, typed input, bay mirror,
  simulated Disto entry, library/layer/recovery, and the eight repair cases).
- 163 core regressions pass; no physical DISTO test was performed.
- Eight new real-program cases: mouse repair, keyboard repair with immediate
  pen-up, touch repair, Undo, Redo, interior exclusion, unrelated-loop detection,
  and a subsequent wall with unique entity IDs.
- Inspected target/e2e/redraw-keyboard-reclosed.png: the boundary is closed and
  the app reports “Area closed and ready to define.”
- Workspace/all-target Clippy with warnings denied and formatting pass.
- The full app/report/reference screenshot matrix was not rerun for this focused
  correction; earlier full-suite and comparison numbers remain historical evidence.

Current optimized package validation and executable fingerprint are recorded in
[BUILD_VALIDATION.md](BUILD_VALIDATION.md). Suite integration and physical
DISTO testing remain deferred.

Additional 0.1.3 check: `e2e_redraw_deleted_boundary_with_remaining_curved_wall_recloses_area` passes. It replaces a straight boundary while another boundary is bowed, verifies the curved edge and curved area are retained, and verifies automatic pen-up. This is one additional unique regression beyond the 69-test run (70 affected tests total, nine repair cases). No production code changed for this additional test.


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
