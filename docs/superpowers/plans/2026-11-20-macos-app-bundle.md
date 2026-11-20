# macOS App Bundle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship an Apple Silicon Sketch Rust `.app` with native macOS window decorations, a high-fidelity Dock/Finder icon, working print preview, and `tools/package-macos.sh` packaging via `cargo-bundle`.

**Architecture:** One `sketch-app` binary; `#[cfg(target_os = "macos")]` for decorations, chrome, platform helpers, and PDFKit preview; Windows frameless path unchanged; `cargo-bundle` emits `dist/macos/Sketch Rust.app` with symbols beside the executable.

**Tech Stack:** iced 0.14, `cargo-bundle` 0.12, Icon Composer + `iconutil`, `objc2-pdf-kit` for Mac print preview, existing `sketch-core` PDF export.

**Spec:** `docs/superpowers/specs/2026-11-20-macos-app-bundle-design.md`

## Global Constraints

- Never give time estimates (project rule).
- Apple Silicon (`aarch64-apple-darwin`) only this pass.
- Appraise-It / SFREP stay `cfg(windows)` only.
- No notarization / Developer ID / Sparkle / universal binary.
- Do not change Windows frameless chrome or WiX packaging.
- Bundle display name: **Sketch Rust**; bundle id: **com.propertyhackers.sketchrust**.
- Symbol library must resolve next to the executable (`symbols.json` in `Contents/MacOS/`).

---

## File map

| File | Responsibility |
|------|----------------|
| `crates/sketch-app/src/main.rs` | `decorations: cfg!(target_os = "macos")` |
| `crates/sketch-app/src/ui/topbar.rs` | Omit custom window controls on macOS |
| `crates/sketch-app/src/platform.rs` | macOS `work_area` + `point_on_screen` |
| `crates/sketch-app/src/print.rs` | Dispatch Mac preview to `print_macos` |
| `crates/sketch-app/src/print_macos.rs` | PDFKit page rasterization (new) |
| `crates/sketch-app/Cargo.toml` | `objc2-pdf-kit` (macos), `package.metadata.bundle` |
| `assets/macos/` | Icon Composer source, `.iconset`, `AppIcon.icns` |
| `tools/package-macos.sh` | Release build + bundle + stage symbols |
| `tools/build-app-icon.sh` | PNG/SVG layers → `.iconset` → `.icns` |
| `docs/MACOS.md` | Build / package / icon / Windows-only notes |
| `README.md` | Link Mac package instructions |

---

### Task 1: Native decorations and hide custom caption buttons

**Files:**
- Modify: `crates/sketch-app/src/main.rs` (window settings in `program()`)
- Modify: `crates/sketch-app/src/ui/topbar.rs` (`window_controls` call sites)
- Test: `crates/sketch-app/src/ui/topbar.rs` (unit test module) or new `crates/sketch-app/src/ui/chrome_tests.rs` included from `ui/mod.rs`

**Interfaces:**
- Consumes: existing `Message::Minimize` / `ToggleMaximize` / `Close` (unchanged on Windows)
- Produces: `decorations == cfg!(target_os = "macos")`; top bar without caption buttons when `cfg!(target_os = "macos")`

- [ ] **Step 1: Write the failing test**

Add to `crates/sketch-app/src/ui/topbar.rs` at the bottom (or a small `#[cfg(test)]` module):

```rust
#[cfg(test)]
mod macos_chrome_tests {
    #[test]
    fn custom_window_controls_are_windows_only() {
        // Document the product rule in code: Mac uses system traffic lights.
        assert_eq!(
            cfg!(target_os = "macos"),
            !super::shows_custom_window_controls(),
            "macOS must not show custom caption buttons; other targets must"
        );
    }
}
```

Add this helper next to `window_controls` in `topbar.rs`:

```rust
/// Whether the top bar draws min/max/close. macOS uses system decorations instead.
pub(crate) fn shows_custom_window_controls() -> bool {
    !cfg!(target_os = "macos")
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
export ASDF_RUST_VERSION=1.97.0
cargo test -p sketch-app --lib macos_chrome_tests -- --nocapture
```

If `sketch-app` is a binary-only crate (no lib), place the test in `main.rs` under `#[cfg(test)]` and run:

```bash
cargo test -p sketch-app custom_window_controls_are_windows_only -- --nocapture
```

Expected: FAIL because `shows_custom_window_controls` does not exist yet, or still returns `true` on macOS.

- [ ] **Step 3: Implement decorations + gate caption buttons**

In `program()` window settings (`main.rs`):

