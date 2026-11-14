# 06. Symbol library

## 1. What exists

- Library file: `assets/symbols.json`, external to the executable, found at run time by
  `sketch_core::symbols::resolve` and `sketch-app/src/symlib.rs` (`library_path`,
  `library`, `edit_library`). 639 symbols in 29 categories. CADdillo symbols are CC0
  (`docs/SYMBOL_LIBRARY.md`).
- Types (`crates/sketch-core/src/symbols.rs`):
  - `SymbolLibrary { version, generated_from, symbol_count, categories, default_favorites, symbols: BTreeMap<String, Symbol>, aliases }`
  - `SymbolCategory { key, name, symbol_ids }`
  - `Symbol { id, category, category_name, name, label, description, units, wall_snap, wall_gap_width, wall_gap_alignment, wall_cap_length, bounds, geometry, native_apex }`
  - `WallSnap { None, Side, Center }` (JSON `NONE`, `SIDE`, `CENTER`)
  - `Entity { entity_type: Line | Polyline | Arc | Ellipse | Text, vertices (with bulge `b`), start_point, end_point, center, axis_point, ratio, start_angle, end_angle, insert_point, rotation, font_size, alignment, text_string, closed, stroke_*, fill_color }`
  - Units are inches. `world_corners` converts with `/ 12`.
- File edits (`crates/sketch-core/src/symbol_edit.rs`), each rewriting the JSON and
  keeping the previous file as `symbols.json.bak`: `delete`, `delete_category`,
  `rename`, `move_to_category`, `rename_category`, `merge`.
- Import: `sketch_core::symbol_import::from_dxf(body, name) -> Result<Symbol, String>`.
- Per-user overrides (`sketch-app/src/symbol_manager.rs`): `Changes { renamed, hidden,
  deleted, deleted_categories, custom }` in `symbol-library.json`; favourites in
  `symbol-favorites.json`.
- `symbol_manager::Msg` already has: Open, Close, Filter, Toggle, SelectAll, Focus,
  Inspect, View, ViewerClose, Name, Category, Rename, RenameInFile, Favorite, Favorites,
  Hide, Restore, RestoreAll, Delete, DeleteCategory, Target, Move, RenameCategoryInFile,
  MergeDuplicates, Import, Place, Confirm, Cancel, Context.
- `assets/symbols.previous.json` is the input of
  `crates/sketch-core/examples/import_caddillo.rs`. It is not a user backup.

So renaming, moving, hiding, restoring, deleting, category rename and delete, duplicate
merging, favourites and DXF import exist. **Missing**: editing a symbol's properties,
editing its shape, creating a symbol from the sketch, duplicating, empty categories,
reordering, export, and restoring the shipped library.

## 2. Requirement

- All 639 symbols and all 29 categories are present and visible by default. Nothing is
  hidden on a fresh install. Add a test that a fresh profile lists `symbol_count`
  symbols.
- Every symbol and every category is editable, including the built-in ones.

## 3. The screen (`ui/symbol_library.rs`)

Opened from Insert drawer > "Edit library", Menu > Tools > Symbol library, a tile's
context menu, and command `symbol-library`. See `mockups.html`, "Symbol library".

Three columns in Wide; in Medium the editor is a slide-over; in Compact each column is a
page (categories, grid, editor).

1. **Categories** (216 wide): "All symbols" with total, Favorites, every category with
   its count, "Hidden" when any are hidden. Row menu: Rename, Delete, Move up, Move down.
   Footer: New category.
2. **Grid**: search within the category, "New symbol", "Import DXF". Tiles show the
   preview, name and wall behaviour. Multi-select with checkboxes or Ctrl+click.
   Selection actions: Move to category, Add to favorites, Hide, Delete, Merge duplicates,
   Export selected.
3. **Editor** for the focused symbol:

| Field | Control | Maps to |
|---|---|---|
| Preview | Canvas with the symbol at scale, wall line shown for wall symbols; "Edit shape" | `geometry`, `bounds` |
| Name | Text | `name` and `label` (label is "<Category> - <Name>") |
| Category | Pick list | `category`, `category_name`, and `symbol_ids` of both categories |
| Description | Text | `description` |
| Snaps to wall | Pick list: No, One side, Centre of wall | `wall_snap` |
| Wall gap | Number with unit | `wall_gap_width` |
| Gap alignment | Number | `wall_gap_alignment` |
| End cap | Number with unit | `wall_cap_length` |
| Units | Read-only "Inches" | `units` (all shipped symbols use INCHES; keep it fixed) |
| Actions | Add to favorites, Duplicate, Hide, Delete, Place | |
| Info | "Placed N times in this sketch" | |

Footer: Export library, Import library, Restore original library, Close.

## 4. New core functions (`sketch-core/src/symbol_edit.rs`)

Follow the existing pattern: go through `rewrite(path, |doc| ...)`, operate on the raw
JSON so unknown fields survive, return the count changed, keep `symbols.json.bak`.

