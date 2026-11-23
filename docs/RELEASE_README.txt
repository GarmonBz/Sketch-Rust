Sketch Rust 0.3.13 - Windows x64

0.3.13 adds Apex AXF import and export to the Menu and fixes the Areas list
across pages. The Menu's Import fly-out gains "Apex AXF sketch…" (a legacy
`.axf` opens as a converted, untitled copy, like File > Open always did) and
the Export fly-out gains "Apex AXF", which writes the current page — every
closed area with its arc walls, plus text labels — as a legacy `.axf` using a
clean-room encoder (no Apex or proprietary code; the importer's own documented
byte layouts, written back out). The Areas list now shows only the current
page's areas: a layer whose area lives on another page appears on that page
instead of every page (the Site Plan screenshot bug).

Sketch Rust 0.3.12 - Windows x64

0.3.12 fixes arrow-key nudging of a selected wall or area: the selection moved
diagonally, about ten feet sideways per key press, because the port used Rust's
f64::signum (which returns 1.0 for zero) where the original used JavaScript's
Math.sign (which returns 0). Nudge steps now use the JS sign semantics, so an
arrow key moves the selection exactly one snap pitch along the arrow — Shift
still moves ten pitches — and shared walls stretch cleanly without shearing the
areas they border.

0.3.11 stops File > Open and Save freezing for good. The window no longer runs
a nested Win32 message pump while a native file dialog initializes — that pump
deadlocked the interface thread whenever the Shell's dialog start stalled (a
remembered network share was enough). Dialogs now open on their own thread and
are awaited by the event loop, every dialog flow is a message-driven
continuation, and each dialog starts in an explicit local folder (Documents
fallback) instead of the last-visited one (0.3.9/0.3.10 hardened the same
anchor).

Sketch Rust 0.3.8 - Windows x64

0.3.8 fixes UI hangs and freezes during drawing, dragging, and panning:
exterior wall-join healing no longer stalls on multi-room floorplans with open
wall chains, T-junction welds split the host wall cleanly, area label placement
skips expensive cell refinement when a room's centroid is already well clear of
its walls, unsaved-change checks no longer re-serialize .sketchie backing data
or run mid-drag, chrome and menu hovers keep the cached floorplan render intact,
and dotted/hatched area patterns clip to the visible canvas viewport.

0.3.7 stops the window freezing ("Not Responding") while an Open or Save
dialog is open and speeds up redrawing sketches with many layers. Dragging a
wall end or area corner, or moving lines and areas, now snaps onto nearby
endpoints as well as the grid. The installer opens .sketchie and .skx files on
a double click and adds Sketch Rust to Open with for .json (.sketchie.json).
The Fallingwater demo is listed under Recent sketches on the first start, and
the start screen has a "Show this screen when Sketch Rust starts" checkbox.

0.3.6 brings an area back when you redraw a wall you deleted from it, even next
to neighbouring areas (it failed on areas whose walls other areas touch). The
Areas list now shows every layer and can delete areas and layers from a row
menu; single-layer export is removed from that menu.

0.3.5 adds one-click eye, lock and menu on every layer row (the eye greys out
when a layer is hidden), an Apply to all labels button, dark label text that
turns light on Dark and Blue canvases, Ctrl+click toggling of labels and
symbols, an angle snap that follows its setting, arrow keys that nudge by the
snap pitch, and layers that paint in list order so walls no longer cover the
areas of layers above them.

0.3.4 opens the Open and Save dialogs on a thread of their own, so they no
longer hang on "Working on it..." in the Downloads folder.

0.3.3 adds a grab handle to every row of the Areas and layers list for dragging,
makes Bold work on area labels (the name starts bold and the button turns it off
or on), and stops Save As Sketchie from refusing older files: Sketchie 0.2 is
the only version supported. fallingwater.sketchie, a full example sketch, is
now in the install folder.

