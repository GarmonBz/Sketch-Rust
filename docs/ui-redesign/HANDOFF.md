# UI redesign handoff note

- **Branch:** `ui-redesign`
- **Workspace version:** `0.2.3`
- **State:** P0 to P16 are done and committed. The owner approved the 11 RETIRED rows on 2026-11-15.

## Verified on 2026-11-15

- `cargo build -p sketch-app`: passes.
- `cargo test --workspace`: passes (app and core).
- `cargo clippy --workspace -- -D warnings`: passes.

Run one `cargo` command at a time; the target directory is shared.

## Fixed after the first audit (2026-11-15)

- Select tool: pressing a wall that is already selected now drags it, so its
  neighbours stretch and the area resizes. Before, only the endpoint dots worked.
  Tests: `e2e_next::select_drag_on_a_wall_resizes_the_area_across_sizes`,
  `e2e_next::select_drag_on_a_wall_endpoint_moves_the_corner_across_sizes`.
- Toasts sit above the tool tray. The length strip moves out from under the Pad.
- Old vocabulary removed from command labels, toasts, dialog titles and undo names.
- `coverage.csv`: 49 rows pointed at tests that did not exist; corrected.
- Duplicate "Manage symbols..." command removed (it opened the Symbol library).

## Second round (2026-11-15)

- **Inspector is paged, not scrolled.** Each selection kind is split into short
  tabs (an area has Area, Math, Look, Edit). `UiState.inspector_tab`,
  `ui::Msg::InspectorTab`. The scroll area remains only as a net for windows
  too short to show a page. Screenshots: `target/e2e-next/inspector-*.png`
  from `e2e_next::inspector_pages_render_without_scrolling`.
- **File dialogs** all go through `platform.rs` (`pick_file`, `pick_files`,
  `save_file`, `save_file_as`, and the async pair). No `rfd` call elsewhere.
- **Drag tests** added for the curve handle, rotate handle and marquee
  (`e2e_next::select_drag_on_*`); coverage rows S6-067, S6-071, S6-072 updated.
- **Calculation marker** is `native.calculation`
  (`sketch_core::area_calc::uses_native_net`); the legacy key is still read.
- **Command search** gained recent sketches, nudge, underlay and label defaults.
- **Removed** the three area-type dialogs nothing could open (JSON table,
  New Area Code, definition editor): `def_editor.rs` is gone and
  `new_code_dialog.rs` keeps only the fill style names.
- Pick lists and sliders use `kit::pick_list_style`, `kit::pick_menu_style`
  and `kit::slider_style`.
- The mini map's dark box was a headless-test artefact: with no window to
  capture, `minimap::blank` is shown. The running app captures the canvas.

## Third round (2026-11-17, owner requests)

- **Icons in colour.** The classic tints are back, by meaning (blue view, orange edit,
  red remove, green file, slate device): `tokens::IconTint`, `icons::tint_of`,
  `icons::tinted`. Every Menu command has an icon (`commands::icon_for`). Spec §1 updated.
- **Menu, Option A.** One column, search field on top, 19 rows, fly-outs for Recent,
  Import, Export, Print, Trace image, Select, Transform sketch, Help (`ui/menu.rs`,
  `ENTRIES`). The search lists all 160 commands. Theme, drawing background and density
  moved to Options > Display > Appearance. The feature-map rule "appear in the Menu under
  its group" is replaced by "found by search, plus one on-screen home"; `coverage.csv`
  homes rewritten to match. Ctrl K still opens the separate command palette.
- **Dialogs fit 1920 x 1080.** Wide dialogs are capped at 1280 x 960 (was 1080 x 740).
  `e2e_next::dialogs_fit_1920x1080_without_scrolling` lays out every dialog and tab and
  fails if any form scroll area overflows (`e2e_support::scroll_areas`). Lists of the
  user's own data may scroll; their scroll areas have ids ending in `-list`.
- **First area fits the view**, centring and zooming out only (`fit_after_first_area`).
- **Area tool hover** no longer prints a size over an area that has its own label.
- **Selection tint** lowered to 7% so the area type's colour shows.

## Still open

1. **Old symbol manager.** `symbol_manager.rs` still holds the classic
   "Manage Symbols" modal. Nothing opens it (`Msg::Open` forwards to the
   Symbol library), but the tile context menu, `Changes` and
   `category_key` in the same file are live. Split the live parts out, then
   delete the modal view and its messages.
2. **Windows-only code in `ui/`.** `ui/commands.rs`, `ui/dialogs.rs` and
   `ui/start.rs` gate the Appraise-It features with `#[cfg(windows)]`. Moving
   them behind `platform.rs` needs a non-Windows build to verify, which this
   machine does not have.
3. **Android (P16)** is a desk study. No Android toolchain is installed, so
   the experiments in `07-touch-and-android.md` section 5 have not been run.
4. **Thin tests elsewhere.** Outside Select-mode drags there is still one
   large headless test per phase.
5. Inspector action buttons do not fill the panel width (cosmetic).

## Constraints to preserve

- Never use fixed coordinates in `crates/sketch-app/src/ui/`.
- One responsive implementation per screen; emit existing `Message` variants.
- Update `coverage.csv` in the same change that moves a feature.
