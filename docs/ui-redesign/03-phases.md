# 03. Phases

Each phase is a package that can be done in one sitting. Do them in order. Each lists what
to read, what to build, and how to know it is done. "Both shells green" means:

```
cargo build -p sketch-app
cargo test --workspace
SKETCH_UI=classic cargo run -p sketch-app -- --check-installation
SKETCH_UI=next cargo run -p sketch-app -- --check-installation
```

On Windows PowerShell set the variable with `$env:SKETCH_UI = 'next'` first.

---

## P0. Preparation

Read: `02-architecture.md` sections 1 to 3 and 8.

Build:

1. Create branch `ui-redesign`.
2. Add `Shell` and `App.shell`, chosen from `SKETCH_UI` then `ui-shell.json` then Classic.
3. Rename the body of `App::view` to `view_classic`. Extract `canvas_element(&self)` and
   `overlay_layers(&self)` from it without changing what classic renders.
4. Add `mod ui;` with `ui::view(app)` returning a column with the text "Sketch Rust
   (next)" over `canvas_element()`, under the shared overlays.
5. Add `Message::Ui(ui::Msg)` and `App.ui: ui::UiState` (empty for now).
6. Move the e2e `Harness` to `e2e_support.rs` (`pub(crate)`). Add `e2e_next.rs` with one
   test that boots Next at the four sizes and expects the placeholder text.
7. When Shell is Next, set the minimum window size to 360 x 560 and the title to
   "Sketch Rust - <name>".

Done when: both shells green; classic e2e screenshots are unchanged
(`bash tools/e2e.sh` reports the same table as before the phase); the new test passes.

---

## P1. Tokens, fonts, widget kit

Read: `01-design-spec.md` sections 1 to 3.

Build:

1. `ui/tokens.rs`: `link()`, `tape()`, `tape_ink()`, `flag()`, `on_accent()` per theme;
   spacing, radii and type size constants; `Density { Comfortable, Touch }` with methods
   `bar_h()`, `row_h()`, `field_h()`, `tray_button()`, `min_target()`.
2. Fonts: add `assets/fonts/Barlow-Regular.ttf`, `Barlow-Medium.ttf`,
   `Barlow-SemiBold.ttf`, `BarlowSemiCondensed-Medium.ttf`,
   `BarlowSemiCondensed-SemiBold.ttf` and their `OFL.txt` (from the Barlow project by
   Jeremy Tribby, SIL OFL 1.1). Load them with `iced::font::load` at boot in both shells.
   `ui/fonts.rs` exposes the `Font` constants. Add the files to the installer
   (`installer/SketchRust.wxs`) and the package scripts only if they are not embedded;
   prefer `include_bytes!` so nothing changes in packaging.
3. `ui/kit.rs`: the widgets listed in `02-architecture.md` section 4. Each takes
   `Density`. Each has hover, pressed, focused and disabled styles from tokens. Keyboard
   focus must be visible (2px accent outline).
4. Icons: keep using `icons::icon` / `icons::ph` (Phosphor, MIT, `assets/icons`). Add any
   missing icon SVGs the tray needs.
5. A hidden gallery page (`SKETCH_UI_GALLERY=1` shows every kit widget in both densities
   and all three themes) for visual checking.

Done when: both shells green; a test renders the gallery at 1400 x 860 in Dark, Blue and
Light and writes screenshots; no kit widget uses `theme::at`.

---

## P2. Size classes and the empty shell

Read: `01-design-spec.md` sections 4 and 6, `02-architecture.md` section 7.

Build:

1. `ui/layout.rs`: `SizeClass`, the four layout helpers, and `canvas_rect(app)`.
2. `App::canvas_origin()` and `App::canvas_rect()`; route classic constants through them
   (classic returns its old values).
3. `ui::view` lays out empty placeholder regions for Wide, Medium and Compact: bar,
   optional page bar, ledger placeholder, drawing, inspector placeholder, tray
   placeholder. Slide-overs and sheets open and close through `ui::Msg`.
4. Density selection as specified. Add `ui::Msg::SetDensity` and switch to Touch on the
   first `Event::Touch`.

Done when: resizing a Next window from 360 to 1600 wide switches layouts at 700 and 1100
with no overlap or clipped region; drawing, pan and zoom on the canvas work in all three;
tests assert the size class at the four test sizes.

---

## P3. Command registry and Menu

Read: `02-architecture.md` section 6, `04-feature-map.md` sections 1 and 2.

Build:

1. `ui/commands.rs` with every command in `04-feature-map.md` section 2.
2. Palette: when Shell is Next, list `commands::all()` instead of `CATALOG`.
3. `ui/menu.rs`: the Menu from the registry, with sub-menus, check marks, disabled
   states, shortcut hints, keyboard navigation, Escape to close.
