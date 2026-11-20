# Building Sketch Rust on macOS

Apple Silicon (`aarch64-apple-darwin`) packaging for a local/internal `.app`.
Windows remains the primary shipping platform (MSI / portable zip).

Design notes: [docs/superpowers/specs/2026-11-20-macos-app-bundle-design.md](superpowers/specs/2026-11-20-macos-app-bundle-design.md).

## Prerequisites

- Apple Silicon Mac
- Rust stable (this repo often pins via asdf; see `.tool-versions` if present)
- Xcode Command Line Tools (`xcode-select --install`)
- Optional: **Icon Composer** (bundled with Xcode) to tune Liquid Glass layers

## Dev run

```bash
cargo run -p sketch-app
```

Release build only:

```bash
cargo build --release -p sketch-app --target aarch64-apple-darwin
```

## Package a `.app`

```bash
./tools/package-macos.sh
```

Produces `dist/macos/Sketch Rust.app` (arm64 binary, `AppIcon.icns`, symbol library beside the executable). Requires [`cargo-bundle`](https://crates.io/crates/cargo-bundle) 0.12 (the script installs it if missing).

Bundle metadata lives in `crates/sketch-app/Cargo.toml` under `[package.metadata.bundle]`.

**Out of scope for this pass:** Developer ID signing, notarization, Sparkle, and universal (Intel + ARM) binaries.

## App icon

Source and regenerate steps: [assets/macos/README.md](../assets/macos/README.md).

- Open `assets/macos/SketchRust.icon` in Icon Composer to adjust glass / appearances.
- Rebuild the classic `.icns` used by `cargo-bundle`:

```bash
./tools/build-app-icon.sh assets/macos/layers/master-1024.png
```

## Windows-only features

These stay Windows-only and are not available in the macOS build:

- Appraise-It Pro / SFREP COM integration
- WiX MSI installer (`tools/installer.ps1`, `installer/`)
- Portable zip packaging via `tools/package.ps1`

On macOS, print preview uses PDFKit; File › Print still opens the PDF via Preview when needed.

## Keyboard shortcuts

Chords follow iced’s [`Modifiers::command`](https://docs.rs/iced/latest/iced/keyboard/struct.Modifiers.html)
(⌘ on macOS, Ctrl elsewhere) — same API as iced’s own `text_input` /
`text_editor`. Labels stay stored as `Ctrl+…` and are rewritten for Apple hosts
in `shortcut_host`. A future wasm build can override chords in `detect_web()`
when the browser client OS differs from the compile target.

Physical **Control** is still used for fine wheel zoom / drawing-arrow snaps
(trackpads often synthesize Control for pinch-zoom). Additive multi-select uses
the iced primary (**⌘-click** on Mac).