0.3.2 combines Areas and Layers into one list. Each area shows its name with
its type and size underneath, folders group areas and layers, and every row has
an eye to hide it and a lock. Drag any row up or down, or into a folder; the
row menu moves an area to another layer. The new-folder button sits beside the
menu. The separate Layers list and its divider are gone.

0.3.2 also opens legacy .axf sketches as a converted new sketch (areas, arc
walls and room labels; doors and fixtures are not imported yet). Save As keeps
the copy as .sketchie; the .axf file is never changed.

0.3.1 adds Door Garage 216, an 18 ft double garage door, to Doors & windows in
place of the old Door - Garage-Dbl. A selected symbol's box and handles now sit
on the symbol instead of being offset from it, and symbols draw in light ink on
the Dark and Blue canvases.

0.3.0 gives Layers more room (drag the divider to resize), replaces the page
tabs with a page switcher that fits the top bar, closes the right-click menu
on a canvas click or Escape, and renames default area names when the type
changes. GBA5-GBA10 are removed and Carport is CPT. Light, Blue, Dark is the
order for both theme selectors. The Arrows, Doors, Windows and Miscellaneous
symbol categories are removed from the default library.

The tool tray and drawing controls now share a layout that prevents overlap.
Zoom and Grid/Snap/Show/Dimensions move to separate top rows when there is
not enough canvas width beside the twelve-tool tray. Menus follow their
controls, including after sidebar resizing and horizontal tool scrolling.

Freeform and Erase are back in the tool tray and command search. Drag with
Freeform to draw without snapping, dimensions, or wall solids. Click or drag
with Erase to remove lines, room labels, and symbols. Each gesture is one
Undo step (Ctrl+Z). Both tools support mouse and touch input, respect hidden
and locked layers, and use the existing save, print, and export formats.

0.2.7 snaps moves to the grid, deselects on empty canvas clicks, and adds
Open and Save buttons. 0.2.6 adds sketch-wide length formats and edit snapping.

0.2.5: Area outlines and fills use their assigned area colors. A magnifier beside
Fit zooms to a dragged rectangle. Selected walls and areas show draggable
endpoints and corners. The inspector combines properties and editing
controls, and wall length accepts decimal feet or inches.

0.2.4 fixes door and window wall gaps so they use the symbol library
alignment. An opening follows its symbol to a new wall or closes when
detached.

0.2.3 tidies the symbol library. Categories have plain names (Doors &
windows, Kitchen, Bath & plumbing, Utilities and so on), and the CADdillo
symbols are merged into the matching categories. 67 older symbols that
a newer one replaces are removed, such as the old 36-inch refrigerator
(now Fridge 36), leaving 572 symbols. When you open a sketch that uses a
removed symbol, the newer one takes its place at the same position,
rotation and size. It can differ a little in shape. The right-click menu
is redesigned with icons, groups and shortcuts, and Options and Keyboard
Shortcuts fit a 250% scaled screen without scrolling.

0.2.2 gives the Dark theme a new palette made for CAD work: deep
blue-black surfaces, a brighter blue for the active tool and selection,
crisper text, and a darker drawing background with subtle grid lines.
The top bar sits on the darkest surface. Choose Options > Display >
Drawing background > Dark to draw on the new dark background. A selected
area is tinted violet in every theme. Area type colours are unchanged.

0.2.1 refines the new interface. Icons are back in colour: blue for
selecting and viewing, orange for drawing and editing, red for removing
and the Area tool, green for files and printing. The Menu is now one
short column with a search field on top; Ctrl K opens it, and typing
finds any command. Interface theme, drawing background and control size
moved to Options > Display. Options, Property Details, Page Layout,
Print Preview and Keyboard Shortcuts fit a 1920 x 1080 screen without
scrolling. The first area you close is centred in view, area numbers
use one decimal everywhere, and a selected area keeps its type's colour.