4. A unit test that every `Message` produced by `run` is handled (call `update` on a
   booted `App` for each command that needs no file dialog and assert no panic).

Done when: every command can be run from the Menu and from command search in Next;
both shells green.

---

## P4. Top bar and page tabs

Read: `01-design-spec.md` section 5.1.

Build `ui/topbar.rs`:

1. Mark, Menu button, document name, save state (`App::save_state()`), page tabs, search,
   undo and redo, primary button, window controls.
2. Window drag on the bar (`Message::TitleDrag`), double-click to maximise
   (`ToggleMaximize`), resize zones (`ResizeDrag`) as classic does.
3. Page tabs from `self.pages`: switch, add, delete with the existing confirmation,
   underlay toggle, first and last. The classic Pages navigator stays reachable as
   "All pages" in the tab menu.
4. Undo history pop-up anchored under the Undo button (give `undo_history::layer` an
   anchor parameter).
5. Medium and Compact: two-row variant.

Done when: tests cover switching, adding and deleting pages, undo and redo, opening
search, and renaming, at all four sizes.

---

## P5. Tool tray and fly-outs

Read: `01-design-spec.md` section 5.2.

Build `ui/tray.rs`:

1. Ten buttons mapping to `Message::Rail(..)` as in `02-architecture.md` section 5.
   Active state from `self.view.mode` (PlaceSymbol highlights Insert, PlaceText highlights
   Label, PlaceShape highlights Shape, ZoomFence highlights Pan).
2. Fly-outs. The Wall and Inner fly-out replaces `shapes::layer`'s opener; reuse its
   messages.
3. Compact: a horizontally scrolling bar fixed at the bottom.
4. Tooltips with names and shortcuts (take texts from `tooltips::rail_title`, reworded).

Done when: every `view_state::Mode` can be entered from the tray or its fly-outs;
keyboard shortcuts still switch tools and the tray follows; tests at four sizes.

---

## P6. Ledger

Read: `01-design-spec.md` section 5.3.

Build `ui/ledger.rs`:

1. Drawn view from `calcs::build(&drawing.polygons, &computed)` and `calcs::summary`.
   Row expansion shows the six figures the classic Calcs tree shows.
2. Types view from `dock::paint_palette(&self.area_codes)`: groups, usage counts, arm,
   drag, context menu, Undefined row, search, "Edit types".
3. Layers section: port the layer tree from `dock.rs` (rows, indent, eye, lock, caret,
   inline rename, drag markers, context menu) to flexible layout.
4. Totals foot and "Show the math".
5. "..." menu: the "Edit areas" sheet actions and label defaults
   (`area_workflow::advanced`, `post_controls`).
6. Width drag handle in Wide (store in `UiState.ledger_width`, persist in the sketch
   settings where the dock width is stored today).
7. Medium slide-over and Compact sheet variants.

Done when: tests cover defining an area by arming a type and clicking an outline, by
dragging a type onto an outline, expanding a row, deleting a type (areas become
Undefined), finding undefined areas, renaming and hiding a layer. All at four sizes
(drag tests may run Wide only).

---

## P7. Inspector

Read: `01-design-spec.md` section 5.4.

Build `ui/inspector.rs` from `App::edit_tools_data()` and the selection:

1. One section per selection kind with every control in the table.
2. Steppers for wall bow, text size, symbol scale and opening width, so nothing needs a
   mouse wheel.
3. The Area section embeds the fields of the Area properties dialog
   (`area_dialog.rs`: code, name, factor, sign, auto-subtract, deduct-from, label
   switches) inline, emitting `AreaDialog(..)` messages; attributes open the existing
   dialog until P13 restyles it.
4. Trace image section when `self.bg_image.edit_mode` is on or an image is loaded and
   the Image menu's "Adjust" is chosen.

Done when: for each selection kind a test selects it and changes one value from the
inspector; the classic Edit Tools tab has no control without an inspector equivalent
(check against `ribbon.rs` `items(Body::EditTools, ..)`).

---

## P8. Insert drawer

Read: `01-design-spec.md` section 5.5.

Build `ui/insert.rs`:

1. Symbols mode: categories from `SymIndex` (all categories, counts), search, tile grid
   with previews (`symlib::render_geometry` / `Preview`), arm, drag, pin, context menu.
2. Room labels mode: search and add, letter filter, list, "Edit list".
3. Compact: full-screen page.

Done when: tests place a symbol by tap-then-tap and a label by tap-then-tap at four
sizes; the category list shows every category in the library file; favourites work.

---

## P9. Drawing controls, length strip, area chooser

