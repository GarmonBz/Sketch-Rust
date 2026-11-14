# 04. Feature map

The rule: **nothing is dropped**. `refs/ui-spec/feature-inventory.md` lists 634
user-reachable features in 12 groups. This file says where each group goes, lists every
command, and explains how to close each row in `coverage.csv`.

## 1. Groups and their new homes

| Inventory group (IDs) | Rows | New home |
|---|---|---|
| S1 Title bar and static chrome | 15 | Top bar. Quick-access Save, Undo, Redo, History, Print become Menu commands plus the bar's Undo and Redo. |
| S2 Ribbon, File menu, Edit Tools | 19 | Menu; Inspector; View commands for canvas background and interface theme. |
| S3 Dock | 20 | Ledger (Drawn, Types, Layers), Insert drawer (Symbols, Room labels). Pin, collapse and resize become the drawer pin and the ledger's collapse and width handle. |
| S4 Status bar | 36 | Drawing controls. Totals chip becomes the ledger foot. Defined-areas select becomes the Drawn list. Page controls move to page tabs. Underlay moves to the page tab menu. Bluetooth and Touch Panel become Laser and Pad buttons. |
| S5 Drawing | 64 | Wall and Inner tools with shape fly-out; Length strip; all keyboard entry forms unchanged; Draw colour in Menu > Tools and in the inspector. |
| S6 Select and edit | 88 | Select tool with Fence and Lasso; Curve tool with Insert vertex; Inspector; Edit menu; press-and-hold menu. |
| S7 Areas | 79 | Area tool; chooser; Ledger Types and Drawn; Inspector Area section; area type editor; breakdown dialog. |
| S8 Symbols | 45 | Insert tool and drawer; Symbol library; Inspector Symbol section. |
| S9 Labels, dimensions, shapes, measure | 46 | Label tool and drawer; Shape tool; Measure tool with Chain dimension; Inspector Text section. |
| S10 Pages, layers, trace image, geo | 57 | Page tabs; Ledger Layers; Image menu and Inspector Trace image section. |
| S11 Files, print, options | 81 | Menu File and Tools; Start screen; Options, Page layout and Print preview dialogs. |
| S12 Pad, laser, search, other dialogs | 84 | Pad; Laser; command search; Property details; Mini map; Shortcut sheet; Email; toasts and confirmations. |

Rail tool mapping (17 to 10):

| Classic rail tool | New place |
|---|---|
| Select | Tray: Select |
| Fence, Lasso | Select fly-out |
| Pan | Tray: Pan |
| Draw Exterior, Draw Interior | Tray: Wall, Inner (shape fly-out on each) |
| Insert Vertex | Curve fly-out |
| Curve | Tray: Curve |
| Measure | Tray: Measure |
| Chain Dimensions | Measure fly-out |
| Shapes | Tray: Shape |
| Define Area | Tray: Area |
| Text | Tray: Label |
| Symbols | Tray: Insert |
| Touch | Pad button on the drawing; Menu > View > Pad |
| Geo | Menu > Image > Map; Inspector Trace image section |
| Pages | Page tabs; "All pages" in the tab menu |

## 2. Command registry contents

Every command below must exist in `ui/commands.rs`, appear in the Menu under its group,
and be found by command search. Ids that exist in `command_palette::CATALOG` are reused.
"Message" names the existing message to emit.

### File

| Id | Label | Shortcut | Message |
|---|---|---|---|
| new | New sketch | Ctrl+N | `New` |
| open | Open | Ctrl+O, F3 | `OpenFile` |
| save | Save | Ctrl+S, F2 | `SaveFile` |
| save-as | Save as | F12, Ctrl+F12 | `SaveFileAs` |
| import | Import a sketch page | | `ImportSketch` |
| import-cad | Import a CAD drawing | | `ImportCad` |
| import-layer | Import a layer | | `ImportLayer` |
| export-png | Export as PNG | | `ExportPng` |
| export-svg | Export as SVG | | `ExportSvg` |
| export-dxf | Export as DXF | | `ExportDxf` |
| export-pdf | Export as PDF | | `ExportPdf` |
| print | Print | Ctrl+P | `Print` |
| print-preview | Print preview | | `PrintPreviewOpen` |
| page-layout | Page layout | | `PageLayout(LayoutMsg::Open)` |
| recent | Recent sketches (sub-menu) | | `RecentOpen(i)`, `RecentClear` |
| report-open | Appraise-It reports | | `Guide(GuideMsg::Open(topic))` with the Appraise-It topic index (see the constant near the top of `guide.rs`) |
| report-check | Check open report | | `Guide(GuideMsg::CheckReport)` (Windows only, `#[cfg(windows)]`) |
| report-preview | Preview sketch for report | | `Guide(GuideMsg::PreviewSketch)` (Windows only) |
| report-areas | Include area totals | | `Guide(GuideMsg::IncludeAreas(bool))` (Windows only) |
| report-send | Send sketch images | | `Guide(GuideMsg::SendSketch)` (Windows only) |
| exit | Exit | | `Close` |

