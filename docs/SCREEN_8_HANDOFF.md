# Screen 8 Handoff — Symbol Placement

**Last commit on main:** `cfecf30` (docs: mark Screen 7k complete)
**Screen 8 status:** partial — compiles clean, one `Scene` construction site not yet updated, tests not run for the new path

---

## What's already coded

### New files
- `crates/sketch-app/src/placed_symbol.rs` — `PlacedSymbol` struct with all browser fields:
  - `id`, `symbol_type`, `x`, `y`, `rotation`, `scale`, `flip_h`
  - `wall_id`, `wall_snap`, `wall_gap_width`, `wall_fraction`, `wall_side_flip`, `wall_side_explicit`
  - `layer_id`
  - `PlacedSymbol::new(id, symbol_type)` constructor

### Modified files

**`crates/sketch-app/src/view_state.rs`**
- `Mode::PlaceSymbol` added to the enum
- `Mode::crosshair()` now includes `PlaceSymbol`
- `Mode::is_placing_symbol()` helper added

**`crates/sketch-app/src/drawing.rs`**
- `Drawing.symbols: Vec<PlacedSymbol>` field added
- Initialized to `Vec::new()` in `Drawing::default()`

**`crates/sketch-app/src/main.rs`**
- `App.placing_symbol: Option<String>` field added
- Initialized to `None` in `program()`
- `Message::Dock` handler: when `self.dock.preview` is `Some(id)`, arms placement:
  ```rust
  self.placing_symbol = Some(id);
  self.set_mode(view_state::Mode::PlaceSymbol);
  ```
- `Message::CanvasClick`: if `placing_symbol` is `Some`, calls `place_symbol_at(world, symbol_type)` and returns early
- `place_symbol_at()` method added:
  - Pushes undo ("Place Symbol")
  - Generates id via `self.drawing.gen_id()`
  - Pushes `PlacedSymbol` to `self.drawing.symbols`
  - Clears selection, returns to `Mode::Select`, clears dock preview
- `symbols_lib()` helper added (returns `&crate::symlib::SymIndex` from the `Arc<Result<...>>`)

**`crates/sketch-app/src/render.rs`**
- `Scene` struct extended with:
  - `pub symbols: &'a [crate::placed_symbol::PlacedSymbol]`
  - `pub symbol_lib: &'a crate::symlib::SymIndex`
- `draw_symbols()` stub added — currently draws blue placeholder squares
- Called from `draw_scene()`

**`crates/sketch-app/src/sketch_canvas.rs`**
- `draw_placement_preview()` added — draws 24×24 blue square at cursor in `PlaceSymbol` mode
- Hooked into `SketchCanvas::draw()` when `self.mode.is_placing_symbol()`

**`crates/sketch-app/src/e2e.rs`**
- `e2e_symbol_placement_arms_and_commits` test added:
  - Opens Symbols dock, clicks a symbol
  - Asserts "Click the drawing to place" toast appears
  - Clicks canvas, asserts mode returns to Select, `drawing.symbols.len() == 1`, `placing_symbol == None`

---

## Current compile blocker

```
error[E0063]: missing fields `symbol_lib` and `symbols` in initializer of `Scene<'_>]
```

**Location:** `crates/sketch-app/src/main.rs` — the `render::Scene { ... }` construction site

**Fix:** Find the `Scene` struct literal and add the two new fields:
```rust
scene: render::Scene {
    drawing: &self.drawing,
    selection: &self.selection,
    ctx: self.draw_ctx(),
    pan: self.view.pan,
    zoom: self.view.zoom,
    filters: self.view.filters,
    dimensions: self.view.dimensions,
    placement: self.view.dimension_placement,
    symbols: &self.drawing.symbols,           // <-- add this
    symbol_lib: self.symbols_lib(),           // <-- add this
},
```

---

## Additional fix needed

**`crates/sketch-app/src/e2e.rs`** — the new test uses `view_state::Mode::Select` but may lack the import. Either:
- Add `use crate::view_state::Mode;` at the top, or
- Use full path `crate::view_state::Mode::Select`

---

## Verification steps once blockers are fixed

```bash
cd /d/AIProjects/Sketch Rust
export PATH="$HOME/.cargo/bin:$PATH"

# Focused test for the new path
cargo test -p sketch-app --bin sketch-app e2e_symbol_placement_arms_and_commits

# Full app suite
cargo test -p sketch-app --bin sketch-app

# Core suite
cargo test -p sketch-core
```

Expected: 220+ app tests, 143 core tests, all green.

---

## What Screen 8 still needs (after the arm-and-commit path is green)

### High priority (behavioural parity)
- **Wall-snap logic** (`resolveSymbolPlacement`): SIDE/CENTER/NONE tolerance, nearest-wall projection, side-flip rules, exterior swing flip
- **Real symbol rendering**: draw library geometry (polylines/arcs/text) with rotation, scale, flipH — replace the blue placeholder
- **Flip/resize/rotate handles**: corner resize (7px hit, 0.25–8 scale), rotation handle (11px hit, 15°/1°-Ctrl)
- **Hosted symbol follow-along**: `repositionHostedSymbol`, `refreshHostedSymbols`, `resnapSelectedSymbols`
- **Opening width adjustment**: `adjustOpeningWidth(±0.5)` with min 0, recalculate

### Medium priority (dock/library management)
- Drag-from-dock placement (S8-002)
- Symbol context menu: Place, Add/Remove favorites, Rename, Delete/Hide, Purge
- Manage Symbols dialog (filter, search, category, rows with checkboxes, preview pane)
- Add from DXF (future — depends on DXF import)
- Favorites persistence (`settings.symbolFavorites`)
- Hidden/purged symbols persistence (`sketch.hiddenSymbols`, `sketch.purgedSymbols`, `sketch.purgedCategories`)

### Low priority / future
- Keyboard shortcut S for symbol library
- Symbol viewer dialog (Type, Category, Wall snap)
- Export purged-symbols.json list
- Legacy "Organize Favorite Symbols" dialog (superseded by Manage Symbols)

---

## Reference files

- `refs/sketch-app.js:1994` — `placeSymbolAt` (original)
- `refs/sketch-app.js:1799` — `resolveSymbolPlacement` (original)
- `docs/FEATURE_INVENTORY.md:388-432` — Screen 8 feature inventory (45 rows)
- `docs/STATUS.md:1009-1050` — P4 symbol placement status (already built in a prior phase; this Screen 8 work is integrating it into the Rust port)

---

## Decision log

- Symbol rendering is a blue placeholder for now — the full geometry path needs `render_geometry` ported from `symlib.rs`'s preview code (already has the polyline/arc/ellipse/text drawing logic for the 92×68 preview canvas)
- Wall-snap logic deferred to a follow-up pass; the data model stores all the fields (`wall_fraction`, `wall_side_flip`, etc.) so the plumbing is ready when the geometry logic arrives
- `placed_symbol.rs` is a new module, not the old `symlib.rs` — `symlib` stays as the library loader; `placed_symbol` is the runtime model
