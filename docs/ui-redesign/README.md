# Sketch Rust interface redesign: build plan

This folder is a complete brief for rebuilding the Sketch Rust interface. It is written so
that a developer or an AI agent with no memory of the design discussion can finish the
build. Read the files in order.

| File | What it holds |
|---|---|
| `README.md` | Goals, decisions, rules, vocabulary, phase tracker |
| `01-design-spec.md` | Tokens, type, sizes, every screen region, wireframes for each window size |
| `02-architecture.md` | How the new interface fits the existing code: modules, state, messages, the shell switch |
| `03-phases.md` | The work, in order, as self-contained packages with acceptance checks |
| `04-feature-map.md` | Where every existing feature goes, the command registry, the coverage checklist |
| `05-area-types.md` | The area type editor and the new default colours |
| `06-symbol-library.md` | The symbol library screen and symbol editing |
| `07-touch-and-android.md` | Touch input, tablet and phone behaviour, what to verify for Android |
| `coverage.csv` | One row per inventoried feature (634). Fill it in as work lands |
| `mockups.html` | The approved renderings. Open in a browser |

## Why this is being done

The app is a Rust port of a browser app. Its screen layout follows the arrangement used by
Apex-style appraisal sketch programs: an Office ribbon with a File backstage, a vertical
tool rail, a right-hand dock listing every area code, and a dense status bar. The owner
wants to distribute the program and wants the interface to be clearly its own.

The redesign changes **layout, vocabulary and typography**. It keeps the colour palette,
because the palette is the app's own. It must not drop any feature.

This is a look-and-feel change only. It does not address questions about source lineage,
file formats or bundled data. Those are listed in "Outside the interface" at the end of
this file and need a separate review.

## Decisions already made by the owner (2026-11-14)

1. The layout in `mockups.html` is approved: top bar, tool tray, ledger, inspector, insert
   drawer, length strip, drawing controls, start screen.
2. Colours stay as the current app's three themes (`theme.rs` DARK, BLUE, LIGHT).
3. The interface must adapt to touch devices. **The tablet is the primary touch target.**
   The phone is supported as the smallest case. An Android port is planned later.
4. No feature may be skipped. The inventory in `refs/ui-spec/feature-inventory.md` has 634
   rows and each must end with a recorded home (`coverage.csv`).
5. All 32 built-in area types and all 639 built-in symbols stay available by default.
6. Area types are fully editable, including the built-in ones.
7. The symbol library is fully editable, including symbol shapes.
8. Living space floors 1 to 4 get four distinct default colours (see `05-area-types.md`).

## Still open (ask the owner before finalising)

- Fourth Floor default colour: red `#C53030` was proposed. Deep indigo is the alternative.
- "Undefined Area" (`UND`) stays undeletable, because areas fall back to it. Owner has not
  confirmed.
- Typeface: Barlow and Barlow Semi Condensed were proposed and not objected to.

## Rules for whoever builds this

1. **Rewrite views, not behaviour.** The app is an iced 0.14 program with one `Message`
   enum and one `App::update` (`crates/sketch-app/src/main.rs`). New screens must emit the
   existing messages wherever one exists. `02-architecture.md` lists them. Add a message
   only when no existing one does the job.
2. **Do not change `sketch-core`** except for the items named in `05-area-types.md` and
   `06-symbol-library.md`. Geometry, calculations, file formats and exports stay as they are.
3. **Both interfaces must run until cutover.** The old one is "classic", the new one is
   "next". See the shell switch in `02-architecture.md`. Every phase must leave both working.
4. **No fixed coordinates in new code.** The classic chrome pins elements at positions
   measured from a 1400 x 860 screenshot (`theme::at`, `ribbon_tabs.rs`). New code uses
   iced rows, columns, containers and `Length`, and must lay out correctly from 360 wide to
   4K.
5. **Every phase ends green:** `cargo build -p sketch-app`, `cargo test --workspace`, and
   the new headless tests for that phase.
6. **Use the new vocabulary** in every new string (table below). Never mention Apex in the
   interface. The guide may mention Appraise-It Pro, which is an integration.
7. **Update `coverage.csv` in the same change** that gives a feature its new home.
8. **Do not delete classic code or `refs/` before the cutover phase.**
9. Keep commits small and per package. Work on a branch named `ui-redesign`.

## Vocabulary

| Classic term | New term | Notes |
|---|---|---|
| Ribbon, File backstage, quick-access toolbar | Top bar, Menu, Start screen | |
| Tool rail | Tool tray | |
| Dock: Areas, Calcs, Layers tabs | Ledger: Drawn, Types, Layers | |
| Dock: Symbols, Text tabs | Insert drawer: Symbols, Room labels | |
| Edit Tools (contextual ribbon tab) | Inspector | |
| Status bar | Drawing controls | |
| Define Area | Area tool; the chooser asks "What is this area?" | |
| Area code | Area type | The short code (GLA1) is still shown and is still called "Code" |
| Calcs, View Calcs | Totals; "Show the math" opens the breakdown | |
| Area Breakdown | Area breakdown | unchanged |
| Subject Info | Property details | |
| Touch Panel | Pad | number and direction pad |
| Disto, Bluetooth | Laser | "Connect laser" |
| Reference Image | Trace image | |
| Manage symbols, Organize | Symbol library | |
| Text library | Room labels | |
| Segment HUD | Length strip | |
| Pen up, pen down | Pen up, pen down | unchanged |
| Mini-Map | Mini map | unchanged |

## Phase tracker

Tick a phase only when its acceptance checks in `03-phases.md` pass.

- [ ] P0 Preparation: branch, shell switch, test harness for "next"
- [ ] P1 Tokens, fonts, widget kit
- [ ] P2 Size classes and the empty shell
- [ ] P3 Command registry and Menu
- [ ] P4 Top bar and page tabs
- [ ] P5 Tool tray and fly-outs
- [ ] P6 Ledger: Drawn, Types, Layers, totals
- [ ] P7 Inspector
- [ ] P8 Insert drawer: symbols and room labels
- [ ] P9 Drawing controls, length strip, area chooser
- [ ] P10 Touch input and the Pad
- [ ] P11 Area type editor and new default colours
- [ ] P12 Symbol library and symbol editing
- [ ] P13 Start screen, dialogs restyle, guide and shortcut sheet rewrite
- [ ] P14 Coverage audit: every row of `coverage.csv` closed
- [ ] P15 Cutover: make "next" the default, delete classic chrome and `refs/`
- [ ] P16 Android feasibility spike (separate from the desktop release)

## Outside the interface (not solved by this plan)

Record of what was found while planning. None of this is fixed by a new layout.

- `refs/` holds the original web app's source and markup. If any of it derives from Apex
  material, that is a question for a lawyer.
- `dist/` holds old builds 0.1.21 to 0.1.29 with `APEX-COMPATIBILITY.md`, describing an
  AXF reader and writer and integration as an Apex replacement. The current code refuses
  AXF (`main.rs`, `read_sketch_path`). Those old builds and documents should not ship.
- Identifiers that still carry the name: the `apex.calculation` attribute key
  (`sketch-core/src/area_calc.rs`, `sketch-app/src/area_repair.rs`) and the `native_apex`
  field on `Symbol` (`sketch-core/src/symbols.rs`). Renaming them with a read-compatible
  alias is a small separate task.
- The window title is still "Appraisal Sketch" (`main.rs`, `program()` and `view()`).
  P4 changes it to "Sketch Rust".
- The built-in area type list and the 25 default room labels should be checked against
  where they came from.
