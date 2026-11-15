# UI redesign handoff note

- **Branch:** `ui-redesign`
- **Workspace version:** `0.2.0`
- **State:** P0 to P13, P15 and P16 are done and committed. P14 is open only for owner sign-off.

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

## Still open

1. **Owner sign-off** on the 11 rows marked RETIRED in `coverage.csv` (P14).
2. **Dead screens.** `symbol_manager.rs`, `new_code_dialog.rs`, `def_editor.rs`,
   `text_manager.rs` and the JSON area-types dialog in `ui/dialogs.rs` are no
   longer opened by any command, but their state and messages are still wired
   into `main.rs` (`symbol_manager::Changes` is used by the insert drawer).
   Remove the views and keep the types `update` needs.
3. **Thin tests.** There is one large headless test per phase. Rotate-handle,
   curve-handle and marquee drags (coverage rows S6-067, S6-071, S6-072) still
   cite a transform unit test that never drags. Add drag tests like the two above.
4. **Platform isolation** (`07-touch-and-android.md` section 5): `rfd` is called
   from `file_dialogs.rs`, `layer_io.rs`, `main.rs`, `symbol_manager.rs`,
   `ui/dialogs.rs` and `ui/symbol_library.rs`; `ui/commands.rs`, `ui/dialogs.rs`
   and `ui/start.rs` contain `#[cfg(windows)]`. Move both behind `platform.rs`.
5. **`apex.calculation`** attribute key in `sketch-core/src/area_calc.rs` and
   `sketch-app/src/area_repair.rs`: rename with a read-compatible alias.
6. **Android (P16)** is a desk study. No Android toolchain was installed, so the
   experiments in `07-touch-and-android.md` section 5 have not been run.
7. **Command search** has no entry for recent sketches, nudge, trace image
   intensity, underlay or label defaults. Each is reachable elsewhere (Menu,
   page tab menu, inspector, ledger menu).
8. **Mini map** draws an empty dark box when the sketch is empty; check in the
   running app.
9. Unstyled controls: the "Snaps to wall" pick list in the Symbol library and
   the fill opacity slider in Area types use iced defaults, not the tokens.

## Constraints to preserve

- Never use fixed coordinates in `crates/sketch-app/src/ui/`.
- One responsive implementation per screen; emit existing `Message` variants.
- Update `coverage.csv` in the same change that moves a feature.
