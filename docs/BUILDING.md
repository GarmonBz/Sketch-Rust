# Building and releasing

You only need this if you are changing the code. To install the app, see
[INSTALL.md](INSTALL.md).

## Requirements

Windows, with:
- the Rust toolchain (stable, MSVC target)
- the Windows SDK (`rc.exe`)
- .NET Framework 4 (`csc.exe`, built into Windows)

## Build and test

```
cargo build --release -p sketch-app
cargo test --workspace
bash tools/e2e.sh                    # headless UI suite + screenshot comparison
```

- `tools\package.ps1 -NativePreview` builds the portable zip into `dist\`.
- `tools\installer.ps1` builds the zip and then `dist\SketchRust-<version>-x64.msi`
  (source in `installer\`). Run it once with `-FetchWix` to download WiX, which needs a
  .NET runtime, not the SDK. Add `-Test` to install, check and remove the result.

See [BUILD_VALIDATION.md](BUILD_VALIDATION.md) for recorded validation runs.

## Publishing a release

Built files are not stored in the repository. GitHub builds them:

1. Bump `version` in `Cargo.toml`, add notes to `docs/RELEASE_README.txt`, and commit.
2. Tag the commit with the same version and push the tag:
   `git tag v0.1.30` then `git push origin v0.1.30`.
3. [`.github/workflows/release.yml`](../.github/workflows/release.yml) builds the zip and the
   MSI on a Windows runner and test-installs the MSI. It then attaches both, with their
   `.sha256` files, to the GitHub Release for that tag. If that release doesn't exist yet,
   it is created as a pre-release.

To build without publishing, run **Actions > Release > Run workflow**. The files appear
as a downloadable artifact on that run.

## Repository layout

| Path | What it is |
|---|---|
| `crates/sketch-core` | Geometry, calculations, file formats, symbol import |
| `crates/sketch-app` | The application (iced UI), Appraise-It/Apex compatibility |
| `installer/` | The WiX source of the Windows installer |
| `.github/workflows/` | Release build |
| `assets/` | Symbol library, icons, cursors |
| `examples/` | Sample files for [TEST_CHECKLIST.md](TEST_CHECKLIST.md) |
| `refs/` | The original web app's source and UI measurements the port is checked against |
| `tests/` | Apex compatibility checks |
| `tools/` | Packaging and test scripts |
| `docs/` | All documentation |

See [ARCHITECTURE.md](ARCHITECTURE.md) for how the code fits together.