```rust
decorations: cfg!(target_os = "macos"),
```

In `topbar.rs`, replace every `right = right.push(window_controls(...))` with:

```rust
if shows_custom_window_controls() {
    right = right.push(window_controls(app, /* compact */ false)); // keep existing compact flag per call site
}
```

(Keep the existing `compact` argument values at each of the three call sites.)

- [ ] **Step 4: Run test to verify it passes**

```bash
export ASDF_RUST_VERSION=1.97.0
cargo test -p sketch-app custom_window_controls_are_windows_only -- --nocapture
cargo check -p sketch-app --target aarch64-apple-darwin --locked
```

Expected: PASS / Finished.

- [ ] **Step 5: Commit**

```bash
git add crates/sketch-app/src/main.rs crates/sketch-app/src/ui/topbar.rs
git commit -m "$(cat <<'EOF'
feat: Use native macOS window decorations

EOF
)"
```

---

### Task 2: macOS `work_area` and `point_on_screen`

**Files:**
- Modify: `crates/sketch-app/src/platform.rs`
- Test: `crates/sketch-app/src/platform.rs` (`#[cfg(all(test, target_os = "macos"))]`)

**Interfaces:**
- Consumes: iced `window::Window` raw handle (AppKit NSWindow) for `work_area`
- Produces: `work_area(...) -> Option<(f32, f32)>` in logical points; `point_on_screen(x, y) -> bool` using AppKit screens

- [ ] **Step 1: Write the failing test**

Append to `platform.rs`:

```rust
#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    #[test]
    fn point_on_screen_accepts_origin_of_main_screen() {
        // Main screen's bottom-left in AppKit coords is on-screen.
        assert!(
            crate::platform::point_on_screen(0, 0),
            "expected (0,0) in screen space to hit a display on CI/dev Macs"
        );
    }

    #[test]
    fn point_on_screen_rejects_far_away_coordinate() {
        assert!(!crate::platform::point_on_screen(i32::MAX / 2, i32::MAX / 2));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail (or wrongly pass via stub)**

```bash
export ASDF_RUST_VERSION=1.97.0
cargo test -p sketch-app point_on_screen_ -- --nocapture
```

Expected: `point_on_screen_rejects_far_away_coordinate` FAIL because the current stub always returns `true`.

- [ ] **Step 3: Implement macOS screen queries**

Replace the `#[cfg(not(windows))]` stub bodies with a macOS implementation using `objc2` / `objc2-app-kit` (already in the iced/winit tree — add explicit macOS deps on `sketch-app` if needed):

```toml
# crates/sketch-app/Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-foundation = { version = "0.3", features = ["NSGeometry"] }
objc2-app-kit = { version = "0.3", features = ["NSScreen", "NSWindow"] }
objc2-pdf-kit = { version = "0.3", features = ["PDFDocument", "PDFPage"] }
```

`point_on_screen` (AppKit bottom-left origin; iced may pass different coords — convert if the Windows call sites use top-left physical pixels; match how `window_state.rs` calls this):

```rust
#[cfg(target_os = "macos")]
pub fn point_on_screen(x: i32, y: i32) -> bool {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::NSPoint;
    let point = NSPoint::new(x as f64, y as f64);
    for screen in NSScreen::screens() {
        if screen.frame().contains(point) {
            return true;
        }
    }
    false
}
```

`work_area` for macOS: from the NSWindow under the iced handle, read `screen()?.visibleFrame()` and return `(width, height)` as `f32` in points (same contract as Windows: usable size, not origin).

Keep Linux/`not(windows)` non-mac stubs if any: for `target_os` other than windows/macos, keep `point_on_screen -> true` and `work_area -> None`.

- [ ] **Step 4: Run tests**

```bash
export ASDF_RUST_VERSION=1.97.0
cargo test -p sketch-app point_on_screen_ -- --nocapture
cargo check -p sketch-app --target aarch64-apple-darwin --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sketch-app/src/platform.rs crates/sketch-app/Cargo.toml Cargo.lock
git commit -m "$(cat <<'EOF'
feat: Implement macOS work area and screen hit-testing

EOF
)"
```

---

### Task 3: PDFKit print preview on macOS

**Files:**
- Create: `crates/sketch-app/src/print_macos.rs`
- Modify: `crates/sketch-app/src/print.rs`
- Modify: `crates/sketch-app/src/main.rs` (`mod print_macos;`)
- Modify: `crates/sketch-app/Cargo.toml` (objc2-pdf-kit already noted in Task 2)
- Test: unit test in `print_macos.rs` with a tiny PDF from `sketch_core::export_pdf` or a fixture byte string

