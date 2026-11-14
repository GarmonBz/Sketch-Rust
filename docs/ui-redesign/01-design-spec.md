# 01. Design specification

`mockups.html` shows the approved look. This file gives the numbers. Where the two differ,
this file wins.

## 1. Colour tokens

Keep the three palettes that exist in `crates/sketch-app/src/theme.rs` (`DARK`, `BLUE`,
`LIGHT`) and their accessor functions. Add the tokens marked "new".

| Token | Accessor | Dark | Blue | Light | Used for |
|---|---|---|---|---|---|
| bar | `bg_ribbon()` | `#2D2D30` | `#263F66` | `#DDE3EA` | Top bar, pop-ups, sheets, tray |
| panel | `bg_panel()` | `#252526` | `#1E3150` | `#F7F8FA` | Ledger, inspector, dialogs |
| field | `bg_app()` | `#1E1E1E` | `#17243A` | `#E9EDF2` (use `#FFFFFF` for inputs in Light) | Inputs, active tab, selected tile |
| hover | `bg_hover()` | `#3E3E42` | `#34547F` | `#CFD8E3` | Hovered and pressed rows and buttons |
| accent | `accent()` | `#007ACC` | `#2B8AD6` | `#1677BD` | Active tool, selection, primary button |
| rule | `border()` | `#3F3F46` | `#4A668C` | `#AAB4C0` | 1px separators and outlines |
| text | `text_alt()` | `#D4D4D4` | `#D4D4D4` | `#20252B` | Primary text |
| muted | `muted_alt()` | `#9A9A9A` | `#9A9A9A` | `#586474` | Secondary text |
| canvas | `bg_canvas()` | `#111111` | `#0B1728` | `#FFFFFF` | Drawing surface (independent setting, see `canvas_theme.rs`) |
| **link** (new) | `link()` | `#63B3ED` | `#63B3ED` | `#1677BD` | Text buttons on panels |
| **tape** (new) | `tape()` | `#F6AD55` | `#F6AD55` | `#F6AD55` | Length strip and placement previews only |
| **tape_ink** (new) | `tape_ink()` | `#1E1E1E` | `#1E1E1E` | `#1E1E1E` | Text on the tape colour |
| **flag** (new) | `flag()` | `#FC8181` | `#FC8181` | `#C53030` | "Needs a type", destructive text |
| **on_accent** (new) | `on_accent()` | `#FFFFFF` | `#FFFFFF` | `#FFFFFF` | Text and icons on accent |

Rules:

- Icons are one colour: `text` normally, `on_accent` on the active tool, `muted` when
  disabled. Do not carry over the per-icon tints (`ICON_BLUE`, `ICON_ORANGE`, `ICON_RED`,
  `ICON_SLATE`) into new chrome.
- Amber (`tape`) appears only on the length strip, the Pad readout and the live preview
  of something being placed. It is never a button colour.
- Area colours are data and come from each area type.
- The canvas background stays a separate user setting (White, Dark, Blue).

## 2. Type

Two families, both SIL Open Font License, bundled in `assets/fonts/` and loaded at boot.

| Role | Family | Weight | Sizes (px) |
|---|---|---|---|
| Interface text | Barlow | 400, 500, 600 | 11, 12, 13, 14, 15, 16 |
| Headings, numbers, dimensions, codes | Barlow Semi Condensed | 500, 600 | 12, 13, 14, 16, 18, 20, 24, 32 |

- Sentence case everywhere. No all-caps labels.
- Numbers that line up in columns use Barlow Semi Condensed, right aligned.
- Fallback when the fonts fail to load: Segoe UI on Windows, the system sans elsewhere.
- Text drawn on the sketch itself (dimensions, labels) keeps using the fonts chosen in
  Options. This spec covers chrome only.
- Remove the double-ink pass (`theme::ink`, `INK_PASSES`) from new chrome. It imitated a
  browser's rendering.

## 3. Density

Two densities. All sizes below are logical pixels.

| Item | Comfortable (mouse) | Touch |
|---|---|---|
| Top bar height | 44 | 56 |
| Row height (lists) | 36 | 48 |
| Field and button height | 32 | 44 |
| Tray button (w x h) | 62 x 58 | 66 to 72 x 68 |
| Minimum hit target | 28 | 48 |
| Ledger width | 264 (drag 240 to 400) | 300 |
| Inspector width | 280 | 320 |
| Icon size | 20 | 20 to 24 |