```rust
pub struct SymbolProps {
    pub name: Option<String>,
    pub description: Option<String>,
    pub wall_snap: Option<WallSnap>,
    pub wall_gap_width: Option<f64>,
    pub wall_gap_alignment: Option<f64>,
    pub wall_cap_length: Option<f64>,
}
pub fn update_properties(path: &Path, id: &str, props: &SymbolProps) -> Edit;
pub fn replace_geometry(path: &Path, id: &str, geometry: &[Entity], bounds: Bounds) -> Edit;
pub fn add(path: &Path, symbol: &Symbol) -> Edit;              // new id must be unique
pub fn duplicate(path: &Path, id: &str, new_id: &str, new_name: &str) -> Edit;
pub fn add_category(path: &Path, key: &str, name: &str) -> Edit;
pub fn reorder_categories(path: &Path, keys: &[String]) -> Edit;
pub fn reorder_symbols(path: &Path, category: &str, ids: &[String]) -> Edit;
pub fn export(path: &Path, dest: &Path, ids: Option<&BTreeSet<String>>) -> Edit;
pub fn import(path: &Path, source: &Path) -> Edit;             // merge by id, report conflicts
```

Each must keep `symbolCount` and each category's `symbolIds` consistent, and each needs
unit tests in the style of the existing ones in that file.

Ids: `<category-key>/<kebab-name>`, as the shipped ids are (`arrows/left-turn`). Placed
symbols reference ids, so **never change an id on rename or move**. This is what the
existing `rename` and `move_to_category` already guarantee.

## 5. Restore original library

The shipped library must be recoverable after any amount of editing.

- At build or package time, ship a second, read-only copy of the library next to it
  named `symbols.original.json` (add to `tools/package.ps1`, `tools/package.sh` and
  `installer/SketchRust.wxs`).
- "Restore original library" confirms, saves the current file as `symbols.json.bak`,
  copies the original over it, and clears `Changes` (hidden, renamed, deleted).
- If `symbols.original.json` is missing, disable the button and say why.

## 6. Shape editing

Two ways in, sharing one conversion.

### 6.1 Make a symbol from the sketch

Command "Make symbol from selection" (Menu > Tools, and the selection's context menu).

1. Take the selected walls, interior lines, curved walls and text labels.
2. Convert to `Entity` values in inches, relative to the selection's centre:
   straight wall to `Line`, curved wall to `Arc` (or a two-vertex `Polyline` with bulge
   `b`), closed chains to a closed `Polyline`, labels to `Text`. Feet times 12.
   Y direction: check `symlib::render_geometry` for the sign convention before writing
   the conversion, and add a round-trip test.
3. Compute `bounds`.
4. Ask for name, category and wall behaviour, then call `add`.

### 6.2 Edit shape

"Edit shape" in the editor.

1. Open a temporary symbol-editing session: a blank page whose contents are the symbol's
   entities converted back to sketch lines and labels (inverse of 6.1). The top bar shows
   "Editing symbol: <name>" with Save shape and Cancel. The ledger and page tabs are
   hidden for the session.
2. The user edits with the normal tools.
3. Save shape converts as in 6.1 and calls `replace_geometry`. Cancel discards.
4. Entities the sketch tools cannot represent (`Ellipse`, text with unusual alignment)
   are carried through unchanged and shown locked; say so in a note on the screen.

This reuses the sketch's drawing tools, so there is no second editor to build.

The session must not touch the user's document or its undo stack. Implement it as a
separate `Drawing` swapped in for the duration, and restore the document on exit.

## 7. Per-user overrides versus file edits

Today some actions change the per-user `Changes` (hide, rename label, custom symbols) and
others rewrite the shared file (the `...InFile` messages). Keep one rule in the new
screen:

- Edits rewrite the library file (with `.bak`), because the requirement is a fully
  editable library.
- "Hide" stays a per-user override, since it is reversible and personal.
- If the library file is not writable (installed under Program Files), store the edit as
  a per-user override in `Changes.custom` and show "Saved for this user only". Verify how
  `symlib::edit_library` reports a write failure before designing the message.

## 8. Cleanup that belongs with this work

- `Symbol.native_apex`: rename to `native_cad` with `#[serde(alias = "nativeApex")]` so
  old files still load, and write the new key. Check `symbol_edit.rs` and exporters for
  uses first.
- Remove `symbol_organizer.rs` (the legacy favourites dialog) at cutover; favourites are
  managed in the library screen.

## 9. Tests

- Fresh profile: all categories and `symbol_count` symbols are listed.
- Edit each property of a built-in symbol; reload the library; values persist; a placed
  instance still renders and still snaps as configured.
- Duplicate, move, rename; placed instances are unaffected.
- Make a symbol from a drawn rectangle with a label; place it; its size matches.
- Edit shape, save, and the preview and placed instances update.
- Delete a category; its symbols are removed; restore original brings everything back.
- Export selected then import into a clean library.
