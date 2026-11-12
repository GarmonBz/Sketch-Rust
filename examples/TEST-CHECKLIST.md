# Sketch Rust 0.1.1 — quick manual test

Use a disposable sketch. Files are in this folder.

## Layer import

1. File > Import Layer… > test-layer.json.
2. Expect an Import Test group with Sample Room and Sample Notes children:
   four blue walls forming a 16 × 12 ft room (192 ft² with default settings),
   two text labels, a north-arrow symbol and a dimension chain.
3. Immediately after importing, Undo should remove the import; Redo should
   restore it.
4. Hide/show Import Test; its contents should disappear/reappear together.
   Save as SKX and reopen; the layers, text, arrow and room should survive.

## Symbol manager

1. Symbols > Manage symbols… > Add DXF / JSON > test-marker.dxf.
2. Find test-marker (a 12-inch square with an X). Select it, preview/View it,
   rename it Test marker renamed, and toggle Favorite. Check the dock updates.
3. Place a copy. Hide it in the manager and confirm: its library row disappears
   from the dock, but the placed copy stays visible. Restore hidden and confirm;
   the library item should return.
4. Optional: purge only this imported marker. Cancel a confirmation first and
   verify nothing changes; then accept both confirmations. Its definition and
   placed copies should disappear. Reimport the DXF if needed.

## Autosave / recovery

1. Options > System > Auto-Save: enable it, set 1 minute, and Apply.
2. Make a visible change and do not manually save. Wait at least 75 seconds.
3. In File Explorer, open %APPDATA%\SketchRust\autosave and check that a .skx
   recovery file has a recent timestamp.
4. Close without manually saving, relaunch dist/sketch-app.exe, and choose
   Recover. Expect the latest autosaved edit, imported layers and symbols.
5. Save the recovered sketch manually, then relaunch. That recovery should no
   longer be offered. Earlier unrelated recovery files may still be offered.

If you choose Later at startup, File > Recover Autosave… offers recovery again.
Choosing Later must preserve that file even if you save a different sketch.