0.2.0 replaces the ribbon, tool rail, dock and status bar with a top bar,
a floating tool tray, the Areas ledger, an inspector for the selection,
an Insert drawer, drawing controls and a start screen. The layout adapts
to tablets and phones, with a number Pad for touch drawing.

0.1.35 adds native open and save for Sketchie 0.2 JSON workfiles and
SQLite revision stores. Existing Sketchy files save in their original
format and version. Use Save As to create a Sketchie copy of a Sketchy
0.2 file. See SKETCHY.md for details.

0.1.34 sends sketch pages to legacy Appraise-It Pro reports that support
repeatable Sketch addenda. In Help > Appraise-It Reports, check the open
report, preview the pages, and select Send sketch images. Review the
new addenda and save the report in Appraise-It Pro. Enter legacy area
totals in Appraise-It Pro.


0.1.33 adds optional area totals to Appraise-It Pro sketch transfer. In
Help > Appraise-It Reports, preview the sketch and select Include area totals
to review the proposed standard finished above-grade, finished below-grade,
and unfinished below-grade values. The option is available for UAD 3.6 URARs
with one dwelling, one living unit, and no ADUs.

0.1.32 transfers sketch page images to a saved, editable UAD 3.6 report
using the installed SFREP COM server. Open Help > Appraise-It Reports to
select the report, preview the pages, and add them to Additional Sketch(es).
Review the images and save the report in Appraise-It Pro.

0.1.31 opens Sketchy 0.2 JSON files and SQLite stores as Saved. Editing
marks the file Unsaved, and saving returns it to Saved.

File > Open accepts Sketchie 0.2 .sketchie.json workfiles and .sketchie
revision stores, as well as the earlier Sketchy file types. Save writes edits
to the open workfile. Save As offers Sketchie JSON and Sketchie Store. Copy
try-sketchie.sketchie.json to Documents, open the copy, move a wall, save,
and reopen it. fallingwater.sketchie is a full multi-page example sketch;
open it from the install folder or copy it first. See SKETCHY.md for format
details.

See LICENSE.txt for the Sketch Rust Free Use License. Paid appraisal work is
permitted. Selling or monetizing the program or modified versions is prohibited.
See PHOSPHOR-LICENSE.txt and SYMBOL-LIBRARY.md for third-party materials.

0.1.30 removes AXF and the former Apex-based Appraise-It Pro integration.
See APPRAISE_IT_PRO_INTEGRATION.md for SFREP transfer requirements and older-preview guidance.

Help > User Guide covers drawing, area calculations, symbols, printing and saving.
A Windows installer (SketchRust-<version>-x64.msi) is available next to this
zip: it installs for the current user without administrator rights, offers a
choice of folder and an optional desktop shortcut, upgrades older installer
builds in place, and can be removed from Windows Settings > Apps. Upgrading
replaces symbols.json, so keep a copy if you have edited the symbol library.

0.1.28 fixes a crash: opening File > Recent Files with at least one entry in the
list closed the app.

0.1.27 reduces input lag while drawing. Frames are now shown as soon as they are
drawn instead of waiting for the display refresh, so the wall preview keeps up
with the mouse cursor. If you see tearing, set the environment variable
ICED_PRESENT_MODE=vsync before starting Sketch to restore the old behavior.
Help > About shows the version, so 0.1.27 can be told apart from earlier builds.

