# Appraisal Sketch 1.1 — complete user-reachable feature inventory

Source of truth: `00) sketch standalone.html` (attachment; line numbers below refer to it with the 9 MB `SKETCH_SYMBOL_LIBRARY` data line intact — it is line 6710). Cross-checked against `refs/ui-spec/chrome-spec-*.json` (every visible button/tooltip) and `ACCEPTANCE.md` (screen numbers). Every handler cited was grepped in the HTML: `Class.method` names were verified against the method list of that class (GeometryEngine 4120, AreaBreakdown 5037, InputController 6096, CanvasEngine 6721, Exporter 10638, UIManager 11097, SketchApp 14398); layer functions (lowercase names) were verified to exist in their layer script. `→` means "calls". `window.Geometry` is the `GeometryEngine` instance.

**Layered scripts after SketchApp** (they change the live UI vs. the raw markup — the gallery/spec reflect the layered result):
- `CAD IMPORT` (l.20139) + `CAD IMPORT UI LAYER` (l.20618): adds *Import CAD...* to the backstage.
- `SYMBOL LIBRARY CONTEXT MENU` (l.20739): symbol right-click menu, *Manage symbols...* (replaces *Organize...*), hide/purge/restore, DXF→symbol import.
- `SIDEBAR REDESIGN LAYER` (CSS l.2458, JS l.22071): collapse/expand chevrons, tab-strip pin toggle, selection/pending zones.
- `SIDEBAR RESIZE LAYER` (l.22263): drag/double-click dock resize handle.
- `AREA DEFINITION WORKFLOW` / optional region classification (l.22401): the Areas dock palette, chooser, *New area / Find undefined / Edit areas…*, renames the Define tab to **Areas**.
- Inline IIFEs before the ribbon markup: command palette (l.2696) and pages navigator + totals chip (l.2814).

Status column refreshed 2026-10-29. See PARITY.md and feature-audit.json for classification, scope and evidence; implementation status is distinct from individual visual acceptance.

## Screens (ACCEPTANCE.md)

| # | Screen |
|---|---|
| 1 | Static chrome, default state |
| 2 | Ribbon tabs + File backstage + contextual Edit Tools |
| 3 | Dock tabs, pin/collapse, resize |
| 4 | Status bar controls |
| 5 | Canvas: grid, pan/zoom, drawing walls, snap, keyboard entry |
| 6 | Selection / edit / undo-redo + history |
| 7 | Areas: define, dock, attributes, Calcs |
| 8 | Symbols: library, placement, wall snap, flip, resize |
| 9 | Text library, chain dimensions, room shapes, measure |
| 10 | Pages, layers, reference image/underlay, geo-referencing |
| 11 | File: new/open/save, print, exports, CAD import, Options |
| 12 | Touch panel, Bluetooth (Disto), command search, remaining dialogs |

## Screen 1 — Static chrome, default state (15 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S1-001 | Title bar shows logo, QAT, doc save state, window title 'Appraisal Sketch - <name>' | title-bar | .title-bar markup; SketchApp.updateTitle | 1 | implemented |
| S1-002 | QAT Save (tooltip 'Save (F2)') | title-bar QAT | data-command save → UIManager.executeCommand → SketchApp.saveToFile | 1 | implemented |
| S1-003 | QAT Undo (tooltip 'Undo (Ctrl+Z)'; disabled when stack empty) | title-bar QAT | data-command undo → SketchApp.undo; UIManager.updateUndoRedoControls | 1 | implemented |
| S1-004 | QAT Redo (tooltip 'Redo (Ctrl+Y)'; disabled when stack empty) | title-bar QAT | data-command redo → SketchApp.redo; UIManager.updateUndoRedoControls | 1 | implemented |
| S1-005 | QAT Undo History popup (disabled when empty) | title-bar QAT | data-command undo-history → UIManager.showUndoHistory | 1 | implemented |
| S1-006 | QAT Print | title-bar QAT | data-command print → SketchApp.exportPDF | 1 | implemented |
| S1-007 | QAT Command search button (tooltip 'Command search (Ctrl+K)') | title-bar QAT | #command-palette-btn → command palette open() IIFE | 1 | implemented |
| S1-008 | Document save-state indicator ('New document' / saved / unsaved) | title-bar #doc-save-state | SketchApp.updateTitle | 1 | implemented |
| S1-009 | Window Minimize button (browser: toast 'Minimize is controlled by the browser…') | title-bar window-controls | data-command window-minimize → UIManager.executeCommand | 1 | implemented |
| S1-010 | Window Toggle Full Screen button | title-bar window-controls | data-command window-maximize → requestFullscreen/exitFullscreen | 1 | implemented |
| S1-011 | Window Close button | title-bar window-controls | data-command exit → SketchApp.exitApp | 1 | implemented |
| S1-012 | Drag title bar to move window (native) | title-bar mouse gesture | (native window; n/a in browser) | 1 | implemented |
| S1-013 | Ribbon tab strip: File, Home, View, Tools, Reference Image, Help (+hidden contextual Edit Tools) | ribbon tab strip | .tab click handler in UIManager.bindEvents | 1 | implemented |
| S1-014 | 17-button left rail (Select…Pages) with active-tool highlight | left rail | SketchApp.setMode (toggles .tool-btn.active) | 1 | implemented |
| S1-015 | Right dock open on Areas/Define tab by default | right dock | UIManager.showPanel / restoreSidebarState | 1 | implemented |

## Screen 2 — Ribbon tabs + File backstage + contextual Edit Tools (19 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S2-001 | Switch ribbon tab (hides other .ribbon-content, shows target) | ribbon tab strip click | UIManager.bindEvents (.tab listener); UIManager.activateRibbonTab | 2 | implemented |
| S2-002 | Canvas Background popup (tooltip 'Choose the drawing background: White, Dark, or Blue') | View ribbon › Background | toggle-canvas-background → UIManager.showCanvasBackgroundMenu | 2 | implemented |
| S2-003 | Interface theme popup (tooltip 'Switch the interface between Light, Dark, and Blue') | View ribbon › Interface | interface-theme → UIManager.showInterfaceThemeMenu | 2 | implemented |
| S2-004 | Canvas background menu: White | View › Background popup | UIManager.showCanvasBackgroundMenu → CanvasEngine.setBackgroundTheme('light') | 2 | implemented |
| S2-005 | Canvas background menu: Dark | View › Background popup | CanvasEngine.setBackgroundTheme('dark') | 2 | implemented |
| S2-006 | Canvas background menu: Blue | View › Background popup | CanvasEngine.setBackgroundTheme('blue') | 2 | implemented |
| S2-007 | Interface theme menu: Light (persisted sketch.interfaceTheme) | View › Interface popup | UIManager.showInterfaceThemeMenu → UIManager.applyInterfaceTheme('light') | 2 | implemented |
| S2-008 | Interface theme menu: Dark | View › Interface popup | UIManager.applyInterfaceTheme('dark') | 2 | implemented |
| S2-009 | Interface theme menu: Blue | View › Interface popup | UIManager.applyInterfaceTheme('blue') | 2 | implemented |
| S2-010 | File tab opens/closes backstage menu (renders recent files); click outside or Esc closes | ribbon tab strip › File | UIManager.bindEvents (.tab file-tab); SketchApp.closeFileMenu | 2 | implemented |
| S2-011 | Contextual 'Edit Tools' tab appears on selection (auto-activates), hides when cleared; groups per selection kind (area/text/line/hosted opening) | ribbon tab strip › Edit Tools | UIManager.updateContextualTools; activateRibbonTab; syncContextualControls | 2 | implemented |
| S2-012 | Canvas background themes applied to drawing (white/dark/blue) | canvas | CanvasEngine.setBackgroundTheme | 2 | implemented |
| S2-013 | localStorage sketch.interfaceTheme | persistence | UIManager.applyInterfaceTheme | 2 | implemented |
| S2-014 | View tab body: Mini-Map, View Calcs \| Previous, Next, First/Last Page, Add, Delete \| Background, Interface, Apply, Grid (highlighted), Filter | ribbon › View | markup #tab-view; UIManager.bindEvents | 2 | implemented |
| S2-015 | Tools tab body: Connect, Disconnected status \| Options, Area Breakdown, Area Codes | ribbon › Tools | markup #tab-tools | 2 | implemented |
| S2-016 | Reference Image tab body: Load/Show/Hide/Remove \| Set Scale, Smaller, Larger, Rotate L/R, Edit Mode, Orient Guide, Flip H/V, Intensity slider \| Geo | ribbon › Reference Image | markup #tab-photometrics | 2 | implemented |
| S2-017 | Help tab body: Keyboard Shortcuts | ribbon › Help | markup #tab-help | 2 | implemented |
| S2-018 | File backstage layout: item list (New…Page Layout incl. Import CAD...), dividers, footer Options/Exit, Recent Files flyout | File backstage | markup #file-menu; CAD IMPORT UI LAYER install | 2 | implemented |
| S2-019 | Reference Image tab can be hidden/shown (module deactivate/activate) | executeCommand activate-module/deactivate-module | UIManager.setPhotometricsModule | 2 | source-inactive |

## Screen 3 — Dock tabs, pin/collapse, resize (20 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S3-001 | Symbols dock: category select (Favorites + 18 library categories) | dock › Symbols › #symbol-category-select | UIManager.populateSymbolCategorySelect → UIManager.populateSymbolLibrary | 3 | implemented |
| S3-002 | Symbols dock: 'Find a symbol...' search filter | dock › Symbols › #symbol-lib-search | UIManager.populateSymbolLibrary | 3 | implemented |
| S3-003 | Symbols dock: symbol list items (icon + label); empty 'No matching symbols.' | dock › Symbols › #symbol-lib-list | UIManager.populateSymbolLibrary; UIManager.getSymbolItems | 3 | implemented |
| S3-004 | Symbol preview pane (canvas render + label + wall-snap note; 'Select a symbol') | dock › Symbols › #symbol-preview | UIManager.updateSymbolPreview; CanvasEngine.renderSymbolPreview | 3 | implemented |
| S3-005 | Dock collapse chevron (tooltip 'Collapse sidebar') → 30px rail | dock tab strip › #sidebar-collapse | SIDEBAR REDESIGN LAYER bindCollapse → UIManager.setSidebarOpen(false) | 3 | implemented |
| S3-006 | Dock expand button on collapsed rail ('Expand sidebar') | dock collapsed rail › #sidebar-expand | bindCollapse → UIManager.setSidebarOpen(true) | 3 | implemented |
| S3-007 | Dock pin toggle (tooltip 'Keep the dock open after placing' / 'Hide the dock after placing'; toast) | dock tab strip › #sidebar-pin-toggle | SIDEBAR REDESIGN LAYER bindPin/syncPin (state.sidebar.pinned) | 3 | implemented |
| S3-008 | Auto-hide dock after text/symbol placement when unpinned | behaviour | state.sidebar.pinned checked in SketchApp.placeTextAt / placeSymbolAt | 3 | implemented |
| S3-009 | Dock resize: drag left-edge handle 320–400 px (canvas ≥220), width saved in sketch state | dock › #sidebar-resize-handle drag | SIDEBAR RESIZE LAYER bindHandle/applyWidth/clamp | 3 | implemented |
| S3-010 | Dock resize handle double-click → reset width | dock › resize handle dblclick | SIDEBAR RESIZE LAYER resetWidth | 3 | implemented |
| S3-011 | Dock tab switching: Areas, Text, Symbols, Calcs, Layers (state.sidebar.active persisted) | dock tab strip | UIManager.bindEvents (.sidebar-tab) → UIManager.showPanel; restoreSidebarState | 3 | implemented |
| S3-012 | Legacy per-panel pin buttons (.sidebar-pin; hidden by redesign, still state-synced) | dock panel headers | UIManager.bindEvents (.sidebar-pin) | 3 | source-inactive |
| S3-013 | Dock state (active tab, open, pinned, width) saved inside the sketch file settings.sidebar | persistence | SketchApp.buildSaveModel; UIManager.restoreSidebarState; SIDEBAR RESIZE LAYER storedWidth | 3 | implemented |
| S3-014 | Areas dock body (palette + links) matches gallery dock-Areas | dock › Areas | AREA DEFINITION WORKFLOW boot; paintPalette | 3 | implemented |
| S3-015 | Text dock body: search, list, alpha rail, Edit | dock › Text | markup #panel-text-library; UIManager.populateTextLibrary | 3 | implemented |
| S3-016 | Symbols dock body: category select, search, list, preview, Manage symbols... | dock › Symbols | markup #panel-symbol-library | 3 | implemented |
| S3-017 | Calcs dock body: Expand All / Collapse All, tree, Calculation Details, summary, right-click hint | dock › Calcs | markup #panel-calcs; UIManager.updateCalcsPanel | 3 | implemented |
| S3-018 | Layers dock body: tree / empty state | dock › Layers | markup #panel-layers; UIManager.updateLayersPanel | 3 | implemented |
| S3-019 | Areas tab label renamed 'Areas' (markup says Define); rail tooltip 'Define Areas: click inside an outline' | dock tab strip / rail | AREA DEFINITION WORKFLOW boot | 3 | implemented |
| S3-020 | Areas dock: selection summary zone & pending-classification readout (legacy zone, kept in sync) | dock › Areas (legacy) | SIDEBAR REDESIGN LAYER updateSelectionSummary/updatePending/refreshZones/hookRender | 3 | source-inactive |