### Edit

| Id | Label | Shortcut | Message |
|---|---|---|---|
| undo | Undo | Ctrl+Z | `Undo` |
| redo | Redo | Ctrl+Y | `Redo` |
| undo-history | Undo history | | `UndoHistory` |
| cut | Cut | Ctrl+X | `Cut` |
| copy | Copy | Ctrl+C | `Copy` |
| paste | Paste | Ctrl+V | `Paste` |
| delete-selected | Delete | Del | `Delete` |
| select-all | Select all | Ctrl+A | `SelectAll` |
| select-areas, select-text, select-dimensions, select-symbols, select-lines | Select all areas / labels / dimensions / symbols / freeform lines | | `SelectByType(kind)` |
| rotate-selected-5, rotate-selected-1 | Rotate selection 5 / 1 degree | Shift+R, Ctrl+Shift+R | `RotateSelected(d)` |
| orient-h, orient-v | Make horizontal / vertical | H, V | `Key(KeyCmd::OrientH / OrientV)` |
| scale-up, scale-down | Larger / Smaller | +, - | `ScaleSelected(1.1 / 0.9)` |
| nudge | Nudge (arrow keys) | arrows | `Key(KeyCmd::Nudge(..))` |
| bring-front, send-back | Bring to front / Send to back | | `LayerBringToFront`, `LayerSendToBack` |
| rotate-sketch | Rotate sketch 90 degrees | | `RotateSketch` |
| rotate-sketch-left, rotate-sketch-right | Rotate sketch left / right | L, R | `RotateSketchBy(-90 / 90)` |
| flip-sketch-h, flip-sketch-v | Flip sketch horizontal / vertical | F, M | `PopupMenu(PopupMsg::Flip(axis))` |

The shortcut column is display text. The real key handling is `input::key_down` and the
full list is `shortcuts::GROUPS`. Copy every entry from there, including: E (Wall),
I or F5 (Inner), P (Select), Insert (Insert vertex), A (Close area), B (Bay mirror),
J (Jump to endpoint), D (Curve by degree or height), G (Grid), C (Centre),
Shift+C (Centre and fit), S (Symbols), T (Room labels), F4 (Area types list),
F6 (Totals), F1 (Shortcut sheet), Ctrl+K (Command search), Ctrl+F1 (Email support),
Space tap (Toggle dimension), Space hold (Pan), and the typed-length forms
`10` + arrow, `10x5`, `10<45`, `10l`, `10r`, Tab for a decimal point.

### View

