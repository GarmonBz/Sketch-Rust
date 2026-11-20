# macOS app bundle for Sketch Rust — Design

**Branch:** `macos-app-bundle`  
**Status:** Approved in brainstorming (2026-11-20)  
**Success bar:** Packaged local/internal `.app` (option B), with native macOS window decorations and a high-fidelity Mac app icon.

## Goal

Ship a first-class **Apple Silicon** macOS build of Sketch Rust as a double-clickable `.app`, using iced’s normal desktop path, without Appraise-It / SFREP, notarization, or a full Mac HIG redesign.

## Context

- UI framework: **iced 0.14** (`iced::application`, winit, wgpu / tiny-skia).
- Host target `aarch64-apple-darwin` already typechecks for `sketch-app`.
- Windows remains the primary shipping platform (WiX MSI / portable zip).
- Existing window icon art: `assets/app-icon.png` (blue squircle + white floor-plan mark).
- Icon Composer is available on this machine (bundled with Xcode).

## Approach (chosen)

**Platform `cfg` + `cargo-bundle` `.app`**, not an Xcode app shell and not `cargo-packager` for this pass.

## Architecture

### Single binary

- One `sketch-app` crate / binary for all desktop targets.
- No separate macOS crate or iced rewrite.
- Windows-only modules stay behind `#[cfg(windows)]` (notably Appraise-It / SFREP COM).

### Window chrome

| Platform | `window::Settings::decorations` | Chrome |
|----------|----------------------------------|--------|
| macOS | `true` | System title bar + traffic lights |
| Windows | `false` (unchanged) | Custom frameless top bar and window controls |

On macOS:

- Hide or no-op custom minimize / maximize / close controls that duplicate traffic lights.
- Keep the in-app top bar for menu, QAT, document title, and search.
- Window move / zoom / close are owned by the system title bar.

Entry point today sets `decorations: false` unconditionally in `crates/sketch-app/src/main.rs` (`program()`); that becomes platform-conditional.

### Packaging

- New script: `tools/package-macos.sh`.
- Steps:
  1. `cargo build --release -p sketch-app` for `aarch64-apple-darwin`.
  2. Stage bundle resources (at minimum the symbol library files already copied by Windows `tools/package.ps1`: `symbols.json`, companions, license/docs as needed).
  3. Run **`cargo-bundle`** to emit `dist/macos/Sketch Rust.app` (exact bundle display name may be `Sketch Rust` or `SketchRust`; pick one and keep Info.plist / Dock name consistent).
- Register document types for:
  - `.sketchie`, `.sketchie.json`
  - legacy `.sketchy`, `.sketchy.json`
- Barlow fonts remain embedded via the existing iced font load path; no separate font copy required for v1.
- **Notarization, Developer ID signing, Sparkle, and universal (Intel + ARM) binaries are out of scope.**

### Icon pipeline

1. Derive **flat vector layers** from `assets/app-icon.png` (background + floor-plan foreground; optional door/window layers for Liquid Glass control).
2. Author **`assets/macos/SketchRust.icon`** (or similar) in **Icon Composer**.
3. Export:
   - Icon Composer source committed to the repo.
   - Flattened marketing PNGs as needed.
   - Classic **`.icns`** via `iconutil` from a generated `.iconset` for `cargo-bundle` / Finder.
4. Keep `assets/app-icon.png` as iced `window::Icon` unless a Mac-sized PNG is wired later.

### Platform gaps

**`crates/sketch-app/src/platform.rs`**

- `work_area`: implement for macOS (AppKit / CoreGraphics usable frame), not Windows-only.
- `point_on_screen`: real multi-monitor test on macOS.
- Single-instance activate-existing: **optional**; leaving the non-Windows stub (always allow a second window) is acceptable for this pass.

**Print / Print Preview (`print.rs`)**

- PDF bytes continue to come from `sketch-core` (already cross-platform).
- Replace `#[cfg(not(windows))]` `render_pages` stub (currently errors) with a **macOS PDFKit** (or equivalent) rasterizer so in-app preview works.
- File › Print may still write the temp PDF and `open` it in Preview as a backup (`platform::open_target` already uses `open` on macOS).

**Fonts / theme**

- Bundled Barlow remains the UI type path.
- Segoe-named fallbacks may resolve differently on Mac; no dedicated Mac font pack required for v1.

## Testing (acceptance)

- Release build of `sketch-app` on Apple Silicon succeeds.
- `tools/package-macos.sh` produces a runnable `.app` under `dist/macos/`.
- App launches from Finder and Dock; traffic lights work; Windows frameless behavior unchanged when building for Windows.
- Open/save Sketchie JSON and store; symbol library loads from the bundle layout.
- Print preview shows page rasters; Print writes/opens PDF.
- App icon appears in Dock, Finder, and Cmd-Tab.

## Documentation

- Add `docs/MACOS.md` (or a clear README section) covering:
  - build and package commands
  - Icon Composer / `.icns` notes
  - what remains Windows-only (Appraise-It / SFREP, WiX installer)

## Out of scope

- Appraise-It Pro / SFREP COM integration on Mac
- Notarization, Developer ID, Mac App Store, Sparkle
- `x86_64-apple-darwin` / universal binary
- Full Mac HIG redesign beyond decorations + icon
- Porting Version History UI from the unmerged Sketchy PR
- Time estimates of any kind (project rule: never)

## Non-goals disguised as stretch

Do not expand this pass into Android, Linux packaging, or iced version upgrades unless a Mac blocker requires it.

## File / tool touch list (indicative)

| Area | Likely paths |
|------|----------------|
| Decorations / chrome | `crates/sketch-app/src/main.rs`, `ui/topbar.rs` (and any window-control widgets) |
| Platform | `crates/sketch-app/src/platform.rs` |
| Print preview | `crates/sketch-app/src/print.rs` (+ thin PDFKit bridge if needed) |
| Bundle metadata | `Cargo.toml` bundle keys and/or `tools/package-macos.sh` |
| Icon | `assets/macos/`, `assets/app-icon.png` (source), generated `.icns` |
| Docs | `docs/MACOS.md`; link from `README.md` Install section |

## Decisions log

| Decision | Choice |
|----------|--------|
| Success bar | Packaged `.app` for local/internal use |
| Window chrome | Native macOS decorations; Windows stays frameless |
| Packaging | `cargo-bundle` + `tools/package-macos.sh` |
| Icon | Icon Composer + `.icns` for bundle |
| Print on Mac | PDFKit raster for preview; Preview app backup for Print |
| Single-instance | Optional; stub OK for v1 |
| Arch | Apple Silicon first |