## Screen 4 — Status bar controls (36 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S4-001 | Filter popover ('Show in sketch') checkbox: Area fills | View › Filter; status bar Filter button | UIManager.toggleFilterPanel (state.filters.areas) | 4 | implemented |
| S4-002 | Filter popover ('Show in sketch') checkbox: Exterior walls | View › Filter; status bar Filter button | UIManager.toggleFilterPanel (state.filters.exterior) | 4 | implemented |
| S4-003 | Filter popover ('Show in sketch') checkbox: Interior lines | View › Filter; status bar Filter button | UIManager.toggleFilterPanel (state.filters.interior) | 4 | implemented |
| S4-004 | Filter popover ('Show in sketch') checkbox: Dimensions | View › Filter; status bar Filter button | UIManager.toggleFilterPanel (state.filters.dimensions) | 4 | implemented |
| S4-005 | Filter popover ('Show in sketch') checkbox: Text labels | View › Filter; status bar Filter button | UIManager.toggleFilterPanel (state.filters.labels) | 4 | implemented |
| S4-006 | Filter popover ('Show in sketch') checkbox: Symbols | View › Filter; status bar Filter button | UIManager.toggleFilterPanel (state.filters.symbols) | 4 | implemented |
| S4-007 | Center sketch | status bar › Center | #btn-center → SketchApp.centerSketch | 4 | implemented |
| S4-008 | Center & Fit | status bar › Center & Fit | #btn-center-fit → SketchApp.centerAndFit | 4 | implemented |
| S4-009 | Zoom Fence (drag rectangle to zoom) | status bar › Zoom Fence | data-command zoom-fence → SketchApp.setMode('zoom-fence') → SketchApp.zoomToWorldRect | 4 | implemented |
| S4-010 | Grid toggle (active state) | status bar › Grid | #btn-grid → SketchApp.toggleGrid | 4 | implemented |
| S4-011 | Snap toggle (active state) | status bar › Snap | #btn-snap → SketchApp.toggleSnap | 4 | implemented |
| S4-012 | Area Grid Snap toggle | status bar › Area Grid Snap | #btn-area-snap → SketchApp.toggleAreaGridSnap | 4 | implemented |
| S4-013 | Snap grid pitch select: 1' / 1/2' / 1/4' / 1" | status bar › #area-snap-pitch | SketchApp.setAreaGridSnapSize | 4 | implemented |
| S4-014 | Filter popover | status bar › Filter | data-command filter → UIManager.toggleFilterPanel | 4 | implemented |
| S4-015 | Totals chip 'GLA x sf · N areas' (click opens Calcs panel) | status bar › #totals-chip | pages-nav IIFE → UIManager.showPanel('panel-calcs'); text by UIManager.updateCalcsPanel | 4 | native-adaptation |
| S4-016 | Zoom out button | status bar › zoom slider − | #btn-zoom-out → SketchApp.zoomOut | 4 | implemented |
| S4-017 | Zoom slider 10–500% | status bar › #zoom-range | UIManager.bindEvents → SketchApp.zoomTo | 4 | implemented |
| S4-018 | Zoom in button | status bar › zoom slider + | #btn-zoom-in → SketchApp.zoomIn | 4 | implemented |
| S4-019 | Zoom/scale readout '+ N% (Imp) 1:' | status bar › #scale-info | UIManager.updateZoomDisplay | 4 | implemented |
| S4-020 | Drawing scale input (1:N, Enter/change applies) | status bar › #scale-input | SketchApp.setDrawingScale; SketchApp.drawingScaleFactor | 4 | implemented |
| S4-021 | Defined areas select (code/name/area rows; choose selects area) | status bar › #area-status-select | UIManager.bindEvents; UIManager.updateCalcsPanel | 4 | native-adaptation |
| S4-022 | Segment HUD: length ' @ angle ° + pen state UP/DOWN | status bar › .status-hud | UIManager.updateHUD; InputController.updateKeyboardHUD | 4 | implemented |
| S4-023 | North indicator: arrow + degrees; click → prompt 'North direction in degrees (0 = up, clockwise)' | status bar › #btn-north | UIManager.bindEvents → SketchApp.setNorthAngle; UIManager.updateNorthDisplay | 4 | implemented |
| S4-024 | Delete page | status bar › Delete Page | #btn-page-del → SketchApp.deletePage | 4 | implemented |
| S4-025 | Previous page (disabled on page 1) | status bar › Previous Page | #btn-page-prev → SketchApp.goToPage; UIManager.updatePageDisplay | 4 | implemented |
| S4-026 | Page display 'N of M' | status bar › #page-display | UIManager.updatePageDisplay | 4 | implemented |
| S4-027 | Next page (confirm add page at last) | status bar › Next Page | #btn-page-next → SketchApp.goToPage / addPage | 4 | implemented |
| S4-028 | Add page | status bar › Add Page | #btn-page-add → SketchApp.addPage | 4 | implemented |
| S4-029 | Underlay select 'Underlay: None / Page N' (ghost another page) | status bar › #underlay-select | SketchApp.setUnderlay | 4 | implemented |
| S4-030 | Bluetooth button | status bar › Bluetooth | data-command disto-connect → SketchApp.connectDisto | 4 | hardware-deferred |
| S4-031 | Touch Panel button | status bar › Touch Panel | data-command touch-panel → UIManager.showTouchPanel | 4 | implemented |
| S4-032 | Center sketch / Center & fit (also while drawing) | keyboard: C / Shift+C | InputController.handleKeyDown → SketchApp.centerSketch / centerAndFit | 4 | implemented |
| S4-033 | Zoom Fence: drag rectangle → zoom to it | left drag (zoom-fence) | CanvasEngine.setupEvents → SketchApp.zoomToWorldRect | 4 | implemented |
| S4-034 | Zoom in/out step; zoomTo clamps; drawing-scale factor ties zoom % to 1:N scale | zoom | SketchApp.zoomIn/zoomOut/zoomTo/drawingScaleFactor/setDrawingScale | 4 | implemented |
| S4-035 | Center sketch / Center & fit to bounds | view | SketchApp.centerSketch/centerAndFit/getSketchBounds | 4 | implemented |
| S4-036 | Grid on/off; Snap on/off; Area grid snap + pitch (also sets grid size) | status bar | SketchApp.toggleGrid/toggleSnap/toggleAreaGridSnap/setAreaGridSnapSize | 4 | implemented |

## Screen 5 — Canvas: grid, pan/zoom, drawing walls, snap, keyboard entry (64 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S5-001 | Draw Color button: opens colour palette/dialog to set drawing colour | Home ribbon › Draw Color | draw-color → UIManager.toggleDrawColorPalette → showColorDialog; #draw-color-input listener sets state.drawColor | 5 | implemented |
| S5-002 | Apply/cycle dimension display mode | View ribbon › Apply (Dims) | cycle-dimensions → SketchApp.cycleDimensions | 5 | implemented |
| S5-003 | Toggle grid | View ribbon › Grid | toggle-grid → SketchApp.toggleGrid | 5 | implemented |
| S5-004 | Colour dialog › Standard tab: colour wheel drag + preset swatch grid | Colour dialog (Draw Color, ribbon colour labels, area-code swatches) | UIManager.showColorDialog | 5 | implemented |
| S5-005 | Colour dialog › Custom tab: saturation/value square + hue strip drag | Colour dialog | UIManager.showColorDialog (svCanvas/hueCanvas; hsvToRgb/rgbToHsv) | 5 | implemented |
| S5-006 | Colour dialog › Red/Green/Blue numeric inputs 0-255 | Colour dialog | UIManager.showColorDialog (applyRgb; rgbToHex/hexToRgb) | 5 | implemented |
| S5-007 | Colour dialog › OK / Cancel / × / click-outside close | Colour dialog | UIManager.showColorDialog | 5 | implemented |
| S5-008 | Rail: Pan tool (mode 'pan', drag to pan) | left rail › Pan | #tool-pan → SketchApp.setMode('pan') | 5 | implemented |
| S5-009 | Rail: Draw Exterior (mode 'draw-exterior') | left rail › Draw Exterior; key E | #tool-draw-exterior → SketchApp.setMode('draw-exterior') | 5 | implemented |
| S5-010 | Rail: Draw Interior (mode 'draw-interior') | left rail › Draw Interior; key I / F5 | #tool-draw-interior → SketchApp.setMode('draw-interior') | 5 | implemented |
| S5-011 | Rail: wall-shape caret ▸ / right-click on Draw Exterior/Interior → shape menu | left rail caret / contextmenu | UIManager.showWallShapeMenu | 5 | implemented |
| S5-012 | Wall shape menu: Line (plain draw mode) | wall shape popup | UIManager.showWallShapeMenu → SketchApp.setMode('draw-<type>') | 5 | implemented |
| S5-013 | Wall shape menu: Rectangle (prompts Width, Height ft) | wall shape popup | UIManager.showWallShapeMenu → SketchApp.addWallShape('rectangle') | 5 | implemented |
| S5-014 | Wall shape menu: Circle (prompt Diameter) | wall shape popup | UIManager.showWallShapeMenu → SketchApp.addWallShape('circle') | 5 | implemented |
| S5-015 | Wall shape menu: Polygon (prompts Width, Height, Number of sides) | wall shape popup | UIManager.showWallShapeMenu → SketchApp.addWallShape('polygon') | 5 | implemented |
| S5-016 | Auto-close current drawing (undocumented in cheat sheet) | keyboard: Ctrl+F3 | InputController.handleKeyDown → SketchApp.autoClose | 5 | implemented |
| S5-017 | Draw interior walls | keyboard: F5 | InputController.handleKeyDown → SketchApp.setMode('draw-interior') | 5 | implemented |
| S5-018 | Erase last typed character | keyboard: Backspace / Delete while drawing with buffer | InputController.handleKeyDown → InputController.buffer (property) slice; UIManager.updateHUD | 5 | implemented |
| S5-019 | Toggle grid | keyboard: G | InputController.handleKeyDown → SketchApp.toggleGrid | 5 | implemented |
| S5-020 | Draw exterior walls | keyboard: E | InputController.handleKeyDown → SketchApp.setMode('draw-exterior') | 5 | implemented |
| S5-021 | Draw interior walls | keyboard: I | InputController.handleKeyDown → SketchApp.setMode('draw-interior') | 5 | implemented |
| S5-022 | Temporary pan (drag) | keyboard: Space (hold) | InputController.handleKeyDown → state.spacePan; CanvasEngine.setupEvents | 5 | implemented |
| S5-023 | Toggle last wall's dimension while drawing / toggle selected dimensions | keyboard: Space (tap, release without panning) | InputController.handleKeyDown → InputController.handleKeyUp → SketchApp.toggleLastDimension / toggleSelectedDimensions | 5 | implemented |
| S5-024 | Start exterior draw mode | keyboard: Enter (not drawing) | InputController.handleKeyDown → SketchApp.setMode('draw-exterior') | 5 | implemented |
| S5-025 | Type distance into buffer (HUD shows it) | keyboard: Digits / '.' while drawing | InputController.handleKeyDown → InputController.buffer (property); UIManager.updateHUD | 5 | implemented |
| S5-026 | Rise × run entry | keyboard: 'x' after digits (e.g. 10x5) | InputController.handleKeyDown → InputController.parseBuffer (riserun) | 5 | implemented |
| S5-027 | Turn left/right relative to last wall (default 90°) | keyboard: 'l' / 'r' after digits (e.g. 10l, 10r45) | InputController.handleKeyDown → InputController.parseBuffer (turn) | 5 | implemented |
| S5-028 | Distance at absolute angle | keyboard: '<' or ',' after digits (e.g. 10<45) | InputController.handleKeyDown → InputController.parseBuffer (angle) | 5 | implemented |
| S5-029 | Insert decimal point | keyboard: Tab while drawing | InputController.handleKeyDown → InputController.handleKeyDown | 5 | implemented |
| S5-030 | Pen down if up; else commit typed entry / cursor move as a wall; Enter at last point with empty buffer → pen up | keyboard: Enter while drawing | InputController.handleKeyDown → SketchApp.penDown / drawLine / penUp; InputController.parseBuffer | 5 | implemented |
| S5-031 | Auto-close area | keyboard: A (pen down) | InputController.handleKeyDown → SketchApp.autoClose | 5 | implemented |
| S5-032 | Bay window mirror | keyboard: B (pen down) | InputController.handleKeyDown → SketchApp.bayWindowMirror | 5 | implemented |
| S5-033 | Jump cursor to nearest endpoint | keyboard: J (drawing) | InputController.handleKeyDown → GeometryEngine.findNearestEndpoint | 5 | implemented |
| S5-034 | Move drawing cursor by typed distance or step unit (Tenths 0.1/Half 0.5/Whole 1; Shift = 0.1); enters keyboard-pointer mode | keyboard: Arrow keys while drawing | InputController.handleKeyDown → InputController.enterKeyboardPointerMode / updateKeyboardHUD | 5 | implemented |
| S5-035 | Rise/run move in arrow quadrant | keyboard: Arrow keys with typed 'NxM' buffer | InputController.handleKeyDown → InputController.handleKeyDown (riserun) | 5 | implemented |
| S5-036 | Diagonal cursor moves (NE / NW / SW / SE) | keyboard: PageUp / Home / End / PageDown while drawing | InputController.handleKeyDown → InputController.handleKeyDown (diagonal) | 5 | implemented |
| S5-037 | Snap cursor to alignment with existing endpoints in that direction | keyboard: Ctrl+Arrow while drawing | InputController.handleKeyDown → GeometryEngine.findAlignment | 5 | implemented |
| S5-038 | Snap cursor to intersecting alignment | keyboard: Ctrl+Shift+Arrow while drawing | InputController.handleKeyDown → GeometryEngine.findIntersectingAlignment | 5 | implemented |
| S5-039 | Mouse wheel → zoom about cursor (×1.1 / ×0.9) | mouse wheel on canvas | CanvasEngine.setupEvents (wheel) → SketchApp.zoomTo | 5 | implemented |
| S5-040 | Shift+wheel while drawing → bow the last wall (±15°, Ctrl 1°; height mode ±1/0.1 ft) | Shift+wheel (draw mode) | CanvasEngine.setupEvents (wheel) → GeometryEngine.clampBowAngle | 5 | implemented |
| S5-041 | Middle-button drag → pan | middle mouse drag | CanvasEngine.setupEvents (pointerdown button 1 → isPanning) | 5 | implemented |
| S5-042 | Left drag in Pan tool → pan | left drag (pan mode) | CanvasEngine.setupEvents (pointerdown)  | 5 | implemented |
| S5-043 | Space+left drag → temporary pan | Space held + drag | CanvasEngine.setupEvents (spacePan) ; InputController.handleKeyDown | 5 | implemented |
| S5-044 | Draw mode: click to drop vertices (pen down on first click), live rubber band with length/angle readouts, HUD updates | left click / move (draw mode) | CanvasEngine.setupEvents (click) → SketchApp.handleCanvasClick → drawLine; CanvasEngine.drawRubberBand/drawLiveAngle | 5 | implemented |
| S5-045 | Draw mode snapping: structural snap to closed-area vertices & wall-solid faces, grid snap, 5° polar snap, join to start vertex closes loop | pointer move (draw mode) | SketchApp.getDrawingTarget/snapPoint/structuralSnap; GeometryEngine.polarSnap/snapToGrid | 5 | implemented |
| S5-046 | Double-click while drawing → pen up (finish chain) | double-click (draw mode) | CanvasEngine.setupEvents (dblclick) → SketchApp.penUp | 5 | implemented |
| S5-047 | Right-click while drawing → pen up, then canvas context menu | right-click (draw mode) | CanvasEngine.setupEvents (contextmenu) → SketchApp.penUp | 5 | implemented |
| S5-048 | Closed exterior loop auto-creates UND area polygon; wall joins healed | draw closing | SketchApp.finishCommittedWall / closeJoinedExteriorLoops / healExteriorWallJoins / recalculate | 5 | implemented |
| S5-049 | Live drawing aids: witness lines, alignment guides, rise/run, return guide, snap preview, vertices | canvas render | CanvasEngine.drawWitnessLines/drawGuideLabel/drawRiseRun/drawReturnGuide/drawSnapPreview/drawVertices | 5 | implemented |
| S5-050 | Keyboard pointer (crosshair) shown while drawing by keys; any mouse move >3px returns to mouse | canvas | CanvasEngine.drawKeyboardPointer/drawKeyboardPreview; InputController.enterKeyboardPointerMode | 5 | native-adaptation |
| S5-051 | Drawing cursor image sized by Options › Drawing Pointer | canvas cursor | CanvasEngine.getDrawingCursor/getDrawingCursorImage/setCursor | 5 | native-adaptation |
| S5-052 | Grid rendering (toggle; pitch) | canvas | CanvasEngine.drawGrid | 5 | implemented |
| S5-053 | Wall dimensions auto-labelled (inside/outside placement, cycle modes) | canvas | CanvasEngine.drawDimension/getDimensionAnchor; SketchApp.cycleDimensions/isDimensionVisible | 5 | implemented |
| S5-054 | Wall solids rendering (exterior/interior thickness) when enabled | canvas | CanvasEngine.drawWallSolids/drawCurvedWallSolid; SketchApp.getWallSolids | 5 | implemented |
| S5-055 | Canvas resize with window / dock changes | window resize | CanvasEngine.resize | 5 | implemented |
| S5-056 | Pen down/up drawing chain; Define First option blocks exterior drawing until an area type is chosen | draw | SketchApp.penDown/penUp/togglePen/handleCanvasClick | 5 | implemented |
| S5-057 | Auto-close (A / Ctrl+F3 / click start point) closes loop into area | draw | SketchApp.autoClose | 5 | implemented |
| S5-058 | Bay window mirror (B): mirror the last segments to form a bay | draw | SketchApp.bayWindowMirror | 5 | implemented |
| S5-059 | Toggle last wall's dimension (Space tap while drawing; suppressDimensionNext) | draw | SketchApp.toggleLastDimension | 5 | implemented |
| S5-060 | Coordinate cleaning (round to 1e-?) for typed/keyboard moves | draw | SketchApp.cleanCoord | 5 | implemented |
| S5-061 | Typed-length walls default first heading up; subsequent typed distance continues last heading | keyboard draw | InputController.handleKeyDown (Enter) → SketchApp.drawLine | 5 | implemented |
| S5-062 | Units/precision formatting (imperial ft-in / metric, decimal precision, strip trailing zeroes) | labels | SketchApp.formatLength/formatFeetInches/formatLengthDual/formatArea | 5 | implemented |
| S5-063 | Cycle dimensions: Outside → Inside → Hidden (toast) and syncs Dims checkbox/placement | View › Apply | SketchApp.cycleDimensions | 5 | implemented |
| S5-064 | Dimension visibility rules: hidden flag, filter, minimum-dimension option, live wall always shown | render | SketchApp.isDimensionVisible | 5 | implemented |

