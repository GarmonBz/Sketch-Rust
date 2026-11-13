Sketch Rust 0.1.32 - Windows x64 preview

0.1.32 transfers sketch page images to a saved, editable UAD 3.6 report
using the installed SFREP COM server. Open Help > Appraise-It Reports to
select the report, preview the pages, and add them to Additional Sketch(es).
Review the images and save the report in Appraise-It Pro.

0.1.31 opens Sketchy 0.2 JSON files and SQLite stores as Saved. Editing
marks the file Unsaved, and saving returns it to Saved.

Sketchy preview: File > Open accepts draft 0.1 and 0.2 .sketchy.json workfiles
and .sketchy revision stores. Save writes edits to the open workfile. Save As
offers Sketchy JSON and Sketchy Store. Copy try-sketchy.sketchy.json to Documents, open the copy,
move a wall, save, and reopen it. See SKETCHY.md for format details.

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
Attributes; View Calcs; and Tools Area Codes. Ribbon command lookup now handles
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

FEATURE-AUDIT.md and feature-audit.json describe the exact scope and remaining
native/reference differences. Parent-suite integration and physical DISTO
testing remain deferred.

manifest.json records the source revision, working-tree fingerprint and file
SHA-256 hashes.

For a non-interactive installation check, run the executable from a terminal
with --check-installation. It verifies the external library, constructs the
native view and performs an in-memory SKX round trip without changing user data.