Density is Touch when the platform is Android, when Options > General > Input mode is
"Touch", or after the first touch event in a session. Otherwise Comfortable.

Shared constants: spacing 4, 8, 12, 16, 24. Corner radius 6 for controls, 8 for tiles,
10 for pop-ups and dialogs, 14 to 16 for the tray and sheets. Separators are 1px `rule`.
Floating elements (tray, pop-ups, Pad, slide-overs) have a soft shadow and a 1px `rule`
outline.

## 4. Size classes

Chosen from the window's logical width on every resize.

| Class | Width | Typical device |
|---|---|---|
| Wide | 1100 and up | Desktop, landscape tablet |
| Medium | 700 to 1099 | Portrait tablet, small window |
| Compact | below 700 | Phone |

| Part | Wide | Medium | Compact |
|---|---|---|---|
| Top bar | One row: mark, menu, name, save state, page tabs centred, search, undo, redo, primary button | Row 1: mark, menu, name, undo, redo, search, overflow. Row 2 (44 to 47 high): page chips, totals at right | Same as Medium |
| Ledger | Left side panel | Slide-over from the left, over the drawing | Bottom sheet with tabs Drawn, Types, Totals |
| Inspector | Right side panel, only while something is selected | Slide-over from the right | Bottom sheet; replaces the tool bar while open |
| Insert drawer | Floats above the tray, 520 wide | Same | Full-screen page |
| Tool tray | Floating, bottom centre of the drawing | Same, full row | Fixed bar on the bottom edge, scrolls sideways |
| Pad | Hidden unless Touch density; then floats bottom right | Floats bottom right | Docked along the bottom, above the tool bar's place |
| Dialogs | Centred, max 1080 x 740 | Centred, margins 24 | Full-screen page with a back arrow |

Slide-overs and sheets close on: their close control, tapping the drawing, Escape, or
starting to draw. They never cover the tool tray.

## 5. Regions

### 5.1 Top bar

Left to right in Wide:

1. App mark, 20 x 20 (24 Touch), accent square with a corner notch. Not a button.
2. Menu button (three lines). Opens the Menu (section 5.9).
3. Document name, 14px 600. Click to rename (runs Save As when untitled).
4. Save state, 12px muted: "Not saved", "Saved", "Edited".
5. Page tabs, centred: one tab per page, the current one filled with `field`. "+" adds a
   page. Press-and-hold or right-click a tab for: Rename, Delete, Use as underlay, Move
   first, Move last. More than fits: the strip scrolls and gains a pages list button.
6. Command search field, 204 wide, placeholder "Search commands", hint "Ctrl K".
   In Medium and Compact it is a magnifier button.
7. Undo, Redo (always visible in Touch density; in Comfortable they live in the Menu and
   keep their shortcuts, with a small pair of buttons left of the search field).
   Press-and-hold or right-click Undo opens Undo history.
8. Primary button: "Send to report" on Windows, opening the existing Appraise-It report
   flow (today it lives inside the guide dialog, `guide.rs`: check report, preview,
   include areas, send). Elsewhere, and when the owner prefers, "Export" with a menu of
   PNG, SVG, DXF, PDF.
9. Window controls (minimise, maximise, close) on desktop only.

The bar is the window drag area on desktop.

### 5.2 Tool tray

Ten buttons, in this order. A small corner mark shows a button has a fly-out. Open the
fly-out by press-and-hold, right-click, or clicking the active tool again.

| Button | Does | Fly-out |
|---|---|---|
| Select | Select and edit | Fence select, Lasso select |
| Wall | Draw exterior walls | Line, Rectangle, Circle, Polygon |
| Inner | Draw interior lines | Line, Rectangle, Circle, Polygon |
| Area | Name areas: tap inside an outline | none |
| Shape | Insert a room shape (opens its form) | none |
| Curve | Bow a wall | Insert vertex |
| Measure | Measure runs | Chain dimension |
| Label | Opens the Insert drawer on Room labels | none |
| Insert | Opens the Insert drawer on Symbols | none |
| Pan | Drag to move the view | Zoom to rectangle |

Active tool: accent fill, `on_accent` icon and label. Each button shows its icon above a
11 to 12px label. Tooltips give the name and shortcut.

### 5.3 Ledger

Header: title "Areas", a two-way switch "Drawn | Types", and a "..." menu.

