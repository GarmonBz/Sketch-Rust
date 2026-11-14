# 05. Area types

## 1. What exists

- Type: `sketch_core::geometry::AreaCode` (`crates/sketch-core/src/geometry.rs`).

| Field | JSON key | Meaning |
|---|---|---|
| `code` | `code` | Short id, upper case, unique (GLA1) |
| `name` | `name` | Display name (First Floor) |
| `description` | `description` | Longer text (Gross Living Area) |
| `group` | `group` | Group key: GLA, GBA, GAR, BSMT, P/P, OTH, LAND, SITE, NCA, UND |
| `multiplier` | `multiplier` | Factor applied to the area |
| `positive` | `positive` | true adds, false subtracts |
| `include_in_gla` | `includeInGLA` | Counts toward gross living area |
| `color` | `color` | Line colour, hex |
| `fill` | `fill` | Fill as `rgba(r,g,b,a)` |
| `fill_color` | `fillColor` | Fill colour, hex, optional |
| `fill_style` | `fillStyle` | `solid`, `none`, `diagonal`, `crosshatch`, `dots` (`new_code_dialog::FILL_STYLES`, labels from `style_label`) |
| `fill_opacity` | `fillOpacity` | 0 to 1 |

The struct is `#[serde(rename_all = "camelCase")]`, with `includeInGLA` renamed by hand.

- Defaults: a JSON literal in `geometry.rs` near `default_area_codes()`. 32 entries.
- Helpers in the same file: `default_area_codes`, `clone_area_codes`,
  `normalize_area_code`, `validate_area_codes`, `from_json`, `to_json`, `as_map`.
- App state: `App.area_codes: Vec<AreaCode>`. Saved in each sketch file (`areaCodes`) and
  in the per-user file `area-codes.json` (`persist.rs`).
- Classic editing surfaces:
  - `new_code_dialog.rs`: create a type with every field.
  - `def_editor.rs`: edit name, line colour, fill colour, fill style, opacity of an
    existing type. Saving also renames every area of that type and repaints its walls.
  - `codes_editor.rs`: raw JSON text editor with Apply, Import JSON, Export JSON,
    Restore defaults.
  - `area_workflow.rs`: `Msg::EditType`, `NewType`, `DeleteType`, `ConfirmDelete`.
    Deleting a type turns its areas into `UND`. `UND` cannot be deleted.
  - `dock::paint_palette(codes)` builds the grouped palette; `palette_group_label` maps
    group keys to headings.

## 2. The 32 built-in types (all must remain by default)

| Group heading | Types (code) |
|---|---|
| Living space | First Floor (GLA1), Second Floor (GLA2), Third Floor (GLA3), Fourth Floor (GLA4) |
| Building area | First to Tenth Floor (GBA1 to GBA10) |
| Garage / carport | Garage (GAR), Carport (GAR2) |
| Basement | Finished BSMT (BSMT), Unfinished BSMT (BSMT2) |
| Outdoor (group `P/P`) | Open Frame Porch (OFP), Screen Frame Porch (SFP), Enclosed Frame Porch (EFP), Patio (PAT), Porch (PORCH), Wood Deck (DK), Balcony (BAL) |
| Storage (group `OTH`) | Pool (POOL), Shed (SHED), Storage (OTH) |
| Land | Land (LAND) |
| Site | Subject Site (SITE) |
| Non-calculated | Non-Calculated Area (NCA) |
| Undefined | Undefined Area (UND) |

Group headings are the ones `dock::palette_group_label` returns today. Keep them. The
mockups label the porch group "Porch/Patio"; the real heading is "Outdoor". "Storage" is
an odd heading for a group that holds Pool and Shed; renaming it to "Other" is a
reasonable improvement but ask the owner first.

Add a unit test in `sketch-core` asserting `default_area_codes()` returns exactly these
32 codes in this order, so the list cannot shrink by accident.

## 3. New default colours for living space

Change only the `color` and `fill` of GLA2, GLA3 and GLA4 in the defaults literal.

| Code | Old colour | New colour | New fill |
|---|---|---|---|
| GLA1 | `#2B6CB0` | `#2B6CB0` (unchanged) | `rgba(43,108,176,.10)` |
| GLA2 | `#2C5282` | `#0987A0` teal | `rgba(9,135,160,.10)` |
| GLA3 | `#3182CE` | `#B83280` magenta | `rgba(184,50,128,.10)` |
| GLA4 | `#4299E1` | `#C53030` red (owner to confirm; alternative deep indigo `#434190`) | `rgba(197,48,48,.10)` |

