# Sketch Rust

A native Windows floor-plan sketching program for appraisers: draw walls to scale,
define areas, and get living-area and building-area calculations. It is a Rust port
of the browser-based Appraisal Sketch app.

## Install

Download the latest release and use one of:

- **Installer** — `SketchRust-<version>-x64.msi` (a Windows Installer package built with
  WiX Toolset 5). Installs for the current Windows user (no administrator permission), with
  a licence page, a choice of folder, an optional desktop shortcut and a Start Menu shortcut.
  Later versions upgrade it in place, and it is removed from Windows Settings > Apps >
  Installed apps. Silent install: `msiexec /i SketchRust-<version>-x64.msi /qn`.
- **Portable zip** — `SketchRust-<version>-native-preview-<date>-windows-x64.zip`.
  Extract it anywhere and run `sketch-app.exe`.

The downloads are not code-signed, so Windows SmartScreen may warn before running
them. Each has a `.sha256` file to check the download against.

Requires 64-bit Windows 10 or 11.

## Using it

Help is built in: **Help > User Guide** covers drawing, areas, symbols, printing and
saving. **Help > Keyboard Shortcuts** (F1) lists the keyboard shortcuts.

Quick start: press `E`, click a start point, type a length and press an arrow key for
each wall, press `A` to close the area, then pick what the area is in the Define Area
panel (`F4`).

### Appraise-It Pro reports

Keep the editable sketch in Sketchy or SKX. Use **File > Export as PNG** or
**File > Export as PDF** for a report attachment where your form supports it,
and enter the area totals in Appraise-It Pro. AXF files can be opened with
the licensed Apex application.

See the [Appraise-It Pro integration plan](docs/APPRAISE_IT_PRO_INTEGRATION.md)
for the proposed SFREP connection and guidance for users of older previews.

## Build from source

Requires the Rust toolchain (stable, MSVC target) and the Windows SDK (`rc.exe`).

```
cargo build --release -p sketch-app
cargo test --workspace
```

- `tools\package.ps1 -NativePreview` builds the portable zip into `dist\`.
- `tools\installer.ps1` builds the zip and then `dist\SketchRust-<version>-x64.msi`
  (source in `installer\`). Run it once with `-FetchWix` to download WiX (it needs a .NET
  runtime, not the SDK); add `-Test` to install, check and remove the result.

## Layout

| Path | What it is |
|---|---|
| `crates/sketch-core` | Geometry, calculations, file formats, symbol import |
| `crates/sketch-app` | The desktop application (iced UI) |
| `installer/` | The WiX source of the Windows installer |
| `assets/` | Symbol library, icons, cursors |
| `refs/` | The original web app's source and UI measurements the port is checked against |
| `docs/` | Release notes, build validation, the Appraise-It Pro plan and symbol-library notes |
| `tools/` | Packaging and test scripts |

`STATUS.md` is the running change ledger; `ACCEPTANCE.md` records the parity checks
against the original app.

## License

Sketch Rust uses the [Sketch Rust Free Use License](LICENSE). Anyone may use,
modify, and share the program for free, including using it for paid appraisals
and charging for the resulting sketches and reports. Selling or otherwise
monetizing the program or modified versions is prohibited.

The [CADdillo residential symbols](docs/SYMBOL_LIBRARY.md) are CC0 1.0.
The [Phosphor icons](assets/icons/LICENSE) are MIT.