Read: `01-design-spec.md` sections 5.6 to 5.8.

Build:

1. `ui/controls.rs`: zoom group with slider and 1:N field, snapping group, Show
   switches, Dimensions cycle, North. Anchor `filter::layer` to the Show button.
2. `ui/tape.rs`: reads `drawing.hud` (`len`, `ang`, `down`) and the typed buffer.
   Positioned from the last wall's end in screen space via `self.view.to_screen`, kept
   inside the canvas rect and away from the pointer.
3. `ui/chooser.rs`: restyle `area_workflow::chooser_layer` with flexible layout. Keep
   its messages. Must list all types when the search is empty.
4. Under-lay, mini map and laser controls placed as in the spec.

Done when: tests type a length and an arrow key and see the strip update; change zoom,
scale, grid, snap pitch, each Show switch, dimensions mode and north; close an area and
name it through the chooser.

---

## P10. Touch input and the Pad

Read: `07-touch-and-android.md`.

Build:

1. Press-and-hold (500 ms, under 8 px travel) on the canvas produces the context menu
   (`Message::RightPress` then `CanvasContext` path). Same on rows and tiles.
2. One-finger drag follows the active tool; two-finger gesture stays pinch and pan.
3. `ui/pad.rs` emitting `Message::Key(KeyCmd::..)`. Implement Turn left and Turn right
   (the classic `TouchAction` handler never implemented them).
4. "Select more" switch on the Select tool (sets the same additive flag as Ctrl).
5. Tap-to-preview in Area mode instead of hover.
6. Stylus: read Options `inputMode`. In "Pen as Ink" the pen draws and fingers pan.

Done when: a headless test drives a full floor plan (three areas) using only Pad
messages and taps; desktop mouse and keyboard behaviour is unchanged.

---

## P11. Area type editor and default colours

Read: `05-area-types.md`. Build what it specifies.

---

## P12. Symbol library and symbol editing

Read: `06-symbol-library.md`. Build what it specifies.

---

## P13. Start screen, dialogs, guide

Build:

1. `ui/start.rs` per spec 5.10, using `recent.rs`, and thumbnails rendered from each
   recent file's first page when cheap; otherwise a placeholder outline.
2. `ui/dialogs.rs`: one dialog frame (title, close, body, footer buttons) that becomes a
   full-screen page in Compact. Move each dialog onto it with flexible layout: Options
   (six tabs, every setting), Page layout, Print preview, Property details (three tabs),
   Area attributes, Area breakdown, Colour, Room shape, Laser, Undo history, Shortcut
   sheet, Room labels list, alerts, confirms and prompts.
3. Rewrite `guide.rs` and `shortcuts.rs` texts in the new vocabulary. Keep every
   shortcut.

Done when: every dialog opens, works and closes in Next at four sizes with nothing
clipped; no dialog in Next uses `theme::at`.

---

## P14. Coverage audit

1. Fill every row of `coverage.csv` (rules in `04-feature-map.md` section 3).
2. Add the unit test that checks command ids in the CSV exist.
3. For every row marked KEEP or MOVED, a Next test or a manual check note must exist.
4. Report any row that cannot be closed to the owner. Do not mark it closed.

Done when: zero rows have an empty disposition; the owner has signed off on rows marked
RETIRED.

---

## P15. Cutover

1. Default shell becomes Next. Run the full suite.
2. Delete: `titlebar.rs`, `ribbon.rs`, `ribbon_tabs.rs`, `backstage.rs`, `rail.rs`,
   the view parts of `dock.rs` and `statusbar.rs`, `touch_panel.rs`,
   `symbol_organizer.rs`, `codes_editor.rs`, `tools/gen_ribbon.py`, `tools/compare.py`
   and the gallery comparison in `tools/e2e.sh`, the classic e2e tests, `refs/`,
   `ACCEPTANCE.md`, `SCREEN_8_HANDOFF.md`, and the parity sections of `STATUS.md`.
   Keep the types that `update` uses (move them first, see `02-architecture.md`).
3. Remove `Shell`, `SKETCH_UI` and `ui-shell.json`.
4. Update `README.md`, `docs/RELEASE_README.txt`, the installer and package scripts.
5. Remove `theme::at`, `nudged`, `ink`, `INK_PASSES` if nothing uses them.

Done when: `cargo test --workspace` passes; `tools/package.ps1 -NativePreview` and
`tools/installer.ps1 -Test` succeed; a search of the repository for "ribbon",
"backstage" and "Appraisal Sketch" finds nothing in shipped code or documents.

---

## P16. Android feasibility spike

Separate from the desktop release. See `07-touch-and-android.md` section 5. The output is
a written finding, not a port.