**Drawn view** (default): the areas on this page, grouped by type group.

- Group header row: group name, group net total at right. Tap to collapse.
- Area row: colour swatch, name, net area right aligned. Tap selects the area on the
  drawing. Tap again, or the chevron, expands it to show Base area, Factor, Total,
  Subtracted, Net area, Perimeter.
- Areas with type `UND` are listed under "Needs a type" in `flag` colour.
- While a wall chain is open, a dashed row reads "Drawing an area" with wall count and
  total length.
- "..." menu: Expand all, Collapse all, Find areas that need a type, Detect regions,
  Label defaults, Area label suffix.

**Types view**: every area type, grouped with today's headings (Living space, Building
area, Garage / carport, Basement, Outdoor, Storage, Land, Site, Non-calculated,
Undefined; see `05-area-types.md`). All 32 built-ins
plus user types. Search field at top.

- Type row: swatch, name, code at right, count of areas using it when above zero.
- Tap arms the type ("First Floor is ready. Tap an outline."). Tap again disarms.
- Drag a row onto an outline to assign it.
- Press-and-hold or right-click: Assign to N selected areas, Select the N areas using it,
  Edit type, Duplicate type, New type, Delete type.
- "Undefined" row clears a type when assigned.
- Footer link "Edit types" opens the area type editor.

**Layers** (a collapsible section under the list in Wide, a tab in sheets): the existing
layer tree with show, lock, rename, drag to reorder, groups, and Export on each row.

**Totals foot** (pinned at the bottom): "Gross living area" with the number at 32px,
"Building area", "Not counted", and "Show the math" (opens the area breakdown).

### 5.4 Inspector

Appears when the selection is not empty. Title is the selected thing. Content depends on
what is selected. Every control of the classic Edit Tools tab must appear here.

| Selection | Controls |
|---|---|
| Wall | Length (typed, with "start" or "end" handle note), Set length, Line style (Solid, Dashed, Dotted), Line colour, Curve (bow amount with stepper, Degree or Height mode), Insert vertex, Toggle dimension |
| Area | Name, Type (opens chooser), Size and dimensions, Counts toward (from its type), Factor, Sign (Adds or Subtracts), Auto-subtract, Deduct from, Fill pattern, Fill colour, Line style, Line thickness, Line colour, Label options, Attributes, Flip horizontal, Flip vertical, Rotate, Clone, Reopen as walls, Walls list, Delete |
| Text label | Text, Font, Size with grow and shrink, Bold, Italic, Underline, Colour, Rotate |
| Symbol | Name, Rotate, Size stepper, and for wall openings: Flip swing, Wider, Narrower |
| Chain dimension | Side, Offset, Delete |
| Trace image in edit mode | Show, Hide, Remove, Set scale, Smaller, Larger, Rotate left, Rotate right, Flip horizontal, Flip vertical, Align guide, Intensity, Map options |
| Several kinds at once | Common actions only: Cut, Copy, Delete, Rotate, Scale, Nudge, Bring to front, Send to back |

### 5.5 Insert drawer

Two modes sharing one drawer.

**Symbols**: category tabs with counts (Favorites first, then all categories, last item
"All N"), search, a grid of tiles (preview, name). Tap a tile to arm placement; the
drawer closes unless pinned. Drag a tile onto the drawing to place it. Footer: placement
hint and "Edit library". Press-and-hold or right-click a tile: Place, Add to or Remove
from favorites, Rename, Hide, Delete, Open in library.

**Room labels**: search field that also accepts a new label ("Enter places it"), letter
filter (All, ABC, DEF, GHI, JKL, MNO, PQR, STU, VWX, YZ), the list of labels, "Edit list".

A pin control keeps the drawer open after placing.

### 5.6 Length strip

Shown while the Wall, Inner or Measure tool is active. It sits just past the end of the
wall being drawn and never under the pointer. Amber background, tick marks along the top
edge like a tape measure.

Contents: typed length (24px 600), angle, pen state ("pen up" or "pen down"). Below it,
one muted hint line. For Measure it shows leg and total and offers Finish, Close, Undo
point, Clear, Copy.

### 5.7 Drawing controls

Three small floating groups. In Wide they sit along the bottom of the drawing, left and
right of the tray. In Medium and Touch they move to the top corners.

- Zoom: minus, percent (opens a slider 10 to 500 and "1:N" scale field), plus, Fit.
  Press-and-hold Fit: Centre, Centre and fit, Zoom to rectangle.