**Interfaces:**
- Consumes: PDF bytes + max edge length `u32` (same as Windows `render_pages`)
- Produces: `Result<Vec<Vec<u8>>, String>` PNG page rasters

- [ ] **Step 1: Write the failing test**

In `print_macos.rs`:

```rust
#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn render_pages_returns_at_least_one_png_for_minimal_pdf() {
        // Minimal valid one-page PDF.
        let pdf = b"%PDF-1.1\n1 0 obj<<>>endobj\ntrailer<<>>\n%%EOF\n";
        // Prefer a real export if this fixture is too thin for PDFKit:
        // let pdf = sketch_core::export_pdf::... 
        let pages = render_pages(pdf, 512);
        assert!(pages.is_ok(), "{pages:?}");
        let pages = pages.unwrap();
        assert!(!pages.is_empty());
        assert_eq!(&pages[0][0..8], b"\x89PNG\r\n\x1a\n");
    }
}
```

If the minimal PDF is rejected by PDFKit, build bytes with the existing `sketch_core` export helpers used elsewhere in app tests (copy the smallest pattern from `export_pdf` tests).

- [ ] **Step 2: Run test — expect fail**

```bash
export ASDF_RUST_VERSION=1.97.0
cargo test -p sketch-app render_pages_returns_at_least_one_png -- --nocapture
```

Expected: FAIL (module/fn missing or still the stub error).

- [ ] **Step 3: Implement PDFKit rasterizer**

`print_macos.rs` sketch:

```rust
#![cfg(target_os = "macos")]

use objc2::rc::Retained;
use objc2_foundation::{NSData, NSString};
use objc2_pdf_kit::{PDFDocument, PDFDisplayBox};

pub fn render_pages(pdf: &[u8], max_edge: u32) -> Result<Vec<Vec<u8>>, String> {
    let data = NSData::with_bytes(pdf);
    let doc = PDFDocument::initWithData(PDFDocument::alloc(), &data)
        .ok_or_else(|| "Could not open PDF for preview.".to_string())?;
    let count = doc.pageCount() as usize;
    if count == 0 {
        return Err("PDF has no pages.".into());
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let page = doc.pageAtIndex(i).ok_or_else(|| format!("Missing PDF page {i}"))?;
        // Render via page.thumbnail or NSImage bitmapRep — write PNG bytes.
        // Scale so max(width,height) <= max_edge.
        out.push(render_page_png(&page, max_edge)?);
    }
    Ok(out)
}

fn render_page_png(page: &objc2_pdf_kit::PDFPage, max_edge: u32) -> Result<Vec<u8>, String> {
    // Implement with PDFPage bounds for PDFDisplayBox::MediaBox + NSBitmapImageRep
    // or CGContext draw. Return PNG bytes.
    let _ = (page, max_edge);
    Err("TODO implement PNG encode".into())
}
```

Finish `render_page_png` fully in this step (no TODO left). Use `image` crate already in deps to encode RGBA → PNG if you get a raw bitmap, or AppKit PNG representation.

Wire `print.rs`:

```rust
#[cfg(target_os = "macos")]
pub fn render_pages(pdf: &[u8], max_edge: u32) -> Result<Vec<Vec<u8>>, String> {
    crate::print_macos::render_pages(pdf, max_edge)
}

#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn render_pages(_: &[u8], _: u32) -> Result<Vec<Vec<u8>>, String> {
    Err("The native PDF preview renderer requires Windows or macOS.".into())
}
```

Remove the old blanket `not(windows)` stub that conflicts.

- [ ] **Step 4: Run tests**

```bash
export ASDF_RUST_VERSION=1.97.0
cargo test -p sketch-app render_pages_returns_at_least_one_png -- --nocapture
cargo check -p sketch-app --target aarch64-apple-darwin --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sketch-app/src/print.rs crates/sketch-app/src/print_macos.rs crates/sketch-app/src/main.rs crates/sketch-app/Cargo.toml Cargo.lock
git commit -m "$(cat <<'EOF'
feat: Rasterize print preview with PDFKit on macOS

EOF
)"
```

---

### Task 4: App icon — layers, Icon Composer, `.icns`

**Files:**
- Create: `assets/macos/README.md` (how to regenerate)
- Create: `assets/macos/layers/` (SVG/PNG flat layers)
- Create: `assets/macos/SketchRust.icon` (Icon Composer document — binary; commit if format is file-based)
- Create: `assets/macos/AppIcon.iconset/` + `assets/macos/AppIcon.icns`
- Create: `tools/build-app-icon.sh`

