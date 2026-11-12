# Architecture

Sketch Rust is a single Windows executable, `sketch-app.exe`. It is a native port of the
browser-based Appraisal Sketch 1.1, and also stands in for Apex as Appraise-It Pro's sketch
program.

```
sketch-app.exe
├── sketch-app (Rust, iced UI)        crates/sketch-app
│   ├── App: Message → update → view  main.rs
│   ├── canvas + renderer             sketch_canvas.rs, render.rs
│   ├── chrome & dialogs              ribbon*, dock, rail, statusbar, *_dialog
│   ├── Apex/Appraise-It bridge       apex_*.rs  ──hosts──▶  apex-compat.dll (C#, embedded)
│   └── platform glue                 platform.rs, windows crate, rfd, btleplug
└── sketch-core (Rust, no UI)         crates/sketch-core
    model · geometry · joins · area_calc · breakdown · skx · exporters · symbols
```

## Crates

| Crate | Role | Key deps |
|---|---|---|
| `sketch-core` | Headless engine: document model, geometry, area calculations, file formats, exporters. No UI or OS calls. | serde, tiny-skia, fontdb, flate2 |
| `sketch-app` | The application: UI, input, Windows integration, Apex compatibility. | iced 0.14, rfd, btleplug, windows |

## sketch-core

A clean-room port of the original JavaScript, which is kept in `refs/` as the reference
implementation. Numbers, sign conventions and tolerances are copied from it, never invented.

- `model.rs`: the persisted document (lines, polygons, labels, symbols, chains). It keeps the
  original's JSON field names so `.skx` files round-trip.
- `geometry.rs`, `joins.rs`, `solids.rs`: wall and polygon geometry, and wall joins.
- `area_calc.rs`, `breakdown.rs`: GLA/GBA totals and area breakdown tables.
- `skx.rs`: the native `.skx` container.
- `export_{pdf,svg,png,dxf}.rs`, `export_model.rs`, `print_layout.rs`: output. PNG is
  rasterised with tiny-skia.
- `symbols.rs`, `symbol_import.rs`, `symbol_edit.rs`, `cad_import.rs`: the symbol library and
  CAD block import.
- `disto.rs`: parses Leica DISTO measurements (the BLE transport lives in the app).

## sketch-app

### UI stack

The UI uses **iced 0.14**, which follows the Elm architecture. iced draws everything itself
through wgpu, with tiny-skia as a fallback renderer. There are no native Windows controls,
web view or Electron. The window has no system border: the title bar, ribbon, dock and dialogs
are all iced widgets. Their placement is measured from the original app and recorded in
`refs/ui-spec/`.

- `main.rs`: the single `App` state, the `Message` enum, `update`, `view`, `subscription`,
  and startup (`iced::application`).
- The canvas is an iced `canvas` widget (`sketch_canvas.rs`), drawn by `render.rs`, with
  `canvas_theme.rs` for styling and `minimap.rs` for the overview.
- Screen chrome: `titlebar`, `ribbon`/`ribbon_tabs`, `rail` (left tool rail), `dock` (right
  panels: Areas, Text, Symbols, Calcs, Layers), `statusbar`, `backstage` (File menu),
  `command_palette`, `context_menu`, `toast`.
- Editing: `drawing`, `input`, `snap`, `selection`, `handles`, `transform`, `shapes`,
  `room_shapes`, `chains`, `undo`/`undo_history`.
- Areas: `define_area`, `area_workflow`, `area_props`, `area_repair`, `calcs`.
- Persistence: `persist`, `autosave`, `recent`, `file_dialogs` (rfd), `window_state`, `options`.

### Apex / Appraise-It compatibility

Appraise-It launches `sketch-app.exe` in place of Apex. The Apex side is written in C#
(`src/apex_*.cs`):

- `apex_file.cs`, `apex_geometry.cs`, `apex_drafts.cs`: the AXF reader and writer (layouts
  1–38, including encrypted payloads).
- `apex_compat.cs`: the host protocol (boot, probe, start).
- `apex_legacy.cs`: legacy Pro report XML.
- `apex_codes.cs`: imports code tables.

`build.rs` compiles these with the .NET Framework 4 `csc.exe` into `apex-compat.dll` and
embeds the DLL in the exe. At runtime, `apex_compat.rs` writes the DLL out and hosts the CLR
in-process through `ICLRRuntimeHost`, calling into it with `ExecuteInDefaultAppDomain`. There
is no separate bridge program or service.

Related Rust modules:
- `apex_startup.rs`: parses the command line Apex is launched with.
- `apex_registration.rs`, `integration_setup.rs`: the Enable/Restore provider switch.
- `report_sketch.rs`: the images sent back to the report (metafiles).

Behaviour and validation are covered in [APEX_REPLACEMENT.md](APEX_REPLACEMENT.md) and
[APPRAISE_IT_PRO_INTEGRATION.md](APPRAISE_IT_PRO_INTEGRATION.md).

### Platform

- `platform.rs` and the `windows` crate: registry, shell, DPI, GDI and the WinRT PDF reader.
- `disto_transport.rs`: Bluetooth LE through btleplug (see [DISTO.md](DISTO.md)).
- `build.rs` also compiles the icon resource with the Windows SDK `rc.exe`.

## Testing

- Unit tests sit inside both crates (`*_tests.rs` and `#[cfg(test)]` modules).
- `src/e2e.rs`: a headless end-to-end suite using `iced_test`. It runs the real program with
  scripted mouse and keyboard input and takes screenshots, with no window. `tools/e2e.sh` runs
  it and compares the screenshots with `refs/ui-spec/gallery/`.
- `tests/apex-compat/`, `tools/test-apex-*.ps1`: Apex/AXF compatibility checks on Windows.
- Parity checks against the original app are recorded in [ACCEPTANCE.md](ACCEPTANCE.md); the change ledger is [STATUS.md](STATUS.md).

## Build and packaging

Building requires Windows, the MSVC Rust toolchain, the Windows SDK (`rc.exe`) and .NET
Framework 4 (`csc.exe`).

- `tools/package.ps1 -NativePreview` builds the portable zip into `dist/`.
- `tools/installer.ps1` builds the per-user WiX 5 MSI from `installer/SketchRust.wxs`. It
  installs to `%LOCALAPPDATA%\Programs\SketchRust`.
- `.github/workflows/release.yml` runs both on a Windows runner when a `v*` tag is pushed
  and attaches the results to the GitHub Release.

See [BUILD_VALIDATION.md](BUILD_VALIDATION.md).