0.1.26 improves the window. The title-bar button now maximizes and restores
instead of going full screen, so the taskbar stays visible. The window remembers
its size, position and maximized state, can be resized from its edges and
corners, and has a minimum size of 800 x 560. The title bar shows Unsaved or
Saved, and closing with unsaved changes asks Save / Don't save / Cancel. Drop a
sketch file on the window to open it, or pass it on the command line; a file
that is already open brings its window forward. The caret beside the title-bar
buttons opens New, Open, Save As and Options. Touch pinch zooms and two-finger
drag pans. CADdillo door symbols now sit on the opening they cut in the wall
(the cut used to start at the door's middle). Help > About shows the version.
The Edit Tools tab is now highlighted only while it is the tab showing.

0.1.25 simplifies symbol deletion. Delete permanently removes definitions from
symbols.json and placed instances from every page, undo history and clipboard.
Delete category does the same for a whole category. The previous library file
is kept as symbols.json.bak. Purge, Purge hidden, the Purged view and its export
option are removed; Delete forever is now labeled Delete, including right-click
actions. Hide/Restore remains available for temporary visibility changes.
A failed library write reports an error and changes nothing, so Delete never
silently becomes a reversible removal. Older removal preferences stay excluded
and cannot be restored through the manager.

0.1.24 adds 177 CADdillo residential symbols in ten categories, with correct
inch sizes, centered door/window insertion and wall snapping. Both resizing
controls now scale the wall opening with its symbol. The DXF importer resolves
BLOCK/INSERT geometry, nested transforms, ellipses and solid arrowheads instead
of silently skipping the fixture. Promotional source labels are excluded.
The original 462 definitions remain available for existing sketches, and the
unchanged original catalog ships separately as symbols.previous.json. Existing
personal favorites and manager preferences are preserved. Read SYMBOL-LIBRARY.md
for dimensions, source/license and restore instructions.

0.1.23 fixes Delete category in the symbol manager, verified with Fire.
Opening Manage symbols from a real dock category keeps that category selected.
Category deletion opens a centered confirmation naming the category and its
symbol count, followed by the final confirmation. Cancel or Escape returns to
the manager. After deletion, both category pickers, the list and preview refresh;
the manager returns to All instead of retaining a deleted filter. Empty
categories can also be removed. In 0.1.25, Delete category permanently updates
symbols.json and keeps symbols.json.bak.

0.1.19 adds side-by-side AREA CALCULATIONS SUMMARY and AREA CALCULATIONS
BREAKDOWN boxes at the bottom of Print Preview, Print and PDF export. Both
boxes are centered within the printable page, with area codes, factors, net
sizes, perimeters, calculation pieces and Net LIVABLE totals. Long tables
continue onto additional numbered pages instead of losing rows. Populated
comment tables print in a separate strip above the calculation boxes; empty
comment grids are omitted. Letter, Legal and A4 share this layout. No logo or
copyright notice is appended. Preview Print writes the exact displayed PDF.
The drawing crosshair now remains visible after area labeling and after a
keyboard-drawn chain ends, including when the area-definition tool stays active.

0.1.18 uses the supplied floorplan crosshair SVG for mouse and keyboard drawing.
The hotspot is the exact center. New settings default to 32 pixels; existing
32/64/128 pointer-size preferences are retained. The vector asset is embedded
in the EXE and needs no cursor file beside the program.
Pointer movement now skips repeated document bookkeeping and reuses cached
floorplan geometry while keeping the pointer, guides and drawing preview live.
Save As was rechecked across SKX and JSON: subsequent File/toolbar/keyboard
saves update the new destination and leave the original file unchanged.

0.1.17 makes Save write directly to the current file. A file dialog appears
only for the first save or Save As. Opened files and newly saved Recent entries
retain their actual disk destination; JSON files keep their JSON format.
Save As switches the current file only after a successful write. Cancellation,
write errors, New and recovery do not redirect Save to an unrelated document.

0.1.16 removes the extra sketch-name prompt when saving. Save and Save As go
directly to the Windows Save dialog. Its chosen filename is used in the saved
document, window title and recent-files list after a successful write.
Cancellation or a write error preserves the existing document name and drawing.

0.1.15 wires all 13 controls identified by the follow-up control audit: File
Open/Save/Save As; Home clipboard, selection, Delete, Rotate Sketch and Area
Attributes; View Calcs; and Tools Area Codes. Command lookup now handles
two-line captions. Regression checks reject enabled controls with no action.

0.1.14 fixes the Center and Center & Fit view buttons and C / Shift+C shortcuts.
Center preserves zoom; Center & Fit uses the current canvas size and sketch
bounds, including curved walls. Empty pages are unchanged; other pages are
excluded. Imported polygons without wall records can also be centered.

Area labels: automatic measurement placement has an extra 6px gap at normal scale.
The gap scales with the label; manually positioned measurements keep their position.

The original HTML Sketch logo is embedded as the executable and taskbar/window icon.

Windows GUI build: launching the program does not open a console window.

Extract the entire ZIP to a folder, then run sketch-app.exe.
Keep symbols.json next to the executable. The external symbol library is
required; it is deliberately not embedded in the program.

Windows 10/11 x64 and the Microsoft Visual C++ runtime are required.
No Rust installation is needed. This package is an unsigned development build.

Draw and edit walls, define areas, calculate totals, place symbols and text,
manage pages/layers, save/open SKX, import DXF, and export PNG/SVG/DXF/PDF.
Tools and the status bar provide the main drawing and measurement controls.
F1 opens Keyboard Shortcuts. Ctrl+K opens command search.

Wall repair (0.1.4): replacing a deleted exterior boundary restores the area's
original name, area code, attributes, factor/subtraction settings, styling, label
settings and layer/group. Geometry and computed area update to the repaired outline.
Mouse, typed/Enter and touch drawing detect closure and end the drawing run. Undo,
Redo, SKX save/open, autosave recovery and layer export/import retain repair metadata.
Explicitly deleting a whole area still removes its definition. Properties already
lost in an older build cannot be reconstructed automatically.

Area codes: Tools > Area Codes. Import JSON loads an editable draft;
Apply updates the library. Export JSON saves a normalized code table.

Symbols: open Symbols > Manage symbols… or right-click a library item.
The list and live preview sit side by side; click a symbol name to update the
preview without changing checked selections. The footer stays visible as the
window resizes. Enlarge or a row View button opens a separate viewer; Escape
returns to the manager. Search/filter, select, rename, toggle favorites, hide/restore,
and add DXF or symbol JSON files. Hidden definitions still render in drawings.
Delete removes definitions from symbols.json and placed instances from every
page and undo/clipboard history after two confirmations. Category deletion also
confirms twice. Restore hidden appears in the dock when needed. Renames, favorites
and visibility preferences are saved separately. Custom definitions
travel with SKX and layer exports. Binary DWG must first be converted to DXF.

Layers: File > Import Layer... accepts Sketch-Layer JSON. It preserves groups,
layer visibility/locking and original coordinates, assigns independent IDs,
and remaps hosts and dimension chains. Import is undoable. Use a layer's Export
action to create a compatible layer file, including its text and symbols.

Autosave: Options > System > Auto-Save enables recovery snapshots at the chosen
1–120 minute interval (default 5). Changed sketches are saved automatically to
%APPDATA%\SketchRust\autosave as SKX files, without opening a dialog. A startup
prompt and File > Recover Autosave... offer the latest valid recovery. Later
keeps it available. A successful manual save removes the current/recovered snapshot;
failed writes preserve the previous snapshot. Later also survives saving a
different sketch. New/Open preserve prior recovery
files. Autosave does not overwrite your manually saved original. Reference
images remain session-only; reattach them after recovery.

User preferences and reusable libraries are stored in %APPDATA%\SketchRust.
Reference images are session-only, as in the captured original; the
Save Image With Sketch option includes them in clean image export.

DISTO software is implemented, but testing with the physical instrument is
deferred. Read DISTO.md before trying the Bluetooth measurement feature.

Parent-suite integration and physical DISTO testing remain deferred.

manifest.json records the source revision, working-tree fingerprint and file
SHA-256 hashes.

For a non-interactive installation check, run the executable from a terminal
with --check-installation. It verifies the external library, constructs the
native view and performs an in-memory SKX round trip without changing user data.