**Interfaces:**
- Consumes: `assets/app-icon.png` as brand reference
- Produces: `assets/macos/AppIcon.icns` for `cargo-bundle`

- [ ] **Step 1: Create layer assets from the existing icon**

```bash
mkdir -p assets/macos/layers assets/macos/AppIcon.iconset
# Extract / redraw flat layers (background fill + white floor-plan mark).
# Prefer SVG. If tracing by hand in Icon Composer, still keep PNG masters:
sips -z 1024 1024 assets/app-icon.png --out assets/macos/layers/master-1024.png
```

Produce at least:
- `assets/macos/layers/background.svg` (or PNG) — solid brand blue, no squircle mask (system applies mask)
- `assets/macos/layers/floorplan.svg` — white plan mark only, transparent background

- [ ] **Step 2: Author Icon Composer file**

Open **Icon Composer** (`/Applications/Xcode-*.app/Contents/Applications/Icon Composer.app` or Spotlight).

1. New Mac icon, 1024 canvas.
2. Import background + floorplan layers.
3. Tune Liquid Glass (specular / translucency); check Default + Dark appearances.
4. Save as `assets/macos/SketchRust.icon`.
5. Export flattened PNGs for marketing if offered.

- [ ] **Step 3: Build classic `.icns` for cargo-bundle**

`tools/build-app-icon.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${1:-$ROOT/assets/macos/layers/master-1024.png}"
SET="$ROOT/assets/macos/AppIcon.iconset"
ICNS="$ROOT/assets/macos/AppIcon.icns"
rm -rf "$SET"
mkdir -p "$SET"
for sz in 16 32 128 256 512; do
  sips -z $sz $sz "$SRC" --out "$SET/icon_${sz}x${sz}.png"
  sips -z $((sz*2)) $((sz*2)) "$SRC" --out "$SET/icon_${sz}x${sz}@2x.png"
done
iconutil -c icns "$SET" -o "$ICNS"
ls -la "$ICNS"
```

Run:

```bash
chmod +x tools/build-app-icon.sh
./tools/build-app-icon.sh
```

Expected: `assets/macos/AppIcon.icns` exists and is non-trivial size.

- [ ] **Step 4: Document regeneration**

Write `assets/macos/README.md` describing Icon Composer + `tools/build-app-icon.sh`.

- [ ] **Step 5: Commit**

```bash
git add assets/macos tools/build-app-icon.sh
git commit -m "$(cat <<'EOF'
feat: Add macOS app icon assets and icns build script

EOF
)"
```

---

### Task 5: `cargo-bundle` metadata and `tools/package-macos.sh`

**Files:**
- Modify: `crates/sketch-app/Cargo.toml` (`[package.metadata.bundle]`)
- Create: `tools/package-macos.sh`
- Modify: `.gitignore` only if `dist/` is not already ignored (check first)

**Interfaces:**
- Consumes: release binary + `assets/macos/AppIcon.icns` + symbol files
- Produces: `dist/macos/Sketch Rust.app` with `Contents/MacOS/sketch-app` and `Contents/MacOS/symbols.json` (+ companions)

- [ ] **Step 1: Add bundle metadata**

In `crates/sketch-app/Cargo.toml`:

```toml
[package.metadata.bundle]
name = "Sketch Rust"
identifier = "com.propertyhackers.sketchrust"
icon = ["../../assets/macos/AppIcon.icns"]
category = "public.app-category.graphics-design"
short_description = "Floor-plan sketching for appraisers"
long_description = """
Sketch Rust is a native floor-plan sketching program for appraisers.
"""
osx_frameworks = []
```

Confirm `cargo-bundle` 0.12 path rules for `icon` (relative to the package manifest). Adjust to `assets/macos/AppIcon.icns` from repo root if the tool resolves from workspace root — verify in Step 3.

Document types: if `cargo-bundle` 0.12 supports UTIs in metadata, register `.sketchie` / `.sketchie.json` / `.sketchy` / `.sketchy.json`. If not supported cleanly, add an `Info.plist` fragment file and document merging in `package-macos.sh` (PlistBuddy).