## Screen 6 — Selection / edit / undo-redo + history (88 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S6-001 | Cut selection | Home ribbon › Cut; Edit Tools › Cut | data-command cut → UIManager.executeCommand → SketchApp.cutSelected | 6 | implemented |
| S6-002 | Copy selection | Home ribbon › Copy; Edit Tools › Copy | copy → SketchApp.copySelected | 6 | implemented |
| S6-003 | Paste clipboard | Home ribbon › Paste; Edit Tools › Paste | paste → SketchApp.paste | 6 | implemented |
| S6-004 | Select All | Home ribbon › Select All; Edit Tools › Select All | select-all → SketchApp.selectAll | 6 | implemented |
| S6-005 | Select (by type) popup button | Home ribbon › Select; Edit Tools › Select | select-by-type → UIManager.showSelectMenu | 6 | implemented |
| S6-006 | Select-by-type menu item: Select All | popup from Select button (action 'all') | UIManager.showSelectMenu → SketchApp.selectByType('all') | 6 | implemented |
| S6-007 | Select-by-type menu item: Select Areas | popup from Select button (action 'areas') | UIManager.showSelectMenu → SketchApp.selectByType('areas') | 6 | implemented |
| S6-008 | Select-by-type menu item: Select Text | popup from Select button (action 'text') | UIManager.showSelectMenu → SketchApp.selectByType('text') | 6 | implemented |
| S6-009 | Select-by-type menu item: Select Dimensions | popup from Select button (action 'dimensions') | UIManager.showSelectMenu → SketchApp.selectByType('dimensions') | 6 | implemented |
| S6-010 | Select-by-type menu item: Select Symbols | popup from Select button (action 'symbols') | UIManager.showSelectMenu → SketchApp.selectByType('symbols') | 6 | implemented |
| S6-011 | Select-by-type menu item: Select Freeform Lines | popup from Select button (action 'freeform') | UIManager.showSelectMenu → SketchApp.selectByType('freeform') | 6 | implemented |
| S6-012 | Select by type via prompt (All/Areas/Text/Dimensions/Symbols/Freeform Lines + aliases) | prompt dialog (legacy path) | UIManager.selectByType → SketchApp.selectByType | 6 | implemented |
| S6-013 | Delete selection | Home ribbon › Delete; Edit Tools › Delete | delete-selected → SketchApp.deleteSelected | 6 | implemented |
| S6-014 | Undo (ribbon, disabled when empty) | Home ribbon › Undo; Edit Tools › Undo | undo → SketchApp.undo | 6 | implemented |
| S6-015 | Redo (ribbon, disabled when empty) | Home ribbon › Redo; Edit Tools › Redo | redo → SketchApp.redo | 6 | implemented |
| S6-016 | Rotate whole sketch 90° | Home ribbon › Rotate Sketch | rotate-sketch → SketchApp.rotateSketch(90) → transformSketch | 6 | implemented |
| S6-017 | Flip Sketch popup button | Home ribbon › Flip Sketch | flip-sketch → UIManager.showFlipMenu | 6 | implemented |
| S6-018 | Flip menu: Flip Horizontal | popup from Flip Sketch | UIManager.showFlipMenu → SketchApp.flipSketch('horizontal') | 6 | implemented |
| S6-019 | Flip menu: Flip Vertical | popup from Flip Sketch | UIManager.showFlipMenu → SketchApp.flipSketch('vertical') | 6 | implemented |
| S6-020 | Filter selection panel | View ribbon › Filter | filter → UIManager.toggleFilterPanel | 6 | implemented |
| S6-021 | Undo History popup: numbered list 'N. <label>' newest first; click → undo to that point; toast 'Nothing to undo yet.' | QAT Undo History; executeCommand undo-history | UIManager.showUndoHistory → SketchApp.undoTo | 6 | implemented |
| S6-022 | Undo/Redo/History buttons enable/disable with stack state | QAT + ribbons | UIManager.updateUndoRedoControls | 6 | implemented |
| S6-023 | Canvas context menu: Delete | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.deleteSelected | 6 | implemented |
| S6-024 | Canvas context menu: Cut | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.cutSelected | 6 | implemented |
| S6-025 | Canvas context menu: Copy | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.copySelected | 6 | implemented |
| S6-026 | Canvas context menu: Paste | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.paste | 6 | implemented |
| S6-027 | Canvas context menu: Flip Horz (selected areas) | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.flipSelectedAreas('horizontal') | 6 | implemented |
| S6-028 | Canvas context menu: Flip Vert | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.flipSelectedAreas('vertical') | 6 | implemented |
| S6-029 | Canvas context menu: Rotate 90° | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.rotateSelected(90) | 6 | implemented |
| S6-030 | Edit Tools › Length field ('Length · Start/End handle') + Enter | Edit Tools (line) | #selected-line-length keydown → SketchApp.setSelectedLineLength | 6 | implemented |
| S6-031 | Edit Tools › Set Length | Edit Tools (line) | apply-line-length → SketchApp.setSelectedLineLength | 6 | implemented |
| S6-032 | Edit Tools › Line Style select (Solid/Dashed/Dotted) for line | Edit Tools (line) | #selected-line-style → SketchApp.setSelectedLineAppearance | 6 | implemented |
| S6-033 | Edit Tools › Line Color for line | Edit Tools (line) | #selected-line-color → SketchApp.setSelectedLineAppearance | 6 | implemented |
| S6-034 | Edit Tools › Curve (switch to curve tool) | Edit Tools (line) | curve-selected-line → SketchApp.setMode('curve') | 6 | implemented |
| S6-035 | Rail: Select/Edit tool (mode 'select') | left rail › Select/Edit; key P | #tool-select → SketchApp.setMode('select') | 6 | implemented |
| S6-036 | Rail: Fence tool (rectangle select, mode 'fence') | left rail › Fence | #tool-fence → SketchApp.setMode('fence') → SketchApp.fenceSelect | 6 | implemented |
| S6-037 | Rail: Lasso tool (freehand select, mode 'lasso') | left rail › Lasso | #tool-lasso → SketchApp.setMode('lasso') → SketchApp.lassoSelect | 6 | implemented |
| S6-038 | Rail: Insert Vertex tool (click a wall to split) | left rail › Insert Vertex; key Insert | #tool-insert-vertex → setMode('insert-vertex') → SketchApp.insertVertexAt | 6 | implemented |
| S6-039 | Rail: Curve Wall tool (drag wall to bow; toast) | left rail › Curve Wall | #tool-curve → setMode('curve') → SketchApp.previewCurveBow/commitCurveBow | 6 | implemented |
| S6-040 | Undo | keyboard: Ctrl+Z | InputController.handleKeyDown → SketchApp.undo | 6 | implemented |
| S6-041 | Redo | keyboard: Ctrl+Shift+Z | InputController.handleKeyDown → SketchApp.redo | 6 | implemented |
| S6-042 | Redo | keyboard: Ctrl+Y | InputController.handleKeyDown → SketchApp.redo | 6 | implemented |
| S6-043 | Select all | keyboard: Ctrl+A | InputController.handleKeyDown → SketchApp.selectAll | 6 | implemented |
| S6-044 | Cut | keyboard: Ctrl+X | InputController.handleKeyDown → SketchApp.cutSelected | 6 | implemented |
| S6-045 | Copy | keyboard: Ctrl+C (with selection, not in measure mode) | InputController.handleKeyDown → SketchApp.copySelected | 6 | implemented |
| S6-046 | Paste | keyboard: Ctrl+V | InputController.handleKeyDown → SketchApp.paste | 6 | implemented |
| S6-047 | Pen up if drawing, else clear selection, else Select tool; clears buffer + bow | keyboard: Esc | InputController.handleKeyDown → SketchApp.penUp / clearSelection / setMode('select') | 6 | implemented |
| S6-048 | Insert-vertex tool | keyboard: Insert | InputController.handleKeyDown → SketchApp.setMode('insert-vertex') | 6 | implemented |
| S6-049 | Delete selection | keyboard: Delete / Backspace (with selection) | InputController.handleKeyDown → SketchApp.deleteSelected | 6 | implemented |
| S6-050 | Select tool | keyboard: P (not drawing) | InputController.handleKeyDown → SketchApp.setMode('select') | 6 | implemented |
| S6-051 | Rotate selection 1° | keyboard: Ctrl+Shift+R (selection) | InputController.handleKeyDown → SketchApp.rotateSelected(1) | 6 | implemented |
| S6-052 | Rotate selection 5° | keyboard: Shift+R (selection) | InputController.handleKeyDown → SketchApp.rotateSelected(5) | 6 | implemented |
| S6-053 | Orient selection horizontal | keyboard: H (selection) | InputController.handleKeyDown → SketchApp.orientSelected('horizontal') | 6 | implemented |
| S6-054 | Orient selection vertical | keyboard: V (selection) | InputController.handleKeyDown → SketchApp.orientSelected('vertical') | 6 | implemented |
| S6-055 | Scale selection ×1.1 | keyboard: + / NumpadAdd (selection) | InputController.handleKeyDown → SketchApp.scaleSelected(1.1) | 6 | implemented |
| S6-056 | Scale selection ×0.9 | keyboard: - / NumpadSubtract (selection) | InputController.handleKeyDown → SketchApp.scaleSelected(0.9) | 6 | implemented |
| S6-057 | Flip sketch horizontal | keyboard: F (not drawing) | InputController.handleKeyDown → SketchApp.flipSketch('horizontal') | 6 | implemented |
| S6-058 | Flip sketch vertical | keyboard: M (not drawing) | InputController.handleKeyDown → SketchApp.flipSketch('vertical') | 6 | implemented |
| S6-059 | Rotate sketch left 90° | keyboard: L (not drawing) | InputController.handleKeyDown → SketchApp.rotateSketch(-90) | 6 | implemented |
| S6-060 | Rotate sketch right 90° | keyboard: R (not drawing) | InputController.handleKeyDown → SketchApp.rotateSketch(90) | 6 | implemented |
| S6-061 | Toggle curve mode Degree/Height (toast) | keyboard: D (empty buffer) | InputController.handleKeyDown → state.curveMode | 6 | implemented |
| S6-062 | Nudge selection 1 ft (Shift = 0.1 ft) | keyboard: Arrow keys (selection, not drawing) | InputController.handleKeyDown → SketchApp.moveSelected + finalizeSelectionMove | 6 | implemented |
| S6-063 | Wheel in Select with selected curved wall → bow ±15° (Shift 5°, Ctrl 1°; height mode 1 ft / 0.1 ft) | mouse wheel (select, curve selected) | CanvasEngine.setupEvents → SketchApp.adjustSelectedCurves | 6 | implemented |
| S6-064 | Wheel in Select with selected text/area label/symbol → scale ×1.08/×0.92 | mouse wheel (select) | CanvasEngine.setupEvents → SketchApp.scaleSelected | 6 | implemented |
| S6-065 | Select: click entity (line, polygon, label, area label/size, symbol, dimension) to select; Ctrl+click additive | left click (select) | CanvasEngine.setupEvents → SketchApp.selectLineAt/hitTestAt | 6 | implemented |
| S6-066 | Select: drag selected entities to move (snaps to grid when snap on; area-grid snap when on) | left drag on selection | CanvasEngine.setupEvents (selection drag) → SketchApp.moveSelected/finalizeSelectionMove | 6 | implemented |
| S6-067 | Select: drag on empty canvas → marquee selection (Ctrl additive) | left drag empty (select) | CanvasEngine.setupEvents (selection-marquee) → SketchApp.fenceSelect | 6 | implemented |
| S6-068 | Fence tool: drag rectangle selects enclosed | left drag (fence) | CanvasEngine.setupEvents → SketchApp.fenceSelect; CanvasEngine.drawFenceRect | 6 | implemented |
| S6-069 | Lasso tool: freehand path selects enclosed (Ctrl additive) | left drag (lasso) | CanvasEngine.setupEvents → SketchApp.lassoSelect; CanvasEngine.drawLasso | 6 | implemented |
| S6-070 | Drag selected line endpoint handle → move endpoint (snaps), updates Edit Tools length | drag endpoint handle (select) | CanvasEngine.setupEvents (line-endpoint) → SketchApp.moveLineEndpoint/snapLineEndpoint/finalizeLineEndpointEdit; CanvasEngine.drawLineEndpointHandles | 6 | implemented |
| S6-071 | Drag rotation handle of selected area → rotate (15° steps, Ctrl 1°) with angle readout | drag rotation handle | CanvasEngine.setupEvents (rotate-area) → SketchApp.rotateSelected; CanvasEngine.drawTransformHandles/drawRotationAngle | 6 | implemented |
| S6-072 | Drag curve handle (midpoint) of selected wall → bow | drag curve handle (select) | CanvasEngine.setupEvents (curve) → CanvasEngine.createCurveDragState; SketchApp.previewCurveBow/commitCurveBow | 6 | implemented |
| S6-073 | Curve tool: press on a wall and drag outward/inward to bow; live area update | left drag (curve mode) | CanvasEngine.setupEvents → SketchApp.previewCurveBow/commitCurveBow; CanvasEngine.drawCurveGuides | 6 | implemented |
| S6-074 | Right-click canvas → selects hit if unselected, opens canvas context menu | right-click canvas | CanvasEngine.setupEvents (contextmenu) → UIManager.showCanvasContextMenu | 6 | implemented |
| S6-075 | Insert-vertex tool: click a wall to split it at the point | click (insert-vertex) | SketchApp.insertVertexAt/splitLineAtPoint | 6 | implemented |
| S6-076 | Undo stack: labelled snapshots (Draw, Place Text, Place Symbol, Import, Scale, Flip Opening, Opening Width, Toggle Dimensions, Move Dimension Chain, Edit Text, Auto-Assign Layer…) of lines/polygons/labels/symbols/layers/north/chains/draw chain | undo model | SketchApp.snapshotModel/pushUndo/undo/redo/undoTo/restoreDrawingChain | 6 | implemented |
| S6-077 | Delete selection (lines, areas, labels, symbols, dimensions, chains) incl. prompt-on-deletion option | Delete | SketchApp.deleteSelected | 6 | implemented |
| S6-078 | Copy/Cut/Paste selection (internal clipboard, pasted offset, re-IDed) | Ctrl+C/X/V | SketchApp.copySelected/cutSelected/paste | 6 | implemented |
| S6-079 | Move selection (drag/arrows) with area-grid snap; finalize heals joins | move | SketchApp.moveSelected/finalizeSelectionMove/getSelectionBounds | 6 | implemented |
| S6-080 | Rotate selection (handle, Shift+R 5°, Ctrl+Shift+R 1°, context 90°) | rotate | SketchApp.rotateSelected/getSelectedGeometryRotationModel | 6 | implemented |
| S6-081 | Orient selection horizontal/vertical (H/V) | keyboard | SketchApp.orientSelected | 6 | implemented |
| S6-082 | Scale selection (+/−, wheel on labels/symbols) | keyboard/wheel | SketchApp.scaleSelected | 6 | implemented |
| S6-083 | Line endpoint edit & Set Length (keeps start/end handle) | Edit Tools / drag | SketchApp.setSelectedLineLength/moveLineEndpoint/getLineEndpointRefs/snapLineEndpoint | 6 | implemented |
| S6-084 | Whole-sketch transforms: rotate ±90°, flip H/V (incl. north angle, labels, symbols) | Home ribbon / keys | SketchApp.transformSketch/rotateSketch/flipSketch | 6 | implemented |
| S6-085 | Selected area flip H/V | Edit Tools / context menu | SketchApp.flipSelectedAreas | 6 | implemented |
| S6-086 | Curve walls: bow angle (±359° max) or arc height mode; live area recalculation | curve tool / wheel / touch | SketchApp.previewCurveBow/commitCurveBow/adjustSelectedCurves; GeometryEngine.clampBowAngle/validateBowAngle/circularSegmentArea | 6 | implemented |
| S6-087 | Hit testing priority & tolerance; selection highlight | select | SketchApp.hitTestAt/isHitSelected/selectLineAt/pointInSymbol/distanceToWall | 6 | implemented |
| S6-088 | Select-by-type selects all of a kind on current page | Select menu | SketchApp.selectByType | 6 | implemented |

