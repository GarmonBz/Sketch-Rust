# 07. Touch, tablet, phone, Android

## 1. Targets, in priority order

1. **Tablet, landscape** (reference 1280 x 800): Wide layout, Touch density. This is the
   main touch experience and must be complete.
2. **Tablet, portrait** (reference 800 x 1280): Medium layout, Touch density.
3. **Phone** (reference 372 x 800): Compact layout. Every feature reachable, less
   comfortable for heavy drawing.

All three must be testable on Windows by resizing the window and turning on
Menu > View > Touch-sized controls. Android comes later.

## 2. Input equivalents

Every row must be implemented. The left column is what exists today.

| Mouse or keyboard | Touch |
|---|---|
| Right-click (context menus on the drawing, type rows, symbol tiles, layer rows, page tabs) | Press and hold 500 ms without moving more than 8 px |
| Wheel zoom | Pinch (exists: `Message::Pinch` from `sketch_canvas.rs`) |
| Middle-drag pan, Space-drag pan | Two-finger drag (exists with pinch) |
| Left drag in Pan tool | One-finger drag in Pan tool |
| Left click | Tap |
| Double-click to finish a run, edit a label, open the chooser | Double-tap, and a visible Done / Edit button |
| Hover preview in Area mode | First tap previews the region and shows its size, second tap on the same region opens the chooser |
| Hover tooltips | Press and hold on a button shows its name |
| Ctrl+click additive select | "Select more" switch on the Select tool |
| Ctrl for fine rotate, resize, curve steps | A "Fine" switch in the inspector's steppers |
| Shift+wheel to bow the last wall | Curve stepper in the length strip while drawing, and in the inspector |
| Wheel to scale a label or symbol | Size stepper in the inspector, and the corner handle |
| Typed length + arrow key | Pad |
| Keyboard shortcuts | Menu and command search list every command |
| Drag from a panel to the drawing | Works by touch drag; tap-to-arm then tap is the equal alternative |
| Drag handles (endpoint, rotate, curve, resize) | Same, with a 48 px hit area around each 12 px handle |
| Window resize, drag, minimise | Not applicable on tablets |

## 3. The Pad (`ui/pad.rs`)

Purpose: draw walls by number and direction without a keyboard. It replaces
`touch_panel.rs`, whose `TouchAction` handler only implemented up, down, left, right and
delete.

Layout: `01-design-spec.md` section 5.11.

| Key | Emits |
|---|---|
| 0 to 9 | `Message::Key(KeyCmd::Buffer("0".."9"))` |
| `'` (feet and inches separator) and `.` | Verify how feet-inches are typed today in `input.rs` and `drawing.rs`; `.` is `KeyCmd::DecimalPoint` |
| Backspace | `KeyCmd::BufferBackspace` |
| Up, Right, Down, Left | `KeyCmd::Arrow { dir, ctrl: false, shift: false }` with `dir` 0, 2, 4, 6 (`input::ARROW_KEYS`) |
| Diagonals (press and hold an arrow to reveal) | `Arrow` with `dir` 1, 3, 5, 7 |
| Turn L, Turn R | Append `l` or `r` to the buffer and commit, the same as typing `10l` or `10r`. Find the buffer letters `input::key_down` accepts and send those |
| Pen | Toggle pen up and down: `KeyCmd::Enter` when the pen is up; `Escape` lifts it. Confirm against `drawing.rs` |
| Close area | `KeyCmd::AutoClose` |
| Laser | `Message::Disto(DistoMsg::Connect)` when not connected; when connected, the next laser reading fills the buffer (see `disto.rs`) |
| More (press and hold Close area) | Bay mirror, Jump to endpoint, rise x run (`x`), angle (`<`) |

Behaviour:

- Shown automatically in Touch density when Wall, Inner or Measure is active. Hidden when
  a hardware keyboard key is pressed; shown again on the next touch. The toggle is
  Menu > View > Pad and a button on the drawing.
- Wide and Medium: a floating panel bottom right, draggable, position remembered.
- Compact: docked along the bottom, above the tool bar's place, replacing the sheet.
- The readout at the top of the Pad mirrors the length strip.

## 4. Layout rules for touch

- Minimum hit target 48 x 48 in Touch density. Rows 48 high. Fields 44 high.
- Undo and Redo are always visible in the top bar.
- Slide-overs and sheets are dismissed by tapping the drawing or dragging them away.
  Sheets have three stops: peek (totals only, 56 high), half, full.
- Nothing essential is hover-only.
- Scroll areas scroll by drag with momentum. iced scrollables already handle touch
  scrolling; verify on a touchscreen.
- Text inputs must not be covered by an on-screen keyboard. On Android, lift the focused
  sheet above the keyboard inset.
- Respect the OS text scale: use logical pixels and `App.ui_scale`.

## 5. Android: what to verify (P16)

None of the following has been tested. Treat each as an open question and answer it with
a small experiment before planning the port.

| Question | Why it matters | Where to look |
|---|---|---|
| Does iced 0.14 (winit + wgpu) run acceptably on Android? | The whole interface depends on it | Build a hello-world with a canvas and a text input for `aarch64-linux-android` with `cargo-apk` or `xbuild`. Check touch, text input, soft keyboard, lifecycle (pause and resume), and frame pacing |
| Text input and the soft keyboard | Lengths, names and search all need typing | Same experiment |
| File open and save | `rfd` 0.15 is used for dialogs and has no Android backend that is known to work | Plan a `platform` trait with an Android implementation over the Storage Access Framework |
| Bluetooth laser | `btleplug` 0.11 lists Android support but needs Java glue and runtime permissions | `disto_transport.rs` |
| Printing and PDF viewing | Uses the `windows` crate (`Data_Pdf`, COM) | `print.rs`, `platform.rs`. PDF export itself is pure Rust in `sketch-core` and should work |
| Per-user files | `persist.rs` chooses a path per OS | Add the Android app data directory |
| Symbol library location | Loaded from a file beside the executable | Bundle as an asset and copy to app data on first run so it stays editable |
| SQLite store format | `rusqlite` with `bundled` compiles SQLite from source; needs the NDK toolchain | Build check |
| Appraise-It Pro integration | Windows program; not available on Android | Hide its commands when the platform is not Windows |
| Custom window frame | `decorations: false` and custom controls | Skip window controls on Android |

Code organisation to keep the port cheap, to do during the desktop work:

- Put every OS call behind `platform.rs` (it already holds `open_target`, `work_area`,
  `point_on_screen`, window activation). Add file pick, file save, share, print and
  storage paths to it instead of calling `rfd` and `windows` directly from features.
- Keep `#[cfg(windows)]` code out of `ui/`.
- Do not assume a mouse exists anywhere in `ui/`.

## 6. Stylus

Options already stores `inputMode` (Pen as Ink, Pen as Mouse, Touch) but nothing reads
it. Give it meaning:

| Mode | Pen | Finger |
|---|---|---|
| Pen as Ink | Draws, selects, drags handles | Pans and zooms only |
| Pen as Mouse | Same as a mouse | Same as touch |
| Touch | Same as a finger | Normal touch |

iced reports pen input as touch or mouse depending on platform. Check what winit
delivers on Windows Ink and on Android before promising pen-specific behaviour, and fall
back to "Touch" where pen and finger cannot be told apart.