- Snapping: Grid on or off, Snap on or off with pitch (1', 1/2', 1/4', 1"), Area snap,
  angle step. A "Show" button opens the six switches: Area fills, Exterior walls,
  Interior lines, Dimensions, Text labels, Symbols. A "Dimensions" button cycles Outside,
  Inside, Hidden.
- North: compass button showing the angle. Tap to type the angle.

Also on the drawing in Touch density: a Pad button and a Laser button.

### 5.8 Area chooser

Opens on the area when an outline closes (if Options > Display on Close is on), when the
Area tool taps an outline, and on double-tap of an area or its label.

Title "What is this area?". Search field with focus. Group filter chips. A list of
matching types showing name and code. Below: optional name, label switches (Code, Name,
Dimensions, Calcs, Base, Factor), suffix, "Use for the next areas too". Enter applies the
first match. Escape leaves it as Undefined. Must be able to reach all types by scrolling
with an empty search.

### 5.9 Menu

Opened from the top bar. A column of groups; in Wide it is a drop-down with sub-menus, in
Compact a full-height sheet. The exact entries are in `04-feature-map.md`. Groups:
File, Edit, View, Areas, Insert, Image, Tools, Help.

### 5.10 Start screen

Shown at launch when no file was passed and nothing is drawn. Left column: "Start a
sketch" with New sketch (primary), Open a file, Import a CAD drawing, Continue from a
photo or plan. Below: Appraise-It Pro status and "Draw the sketch for this report" when a
report is open. Right: recent sketches as cards (outline thumbnail, name, living area,
last edited), with Clear recent. Dropping a file anywhere opens it.

### 5.11 Pad

See `07-touch-and-android.md` for behaviour. Layout: 4 columns x 5 rows of keys
(54 x 52, gap 6) and a full-width "Close area" button.

```
Turn L   Up     Turn R   Backspace
Left     Down   Right    Laser
7        8      9        '
4        5      6        .
1        2      3        0
[            Close area            ]
```

## 6. Wireframes

Wide (1400 x 860 reference):

```
+----------------------------------------------------------------------------------+
| [m] = Name  Saved        [Main floor][Upper][Site][+]     [Search  Ctrl K] [Send] |
+---------------+------------------------------------------------------+-----------+
| Areas [D|T] . |                                                  (N) | Inspector |
| Living space  |                                                      | (only if  |
|  # Main  1224 |                 drawing                              | selected) |
| Garage        |                                                      |           |
|  # Garage 484 |                 [length strip]                       |           |
| Layers      > |                                                      |           |
|---------------|  [zoom]  [ Select Wall Inner Area ... Pan ]  [snap]  |           |
| GLA 1,224 sf  |                                                      |           |
+---------------+------------------------------------------------------+-----------+
```

Medium (800 x 1280, portrait tablet):

```
+------------------------------------------------+
| [m] = Name                    undo redo  Q  :  |
| (Main floor) Upper  Site  +        GLA 1,224   |
+------------------------------------------------+
| [zoom]                                     (N) |
|+-----------+                    +-------------+|
|| Ledger    |     drawing        | Inspector   ||
|| slide-over|                    | slide-over  ||
|+-----------+                    +-------------+|
|                                        [ Pad ] |
|   [ Select Wall Inner Area Shape ... Pan ]     |
+------------------------------------------------+
```

Compact (372 x 800, phone):

```
+----------------------------+
| = Name          undo redo :|
| (Main) Upper Site +  1,224 |
+----------------------------+
|          drawing           |
+----------------------------+
| Select Wall Inner Area >   |  tool bar, scrolls
+----------------------------+
| [ 16' 0"      90 pen down ]|  Pad while drawing,
| 7 8 9 | TurnL Up TurnR     |  or the sheet
| 4 5 6 | Left Pen Right     |  (Drawn | Types | Totals)
| 1 2 3 | <x   Down Laser    |
| ' 0 . | [  Close area  ]   |
+----------------------------+
```

## 7. Interface copy

- Buttons say what happens: "Save", "Place", "Delete type", "Close area".
- Empty states tell the user what to do: "No areas yet. Draw walls and close them to make
  an area."
- Errors state the problem and the fix. Keep the existing wording of file and validation
  errors.
- Toasts stay (bottom centre, 3.2 seconds, `toast.rs`).
