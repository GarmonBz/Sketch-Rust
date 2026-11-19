# Sketch Rust interface design

This is the design of the Sketch Rust interface as it is built on the `ui-redesign`
branch (17 November 2026). It describes what the app does now. Ideas that were proposed
but not built are listed at the end under "Not built yet".

The detailed build brief is in `docs/ui-redesign/` (`01-design-spec.md` for tokens and
regions, `04-feature-map.md` for every command, `coverage.csv` for where each of the 634
inventoried features lives). Where this file and the code differ, the code wins; please
fix this file.

## 1. Principles

- **The drawing comes first.** Chrome floats over or beside the plan and gets out of the
  way while drawing.
- **One home per command.** Every command has one on-screen place (the tool tray, drawing
  controls, inspector, page tabs, ledger, Options, top bar or Menu), and every command can
  also be found by search (Ctrl K).
- **Colour carries meaning.** Area colours come from area types. Icon colours say what
  kind of action a control does. Amber is reserved for live measurement.
- **Forms fit on a desktop screen.** At 1920 x 1080 and up, forms and settings dialogs fit
  the window. The inspector and lists of the user's own data scroll as needed.
- **One layout per screen** that adapts from a 372 px phone to a 4K monitor. No fixed
  pixel positions in `crates/sketch-app/src/ui/`.
- **Plain words.** Buttons say what happens ("Close area", "Delete type"). Sentence case
  everywhere. The app never mentions Apex.

## 2. Themes and colour

Three interface themes: Dark (default), Blue and Light. The drawing background is a
separate setting: White (default), Dark or Blue. Both are in Options > Display >
Appearance and apply as soon as they are picked.

Dark uses the "field precision CAD" palette (adopted 17 November 2026, version 0.2.2):
deep blue-black surfaces in four steps, a bright blue accent and cool grey text.