## Screen 7 — Areas: define, dock, attributes, Calcs (79 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S7-001 | Area Attributes dialog (read-only view of selected areas' attributes; toast if none selected) | Home ribbon › Area Attributes | area-attributes → UIManager.editAreaAttributes | 7 | implemented |
| S7-002 | View Calcs (toggle Calcs panel) | View ribbon › View Calcs | toggle-calcs → UIManager.toggleCalcsPanel | 7 | implemented |
| S7-003 | Area Breakdown report dialog | Tools ribbon › Area Breakdown | area-breakdown → AreaBreakdown.show(SketchApp.buildExportModel()) | 7 | implemented |
| S7-004 | Area Codes editor dialog | Tools ribbon › Area Codes | area-code-settings → UIManager.showAreaCodeEditor | 7 | implemented |
| S7-005 | Define Area properties dialog: Area code select, Name, Factor, Positive/Negative sign, Auto-Subtract (Automatic/None), Measure To (Centerline/Interior…), auto-label checkboxes Code/Name/Dims/Calcs/Base/Factor, Suffix; Apply/Cancel | Edit Tools › Area Props; canvas context menu › Properties… | UIManager.showSelectedAreaProperties → SketchApp.updateSelectedAreaProperties | 7 | native-adaptation |
| S7-006 | Area Attributes dialog (editable): Plan Ref Number, Floor Code, FloorCodeText, Year Built for all selected areas | Edit Tools › Area Attributes | selected-area-attributes → UIManager.showSelectedAreaAttributes → SketchApp.updateSelectedAreaAttributes | 7 | implemented |
| S7-007 | Edit area definition dialog: Area name, Line color swatch, Fill color swatch, Fill style (Solid/None/Diagonal/Crosshatch…), Fill opacity slider, live preview; 'Save Permanently' (localStorage sketch.areaCodes, recolours lines of that code) | Areas dock › Edit (with code selected) | UIManager.showAreaDefinitionEditor → SketchApp.setAreaCodes | 7 | native-adaptation |
| S7-008 | Edit selected area via prompts: name, factor override, subtraction mode auto/always/never, deduct-from area id | Areas dock › Edit (no code selected, area selected) | UIManager.bindEvents (#btn-edit-area) | 7 | implemented |
| S7-009 | New Area Code dialog: Code (uppercased, unique), Name, Description, Group (datalist), Multiplier, Include in GLA, Line/Fill colour, Fill style, Fill opacity, preview; Create Code | Areas dock › New / New area | UIManager.showAddAreaCode; AreaCodes.normalizeAreaCode | 7 | native-adaptation |
| S7-010 | Area code table editor: JSON textarea; Apply | Tools › Area Codes | UIManager.showAreaCodeEditor; AreaCodes.fromJSON | 7 | implemented |
| S7-011 | Area code table: Import JSON (file picker) | Area code table dialog | UIManager.showAreaCodeEditor | 7 | implemented |
| S7-012 | Area code table: Export JSON (download sketch-area-codes.json) | Area code table dialog | UIManager.showAreaCodeEditor; AreaCodes.toJSON | 7 | implemented |
| S7-013 | Area code table: Restore defaults | Area code table dialog | UIManager.showAreaCodeEditor; AreaCodes.DEFAULT_AREA_CODES | 7 | implemented |
| S7-014 | Calcs tree: groups per area code (header code + group net total), expandable | dock › Calcs | UIManager.updateCalcsPanel | 7 | implemented |
| S7-015 | Calcs area node: '<name> : net' expand → Base Area, Factor, Total, Subtracted, Net Area, Perimeter; click selects area | dock › Calcs | UIManager.updateCalcsPanel | 7 | implemented |
| S7-016 | Calcs Expand All | dock › Calcs › Expand All | UIManager.bindEvents (#btn-expand-all) | 7 | implemented |
| S7-017 | Calcs Collapse All | dock › Calcs › Collapse All | UIManager.bindEvents (#btn-collapse-all) | 7 | implemented |
| S7-018 | Calculation Details (visual Area Breakdown) | dock › Calcs › Calculation Details | AreaBreakdown.show(..., {visual:true}) | 7 | implemented |
| S7-019 | Calcs summary: Defined Areas, Undefined Areas (red), Total Gross Living Area, Total Gross Building Area | dock › Calcs | UIManager.updateCalcsPanel | 7 | implemented |
| S7-020 | Areas dock: status line (#area-workflow-status) with contextual guidance messages | dock › Areas | notify (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-021 | Areas dock: area-type palette — group headers (Living space, Building area, Garage/carport, Basement, Outdoor, Storage, Land, Site, Non-calculated) collapsible with counts | dock › Areas › #area-palette | paintPalette; paletteGroups; groupLabel (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-022 | Areas dock: type tiles (colour swatch, name, usage-count badge, code); tooltip 'Drag onto an outline, or click to arm. Right-click for options.' | dock › Areas palette | makeTile (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-023 | Area tile click / Enter / Space → arm type; click outlines to assign; click again to disarm | dock › Areas palette | setArmed; click → assignTo → SketchApp.defineAreaTarget (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-024 | Area tile drag onto outline/enclosed region (ghost + live coloured drop preview) → define area | dock › Areas palette drag to canvas | startTileDrag; regionAtClient; paintOverlay; SketchApp.resolveRegionAt/regionAreaAt (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-025 | 'Undefined · clear' tile (drag onto an area to clear classification) | dock › Areas palette last tile | makeTile(undefinedEntry,true) (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-026 | Area type context menu: Assign to N selected areas / Clear N selected | dock › Areas tile right-click | showTypeMenu (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-027 | Area type context menu: Assign by clicking outlines / Stop assigning | dock › Areas tile right-click | showTypeMenu → setArmed (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-028 | Area type context menu: Select the N areas using it | dock › Areas tile right-click | showTypeMenu (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-029 | Area type context menu: Edit type... | dock › Areas tile right-click | showTypeMenu → UIManager.showAreaDefinitionEditor (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-030 | Area type context menu: New type... | dock › Areas tile right-click | showTypeMenu → UIManager.showAddAreaCode (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-031 | Area type context menu: Delete type... (confirm; areas become UND; UND undeletable) | dock › Areas tile right-click | showTypeMenu → SketchApp.setAreaCodes (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-032 | 'New area' link: activates define mode + highlights undefined outlines, or switches to drawing | dock › Areas › #area-new | #area-new onclick → activate / beginDrawing (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-033 | 'Find undefined' toggle link: dashed highlight of all UND outlines | dock › Areas › #area-find-undefined | paintOverlay (findUndefined) (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-034 | 'Edit areas…' link → Edit areas sheet over panel (× close / Esc) | dock › Areas › #area-open-library | openAdvanced/closeAdvanced; buildEditSheet (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-035 | Edit areas sheet › Area operations: Redefine (armed type → selected areas) | Edit areas sheet › #btn-redefine-area | UIManager.bindEvents (#btn-redefine-area) | 7 | implemented |
| S7-036 | Edit areas sheet › Clone (cloneSelectedAreas) | Edit areas sheet › #btn-clone-area | SketchApp.cloneSelectedAreas | 7 | implemented |
| S7-037 | Edit areas sheet › Reopen (turn closed area back into open wall chain) | Edit areas sheet › #btn-reopen-area | SketchApp.reopenSelectedArea | 7 | implemented |
| S7-038 | Edit areas sheet › Detect Regions (find enclosed faces in wall network → UND areas) | Edit areas sheet › #btn-detect-regions | SketchApp.detectRegions; GeometryEngine.findEnclosedFaces | 7 | implemented |
| S7-039 | Edit areas sheet › Label defaults: auto-label checkboxes Code / Name / Dims(+Out/In placement) / Calcs / Base / Factor | Edit areas sheet (.auto-post) | UIManager.bindEvents (autoPostBindings, #autopost-dims, #dimension-placement) | 7 | implemented |
| S7-040 | Edit areas sheet › Area Label Suffix select (sf, ft², SF, FT², s.f., S.F., sq. ft., Sq. Ft., m, m², sq. m, Sq. M) | Edit areas sheet › #autopost-suffix | UIManager.bindEvents (state.autoPost.suffix) | 7 | implemented |
| S7-041 | Legacy Define panel controls (Selection summary + Edit, Classification tree + New + filter, custom name, pending code, Apply) — moved/hidden by redesign but still wired | dock › Areas (legacy DOM) | UIManager.populateAreaCodes/bindAreaTreeEvents/updateAreaActionStates; SIDEBAR REDESIGN updatePending/updateSelectionSummary | 7 | source-inactive |
| S7-042 | Legacy area tree: group expand/collapse, right-click group → 'Add Code to <group>', code button click selects, drag code onto canvas defines area | dock › Areas legacy tree | UIManager.bindAreaTreeEvents; UIManager.bindEvents drop → SketchApp.defineAreaAt | 7 | source-inactive |
| S7-043 | Legacy Apply: apply selected code/custom name to selected polygons (or first UND) | dock › Areas › #btn-apply-area | UIManager.bindEvents (#btn-apply-area) | 7 | source-inactive |
| S7-044 | Define mode (rail 'Define Areas: click inside an outline'): hover shades region + shows area value | left rail › Define Areas; canvas hover | activate → SketchApp.setMode('define-area'); trackHover/paintOverlay (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-045 | Define mode click inside outline → area chooser popup ('Define this area' / 'Edit definition', area sf) | canvas click in define-area mode | click → open (AREA DEFINITION WORKFLOW layer); SketchApp.regionAreaAt | 7 | implemented |
| S7-046 | Area chooser: 'What is this space?' search, group buttons, Classification select, Name · optional, Label settings (Code/Name/Dimensions/Area/Base/Factor + suffix), 'Keep assigning this type', Cancel / Define area\|Save changes; focus-trapped Tab | Area chooser popup | open; drawChoices (AREA DEFINITION WORKFLOW layer) → SketchApp.defineAreaTarget | 7 | implemented |
| S7-047 | Esc in define mode / chooser → back to previous drawing tool; Esc closes Edit areas sheet | keyboard Esc | document keydown capture (AREA DEFINITION WORKFLOW layer) → beginDrawing/closeAdvanced | 7 | implemented |
| S7-048 | Right-click area row in Calcs tree → area chooser (hint 'Right-click an area here or on the drawing to edit its definition.') | dock › Calcs right-click | document contextmenu (AREA DEFINITION WORKFLOW layer) → open | 7 | implemented |
| S7-049 | Canvas context menu 'Edit Definition...' when right-clicking an area | canvas right-click | UIManager.showCanvasContextMenu → areaWorkflow.open (AREA DEFINITION WORKFLOW layer) | 7 | implemented |
| S7-050 | Canvas context menu: Make Negative | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.makeNegative | 7 | implemented |
| S7-051 | Canvas context menu: Clone Area | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.cloneSelected | 7 | implemented |
| S7-052 | Canvas context menu: Auto-Subtract Area toggle | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.toggleSelectedAreaAutoSubtract | 7 | implemented |
| S7-053 | Canvas context menu: Properties… (area → Define Area dialog; else toast) | canvas right-click | UIManager.showCanvasContextMenu → UIManager.showSelectedAreaProperties | 7 | implemented |
| S7-054 | Edit Tools › Fill Pattern select (Solid/None/Diagonal/Crosshatch/…) | Edit Tools (area) | #area-fill-pattern → SketchApp.setSelectedAreaStyle('fillPattern') | 7 | native-adaptation |
| S7-055 | Edit Tools › Fill Color (colour dialog) | Edit Tools (area) | #area-fill-color → UIManager.showColorDialog → SketchApp.setSelectedAreaStyle('fillColor') | 7 | implemented |
| S7-056 | Edit Tools › Area Props | Edit Tools (area) | area-properties → UIManager.showSelectedAreaProperties | 7 | implemented |
| S7-057 | Edit Tools › Area Attributes | Edit Tools (area) | selected-area-attributes → UIManager.showSelectedAreaAttributes | 7 | implemented |
| S7-058 | Edit Tools › Auto Subtract toggle | Edit Tools (area) | auto-subtract-selected → SketchApp.toggleSelectedAreaAutoSubtract | 7 | implemented |
| S7-059 | Edit Tools › Line Style select (Solid/Dashed/Dotted) for area | Edit Tools (area) | #area-line-style → SketchApp.setSelectedAreaStyle('lineStyle') | 7 | native-adaptation |
| S7-060 | Edit Tools › Line Thickness select (1 px…) | Edit Tools (area) | #area-line-thickness → SketchApp.setSelectedAreaStyle('lineThickness') | 7 | native-adaptation |
| S7-061 | Edit Tools › Line Color (colour dialog) for area | Edit Tools (area) | #area-line-color → SketchApp.setSelectedAreaStyle('lineColor') | 7 | implemented |
| S7-062 | Edit Tools › Flip Area (horizontal) | Edit Tools (area) | flip-selected-area → SketchApp.flipSelectedAreas('horizontal') | 7 | implemented |
| S7-063 | Rail: Define Areas (arms define mode + opens Areas dock) | left rail › Define Areas | #tool-define-area → areaWorkflow.activate; UIManager.showPanel('panel-define-area') | 7 | implemented |
| S7-064 | Show Define Area (Areas) panel | keyboard: F4 | InputController.handleKeyDown → UIManager.showPanel('panel-define-area') | 7 | implemented |
| S7-065 | Toggle Calculations panel | keyboard: F6 | InputController.handleKeyDown → UIManager.toggleCalcsPanel | 7 | implemented |
| S7-066 | Double-click an area / area label / area size → area chooser | double-click area | CanvasEngine.setupEvents (dblclick) → areaWorkflow.open | 7 | implemented |
| S7-067 | Area Breakdown 'Breakdown Details' dialog: text breakdown (rectilinear partition / triangulation per area) + SVG preview | Tools › Area Breakdown; Calcs › Calculation Details | AreaBreakdown.show/computeAll/toText/previewSvg/computeForPolygon/rectilinearPartition/triangulate | 7 | implemented |
| S7-068 | Breakdown: Copy for report (clipboard) | Breakdown dialog | AreaBreakdown.show | 7 | implemented |
| S7-069 | Breakdown: Feet & inches / Decimal feet toggle | Breakdown dialog | AreaBreakdown.show (options.dimensionStyle); fmtDim | 7 | implemented |
| S7-070 | Breakdown: Download JSON (area-breakdown.json) | Breakdown dialog | AreaBreakdown.toJSON | 7 | implemented |
| S7-071 | Breakdown: Download XML (area-breakdown.xml) | Breakdown dialog | AreaBreakdown.toXml | 7 | implemented |
| S7-072 | Breakdown: toast 'Define at least one area first.' when no areas | Breakdown dialog | AreaBreakdown.show | 7 | implemented |
| S7-073 | Area calculation pipeline: gross/net, factor (effectiveMultiplier), sign (negative areas), auto-subtract overlapped/contained areas, subtractMode auto/always/never + target, measure-to centerline/interior offset rings, GLA/GBA totals per group, basement rollup | recalc | SketchApp.recalculate/recalcAutoSubtract/measureToOffsetRing/faceMeasurePolygon/basementRollupFrom; AreaCalc | 7 | implemented |
| S7-074 | Make Negative (selected area sign = −1) | context menu | SketchApp.makeNegative | 7 | implemented |
| S7-075 | Toggle auto-subtract on selected area | Edit Tools / context | SketchApp.toggleSelectedAreaAutoSubtract | 7 | implemented |
| S7-076 | Clone selected area(s) / clone selection | Areas sheet / context | SketchApp.cloneSelectedAreas/cloneSelected | 7 | implemented |
| S7-077 | Define area at point (drop code from tree), resolve enclosed region into a new area | drop / define click | SketchApp.defineAreaAt/resolveRegionAt/regionAreaAt/defineAreaTarget/ownBoundaryLine/edgeBorrowsPartOfWall | 7 | implemented |
| S7-078 | Area fill patterns / colours / opacity per code; area label auto-post (code, name, dims, area, base, factor, suffix) positioned at pole of inaccessibility; draggable/rotatable area label & size | render | CanvasEngine.drawPolygons/drawLabels/getAreaLabelGroups/getAreaLabelTransform/getAreaSizeTransform; GeometryEngine.areaLabelAnchor/poleOfInaccessibility | 7 | implemented |
| S7-079 | Selected area style setters (fill pattern/colour, line style/thickness/colour) | Edit Tools | SketchApp.setSelectedAreaStyle/updateSelectedAreaProperties/updateSelectedAreaAttributes | 7 | implemented |

## Screen 8 — Symbols: library, placement, wall snap, flip, resize (45 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S8-001 | Click symbol item → preview + arm placement (click canvas to place) | dock › Symbols list click | UIManager.updateSymbolPreview; SketchApp.startSymbolPlacement | 8 | implemented |
| S8-002 | Drag symbol item onto canvas → live placement preview while dragging, drop places | dock › Symbols list drag; canvas dragover/drop | UIManager.populateSymbolLibrary (dragstart, state.dragSymbol); UIManager.bindEvents drop → SketchApp.placeSymbolAt | 8 | implemented |
| S8-003 | Symbol context menu: Place | dock › Symbols right-click item | showSymbolContextMenu (SYMBOL LIBRARY CONTEXT MENU layer) → SketchApp.startSymbolPlacement | 8 | native-enhancement |
| S8-004 | Symbol context menu: Add to favorites / Remove from favorites | dock › Symbols right-click item | toggleFavorite (SYMBOL LIBRARY CONTEXT MENU layer); UIManager.persistLibraryState | 8 | native-enhancement |
| S8-005 | Symbol context menu: Rename... (prompt) | dock › Symbols right-click item | renameSymbol (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-006 | Symbol context menu: Delete from library (hide; confirm; placed copies keep rendering; localStorage sketch.hiddenSymbols) | dock › Symbols right-click item | deleteSymbol → hideTypes/applyHidden (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-007 | Symbol context menu: Delete permanently... (double confirm; removes placed instances; sketch.purgedSymbols) | dock › Symbols right-click item | purgeSymbol → destroyType (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-008 | Manage symbols... button (replaces Organize...; tooltip 'View, rename, delete and add symbols') | dock › Symbols › #btn-organize-symbols | install → showSymbolManager (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-009 | Restore hidden... button (shown only when hidden symbols exist; confirm) | dock › Symbols › #btn-restore-symbols | restoreAll → restoreType (SYMBOL LIBRARY CONTEXT MENU layer); syncHiddenButtons | 8 | native-enhancement |
| S8-010 | Purge hidden... button (shown only when hidden exist; double confirm) | dock › Symbols › #btn-purge-hidden | purgeHidden (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-011 | Manage Symbols dialog: filter search + category filter (All / categories / Hidden) | Manage Symbols dialog | showSymbolManager; categoryOptions; render (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-012 | Manage Symbols: rows with checkbox, icon, label, category · size in · N placed · hidden; 'N shown'; Select all shown | Manage Symbols dialog | render; selected; syncButtons (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-013 | Manage Symbols: click row → preview pane (canvas 260x200, name, meta) + Enlarge | Manage Symbols dialog | renderPreview (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-014 | Manage Symbols: row View button / Enlarge → Symbol viewer dialog (Type, Category, Wall snap) | Manage Symbols dialog | showSymbolViewer (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-015 | Manage Symbols: Add from DXF... (file picker .dxf/.dwg → new symbols in category) | Manage Symbols dialog | pickDxfFiles → importDxfFiles → dxfToSymbols/installSymbols (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-016 | Manage Symbols: Rename (exactly 1 selected) | Manage Symbols dialog | renameSymbol (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-017 | Manage Symbols: Delete / Delete (N) (hide) | Manage Symbols dialog | hideTypes/applyHidden (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-018 | Manage Symbols: Purge permanently (N) (double confirm) | Manage Symbols dialog | destroyType (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-019 | Manage Symbols: Delete category "X" (double confirm; sketch.purgedCategories) | Manage Symbols dialog | purgeCategory/addPurgedCategory (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-020 | Manage Symbols: Close / × / click outside | Manage Symbols dialog | showSymbolManager (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-021 | Export purged-symbols.json list (API) | window.SymbolLibraryMenu | exportPurged (SYMBOL LIBRARY CONTEXT MENU layer) | 8 | native-enhancement |
| S8-022 | Legacy 'Organize Favorite Symbols' dialog (filter, checkboxes, Restore Defaults, Save Favorites) — superseded by Manage symbols | UIManager (captured/overridden) | UIManager.showSymbolOrganizer | 8 | implemented |
| S8-023 | Symbol favorites persisted (localStorage sketch.symbolFavorites + saved in sketch) | persistence | UIManager.persistLibraryState / readStoredList | 8 | implemented |
| S8-024 | Canvas context menu (hosted opening selected): Flip Swing | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.flipOpening | 8 | implemented |
| S8-025 | Canvas context menu: Opening Wider (+0.5 ft) | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.adjustOpeningWidth(0.5) | 8 | implemented |
| S8-026 | Canvas context menu: Opening Narrower (−0.5 ft) | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.adjustOpeningWidth(-0.5) | 8 | implemented |
| S8-027 | Edit Tools › Flip Swing | Edit Tools (hosted opening) | flip-opening → SketchApp.flipOpening | 8 | implemented |
| S8-028 | Edit Tools › Wider | Edit Tools (hosted opening) | opening-wider → SketchApp.adjustOpeningWidth(0.5) | 8 | implemented |
| S8-029 | Edit Tools › Narrower | Edit Tools (hosted opening) | opening-narrower → SketchApp.adjustOpeningWidth(-0.5) | 8 | implemented |
| S8-030 | Rail: Symbol Library (opens Symbols dock) | left rail › Symbol Library; key S | #tool-symbols → UIManager.showPanel('panel-symbol-library') | 8 | implemented |
| S8-031 | Open Symbol library | keyboard: S (not drawing) | InputController.handleKeyDown → UIManager.showPanel('panel-symbol-library') | 8 | implemented |
| S8-032 | Drag rotation handle of symbol → rotate (15°/Ctrl 1°); detaches from wall | drag symbol rotation handle | CanvasEngine.setupEvents (rotate-symbol) | 8 | implemented |
| S8-033 | Drag symbol corner resize handle → scale 0.25–8 (step 0.05, Ctrl 0.01) | drag symbol resize handle | CanvasEngine.setupEvents (resize-symbol); CanvasEngine.getSymbolResizeHandleHit | 8 | implemented |
| S8-034 | Place symbol: click to place armed symbol with live preview; wall-hosted symbols snap into walls and orient | click (place-symbol) | SketchApp.placeSymbolAt/resolveSymbolPlacement; CanvasEngine.drawPlacementPreview | 8 | implemented |
| S8-035 | Wall-hosted openings (doors/windows): exterior swing flip, reposition on wall edit, re-snap | behaviour | SketchApp.exteriorSwingFlip/repositionHostedSymbol/refreshHostedSymbols/resnapSelectedSymbols | 8 | implemented |
| S8-036 | localStorage sketch.hiddenSymbols / sketch.purgedSymbols / sketch.purgedCategories | persistence | readHidden/writeHidden/readPurged/addPurged/readPurgedCategories (SYMBOL LIBRARY CONTEXT MENU) | 8 | native-enhancement |
| S8-037 | Wall-snap symbols (definition.wallSnap ≠ NONE): snap to nearest straight wall within tolerance, rotate to wall, store wallFraction; curved walls skipped | place-symbol | SketchApp.resolveSymbolPlacement | 8 | implemented |
| S8-038 | SIDE-snap symbols: side chosen by cursor side of wall; exterior walls default swing outward | place-symbol | SketchApp.resolveSymbolPlacement/exteriorSwingFlip | 8 | implemented |
| S8-039 | Wall gap: hosted openings cut a gap of wallGapWidth (in→ft) in the wall rendering/solids | render | SketchApp.resolveSymbolPlacement; WallSolids.openingRuns; CanvasEngine.drawWallSolids | 8 | implemented |
| S8-040 | Hosted symbols follow their wall when the wall is edited (keep fraction/side) | behaviour | SketchApp.repositionHostedSymbol/refreshHostedSymbols | 8 | implemented |
| S8-041 | Dragging a symbol re-snaps it to walls on drop | drag symbol | CanvasEngine.setupEvents pointerup → SketchApp.resnapSelectedSymbols | 8 | implemented |
| S8-042 | Flip Swing toggles flipH of selected hosted openings (toast if none) | Edit Tools / context menu | SketchApp.flipOpening | 8 | implemented |
| S8-043 | Opening width ±0.5 ft (min 0), recalculates | Edit Tools / context menu | SketchApp.adjustOpeningWidth | 8 | implemented |
| S8-044 | Placed symbol is auto-selected; tool returns to Select; symbols go on 'Symbols' layer | place-symbol | SketchApp.placeSymbolAt/ensureSymbolsLayer | 8 | implemented |
| S8-045 | Symbol rendering from library geometry (polylines/arcs/text), selection transform handles | render | CanvasEngine.drawSymbols/renderSymbolGeometry/traceSymbolPolyline/getSymbolDefinition/getSymbolTransform | 8 | implemented |

## Screen 9 — Text library, chain dimensions, room shapes, measure (46 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S9-001 | Insert Room Shape dialog: Shape (Rectangle/L-Shape), Width, Depth, Notch Width/Depth (L only), Orientation 0/90/180/270, Define as area; Place/Cancel/Esc; feet-inches parsing | left rail › Insert Room Shape | UIManager.showShapes; UIManager.parseShapeLength → SketchApp.setMode('place-shape') | 9 | implemented |
| S9-002 | Place pending room shape by clicking canvas (optionally defined as area) | canvas click in place-shape mode | SketchApp.placeShapeAt; SketchApp.getShapePoints | 9 | implemented |
| S9-003 | Text dock: 'Find or type a label...' search filters list | dock › Text › #text-lib-search | UIManager.populateTextLibrary | 9 | implemented |
| S9-004 | Text dock: Enter in search → place typed custom text (toast 'Click the drawing to place …') | dock › Text search Enter | UIManager.bindEvents → SketchApp.startTextPlacement | 9 | implemented |
| S9-005 | Text dock: label list (25 defaults ½ Bath…Wood Deck, sorted); click arms placement | dock › Text › #text-lib-list | UIManager.populateTextLibrary → SketchApp.startTextPlacement | 9 | implemented |
| S9-006 | Text dock: drag label onto canvas → drop places label (fontSize 12, area layer) | dock › Text drag; canvas drop | UIManager.populateTextLibrary (dragstart); UIManager.bindEvents drop | 9 | implemented |
| S9-007 | Text dock: alpha filter All/ABC/DEF/GHI/JKL/MNO/PQR/STU/VWX/YZ | dock › Text › .alpha-filter-btn | UIManager.bindEvents (activeTextRange) → populateTextLibrary | 9 | implemented |
| S9-008 | Text dock: Edit → Manage Text Library dialog | dock › Text › Edit | UIManager.showTextLibraryManager | 9 | implemented |
| S9-009 | Manage Text Library: list (size 14) + label input + alpha range buttons | Manage Text Library dialog | UIManager.showTextLibraryManager | 9 | implemented |
| S9-010 | Manage Text Library: Add (dup check) | Manage Text Library dialog | UIManager.showTextLibraryManager; persistLibraryState | 9 | implemented |
| S9-011 | Manage Text Library: Remove | Manage Text Library dialog | UIManager.showTextLibraryManager | 9 | implemented |
| S9-012 | Manage Text Library: Replace | Manage Text Library dialog | UIManager.showTextLibraryManager | 9 | implemented |
| S9-013 | Text library persisted (localStorage sketch.textLibraryLabels + saved in sketch) | persistence | UIManager.persistLibraryState / readStoredList / loadLibraryState | 9 | implemented |
| S9-014 | Edit Tools › Font select (Arial/Calibri/Georgia/Segoe UI/Times New Roman/Verdana…) | Edit Tools (text) | #text-font-family → SketchApp.setSelectedTextStyle('fontFamily') | 9 | native-adaptation |
| S9-015 | Edit Tools › Size number 6–72 | Edit Tools (text) | #text-font-size → SketchApp.setSelectedTextStyle('fontSize') | 9 | implemented |
| S9-016 | Edit Tools › Bold toggle | Edit Tools (text) | text-bold → SketchApp.toggleSelectedTextStyle('bold') | 9 | implemented |
| S9-017 | Edit Tools › Italic toggle | Edit Tools (text) | text-italic → SketchApp.toggleSelectedTextStyle('italic') | 9 | implemented |
| S9-018 | Edit Tools › Underline toggle | Edit Tools (text) | text-underline → SketchApp.toggleSelectedTextStyle('underline') | 9 | implemented |
| S9-019 | Edit Tools › Text Color (colour dialog) | Edit Tools (text) | #text-color → SketchApp.setSelectedTextStyle('color') | 9 | implemented |
| S9-020 | Edit Tools › Grow text (+1) | Edit Tools (text) | text-grow → SketchApp.adjustSelectedTextSize(1) | 9 | implemented |
| S9-021 | Edit Tools › Shrink text (−1) | Edit Tools (text) | text-shrink → SketchApp.adjustSelectedTextSize(-1) | 9 | implemented |
| S9-022 | Edit Tools › Text value field (edit label / area label text) | Edit Tools (text) | #selected-text-value → SketchApp.setSelectedTextStyle('text') | 9 | implemented |
| S9-023 | Rail: Measure tool (toast with instructions) | left rail › Measure | #tool-measure → setMode('measure') | 9 | implemented |
| S9-024 | Rail: Chain Dimensions tool (click a wall to dimension its full run) | left rail › Chain Dimensions | #tool-chain-dim → setMode('chain-dim') → SketchApp.handleChainClick | 9 | implemented |
| S9-025 | Rail: Insert Room Shape (opens dialog) | left rail › Insert Room Shape | #tool-shapes → UIManager.showShapes | 9 | implemented |
| S9-026 | Rail: Text Library (opens Text dock) | left rail › Text Library; key T | #tool-text → UIManager.showPanel('panel-text-library') | 9 | implemented |
| S9-027 | Finish measure run (toast if <2 points) | keyboard: Measure: Enter | InputController.handleKeyDown → SketchApp.finishMeasureRun(false) | 9 | implemented |
| S9-028 | Close run and read its area | keyboard: Measure: Shift+Enter | InputController.handleKeyDown → SketchApp.finishMeasureRun(true) | 9 | implemented |
| S9-029 | Undo last measure point | keyboard: Measure: Backspace | InputController.handleKeyDown → SketchApp.undoMeasurePoint | 9 | implemented |
| S9-030 | Clear all measurements | keyboard: Measure: Delete | InputController.handleKeyDown → SketchApp.clearMeasurements | 9 | implemented |
| S9-031 | Copy measurements as text | keyboard: Measure: Ctrl+C | InputController.handleKeyDown → SketchApp.copyMeasurements → measureSummaryText | 9 | implemented |
| S9-032 | Cancel run → clear → exit to select | keyboard: Measure: Esc | InputController.handleKeyDown → SketchApp.cancelMeasureRun / clearMeasurements / setMode('select') | 9 | implemented |
| S9-033 | Open Text library | keyboard: T (not drawing) | InputController.handleKeyDown → UIManager.showPanel('panel-text-library') | 9 | implemented |
| S9-034 | Drag rotation handle of text/area label/area size → rotate (15°/Ctrl 1°) | drag text rotation handle | CanvasEngine.setupEvents (rotate-text) | 9 | implemented |
| S9-035 | Click chain dimension to select (Ctrl toggles); drag to change offset/side | click/drag chain (select) | CanvasEngine.setupEvents (chain-offset); CanvasEngine.getChainHit | 9 | implemented |
| S9-036 | Double-click a text label (select) → prompt 'Text:' to edit | double-click label | CanvasEngine.setupEvents (dblclick) → UIManager.promptDialog | 9 | implemented |
| S9-037 | Right-click while measuring → finish run | right-click (measure) | CanvasEngine.setupEvents (contextmenu) → SketchApp.finishMeasureRun | 9 | implemented |
| S9-038 | Measure: click adds point; Shift locks leg to 45°; snaps to wall endpoints/edges; live leg readout | click/move (measure) | CanvasEngine.setupEvents → SketchApp.handleMeasureClick/measureTarget/measureSnapTarget; CanvasEngine.drawMeasurements/drawMeasureCursor/getMeasureReadout | 9 | implemented |
| S9-039 | Measure: double-click finishes run | double-click (measure) | CanvasEngine.setupEvents → SketchApp.finishMeasureRun | 9 | implemented |
| S9-040 | Chain-dim tool: click a wall to add a chain dimension of its full collinear run | click (chain-dim) | SketchApp.handleChainClick/computeChain/getChainCoverage; CanvasEngine.drawChains; GeometryEngine.computeDimensionChain | 9 | implemented |
| S9-041 | Place text: click to place armed label (dock auto-hides if unpinned) | click (place-text) | SketchApp.placeTextAt; CanvasEngine.drawPlacementPreview | 9 | implemented |
| S9-042 | Placed text uses Options text font/size/bold/italic/underline/colour; layer = area under point | place-text | SketchApp.placeTextAt/getAreaLayerIdAt | 9 | implemented |
| S9-043 | Text labels render with background/opacity options; selection box; rotation | render | CanvasEngine.drawLabels/getTextBoxTransform | 9 | implemented |
| S9-044 | Measure runs: chained legs with per-leg + total readout, close for area, snaps to corners/midpoints/edges, multiple runs kept until cleared; cleared when leaving tool | measure tool | SketchApp.handleMeasureClick/finishMeasureRun/cancelMeasureRun/undoMeasurePoint/clearMeasurements/measureSnapTarget/measureSummaryText | 9 | implemented |
| S9-045 | Chain dimensions stored per page (dimensionChains), selectable/deletable, side + offset | chain-dim | SketchApp.handleChainClick/computeChain/getChainCoverage; CanvasEngine.drawChains/getChainGeometry | 9 | implemented |
| S9-046 | Room shape templates: rectangle / L-shape with notch, rotation, optional define-as-area | place-shape | SketchApp.getShapePoints/placeShapeAt | 9 | implemented |

## Screen 10 — Pages, layers, reference image/underlay, geo-referencing (57 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S10-001 | Previous page (toast 'This is the first page.' on page 1) | View ribbon › Previous | page-prev → SketchApp.goToPage | 10 | implemented |
| S10-002 | Next page (on last page: confirm 'Add a page after the current page?' → addPage) | View ribbon › Next | page-next → SketchApp.goToPage / UIManager.confirmDialog / SketchApp.addPage | 10 | implemented |
| S10-003 | First Page | View ribbon › First Page | page-first → SketchApp.goToPage(0) | 10 | implemented |
| S10-004 | Last Page | View ribbon › Last Page | page-last → SketchApp.goToPage(last) | 10 | implemented |
| S10-005 | Add page | View ribbon › Add | page-add → SketchApp.addPage | 10 | implemented |
| S10-006 | Delete page (with confirm) | View ribbon › Delete | page-delete → SketchApp.deletePage | 10 | implemented |
| S10-007 | Load reference image (file picker image/*) | Reference Image ribbon › Load Image | #btn-photo-load → #photo-file-input → SketchApp.loadBgImage | 10 | implemented |
| S10-008 | Show reference image | Reference Image ribbon › Show Image | #btn-photo-show → SketchApp.showBgImage | 10 | implemented |
| S10-009 | Hide reference image | Reference Image ribbon › Hide Image | #btn-photo-hide → SketchApp.hideBgImage | 10 | implemented |
| S10-010 | Remove reference image | Reference Image ribbon › Remove Image | #btn-photo-remove → SketchApp.removeBgImage | 10 | implemented |
| S10-011 | Set Scale: click two points on image + enter real distance | Reference Image ribbon › Set Scale | #btn-photo-set-scale → SketchApp.startSetScale → handleScaleClick | 10 | implemented |
| S10-012 | Image Smaller (×0.8) | Reference Image ribbon › Image Smaller | SketchApp.scaleBgImageBy(0.8) | 10 | implemented |
| S10-013 | Image Larger (×1.25) | Reference Image ribbon › Image Larger | SketchApp.scaleBgImageBy(1.25) | 10 | implemented |
| S10-014 | Rotate image left 90° | Reference Image ribbon › Rotate Left | SketchApp.rotateBgImage(-90) | 10 | implemented |
| S10-015 | Rotate image right 90° | Reference Image ribbon › Rotate Right | SketchApp.rotateBgImage(90) | 10 | implemented |
| S10-016 | Image Edit Mode toggle (drag/scale image on canvas) | Reference Image ribbon › Edit Mode | photo-edit → SketchApp.togglePhotoEditMode | 10 | implemented |
| S10-017 | Orient Guide (click two points to align image) | Reference Image ribbon › Orient Guide | photo-orient → SketchApp.startPhotoOrient → handleOrientClick | 10 | implemented |
| S10-018 | Flip image horizontal | Reference Image ribbon › Horizontal | SketchApp.flipBgImageH | 10 | implemented |
| S10-019 | Flip image vertical | Reference Image ribbon › Vertical | SketchApp.flipBgImageV | 10 | implemented |
| S10-020 | Image intensity slider 0-100 (default 50) | Reference Image ribbon › Intensity range | #photo-intensity → SketchApp.setBgIntensity | 10 | implemented |
| S10-021 | Geo popup button | Reference Image ribbon › Geo | geo → UIManager.showGeoMenu | 10 | implemented |
| S10-022 | Geo menu: Load Map Image… | Geo popup | UIManager.showGeoMenu → #photo-file-input click | 10 | implemented |
| S10-023 | Geo menu: Auto Locate by Coordinate… (lat,lon prompt, decimal or DMS) | Geo popup | UIManager.showGeoReferenceDialog('coordinate'); parseGpsPair | 10 | implemented |
| S10-024 | Geo menu: Auto Locate by Address… (address prompt) | Geo popup | UIManager.showGeoReferenceDialog('address') | 10 | implemented |
| S10-025 | Layers tree: rows per area layer/group with depth indent, active highlight, hidden/locked styling; empty text 'Closed areas create layers automatically.' | dock › Layers | UIManager.updateLayersPanel; SketchApp.getAreaLayersForCurrentPage | 10 | native-adaptation |
| S10-026 | Layer row click → set active layer | dock › Layers row | UIManager.bindLayerTreeEvents → SketchApp.setActiveLayer | 10 | implemented |
| S10-027 | Layer eye button: Hide/Show layer | dock › Layers row | SketchApp.setLayerVisible | 10 | implemented |
| S10-028 | Layer lock button: Lock/Unlock layer | dock › Layers row | SketchApp.setLayerLocked | 10 | implemented |
| S10-029 | Group caret: Expand/Collapse group | dock › Layers group row | SketchApp.setLayerCollapsed | 10 | implemented |
| S10-030 | Double-click layer name → inline rename (Enter commits, Esc cancels, blur commits) | dock › Layers | UIManager.startLayerRename → SketchApp.renameLayer | 10 | implemented |
| S10-031 | Drag-and-drop layer rows: before / after / inside group | dock › Layers | UIManager.bindLayerTreeEvents → SketchApp.moveLayer | 10 | implemented |
| S10-032 | Layer context menu: Rename | dock › Layers right-click | UIManager.showLayerContextMenu → startLayerRename | 10 | implemented |
| S10-033 | Layer context menu: Hide/Show | dock › Layers right-click | SketchApp.setLayerVisible | 10 | implemented |
| S10-034 | Layer context menu: Lock/Unlock | dock › Layers right-click | SketchApp.setLayerLocked | 10 | implemented |
| S10-035 | Layer context menu: Export (layer JSON download) | dock › Layers right-click | SketchApp.exportLayer | 10 | implemented |
| S10-036 | Import layer file (.json, 'Sketch-Layer' format) into sketch — handler exists but NO UI entry point wires pickLayerImportFile in the original | layer import picker | UIManager.pickLayerImportFile → SketchApp.importLayerFile | 10 | native-enhancement |
| S10-037 | Canvas context menu: Send to Back | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.sendToBack → reorderSelected | 10 | implemented |
| S10-038 | Canvas context menu: Bring to Front | canvas right-click | UIManager.showCanvasContextMenu → SketchApp.bringToFront → reorderSelected | 10 | implemented |
| S10-039 | Rail: Sketch Geo-Referencing (Geo menu) | left rail › Geo | #tool-geo → UIManager.showGeoMenu | 10 | implemented |
| S10-040 | Rail: Pages navigator toggle (aria-pressed) | left rail › Pages navigator | #tool-pages → pages-nav IIFE setOpen | 10 | native-adaptation |
| S10-041 | Pages navigator: rows 'Page N' (click switches; current tagged 'current') | pages navigator panel | pages-nav IIFE render → SketchApp.goToPage | 10 | implemented |
| S10-042 | Pages navigator: 'ghost' toggle per row (underlay) | pages navigator row | pages-nav IIFE → SketchApp.setUnderlay | 10 | implemented |
| S10-043 | Pages navigator: delete page per row (hover; confirm) | pages navigator row | pages-nav IIFE → SketchApp.goToPage + deletePage | 10 | implemented |
| S10-044 | Pages navigator: close button | pages navigator header › #pages-nav-close | pages-nav IIFE setOpen(false) | 10 | implemented |
| S10-045 | Pages navigator: underlay select footer | pages navigator › #nav-underlay-select | pages-nav IIFE → SketchApp.setUnderlay | 10 | implemented |
| S10-046 | Underlay page ghost-rendered beneath current page | canvas | CanvasEngine.drawUnderlay; SketchApp.getUnderlayPage | 10 | implemented |
| S10-047 | Wheel in reference-image Edit Mode → scale image ×1.05/×0.95 | mouse wheel (photo edit mode) | CanvasEngine.setupEvents → SketchApp.scaleBgImageBy | 10 | implemented |
| S10-048 | Reference image Edit Mode: drag image to move | left drag (photo edit mode) | CanvasEngine.setupEvents (photo drag) | 10 | implemented |
| S10-049 | Set Scale / Orient Guide clicks on canvas | click during scale/orient | SketchApp.handleScaleClick/handleOrientClick; CanvasEngine.drawScalePoints | 10 | implemented |
| S10-050 | Layer export file '<name>-layer.json' (version 'Sketch-Layer') | Layers context › Export | SketchApp.exportLayer | 10 | implemented |
| S10-051 | Reference image file input (image/*) incl. map images; saved with sketch when option on | Reference Image › Load / Geo › Load Map Image | SketchApp.loadBgImage | 10 | implemented |
| S10-052 | Layers model: auto per-area layers (one per closed area), Symbols layer, groups, visibility/lock affect hit-testing and rendering, paint order | layers | SketchApp.ensureLayers/assignAreaToDedicatedLayer/ensureSymbolsLayer/removeEmptyAreaLayer/isEntityVisible/isEntityLocked/applyLayerOrder/flattenLayerOrder | 10 | implemented |
| S10-053 | Send to Back / Bring to Front moves selection to back/front layer + reorders | context menu | SketchApp.sendToBack/bringToFront/autoAssignLayer/reorderSelectionWithinLayers | 10 | implemented |
| S10-054 | Pages model: per-page lines/polygons/labels/symbols/northAngle/dimensionChains; switching saves current page | pages | SketchApp.getPageData/loadPageData/saveCurrentPage/goToPage/addPage/deletePage | 10 | implemented |
| S10-055 | Delete page: alert 'Cannot delete the last page.' / confirm 'Delete page N?' | pages | SketchApp.deletePage | 10 | implemented |
| S10-056 | North angle per page; 0–359 normalized; shown on status bar and in exports | north | SketchApp.setNorthAngle/normalizeNorthAngle | 10 | implemented |
| S10-057 | Reference image render with intensity, rotation, flip, scale; scale points overlay | render | CanvasEngine.drawBgImage/drawScalePoints | 10 | implemented |

## Screen 11 — File: new/open/save, print, exports, CAD import, Options (81 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S11-001 | Options dialog | Tools ribbon › Options; File backstage › Options | options → UIManager.showOptions | 11 | implemented |
| S11-002 | Options dialog shell: 6 category tabs General/Display/Hardware/System/Fonts/Drawing; × close | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-003 | Options › General › Input Options: Input mode (Pen as Ink / Pen as Mouse / Touch); Prompt on Deletion  (keys: inputMode, promptOnDeletion) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-004 | Options › General › Ink Options: Ink Confirmation; Ink Recognition (0.1/0.25/0.5)  (keys: inkConfirmation, inkRecognition) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-005 | Options › General › Unit of Measure: Imperial / Metric radio  (keys: unitSystem) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-006 | Options › General › Decimal Precision radio: 1 / .1 / .01 / .001 / .0001  (keys: decimalPrecision) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-007 | Options › General › Step Unit radio: Tenths / Half / Whole  (keys: stepUnit) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-008 | Options › General › Dimension Options: Dimension (number), Ortho ° (1-90), Auto Placement, Strip Trailing Zeroes, Same Color As Line, Area Label Suffix  (keys: dimensionOption, orthoAngle, autoPlacement, stripTrailingZeroes, sameColorAsLine, areaLabelSuffix) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-009 | Options › Display › Alignment Indicators: Lines Visible, Points Visible, Test Close Indicators, Color, Style (Dash/Solid)  (keys: alignmentLines, alignmentPoints, testCloseIndicators, alignmentColor, alignmentStyle) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-010 | Options › Display › Grid: Enabled  (keys: gridEnabled) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-011 | Options › Display › Minimum Dimension Properties: Enabled, Minimum length  (keys: minimumDimensionEnabled, minimumDimension) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-012 | Options › Display › Auto-Shade: Enabled, Percent slider, Selection Highlight  (keys: autoShade, autoShadePercent, selectionHighlight) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-013 | Options › Hardware › DISTO Discovery: Automatic / Manual, COM Port  (keys: distoDiscovery, distoComPort) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-014 | Options › Hardware › Configuration: Rounding (None/Nearest Half/Hundredth/Tenth/Whole), Drawing Mode (Standard/Continuous)  (keys: distoRounding, distoDrawingMode) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-015 | Options › System › Auto-Save: Enabled, Interval (minutes 1-120)  (keys: autoSaveEnabled, autoSaveInterval) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-enhancement |
| S11-016 | Options › System › Image Files: Size (4800x4800…1280x720), 8-Bit/24-Bit depth, Append Page Number, formats PNG/JPG/BMP  (keys: imageSize, imageDepth, appendPageNumber, imageFormats) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-017 | Options › System › Application: Disable Video Tutorial, Auto Heal Files  (keys: disableVideoTutorial, autoHealFiles) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-018 | Options › Fonts › Dimension Properties: Font (Arial/Calibri/Georgia/Segoe UI/Times New Roman/Verdana), Size (8-72), Bold, Italic, Underline  (keys: dimensionFont, dimensionFontSize, dimensionBold/Italic/Underline) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-019 | Options › Fonts › Text Label Properties: Font, Size, Bold, Italic, Underline, Color  (keys: textFont, textFontSize, textBold/Italic/Underline, textColor) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-020 | Options › Fonts › Automatic labels Properties: Font, Size, Bold, Italic, Underline, Circle Area Calcs  (keys: autoPostFont, autoPostFontSize, autoPostBold/Italic/Underline, circleAreaCalcs) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-021 | Options › Fonts › Text Background: Color, Opacity  (keys: textBackground, textBackgroundOpacity) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-022 | Options › Drawing › Area Line Properties: Drawing Color, Thickness  (keys: exteriorColor, exteriorWidth) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-023 | Options › Drawing › Interior Line Properties: Drawing Color, Thickness  (keys: interiorColor, interiorWidth) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-024 | Options › Drawing › Wall Solids: Enable wall solids, Exterior thickness (in), Interior thickness (in)  (keys: wallSolids, exteriorThickness, interiorThickness) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-025 | Options › Drawing › Witness Line Properties: Base, Interior, Curve colours, Label Opacity  (keys: witnessBaseColor, witnessInteriorColor, witnessCurveColor, labelOpacity) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-026 | Options › Drawing › Define Area Properties: Display on Close, Define First, Auto-Subtract Overlapped Areas, Quick-Shape Define, Re-Define, Split wall dimensions at area junctions  (keys: displayOnClose, defineFirst, autoSubtractOverlapped, quickShapeDefine, redefine, splitWallsAtJunctions) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-027 | Options › Drawing › Keyboard Modifiers: Pan-Zoom / Delete  (keys: panZoomDelete) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-028 | Options › Drawing › Drawing Pointer: Size 128/64/32  (keys: pointerSize) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-029 | Options › Drawing › Reference image: Save Image With Sketch, Image Quality  (keys: saveImageWithSketch, imageQuality) | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-030 | Options › Restore Defaults button | Options dialog [data-defaults] | UIManager.showOptions → SketchApp.applyOptions(defaultOptions) | 11 | native-adaptation |
| S11-031 | Options › Save as Default (localStorage sketch.defaultOptions) | Options dialog [data-save-defaults] | UIManager.showOptions; UIManager.loadLibraryState/constructor read stored defaults | 11 | native-adaptation |
| S11-032 | Options › OK applies / Cancel | Options dialog | UIManager.showOptions → SketchApp.applyOptions | 11 | native-adaptation |
| S11-033 | Page Layout Settings: Paper Size Legal/Letter/A4 radio | File › Page Layout | page-layout → UIManager.showPageLayout → SketchApp.setPageLayout | 11 | implemented |
| S11-034 | Page Layout › Current Form: Standard / Full Page / <size> Size Sketch Only / <size> W/ Calculations (per paper) | Page Layout dialog | UIManager.showPageLayout | 11 | implemented |
| S11-035 | Page Layout › Additional Options: Show Sketch Comments, Show Comment Tables, Show Page Numbers, Calculation Addendum, Deed Calls Addendum | Page Layout dialog | UIManager.showPageLayout | 11 | implemented |
| S11-036 | Page Layout › Form Captions: Sketch Page Title, Subject Label, Sketch Label, Summary Label, Comment Table 1-3 Labels | Page Layout dialog | UIManager.showPageLayout; SketchApp.normalizePageLayout | 11 | implemented |
| S11-037 | Page Layout › Restore Defaults / Apply / Cancel | Page Layout dialog | UIManager.showPageLayout; SketchApp.getDefaultPageLayout | 11 | implemented |
| S11-038 | Import CAD... (.dxf; .dwg rejected with conversion hint); result dialog lists lines/shapes/labels, units detected, skipped entities | File backstage › Import CAD... | CAD IMPORT UI LAYER install/runImport → CadImport.importDxfFile; describe | 11 | implemented |
| S11-039 | DXF parse: LINE, LWPOLYLINE/POLYLINE+VERTEX (bulge arcs), CIRCLE, ARC, TEXT/MTEXT (INSERT/POINT handled/skipped & reported), $INSUNITS or extent-based scale pick, insert into sketch | CAD import engine | parseDxfPairs, headerInsunits, readEntities, pickScale, bulgeToArcPoints, entitiesToSketch, insertIntoApp (CAD IMPORT) | 11 | implemented |
| S11-040 | New sketch (confirm if content) | File backstage › New; Ctrl+N | #file-new → SketchApp.newSketch → resetToBlank | 11 | implemented |
| S11-041 | Open... (.skx, .apx, .json, .sketch) | File backstage › Open...; Ctrl+O / F3 | #file-open → #file-open-input → SketchApp.loadFromFile → readSketchSource → applySaveModel | 11 | implemented |
| S11-042 | Save (.skx) | File backstage › Save; QAT; F2 / Ctrl+S | #file-save → SketchApp.saveToFile → buildSaveModel | 11 | implemented |
| S11-043 | Save As... (prompt name) | File backstage › Save As...; F12 / Ctrl+F12 | #file-save-as → SketchApp.saveAs | 11 | implemented |
| S11-044 | Import... (merge another sketch file into current) | File backstage › Import... | #file-import → #file-import-input → SketchApp.importFromFile → importState | 11 | implemented |
| S11-045 | Export as PNG | File backstage › Export as PNG | #file-export-png → SketchApp.exportPNG; getExportBounds | 11 | implemented |
| S11-046 | Export as SVG | File backstage › Export as SVG | #file-export-svg → SketchApp.exportSVG → Exporter.exportSVG | 11 | implemented |
| S11-047 | Export as DXF | File backstage › Export as DXF | #file-export-dxf → SketchApp.exportDXF → Exporter.exportDXF | 11 | implemented |
| S11-048 | Export as PDF (print report) | File backstage › Export as PDF | #file-export-pdf → SketchApp.exportPDF | 11 | implemented |
| S11-049 | Recent Files submenu toggle | File backstage › Recent Files | #file-recent → UIManager.renderRecentFiles | 11 | implemented |
| S11-050 | Recent file entries (name + last-opened date) → reopen from stored data | Recent Files submenu | UIManager.openRecentFile → SketchApp.loadFromFile | 11 | implemented |
| S11-051 | Recent Files › Clear ('Recent files cleared.') | Recent Files submenu › Clear | #recent-files-clear → UIManager.clearRecentFiles | 11 | implemented |
| S11-052 | Recent files empty state 'No recent files yet…' | Recent Files submenu | UIManager.renderRecentFiles | 11 | implemented |
| S11-053 | Recent files persisted: localStorage sketch.recentFiles (max 10, ≤1.5 MB each) | persistence | UIManager.recordRecentFile / getRecentFiles | 11 | implemented |
| S11-054 | Print... (print report / save PDF) | File backstage › Print...; QAT Print; Ctrl+P | data-command print → SketchApp.exportPDF | 11 | implemented |
| S11-055 | Print Preview overlay: rendered report pages, 'Print / Save PDF', 'Close Preview', page count · paper portrait | File backstage › Print Preview... | print-preview → UIManager.showPrintPreview; SketchReport.render | 11 | implemented |
| S11-056 | Page Layout dialog | File backstage › Page Layout | page-layout → UIManager.showPageLayout | 11 | implemented |
| S11-057 | Options (backstage footer) | File backstage › Options | data-command options → UIManager.showOptions | 11 | implemented |
| S11-058 | Exit (backstage footer) | File backstage › Exit | data-command exit → SketchApp.exitApp | 11 | implemented |
| S11-059 | Save | keyboard: Ctrl+S | InputController.handleKeyDown → SketchApp.saveToFile | 11 | implemented |
| S11-060 | Open (file picker) | keyboard: Ctrl+O | InputController.handleKeyDown → #file-open-input click | 11 | implemented |
| S11-061 | New sketch | keyboard: Ctrl+N | InputController.handleKeyDown → SketchApp.newSketch | 11 | implemented |
| S11-062 | Save As | keyboard: Ctrl+F12 | InputController.handleKeyDown → SketchApp.saveAs | 11 | implemented |
| S11-063 | Save | keyboard: F2 | InputController.handleKeyDown → SketchApp.saveToFile | 11 | implemented |
| S11-064 | Open (file picker) | keyboard: F3 | InputController.handleKeyDown → #file-open-input click | 11 | implemented |
| S11-065 | Save As | keyboard: F12 | InputController.handleKeyDown → SketchApp.saveAs | 11 | implemented |
| S11-066 | Close File menu | keyboard: Esc (backstage open) | UIManager.bindEvents keydown capture → SketchApp.closeFileMenu | 11 | implemented |
| S11-067 | .skx write: ZIP container (mimetype 'application/vnd.appraisal-suite.sketch' stored first, manifest.json, meta/summary.json, state/sketch.json; SHA-256 per part; profile 'sketch') | Save | SketchFile.write; WorkfileContainer.writeZip/buildManifest/sha256Hex | 11 | implemented |
| S11-068 | .skx write failure fallback → legacy JSON '.json' + toast | Save | SketchApp.saveToFile | 11 | implemented |
| S11-069 | Open format sniffing by bytes: ZIP magic → container (.skx or full .apx with state/sketch.json part); else legacy JSON (.json/.sketch) | Open / Import / Recent | SketchFile.readAny/read; WorkfileContainer.isZipBytes/readZipEntries/readEntry/parseManifest | 11 | implemented |
| S11-070 | Save model fields: version 'Sketch-Web', areaModelVersion, fileName, nextId, areaCodes, layers, activeLayerId, pages, subjectInfo, subjectInfoAutoFilled, extendedData, settings{defineFirst, dimensionsEnabled, dimensionPlacement, drawColor, canvasTheme, pageLayout, autoPost, sidebar, textLibraryLabels, symbolFavorites, options}, savedAt | save model | SketchApp.buildSaveModel | 11 | implemented |
| S11-071 | Load: migrate pages (ensurePageModel, migratePageIds, recalculateSavedPages), restore settings/areaCodes/layers, invalid-file alert | Open | SketchApp.applySaveModel/ensurePageModel/migratePageIds/recalculateSavedPages | 11 | implemented |
| S11-072 | First save of Untitled prompts 'Name your sketch:'; Save As prompts 'Save as:' | Save / Save As | SketchApp.saveToFile / saveAs | 11 | implemented |
| S11-073 | Import merges first page of another sketch offset +2 ft (lines, polygons, labels, symbols re-IDed, undo 'Import') | File › Import... | SketchApp.importFromFile | 11 | implemented |
| S11-074 | PNG export: clean render (no selection), size from Options › Image Files (default 2048x1536), fit with 60px margin, transparent, includes ref image if 'Save Image With Sketch' | File › Export as PNG | SketchApp.exportPNG/getExportBounds; CanvasEngine.renderTo | 11 | implemented |
| S11-075 | SVG export: walls (curved arcs), wall solids, areas, dimensions, auto-post labels, text, symbols | File › Export as SVG | Exporter.exportSVG/curvedWallPath/wallSolidsSVG/symbolEntities/autoPostLines/download | 11 | implemented |
| S11-076 | DXF export (ASCII DXF: LINE/POLYLINE walls, ARC curves, TEXT labels, symbol entities) | File › Export as DXF | Exporter.exportDXF/dxfPair/dxfText/flattenedPoints | 11 | implemented |
| S11-077 | PDF/Print: opens report window (SketchReport.documentHTML) per Page Layout — sketch page, subject info, area calculations summary, comment tables, addenda; popup-blocked toast | File › Export as PDF / Print | SketchApp.exportPDF; SketchReport.documentHTML/render | 11 | implemented |
| S11-078 | localStorage sketch.defaultOptions (Options › Save as Default) | persistence | UIManager.constructor / showOptions | 11 | implemented |
| S11-079 | localStorage sketch.areaCodes (custom area-code library) | persistence | UIManager.constructor / showAreaDefinitionEditor / showAddAreaCode | 11 | implemented |
| S11-080 | Exit: confirm 'Close this sketch? Save your work first if needed.' then close | File › Exit; title-bar Close | SketchApp.exitApp | 11 | implemented |
| S11-081 | New: confirm 'Start a new sketch? Unsaved changes will be lost.' → blank page, Untitled | File › New | SketchApp.newSketch/resetToBlank | 11 | implemented |

## Screen 12 — Touch panel, Bluetooth (Disto), command search, remaining dialogs (84 rows)

| ID | Feature (what the user does) | Where | Original handler | Screen # | Status |
|---|---|---|---|---|---|
| S12-001 | Subject Info dialog | Home ribbon › Subject Info | subject-info → UIManager.editSubjectInfo | 12 | implemented |
| S12-002 | Subject Info › tabs Parcel Info / General Comments / Comment Tables | Subject Info dialog | UIManager.editSubjectInfo ([data-subject-tab]) | 12 | implemented |
| S12-003 | Subject Info › Parcel fields: Parcel No., File No., Property Address, City, State, ZipCode, Owner, County, Client, Client Address, Appraiser Name, Inspection Date, GPS Coordinates (validated decimal/DMS) | Subject Info dialog | UIManager.editSubjectInfo; parseGpsPair/formatGpsPair | 12 | implemented |
| S12-004 | Subject Info › Auto-Fill from suite report (embedded only) | Subject Info dialog [data-auto-fill] | UIManager.autoFillSubjectInfo; mergeSubjectInfo | 12 | suite-only |
| S12-005 | Subject Info › read-only metadata (Created By / Modified By / Format Version) | Subject Info dialog | UIManager.editSubjectInfo | 12 | implemented |
| S12-006 | Subject Info › 'Expand additional fields' 3 name/value pairs | Subject Info dialog details | UIManager.editSubjectInfo (extraName/extraValue) | 12 | implemented |
| S12-007 | Subject Info › General Comments textarea | Subject Info dialog | UIManager.editSubjectInfo | 12 | implemented |
| S12-008 | Subject Info › Save Default comments (localStorage sketch.defaultSubjectComments) | Subject Info dialog [data-save-default] | UIManager.editSubjectInfo | 12 | implemented |
| S12-009 | Subject Info › Apply Default comments | Subject Info dialog [data-apply-default] | UIManager.editSubjectInfo | 12 | implemented |
| S12-010 | Subject Info › Comment Table 1-3 textareas (line lists) | Subject Info dialog | UIManager.editSubjectInfo | 12 | implemented |
| S12-011 | Subject Info › OK (validate + save) / Cancel / × close | Subject Info dialog | UIManager.editSubjectInfo (form submit) | 12 | implemented |
| S12-012 | Mini-Map toggle | View ribbon › Mini-Map | mini-map → UIManager.toggleMiniMap | 12 | implemented |
| S12-013 | Connect Bluetooth (Disto) device | Tools ribbon › Connect | disto-connect → SketchApp.connectDisto | 12 | hardware-deferred |
| S12-014 | Disto connection status indicator ('Disconnected'/connected device) | Tools ribbon › #disto-status (disabled) | SketchApp.connectDisto updates #disto-status | 12 | hardware-deferred |
| S12-015 | Keyboard Shortcuts cheat sheet (7 groups, Close/Esc/F1 closes) | Help ribbon › Keyboard Shortcuts; F1 | shortcuts → UIManager.showShortcuts | 12 | implemented |
| S12-016 | Hidden commands activate-module / deactivate-module (photometrics module toggle) | executeCommand only | UIManager.setPhotometricsModule | 12 | source-inactive |
| S12-017 | Extended Data command → toast 'Extended Data is password-protected… intentionally omitted' | executeCommand 'extended-data' | UIManager.editExtendedData | 12 | source-inactive |
| S12-018 | Mini-Map overlay (PNG snapshot of canvas) with × Close Mini-Map | View › Mini-Map | UIManager.toggleMiniMap | 12 | implemented |
| S12-019 | Touch Panel floating 'Touch Drawing' window toggle (× close) | left rail › Touch Panel; status bar Touch Panel; executeCommand touch-panel | UIManager.showTouchPanel | 12 | implemented |
| S12-020 | Touch Panel › Distance input (default 1, min 0.1) | Touch Panel | UIManager.showTouchPanel | 12 | implemented |
| S12-021 | Touch Panel › ↑ ← ↓ → draw a wall of the distance (auto-starts exterior draw + pen down) | Touch Panel | UIManager.showTouchPanel → SketchApp.penDown / SketchApp.drawLine | 12 | implemented |
| S12-022 | Touch Panel › Turn Left / Turn Right (loads '<d>l90'/'<d>r90' into input buffer) | Touch Panel | UIManager.showTouchPanel (InputController.buffer (property)) | 12 | implemented |
| S12-023 | Touch Panel › Curve Left / Curve Right (±15° bow on selected) | Touch Panel | SketchApp.adjustSelectedCurves(±15) | 12 | implemented |
| S12-024 | Touch Panel › Delete | Touch Panel | SketchApp.deleteSelected | 12 | implemented |
| S12-025 | Touch Panel › Insert Vertex | Touch Panel | SketchApp.setMode('insert-vertex') | 12 | implemented |
| S12-026 | Email... (email support) | File backstage › Email...; Ctrl+F1 | data-command email → SketchApp.emailSupport | 12 | implemented |
| S12-027 | Rail: Touch Panel | left rail › Touch Panel | #tool-touch → UIManager.showTouchPanel | 12 | implemented |
| S12-028 | Command palette: Ctrl+K / QAT magnifier opens; type to filter; ↑/↓ move; Enter runs; Esc / backdrop closes; 'No matching command.' | command palette | command palette IIFE open/render/move/run/close → UIManager.executeCommand | 12 | implemented |
| S12-029 | Command palette entry 'Save' (id save) | command palette | CATALOG → UIManager.executeCommand('save') | 12 | implemented |
| S12-030 | Command palette entry 'Save As...' (id save-as) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('save-as') | 12 | native-enhancement |
| S12-031 | Command palette entry 'New sketch' (id new) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('new') | 12 | native-enhancement |
| S12-032 | Command palette entry 'Open...' (id open) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('open') | 12 | native-enhancement |
| S12-033 | Command palette entry 'Import...' (id import) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('import') | 12 | native-enhancement |
| S12-034 | Command palette entry 'Export as PNG' (id export-png) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('export-png') | 12 | native-enhancement |
| S12-035 | Command palette entry 'Export as SVG' (id export-svg) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('export-svg') | 12 | native-enhancement |
| S12-036 | Command palette entry 'Export as DXF' (id export-dxf) — NOT in executeCommand map: original shows toast "<id> is not available." | command palette | CATALOG → UIManager.executeCommand('export-dxf') | 12 | native-enhancement |
| S12-037 | Command palette entry 'Print' (id print) | command palette | CATALOG → UIManager.executeCommand('print') | 12 | implemented |
| S12-038 | Command palette entry 'Print preview' (id print-preview) | command palette | CATALOG → UIManager.executeCommand('print-preview') | 12 | implemented |
| S12-039 | Command palette entry 'Page layout' (id page-layout) | command palette | CATALOG → UIManager.executeCommand('page-layout') | 12 | implemented |
| S12-040 | Command palette entry 'Options' (id options) | command palette | CATALOG → UIManager.executeCommand('options') | 12 | implemented |
| S12-041 | Command palette entry 'Undo' (id undo) | command palette | CATALOG → UIManager.executeCommand('undo') | 12 | implemented |
| S12-042 | Command palette entry 'Redo' (id redo) | command palette | CATALOG → UIManager.executeCommand('redo') | 12 | implemented |
| S12-043 | Command palette entry 'Undo history' (id undo-history) | command palette | CATALOG → UIManager.executeCommand('undo-history') | 12 | implemented |
| S12-044 | Command palette entry 'Cut' (id cut) | command palette | CATALOG → UIManager.executeCommand('cut') | 12 | implemented |
| S12-045 | Command palette entry 'Copy' (id copy) | command palette | CATALOG → UIManager.executeCommand('copy') | 12 | implemented |
| S12-046 | Command palette entry 'Paste' (id paste) | command palette | CATALOG → UIManager.executeCommand('paste') | 12 | implemented |
| S12-047 | Command palette entry 'Select all' (id select-all) | command palette | CATALOG → UIManager.executeCommand('select-all') | 12 | implemented |
| S12-048 | Command palette entry 'Select by type' (id select-by-type) | command palette | CATALOG → UIManager.executeCommand('select-by-type') | 12 | implemented |
| S12-049 | Command palette entry 'Delete selection' (id delete-selected) | command palette | CATALOG → UIManager.executeCommand('delete-selected') | 12 | implemented |
| S12-050 | Command palette entry 'Rotate sketch' (id rotate-sketch) | command palette | CATALOG → UIManager.executeCommand('rotate-sketch') | 12 | implemented |
| S12-051 | Command palette entry 'Flip sketch' (id flip-sketch) | command palette | CATALOG → UIManager.executeCommand('flip-sketch') | 12 | implemented |
| S12-052 | Command palette entry 'Draw color' (id draw-color) | command palette | CATALOG → UIManager.executeCommand('draw-color') | 12 | implemented |
| S12-053 | Command palette entry 'Subject info' (id subject-info) | command palette | CATALOG → UIManager.executeCommand('subject-info') | 12 | implemented |
| S12-054 | Command palette entry 'Area attributes' (id area-attributes) | command palette | CATALOG → UIManager.executeCommand('area-attributes') | 12 | implemented |
| S12-055 | Command palette entry 'Mini map' (id mini-map) | command palette | CATALOG → UIManager.executeCommand('mini-map') | 12 | implemented |
| S12-056 | Command palette entry 'View calcs' (id toggle-calcs) | command palette | CATALOG → UIManager.executeCommand('toggle-calcs') | 12 | implemented |
| S12-057 | Command palette entry 'Previous page' (id page-prev) | command palette | CATALOG → UIManager.executeCommand('page-prev') | 12 | implemented |
| S12-058 | Command palette entry 'Next page' (id page-next) | command palette | CATALOG → UIManager.executeCommand('page-next') | 12 | implemented |
| S12-059 | Command palette entry 'Add page' (id page-add) | command palette | CATALOG → UIManager.executeCommand('page-add') | 12 | implemented |
| S12-060 | Command palette entry 'Delete page' (id page-delete) | command palette | CATALOG → UIManager.executeCommand('page-delete') | 12 | implemented |
| S12-061 | Command palette entry 'Canvas background' (id toggle-canvas-background) | command palette | CATALOG → UIManager.executeCommand('toggle-canvas-background') | 12 | implemented |
| S12-062 | Command palette entry 'Interface theme' (id interface-theme) | command palette | CATALOG → UIManager.executeCommand('interface-theme') | 12 | implemented |
| S12-063 | Command palette entry 'Toggle grid' (id toggle-grid) | command palette | CATALOG → UIManager.executeCommand('toggle-grid') | 12 | implemented |
| S12-064 | Command palette entry 'Filter selection' (id filter) | command palette | CATALOG → UIManager.executeCommand('filter') | 12 | implemented |
| S12-065 | Command palette entry 'Connect Bluetooth device' (id disto-connect) | command palette | CATALOG → UIManager.executeCommand('disto-connect') | 12 | hardware-deferred |
| S12-066 | Command palette entry 'Area breakdown' (id area-breakdown) | command palette | CATALOG → UIManager.executeCommand('area-breakdown') | 12 | implemented |
| S12-067 | Command palette entry 'Area codes' (id area-code-settings) | command palette | CATALOG → UIManager.executeCommand('area-code-settings') | 12 | implemented |
| S12-068 | Command palette entry 'Keyboard shortcuts' (id shortcuts) | command palette | CATALOG → UIManager.executeCommand('shortcuts') | 12 | implemented |
| S12-069 | Command palette entry 'Zoom fence' (id zoom-fence) | command palette | CATALOG → UIManager.executeCommand('zoom-fence') | 12 | implemented |
| S12-070 | Command palette entry 'Email' (id email) | command palette | CATALOG → UIManager.executeCommand('email') | 12 | implemented |
| S12-071 | Email support | keyboard: Ctrl+F1 | InputController.handleKeyDown → SketchApp.emailSupport | 12 | implemented |
| S12-072 | Keyboard shortcuts cheat sheet | keyboard: F1 | InputController.handleKeyDown → UIManager.showShortcuts | 12 | implemented |
| S12-073 | Blur the field (keys in inputs don't trigger shortcuts) | keyboard: Esc in focused text input/select | InputController.handleKeyDown → InputController.handleKeyDown | 12 | implemented |
| S12-074 | Open/close command palette | keyboard: Ctrl+K | command palette IIFE keydown | 12 | implemented |
| S12-075 | Cancel / OK (alert/confirm/prompt dialogs) | keyboard: Esc / Enter in modal dialogs | UIManager.showDialog | 12 | implemented |
| S12-076 | Close shortcuts dialog | keyboard: Esc / F1 in cheat sheet | UIManager.showShortcuts | 12 | implemented |
| S12-077 | localStorage sketch.defaultSubjectComments | persistence | UIManager.editSubjectInfo | 12 | implemented |
| S12-078 | Suite embedding: title bar QAT/title moved into tab row when inside a parent frame | embedded mode | inline IIFE (suite-embedded) | 12 | suite-only |
| S12-079 | Suite hooks: __exportState, __importState, __exportValidationState, __exportGla, __exportBasement, __setSubjectInfo, __clearSketch, __refreshSuitePreview, __exportPreviews/__exportPreview; 'suite:sketchpreview' event with rendered report-page JPEGs | window API (embedded) | window.onload hooks; SketchApp.pushSuitePreview/renderSuitePreviews | 12 | suite-only |
| S12-080 | Toast notifications (bottom, 3.2 s) | global | UIManager.notify | 12 | implemented |
| S12-081 | Modal alert / confirm / prompt dialogs (title 'SketchQuoder'; Enter=OK, Esc=Cancel, click-outside cancels) | global dialogs | UIManager.alertDialog/confirmDialog/promptDialog/showDialog | 12 | implemented |
| S12-082 | Command popups / context menus close on outside pointerdown; positioned within viewport | global | UIManager.showCommandPopup | 12 | implemented |
| S12-083 | Disto connect: Web Bluetooth requestDevice (acceptAllDevices); status label shows device name; toasts for unavailable / not completed | Tools › Connect; status Bluetooth | SketchApp.connectDisto | 12 | hardware-deferred |
| S12-084 | Email support: mailto thatappraiserx@gmail.com subject 'Appraisal Sketch Support' | File › Email...; Ctrl+F1 | SketchApp.emailSupport | 12 | implemented |

## Totals

| Screen | Rows |
|---|---|
| 1 — Static chrome, default state | 15 |
| 2 — Ribbon tabs + File backstage + contextual Edit Tools | 19 |
| 3 — Dock tabs, pin/collapse, resize | 20 |
| 4 — Status bar controls | 36 |
| 5 — Canvas: grid, pan/zoom, drawing walls, snap, keyboard entry | 64 |
| 6 — Selection / edit / undo-redo + history | 88 |
| 7 — Areas: define, dock, attributes, Calcs | 79 |
| 8 — Symbols: library, placement, wall snap, flip, resize | 45 |
| 9 — Text library, chain dimensions, room shapes, measure | 46 |
| 10 — Pages, layers, reference image/underlay, geo-referencing | 57 |
| 11 — File: new/open/save, print, exports, CAD import, Options | 81 |
| 12 — Touch panel, Bluetooth (Disto), command search, remaining dialogs | 84 |
| **Total** | **634** |

## Behaviours present in the UI but inert or incomplete in the original (port the UI + persistence; do not invent behaviour)

- **Command-palette ids with no executeCommand entry**: `save-as`, `new`, `open`, `import`, `export-png`, `export-svg`, `export-dxf` are listed in the palette CATALOG (l.2699) but absent from the `UIManager.executeCommand` map (l.12265), so the original shows the toast "`<id> is not available.`" Decide explicitly whether the port reproduces or fixes this.
- **Options keys collected/saved but never read by any behaviour** (outside the dialog and defaults): `inputMode, inkConfirmation, inkRecognition, orthoAngle, autoPlacement, testCloseIndicators, alignmentColor, alignmentStyle, autoShade, autoShadePercent, selectionHighlight, distoDiscovery, distoComPort, distoRounding, distoDrawingMode, autoSaveEnabled, autoSaveInterval, imageDepth, appendPageNumber, imageFormats, disableVideoTutorial, autoHealFiles, circleAreaCalcs, labelOpacity, displayOnClose, panZoomDelete, imageQuality`.
- **No autosave, no checkpoints, no sessionStorage/IndexedDB** in the standalone: despite the Auto-Save option, there is no timer. Persistence is limited to the explicit save file, `sketch.recentFiles` (stored file text) and the localStorage preference keys listed in the rows.
- **Layer API with no UI entry point**: `SketchApp.addLayer` (internal via `autoAssignLayer`), `duplicateLayer`, `moveSelectionToLayer`, `removeLayer`, and `UIManager.pickLayerImportFile` → `SketchApp.importLayerFile` are defined but never wired to a button or menu. They are listed only where reachable (the import row is flagged).
- `extended-data` only shows a toast ("password-protected… intentionally omitted"). `window-minimize` only shows a toast in the browser; the native port should really minimise.
- Disto: Web Bluetooth `requestDevice({acceptAllDevices:true})` only names the device. No measurement stream is consumed, and it can't be verified without hardware.
- `AreaBreakdown.show(..., {visual:true})` (Calculation Details) ignores `visual`, so it is the same dialog as Tools › Area Breakdown.
- In drawing, there is no Shift-ortho. Ortho/angle constraint while drawing comes from `GeometryEngine.polarSnap` (5° steps, when Snap is on). Shift locks to 45° only in the Measure tool (`GeometryEngine.constrainToAngleStep`). Ctrl changes rotate/resize/curve step sizes.
- Legacy *Organize Favorite Symbols* dialog (`UIManager.showSymbolOrganizer`) is shadowed by *Manage symbols...* (capture-phase handler). Legacy Define-panel controls (tree, Apply, Edit, name input) are moved into the *Edit areas* sheet or hidden by the redesign layers, but they are still wired.

## Items that could not be classified cleanly (screen chosen, and why)

- Mini-Map (View tab): no screen names it. Put in **12** (remaining dialogs).
- Canvas background / Interface theme menus and `sketch.interfaceTheme`: put in **2** (View tab). They also affect every screen's look.
- Subject Info dialog, Email, Extended Data, suite-embedding hooks (`window.__export*`, `suite:sketchpreview`), toasts and modal-dialog infrastructure: put in **12**.
- Filter popover (View › Filter and status-bar Filter): placed in **4** (status bar). The View-ribbon entry is row S6 (filter selection).
- Draw Color + colour dialog: placed in **5** (drawing). The colour dialog is reused by Edit Tools (7/9) and area-code editors (7).
- Area Breakdown / area-code table / Calcs: placed in **7**, although the buttons live on the Tools ribbon (screen 2 chrome).
- Send to Back / Bring to Front: placed in **10** (layers), because they reassign layers. They are reached from the selection context menu (6).
- Options dialog: all groups placed in **11** per ACCEPTANCE, even where a setting drives screen-5 drawing behaviour (wall solids, pointer size, step unit).
- Command-palette entries: all in **12**. Their target behaviours are also listed under their own screens.
- Window title-bar drag (native only; the browser has no equivalent handler): **1**.
