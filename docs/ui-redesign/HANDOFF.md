# UI Redesign Handoff Note

- **Branch:** `ui-redesign`
- **Workspace version:** `0.2.0` (`Cargo.toml`)
- **Working tree:** Uncommitted P0–P9 changes on `ui-redesign` (mapped from `XPS152023` `X:\Sketch Rust` directly to `\\mediaserver\AIProjects\Sketch Rust` — all changes are already on Mediaserver). Open the **local folder path** directly on Mediaserver rather than a UNC share for normal `git` ownership and faster builds.

## Status: P0–P9 Complete & Verified

Phases **P0 through P9** are implemented, visually inspected via headless `iced_test` screenshots (`target/e2e-next/`), and ticked `[x]` in `docs/ui-redesign/README.md`:

- **P0–P1:** `ui/tokens.rs`, `ui/fonts.rs`, `ui/kit.rs`, `ui/gallery.rs`, `e2e_support.rs`, `e2e_next.rs` (`Shell::Classic` / `Shell::Next` via `SKETCH_UI=next`).
- **P2:** `ui/layout.rs` (responsive `Compact < 700px`, `Medium 700..1100px`, `Wide >= 1100px`), `ui/mod.rs`.
- **P3:** `ui/commands.rs`, `ui/menu.rs` (unified command registry + Menu + `Ctrl+K` command search).
- **P4:** `ui/topbar.rs` (top bar, page tabs, page context menu, undo/redo history menu, export primary/menu, Areas/Totals drawer triggers).
- **P5:** `ui/tray.rs` (floating/compact tool tray + fly-outs + keyboard shortcuts).
- **P6:** `ui/ledger.rs` (Drawn / Types / Layers / totals breakdown trigger / drag-to-place & drag-to-assign + ledger resize).
- **P7:** `ui/inspector.rs` (contextual properties panel / slide-over / bottom sheet across all selection kinds).
- **P8:** `ui/insert.rs` (Symbols & Room labels drawer, Favorites, vector symbol previews via `symlib::render_fitted`, Pin toggle, `dock::ALPHA` filters & custom label placement).
- **P9:** `ui/controls.rs`, `ui/tape.rs`, `ui/chooser.rs` (Zoom/Fit/Scale/Grid/Snap/Pitch/AreaSnap/AngleStep/Show/Dimensions/North/Underlay/Mini-map/Touch `Pad`+`Laser` controls, amber Length Strip in `Wall`/`Inner`/`Measure`, responsive Area Chooser with empty-search all-types list in `area_workflow.rs`).

### `crates/sketch-app/src/ui/` Modules
`mod.rs`, `tokens.rs`, `fonts.rs`, `kit.rs`, `gallery.rs`, `layout.rs`, `commands.rs`, `menu.rs`, `topbar.rs`, `tray.rs`, `ledger.rs`, `inspector.rs`, `insert.rs`, `controls.rs`, `tape.rs`, `chooser.rs`.

## Last Verification Snapshot (before pause at P9)
- `cargo clippy --workspace --offline -- -D warnings`: passed (0 warnings).
- `cargo test -p sketch-core --offline`: `178 passed; 0 failed; 0 ignored`.
- `cargo test -p sketch-app --offline --bin sketch-app -- --skip e2e::`: `348 passed; 0 failed; 3 ignored; 281 filtered out` (includes `e2e_next::next_shell_boots_at_all_four_sizes` and `e2e_next::p1`–`p9` tests).
- `cargo build -p sketch-app --offline`: passed.
- `target\debug\sketch-app.exe --check-installation` with `SKETCH_UI=classic` and `SKETCH_UI=next`: both exited `0`.
- `rg -n "theme::at|theme::ink|INK_PASSES" crates/sketch-app/src/ui/`: matches only doc comments in `kit.rs` and `tokens.rs`.

*(On Mediaserver, omit `--offline` on the first Cargo run if Cargo's local registry cache needs to fetch crates, and expect a one-time rebuild if the local disk path differs from `X:\Sketch Rust`.)*

## Constraints to Preserve
- Keep `Shell::Classic` building and passing `--check-installation` alongside `Shell::Next` until **P15** cutover.
- Never use `theme::at`, `theme::ink`, or `INK_PASSES` in `crates/sketch-app/src/ui/`; use single-pass `icons::mono` and `ui/kit.rs` primitives.
- Keep one responsive implementation per screen (no per-size-class duplication) and emit existing `Message` variants whenever possible.
- Never run two `cargo` commands concurrently on the same target directory.
- Tick each phase `[x]` in `docs/ui-redesign/README.md` only after all acceptance checks and `e2e_next` tests + screenshot checks pass.

## Next Up: P10 (Touch input and the Pad) -> P16
1. Read `docs/ui-redesign/07-touch-and-android.md`, `docs/ui-redesign/01-design-spec.md` §5.11, and `docs/ui-redesign/03-phases.md` P10.
2. Implement long-press (500 ms, <= 8 px drift) context menu triggers, touch 1-finger/2-finger canvas gestures, `ui/pad.rs` (including diagonals on hold, `Turn L`/`Turn R`, `Pen`, `Close area`, `Laser`, hold More actions, docked in Compact / draggable floating panel in Wide+Medium, auto-show in Touch density during `Wall`/`Inner`/`Measure` and auto-hide on hardware key), `"Select more"` additive toggle, tap-to-preview + second-tap-to-open chooser in Area mode, `"Pen as Ink"` stylus behavior, and a full 3-area floor plan `p10_*` test in `e2e_next.rs`.
3. Continue sequentially through **P11** (`05-area-types.md`), **P12** (`06-symbol-library.md`), **P13**, **P14** (`04-feature-map.md` + `coverage.csv`), **P15**, and **P16**.