Notes:

- Sketches saved with their own `areaCodes`, and users with an `area-codes.json`, keep
  their colours. Verify this by reading how `App` loads `area_codes` at boot and on open
  before relying on it; if the per-user file always wins, add nothing, and if defaults
  are merged in, make sure the merge does not overwrite user values.
- `#3182CE` is also `color_dialog::DEFAULT_COLOR` (the draw colour). That is a different
  setting. Do not change it.
- Check the other built-in colours for clashes with the new three before shipping. Print
  all 32 and look. Only GBA1 (`#024F39`), GAR (`#805AD5`), BSMT (`#4A5568`) and OFP
  (`#B7791F`) were compared during planning.

## 4. The area type editor (`ui/type_editor.rs`)

Replaces `codes_editor.rs`, `new_code_dialog.rs` and `def_editor.rs` in the Next shell.
Opened from Ledger > Types > "Edit types", Menu > Areas > Area types, the type row
context menu ("Edit type", which opens it on that type), and command `area-types`.

### Layout (see `mockups.html`, "Area types")

Dialog 1080 x 740 in Wide and Medium; full-screen page in Compact (list first, tap a
type to open its form with a back arrow).

- Left column, 336 wide: search field; the grouped list of every type (swatch, name,
  code, "N used"); scrolls.
- Right: the form for the selected type.
- Footer left: New type, Duplicate, Import, Export, Restore defaults.
  Footer right: Cancel, Save.

### Form fields

| Field | Control | Validation |
|---|---|---|
| Code | Text, upper-cased as typed | Required, unique, same rules `validate_area_codes` enforces |
| Name | Text | Required |
| Description | Text | |
| Group | Pick list of existing groups, plus "New group" | |
| Multiplier | Number | Finite |
| Sign | Segmented: Adds, Subtracts | maps to `positive` |
| Count in gross living area | Checkbox | `include_in_gla` |
| Line colour | Swatch that opens the colour dialog, plus hex text | Valid hex |
| Fill colour | Same | |
| Fill style | Pick list: Solid, None, Diagonal, Crosshatch, Dots | |
| Fill opacity | Slider 0 to 100 percent | |
| Preview | A sample area drawn with these values, name and code | |

Header line states "Built in" or "Added by you" and "Used by N areas in this sketch".

### Behaviour

- Edits apply to a draft. Save validates the whole table with `validate_area_codes`,
  then replaces `App.area_codes`, writes `area-codes.json`, marks the sketch edited, and
  pushes one undo step "Edit area types".
- **Changing a code** on a type that is in use must update every area that references the
  old code, on every page. Reuse the logic `def_editor` runs on save (rename areas,
  repaint walls) and extend it to rewrite the code. Add a test.
- **Delete type**: confirm with "N areas use this type and will become Undefined."
  Reuse the `DeleteType` / `ConfirmDelete` path. `UND` shows the button disabled with
  the reason.
- **Duplicate**: copies the selected type with code suffixed (GLA1 becomes GLA1A or the
  next free code) and name suffixed " copy". Opens it for editing.
- **New type**: empty form with the group of the selected type.
- **Reorder**: drag rows within a group, or Move up / Move down in the row menu. Order is
  the order of `App.area_codes`.
- **Import / Export**: the JSON format `from_json` / `to_json` already use. Import
  replaces the table after validation and confirmation.
- **Restore defaults**: confirm, then `default_area_codes()`. Areas whose type no longer
  exists become `UND`.
- Cancel with unsaved changes asks for confirmation.

### Tests

- Edit each field of a built-in type and save; reopen; values persist.
- Rename a code in use; the area keeps its type and totals.
- Delete a type in use; areas become UND; undo restores both.
- Duplicate, reorder, import, export, restore defaults.
- The Types list in the ledger and the chooser reflect every change immediately.

## 5. Editing a drawn area

Every property of a drawn area stays editable in the inspector's Area section
(`01-design-spec.md` 5.4): name, type, factor override, sign, auto-subtract mode,
deduct-from, attributes (plan reference, floor code, floor code text, year built), fill
and line style, label switches, and geometry by handles, typed wall length, insert
vertex, curve, flip, rotate, clone, reopen. These exist today across `area_dialog.rs`,
`attr_dialog.rs`, `area_props.rs`, the Edit Tools tab and the context menu. The task is
to surface them in one place, not to add behaviour.