| Id | Label | Message |
|---|---|---|
| zoom-in, zoom-out | Zoom in / out | `Status(ZoomIn / ZoomOut)` |
| center | Centre sketch | `Status(CenterSketch)` |
| center-fit | Centre and fit | `Status(CenterFit)` |
| zoom-fence | Zoom to rectangle | `Status(ZoomFence)` |
| toggle-grid | Grid | `Status(ToggleGrid)` |
| toggle-snap | Snap | `Status(ToggleSnap)` |
| toggle-area-snap | Area snap | `Status(ToggleAreaSnap)` |
| snap-pitch | Snap pitch (1', 1/2', 1/4', 1") | `Status(Pitch(..))` |
| filter | Show in sketch (six switches) | `Status(ToggleFilter)`, `Status(Filter(i))` |
| cycle-dimensions | Dimensions: outside, inside, hidden | `Status(CycleDimensions)` |
| north | North direction | `Status(North)` |
| mini-map | Mini map | `ToggleMiniMap` |
| toggle-canvas-background | Drawing background (White, Dark, Blue) | `PopupMenu(PopupMsg::Canvas(..))` |
| interface-theme | Interface theme (Light, Dark, Blue) | `PopupMenu(PopupMsg::Interface(..))` |
| toggle-pad | Pad | `TouchPanelToggle` |
| density | Touch-sized controls | `Ui(SetDensity(..))` |
| page-prev, page-next, page-first, page-last, page-add, page-delete | Pages | `Status(Page..)` |
| underlay | Underlay page | `Status(Underlay(..))` |
| wall-solids | Show wall thickness | `WallSolids(bool)` |

### Areas

| Id | Label | Message |
|---|---|---|
| area-tool | Name areas | `Rail(DefineArea)` |
| new-area | New area | `AreaWorkflow(Msg::NewArea)` |
| find-undefined | Find areas that need a type | `AreaWorkflow(Msg::FindUndefined)` |
| detect-regions | Detect regions | `AreaWorkflow(Msg::Detect)` |
| redefine | Apply armed type to selection | `AreaWorkflow(Msg::Redefine)` |
| clone-area | Clone area | `AreaWorkflow(Msg::Clone)` |
| reopen-area | Reopen as walls | `AreaWorkflow(Msg::Reopen)` |
| make-negative | Make negative | `ContextAction(MakeNegative)` |
| auto-subtract | Auto-subtract | `ContextAction(AutoSubtract)` |
| area-properties | Area properties | `AreaProps` |
| area-attributes | Area attributes | `OpenAreaAttributes(true)` |
| toggle-calcs | Show areas and totals | `Ui(OpenLedger(Drawn))` (was `ToggleCalcs`) |
| area-breakdown | Area breakdown | `OpenBreakdown` |
| area-types | Area types | `Ui(OpenTypeEditor)` (was `OpenCodesEditor`) |
| label-defaults | Label defaults and suffix | `AreaWorkflow(Msg::Advanced(true))` |
| define-first | Choose the type before drawing | `DefineFirst(bool)` |

### Insert

| Id | Label | Message |
|---|---|---|
| draw-exterior, draw-interior | Wall / Inner | `Rail(DrawExterior / DrawInterior)` |
| wall-line | Straight walls | `Shapes(ShapesMsg::Line(wall_type))` |
| wall-rect, wall-circle, wall-polygon | Rectangle / Circle / Polygon walls | `Shapes(ShapesMsg::Choose(WallShape::..))`, then its prompts answer with `ShapesMsg::Value(text)`. `ShapesMsg::Open(wall_type, anchor_y)` is the classic rail's opener and takes a rail coordinate; the tray fly-out sends `Line` and `Choose` directly |
| room-shape | Room shape | `RoomShapeOpen` |
| symbols | Symbols | `Rail(Symbols)` |
| labels | Room labels | `Rail(Text)` |
| measure | Measure | `Rail(Measure)` |
| chain-dim | Chain dimension | `Rail(ChainDim)` |
| insert-vertex | Insert vertex | `Rail(InsertVertex)` |
| curve | Curve wall | `Rail(Curve)` |
| curve-mode | Curve by degree or height | `Key(KeyCmd::CurveMode)` |
| auto-close | Close area | `Key(KeyCmd::AutoClose)` |
| bay-mirror | Bay window mirror | `Key(KeyCmd::BayMirror)` |
| jump-endpoint | Jump to nearest endpoint | `Key(KeyCmd::JumpEndpoint)` |

### Image

| Id | Label | Message |
|---|---|---|
| photo-load | Load trace image | `PhotoLoad` |
| photo-show, photo-hide, photo-remove | Show / Hide / Remove | `PhotoShow`, `PhotoHide`, `PhotoRemove` |
| photo-scale | Set scale from two points | `PhotoScaleStart` |
| photo-smaller, photo-larger | Smaller / Larger | `PhotoSmaller`, `PhotoLarger` |
| photo-rotate-left, photo-rotate-right | Rotate | `PhotoRotateLeft`, `PhotoRotateRight` |
| photo-flip-h, photo-flip-v | Flip | `PhotoFlipH`, `PhotoFlipV` |
| photo-edit | Move and size image | `PhotoEditModeToggle` |
| photo-orient | Align with a guide | `PhotoOrientStart` |
| photo-intensity | Intensity | `PhotoIntensity(v)` |
| geo-map | Load map image | `GeoLoadMap` |
| geo-coordinate | Locate by coordinate | `GeoCoordinate` |
| geo-address | Locate by address | `GeoAddress` |

### Tools

| Id | Label | Message |
|---|---|---|
| options | Options | `Options(OptMsg::Open)` |
| draw-color | Draw colour | `OpenColorDialog` |
| subject-info | Property details | `OpenSubjectInfo` |
| symbol-library | Symbol library | `SymbolManager(Msg::Open)` |
| room-labels | Edit room labels | `Dock(DockMsg::EditTextLibrary)` |
| disto-connect | Connect laser | `Disto(DistoMsg::Connect)` |
| units | Units (imperial, metric) | `Units(..)` |

### Help

| Id | Label | Message |
|---|---|---|
| guide | User guide | `Guide(..)` |
| shortcuts | Keyboard shortcuts (F1) | `Shortcuts(ShortcutsMsg::Open)` |
| email | Email support | `Email` |
| about | About | `About` |

Before finalising the registry, read these and add anything missing: the Home body in
`ribbon.rs` (`items(Body::Home, ..)`), `ribbon_tabs.rs` (View, Tools, Reference Image,
Help), `backstage.rs`, `context_menu::Action`, `select_menu::SelectKind`,
`popup_menu::Kind` and `Quick`, `shortcuts::GROUPS`, `input::KeyCmd`, and
`guide.rs` for the Appraise-It commands.

## 3. Closing rows in `coverage.csv`

Columns: `id, screen, feature, classic_location, home, disposition, command_or_message,
test, notes`. The first four are copied from the inventory. `home` is pre-filled with a
coarse guess per group; replace it with the exact control. Fill the rest. The file is
UTF-8 CSV with 634 data rows. It exists so the record survives the deletion of `refs/`.

Dispositions:

| Value | Meaning | Requires |
|---|---|---|
| KEEP | Same behaviour, same kind of control, new place | `command_or_message`, and `test` or a manual check note |
| MOVED | Same behaviour through a different kind of control (for example a ribbon button that is now a Menu command and an inspector field) | same as KEEP, and say what the new control is in `notes` |
| REPLACED | The row describes a classic chrome mechanism whose purpose is met differently (for example "ribbon tab switching", "dock collapse chevron") | `notes` naming the replacement |
| SHARED | Behaviour in `update` or `sketch-core` with no control of its own (calculation pipeline, file format, rendering) | `notes: unchanged` |
| RETIRED | Only for rows that were browser-only, hidden legacy controls, or inert in the original (see the inventory's "inert or incomplete" list) | Owner sign-off recorded in `notes` |

Rules:

- A row about a *capability* (the user can do X) is never RETIRED.
- Candidates for RETIRED, to be confirmed with the owner one by one: S3-012 (legacy
  per-panel pins, hidden), S7-041 to S7-043 (legacy Define panel controls, hidden),
  S8-022 (legacy Organize Favorites dialog, superseded by the library), S12-016 and
  S12-017 (hidden module toggle, Extended Data toast), S12-078 and S12-079 (browser
  suite embedding hooks), S1-009 as written (browser minimise toast; the native minimise
  is KEEP).
- When a row maps to several new controls, list them all.
- Persistence rows (localStorage keys) are SHARED and map to the files in `persist.rs`.

## 4. Things the first draft left out and the plan restores

Listed so nobody removes them again.

- The full type list with drag-to-assign, arming, usage counts and its context menu.
- The calculation tree detail per area (base, factor, total, subtracted, net, perimeter).
- Fence and Lasso select, Insert vertex, Chain dimension, wall shape menu.
- Zoom slider, 1:N scale, zoom to rectangle, area snap, snap pitch, Show switches,
  dimension mode, underlay, north prompt, mini map.
- All trace image tools and the three map options.
- Pen up and pen down state and angle readout in the length strip.
- Layer import, layer export, send to back, bring to front.
- Label defaults and area label suffix.