| Token | Dark | Blue | Light | Used for |
|---|---|---|---|---|
| chrome | `#0A0E15` | `#263F66` | `#DDE3EA` | Top bar (Dark's deepest surface) |
| bar | `#181C23` | `#263F66` | `#DDE3EA` | Pop-ups, Menu, tray, sheets |
| panel | `#181C23` | `#1E3150` | `#F7F8FA` | Ledger, inspector, dialogs |
| field | `#20252E` | `#17243A` | `#E9EDF2` (inputs `#FFFFFF`) | Inputs, active tab, cards, selected tile |
| hover | `#353941` | `#34547F` | `#CFD8E3` | Hovered and pressed rows |
| accent | `#0091FF` | `#2B8AD6` | `#1677BD` | Active tool, selection, primary button |
| rule | `#282F3B` | `#4A668C` | `#AAB4C0` | 1 px separators and outlines |
| text | `#F0F4F8` | `#D4D4D4` | `#20252B` | Primary text |
| muted | `#8B98A5` | `#9A9A9A` | `#586474` | Secondary text, hotkeys, micro-labels |
| link | `#63B3ED` | `#63B3ED` | `#1677BD` | Text buttons on panels |
| flag | `#FC8181` | `#FC8181` | `#C53030` | "Needs a type", destructive actions |
| tape | `#F6AD55` | `#F6AD55` | `#F6AD55` | Length strip, Pad readout, live previews only |

The Dark drawing background is `#0F141A` with grid rules in `#282F3B` (minor lines at half
strength) and dimension text in `#F0F4F8`.

Tokens live in `ui/tokens.rs` and read the palettes in `theme.rs`. The Dark values are
pinned by `tokens::tests::dark_theme_uses_the_cad_palette`.

On the drawing, a selected area gets a 7% purple (`#A855F7`) wash and a dashed purple
outline above its walls. Selected walls are outlined in purple. Drag the handles at wall
endpoints or area corners to reshape the selection.

Area colours are data and come from each area type. Each floor of living area keeps its
own default colour (First `#2B6CB0`, Second `#0987A0`, Third `#B83280`, Fourth
`#C53030`); the owner chose to keep these rather than one blue for all living area.

## 3. Icons

Phosphor icons (MIT), embedded from `assets/icons/`. Icons carry a tint by meaning when
at rest. These are the classic ribbon and tool-rail colours, restored at the owner's
request.

| Tint | Dark and Blue | Light | Meaning | Examples |
|---|---|---|---|---|
| Blue | `#63B3ED` | `#1D6FB8` | Select, view, navigate, information | Select, Wall, Pan, zoom, grid, compass, undo, Property details |
| Orange | `#F6AD55` | `#C05621` | Draw, edit, measure, annotate | Inner, Shape, Curve, Measure, Label, Insert, copy, calculator |
| Red | `#FC8181` | `#C53030` | Remove, hide, the Area tool | Area, Delete, trash, hide, Close window |
| Green | `#5DAE84` | `#2F7D4F` | Files, add, export, print | New, Open, Save, Export, Print, add page |
| Slate | `#A0AEC0` | `#4A5568` | Devices, locks, history | Laser, lock, Recent, Undo history |

Rules:

- Structural glyphs stay in the text colour: carets, dots, the menu lines, close, plain
  arrows, check marks, plus and minus.
- The active tool's icon is white on the accent fill. Disabled icons are muted. State
  always reads before meaning.
- The mapping is one function, `icons::tint_of`. Draw a tinted icon with
  `icons::tinted`. Every command's icon is `commands::icon_for`.

## 4. Type

- Interface text: Barlow 400, 500, 600 at 11 to 16 px.
- Headings, numbers, codes and shortcuts: Barlow Semi Condensed 500 and 600.
- Numbers in columns are right aligned with tabular figures.
- Text drawn on the plan (dimensions, labels) uses the fonts chosen in Options > Fonts.
- Areas and lengths show one decimal everywhere (100.0 sf, 10.0'). Factors show two.

## 5. Density and spacing

Two densities. Touch turns on with Options > Display > "Controls sized for: Touch", or
after the first touch in a session.

| Item | Mouse | Touch |
|---|---|---|
| Top bar height | 44 | 56 |
| List row | 36 | 48 |
| Field and button | 32 | 44 |
| Tray button | 62 x 58 | 68 x 68 |
| Smallest hit target | 28 | 48 |
| Ledger width | 264 | 300 |
| Inspector width | 280 | 320 |
| Icon | 20 | 22 |

Spacing steps: 4, 8, 12, 16, 24. Corner radius: 6 for controls, 8 for tiles, 10 for
pop-ups and dialogs, 14 for the tray, 16 for sheets. Floating parts have a soft shadow
and a 1 px rule outline.

## 6. Window sizes

| Class | Width | Layout |
|---|---|---|
| Wide | 1100 and up | One-row top bar. Ledger docked left, inspector docked right while something is selected, tray floating at the bottom of the drawing. |
| Medium | 700 to 1099 | Two-row top bar (page tabs on row 2). Ledger and inspector slide over the drawing. |
| Compact | below 700 | Two-row top bar, tool bar fixed to the bottom edge, ledger and inspector as bottom sheets, dialogs as full-screen pages with a back arrow. |

## 7. Regions

### Top bar

App mark, Menu button, document name (click to rename), save state, page tabs (centred,
with a ⋯ menu per tab and "+" to add a page), search field ("Search commands", Ctrl K),
undo and redo, "Send to report" (Appraise-It Pro, Windows), export, and the window
controls. The bar is the window's drag area.

### Menu

Opened from the Menu button, Ctrl K, the top bar's search field or the phone's
magnifier. One column with a search field (focused on open) and 19 rows in four
sections, each row with its tinted icon:

1. **The file:** New sketch, Open, Recent ›, Save, Save as, Import ›, Export ›, Print ›
2. **This sketch:** Property details, Area types, Area breakdown, Trace image ›, Symbol library
3. **Editing:** Undo history, Select ›, Transform sketch ›
4. **The app:** Options, Help ›, Close window

Fly-outs (›) open beside their row on hover, click or the Right arrow:

| Fly-out | Rows |
|---|---|
| Recent | Recent files, Show start screen, Recover autosave, Clear recent sketches |
| Import | Sketch file, CAD drawing (DXF), Layer from another sketch |
| Export | PDF, PNG image, SVG drawing, DXF for CAD |
| Print | Print, Print preview, Page layout |
| Trace image | Load, Show, Hide, Edit image, Remove, Load map area, Search address, Jump to coordinates |
| Select | Everything, Areas, Freeform lines, Text labels, Dimensions, Symbols |
| Transform sketch | Flip horizontally, Flip vertically, Rotate +90°, Rotate −90° |
| Help | User guide, Keyboard shortcuts, Appraise-It guide, Email support, About Sketch Rust |

Typing in the search field replaces the rows with every matching command in the app
(160), each with its group and shortcut. An empty result says "No matching command."

Keys: Up and Down move, Enter runs, Right opens a fly-out, Left closes it (while search
text is typed, Left and Right move the text cursor instead), Escape closes the Menu,
Ctrl K again closes it.

On a phone the Menu is a full-height sheet with 48 px rows; a fly-out replaces the list
and adds a Back row. The rows are `ENTRIES` in `ui/menu.rs`.

### Tool tray

Ten tools, floating at the bottom of the drawing (Wide, Medium) or fixed to the bottom
edge (Compact). Each button is an icon over an 11 px label. A small corner mark means the
button has a fly-out (open with press-and-hold, right-click, or clicking the active tool
again).

| Tool | Icon tint | Fly-out |
|---|---|---|
| Select | Blue | Fence, Lasso |
| Wall | Blue | Line, Rectangle, Circle, Polygon |
| Inner | Orange | Line, Rectangle, Circle, Polygon |
| Area | Red | none |
| Shape | Orange | none |
| Curve | Orange | Insert vertex |
| Measure | Orange | Chain dimension |
| Label | Orange | none (opens the Insert drawer on Room labels) |
| Insert | Orange | none (opens the Insert drawer on Symbols) |
| Pan | Blue | Zoom to rectangle |

### Ledger

Left panel titled "Areas" with a Drawn / Types switch and a ⋯ menu (expand and collapse,
find areas that need a type, detect closed areas, label defaults, auto-subtract).

- **Drawn:** areas on this page grouped by type group, with group totals. Untyped areas
  are listed under "Needs a type" in the flag colour. While a wall chain is open, a row
  reads "Drawing an area" with its wall count and length.
- **Types:** every area type (32 built in plus the user's) with search, swatches, codes
  and use counts. Tap to arm a type, drag onto an outline to assign it.
- **Layers:** a collapsible section (a tab on the phone).
- **Totals foot:** gross living area in large type, then building area, not counted, needs
  a type, and "Show the math" (opens the area breakdown).

### Inspector

Right panel with one scrollable view for the active selection or trace image. Object
properties and edit actions appear together in labeled sections. The wall length field
accepts decimal feet, inches with a unit marker, and feet-and-inches notation. Move a
trace image on the canvas from its inspector alongside size, rotation, and intensity.
The Area Properties command opens the inspector when only areas are selected.

### Insert drawer

Symbols (category chips, search, tile grid, favourites, "Edit library").
Categories have plain names (Doors & windows, Kitchen, Bath & plumbing…); the
CADdillo set is merged into them and 67 older duplicates are retired, per
`tools/symbol-curation.json`. Retired symbols are deleted; `aliases` in
`symbols.json` maps each to its replacement, which takes its place when an old
sketch opens.

Room labels (search that also adds a label, letter filter, list, "Edit list"). A pin
keeps the drawer open after placing.

### Drawing controls

Floating groups on the drawing: zoom (minus, percent with a slider and 1:N scale field,
plus, magnifier for zooming to a dragged rectangle, Fit), snapping and display
(Grid, Snap with pitch, Show filters, Dimensions outside / inside / hidden), and North.
In Touch density, Pad and Laser buttons join them.

### Length strip

Amber strip beside the wall being drawn: typed length, angle, pen state and a hint line.
For Measure it shows leg and total.

### Pad

Number and direction pad for touch drawing: turn left and right, four arrows, backspace,
laser, digits, feet mark and decimal point, and a full-width "Close area".

### Area chooser

"What is this area?" opens when an outline closes (if Options says so), when the Area
tool taps an outline, and on double-tap of an area. Search, group chips, the list of
types, optional name, label switches, "Use for the next areas too". The type list
scrolls; the rest fits.

### Start screen

Shown at launch with nothing open: Start a sketch (New sketch, Open a file, Import a CAD
drawing, Continue from a photo or plan), Appraise-It Pro status with "Draw the sketch for
this report" when a report is open, and recent sketches.

### Toasts

Short confirmations at the bottom centre for 3.2 seconds.

## 8. Dialogs

- **Frame:** title and optional subtitle, close button, body, footer with the primary
  action on the right. Compact shows dialogs as full-screen pages with a back arrow.
- **Size:** in Wide, dialogs are capped at 1280 x 960 and centred with a 32 px margin;
  in Medium they use the window less 24 px margins.
- **Fit rule:** at 1920 x 1080 and up, no form scrolls. A list of the user's own data
  (undo history, room labels, laser devices, the breakdown report, guide articles, symbol
  and type lists, the chooser's type list) may scroll inside its own box. Those scroll
  areas have ids ending in `-list`. Below 1920 x 1080 a form may fall back to scrolling
  rather than being cut off.
- **Checked by** `e2e_next::dialogs_fit_1920x1080_without_scrolling`. It opens every
  dialog and tab at 1920 x 1080 and fails if any other scroll area overflows. Screenshots
  go to `target/e2e-next/fit-1920/`.

Dialogs sized for that rule:

| Dialog | Size (Wide) | Layout |
|---|---|---|
| Options | 840 x 840 | Tabs down the left: General, Display (with Appearance), Hardware, System, Fonts, Drawing |
| Property Details | 900 x 760 | Parcel Info in three columns over five rows; General Comments; Comment Tables |
| Page Layout | 900 x 640 | Paper size, form and options on the left; the seven captions on the right |
| Keyboard Shortcuts | 1200 x 860 | Seven groups in three balanced columns |

## 9. Where commands live

All 160 commands are in `ui/commands.rs` and all are found by search. Their on-screen
homes:

| Home | Count |
|---|---|
| Menu (rows and fly-outs) | 49 |
| Inspector or right-click menu | 29 |
| Drawing controls (including snap pitches and Show filters) | 24 |
| Tool tray | 15 |
| Options | 14 |
| Keyboard shortcut only | 8 |
| Page tabs | 8 |
| Ledger, drawers and editors | 8 |
| Top bar | 5 |

`coverage.csv` records each feature's home.

## 10. Words

| Say | Not |
|---|---|
| Top bar, Menu, Start screen | Ribbon, backstage, quick-access toolbar |
| Tool tray | Tool rail |
| Ledger: Drawn, Types, Layers | Dock |
| Insert drawer | Symbols / Text tabs |
| Inspector | Edit Tools |
| Drawing controls | Status bar |
| Area type ("Code" for the short code) | Area code |
| Totals; "Show the math" | Calcs |
| Property details | Subject info |
| Pad | Touch panel |
| Laser, "Connect laser" | Disto, Bluetooth |
| Trace image | Reference image |
| Symbol library | Manage symbols |
| Room labels | Text library |
| Length strip | Segment HUD |

## 11. Not built yet

Proposed in the interface review and not built:

- Area chooser: group headers in the type list, "Used in this sketch" first with number
  keys, one row of chips, the outline and size at the top.
- Pad: one readout for length and pen state, eight-direction compass, feet and inches
  keys, Undo wall key.
- Tray: group dividers, Label folded into Insert, fly-out mark moved to the bottom-right
  corner.
- Ledger: Layers as a third tab on desktop, "Pick type" on untyped rows, totals foot
  hidden until there are areas.
- Inspector: fields that save on Enter or blur (no Apply or Set buttons), one Type control
  instead of a pick list plus Chooser, net area and perimeter first.
- Phone: tool bar pinned to the bottom with a summary sheet above it, drawing controls
  collapsed to three icons, Pad replacing the tool bar while drawing.
- Start screen: no outer frame, one primary action, no "Blank canvas", formats named on
  each tile.
- Toasts: drawn above dialogs; they should wait while a dialog or sheet is open.
- Section headings in capitals in a few places (ledger "LAYERS", inspector headings,
  the kit gallery); the rule is sentence case.
