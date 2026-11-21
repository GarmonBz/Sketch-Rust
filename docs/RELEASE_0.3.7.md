## What's new

- The installer opens `.sketchie` and `.skx` files in Sketch Rust on a double click, and adds Sketch Rust to **Open with** for `.json` files, so `.sketchie.json` sketches open with it too (Windows only matches the last extension, so it does not take over every JSON file).
- The Fallingwater demo appears under Recent sketches the first time Sketch Rust starts after installing. It is added once, after your own recent sketches.
- The start screen shows when Sketch Rust starts, with a **Show this screen when Sketch Rust starts** checkbox to turn that off.
- Fixed the window going "Not Responding" while an Open or Save dialog is open: it keeps handling its messages while it waits.
- Fixed slow redrawing of sketches with many layers: the wall data is now worked out once per redraw instead of once per layer.
- Dragging a wall end or an area corner now joins a nearby wall end or corner (within 10 px) before falling back to the grid. Moving lines and areas pulls them onto nearby endpoints that stay put.

## Windows

Run **SketchRust-0.3.7-x64.msi** to install or upgrade Sketch Rust for the current Windows user. A portable ZIP and SHA-256 checksums are also provided.
