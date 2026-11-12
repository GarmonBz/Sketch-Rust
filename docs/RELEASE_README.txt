Sketch Rust 0.1.29 - Windows x64 native compatibility development build, 2026-11-11

Native Apex compatibility is in development. This package is NOT a completed
universal Apex replacement. Read APEX-COMPATIBILITY.md for supported interfaces,
verification and remaining gaps before changing a host's sketch provider.

0.1.29 adds built-in help. Help > User Guide covers drawing walls, areas and
calculations, symbols and text, pages and printing, and saving. Help >
Integration Guide explains how to set up and use Appraise-It integration, how
to go back to Apex, and what to check when something goes wrong.
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

0.1.22 adds a report-aware close prompt in integrated mode:
- Save to report transfers the current sketch, measurements and report images,
  then closes only after the host acknowledges success.
- Discard changes closes without sending further edits to the report.
- Cancel (or Escape) returns to editing.
The title-bar close button, File > Exit and Alt+F4 use the same prompt. Closing
is blocked during a pending transfer, and transfer failures keep Sketch open.
Standalone closing retains its existing confirmation.

0.1.21 keeps the desktop host compatibility runtime inside Sketch without an
apex-compat.dll console window. Both the main EXE and embedded managed entry
assembly use the Windows GUI subsystem. PDF/email opening uses the native
Windows shell, without launching cmd.exe.
Live report-image refreshes render on a background thread and reuse unchanged
snapshots. Host idle polls preserve the cached drawing. Late renders cannot
replace a newer explicit Save/Return. Rapid mouse events redraw the live pointer
and drawing guides locally, coalescing full HUD updates to a frame interval.
To update an existing Pro integration, open this 0.1.21 EXE, choose Help >
Appraise-It Integration > Enable integration, then restart Appraise-It Pro.

0.1.20 adds Help > Appraise-It Integration. Keep the complete extracted build
in a permanent folder, open that panel and click Enable integration. Approve
Windows administrator permission if requested, wait for success, then restart
Appraise-It Pro and use its existing Apex button. No terminal command or EXE
rename is needed. Restore previous provider reverses the registration.
The panel shows the current host target, success/errors and the Sketch EXE it
will register. Setup runs in the background inside this EXE. It changes the
shared Apex launch configuration, not Appraise-It Pro's program files. Protected
upgrades retain the prior registration record so an elevated retry succeeds.

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

0.1.13 fixes stretched report images by matching the native host's fixed canvas
aspect ratio. The 10 x 10 test area stays square in the actual legacy report
rendering and UAD stored image. Summary/Breakdown boxes remain centered and compact.

The EXE transfers a complete report image: the floorplan above side-by-side
AREA CALCULATIONS SUMMARY and AREA CALCULATIONS BREAKDOWN sections. These include
area codes/names, factors, sizes, perimeters, net totals, calculation pieces and
Net LIVABLE totals. Tables grow to retain all rows. This applies to default and
integration images for host returns and saves, independently of print settings.
No copyright notice or branding logo is appended to the transferred image.
The boxes have equal side margins and compact heights based on their contents.
The native EMF export fixes asymmetric padding at scaled Windows display DPIs.
Both actual legacy and UAD 3.6 stored report images were read back and verified.
For UAD 3.6, finish Pro's Transfer sketches dialog to apply the images; the SDK
return receipt alone does not mean that Pro has committed them to the report.

0.1.10 verifies actual Appraise-It Pro legacy and UAD 3.6 disposable-report
measurement/image transfers, including a repeated native reopen/return.
It fixes missing finished-basement measurements and reentrant host shutdown.
See BUILD_VALIDATION.md for exact checked fields and remaining qualification.

Compatibility code and the host transport are embedded in sketch-app.exe and
run inside that process. No bridge EXE or service is used. Windows .NET Framework
4.8 is required for native host/file compatibility; a genuine Apex drawing engine
is not used at runtime. The symbol library below remains editable external data.

SDK hosts: shared launch redirection uses --enable-apex-compat; exact restoration
uses --restore-apex-launch. Setup is built into the EXE and may require Windows
administrator rights. Choose a permanent extracted folder before setup. Shared
SDK configuration is only one host registration route; this does not configure
every legacy application automatically. This package does not activate setup.
If Windows protects the shared configuration, setup requests administrator
permission using this same EXE. Windows permission prompts must be approved by
the user. Setup and restoration show a visible success/error dialog; no console
or bridge program is required. --apex-registration-quiet suppresses that result
dialog for scripted use; it does not suppress a required Windows permission prompt.

When a host launches a linked sketch, Save/Ctrl+S saves back without closing.
Return to host saves and closes. Close can discard further changes after its
confirmation. Native file launch accepts an AX2 through AX7 filename. Multiple native filenames open separate sketch windows with independent saves.
/areacodetableurl accepts native Apex XML over HTTP/HTTPS or a local/file URL.
Host codes, signed factors, categories, colors and wall defaults are loaded for
the session. Metric files keep their native unit on save/return; the editor UI
uses feet. Unfinished native drafts and editable face measurements now preserve
their repair/centerline history inside AXF. Nested/nonuniform/reflected native
blocks and point entities are supported. Older
TOTAL/WinTOTAL command-line entry modes are implemented but unverified against
the actual host software on this PC.

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
SHA-256 hashes. This release includes the current uncommitted workspace changes.

For a non-interactive installation check, run the executable from a terminal
with --check-installation. It verifies the external library, constructs the
native view and performs an in-memory SKX round trip without changing user data.