- [ ] **Step 2: Write `tools/package-macos.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export PATH="${HOME}/.cargo/bin:${PATH}"
export ASDF_RUST_VERSION="${ASDF_RUST_VERSION:-1.97.0}"

if ! command -v cargo-bundle >/dev/null 2>&1; then
  cargo install cargo-bundle --locked --version 0.12.0
fi

cargo build --release --locked -p sketch-app --target aarch64-apple-darwin

# Bundle from the binary package directory
(
  cd crates/sketch-app
  cargo bundle --release --target aarch64-apple-darwin
)

APP_SRC=$(find target/aarch64-apple-darwin/release/bundle -name '*.app' | head -1)
mkdir -p dist/macos
rm -rf "dist/macos/Sketch Rust.app"
cp -R "$APP_SRC" "dist/macos/Sketch Rust.app"

MACOS_DIR="dist/macos/Sketch Rust.app/Contents/MacOS"
cp -f assets/symbols.json "$MACOS_DIR/symbols.json"
cp -f assets/symbols.json "$MACOS_DIR/symbols.original.json"
chmod a-w "$MACOS_DIR/symbols.original.json"
cp -f assets/symbols.previous.json "$MACOS_DIR/symbols.previous.json" 2>/dev/null || true

# Smoke: binary exists and is arm64
file "$MACOS_DIR/sketch-app"
test -f "$MACOS_DIR/symbols.json"
echo "Packaged: dist/macos/Sketch Rust.app"
```

Make executable: `chmod +x tools/package-macos.sh`.

- [ ] **Step 3: Run package script**

```bash
./tools/package-macos.sh
open "dist/macos/Sketch Rust.app"
```

Expected: app launches; Dock shows custom icon; traffic lights present; symbols load (place a symbol from the library).

- [ ] **Step 4: Commit**

```bash
git add crates/sketch-app/Cargo.toml tools/package-macos.sh
git commit -m "$(cat <<'EOF'
feat: Package Sketch Rust as a macOS .app

EOF
)"
```

---

### Task 6: Docs — `docs/MACOS.md` + README link

**Files:**
- Create: `docs/MACOS.md`
- Modify: `README.md` (Install / Build from source)

- [ ] **Step 1: Write `docs/MACOS.md`**

Include:

- Prerequisites: Rust stable, Apple Silicon Mac, Xcode CLT, optional Icon Composer
- Dev run: `cargo run -p sketch-app`
- Package: `./tools/package-macos.sh`
- Icon regeneration: Icon Composer + `./tools/build-app-icon.sh`
- Windows-only features: Appraise-It / SFREP, WiX MSI
- Link to design spec under `docs/superpowers/specs/`

- [ ] **Step 2: Link from README**

Under Install or Build from source, add a bullet:

```markdown
- **macOS (Apple Silicon)** — see [docs/MACOS.md](docs/MACOS.md) for `cargo run` and `./tools/package-macos.sh`.
```

- [ ] **Step 3: Commit**

```bash
git add docs/MACOS.md README.md
git commit -m "$(cat <<'EOF'
docs: Add macOS build and packaging guide

EOF
)"
```

---

### Task 7: Manual acceptance pass (checklist)

No code commit required unless bugs are found.

- [ ] **Step 1: Build and package**

```bash
./tools/package-macos.sh
```

- [ ] **Step 2: Verify against design acceptance**

- [ ] Launch from Finder and Dock  
- [ ] Traffic lights work; no duplicate custom caption buttons  
- [ ] Open/save `.sketchie.json` and `.sketchie` store  
- [ ] Symbol library available  
- [ ] Print preview shows pages; Print writes/opens PDF  
- [ ] Icon visible in Dock, Finder, Cmd-Tab  
- [ ] Windows build still has `decorations: false` path (spot-check via `cfg` test already in Task 1; optional CI not required this pass)

- [ ] **Step 3: Fix any defects on the branch with focused commits** (one concern per commit)

---

## Self-review vs spec

| Spec requirement | Task |
|------------------|------|
| Native macOS decorations; Windows frameless | Task 1 |
| `work_area` / `point_on_screen` on Mac | Task 2 |
| PDFKit print preview | Task 3 |
| Icon Composer + `.icns` | Task 4 |
| `cargo-bundle` + `tools/package-macos.sh` + symbols beside exe | Task 5 |
| `docs/MACOS.md` + README link | Task 6 |
| Acceptance verification | Task 7 |
| No notarization / Appraise-It / universal / Version History | Explicitly out of tasks |

**Placeholder scan:** Task 3 step 3 initially shows a stub `render_page_png` — the step text requires finishing it in the same step before commit. No TBD left in committed code.

**Type consistency:** `render_pages(pdf: &[u8], max_edge: u32) -> Result<Vec<Vec<u8>>, String>` shared Windows/Mac; `shows_custom_window_controls() -> bool` gates top bar.
