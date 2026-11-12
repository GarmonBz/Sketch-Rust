# Sketch Rust

A Windows floor-plan sketching program for appraisers: draw walls to scale, define areas,
and get living-area and building-area calculations. It can also stand in for Apex as the
sketch program Appraise-It Pro opens.

> **Status: development preview.** Appraise-It integration has been checked with
> Appraise-It Pro legacy and UAD 3.6 reports only. See
> [docs/APEX_REPLACEMENT.md](docs/APEX_REPLACEMENT.md).

## Download

1. Go to **[Releases](https://github.com/cpursley/Sketch-Rust/releases)** and download the
   **`.msi`** file from the newest release.
2. Open it. If Windows warns you, click **More info** > **Run anyway**.
3. Click through the installer. Sketch Rust then appears in the Start menu.

Requires 64-bit Windows 10 or 11, with no other software needed. For step-by-step help,
updating, the portable version and Appraise-It setup, see
**[docs/INSTALL.md](docs/INSTALL.md)**.

## Using it

Help is built in: **Help > User Guide**, **Help > Keyboard Shortcuts** (F1) and
**Help > Integration Guide**.

## Documentation

| Doc | What it covers |
|---|---|
| [INSTALL](docs/INSTALL.md) | Download, install, update, Appraise-It setup |
| [BUILDING](docs/BUILDING.md) | Building from source, releases, repository layout |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | How the code is organised |
| [APEX_REPLACEMENT](docs/APEX_REPLACEMENT.md), [APPRAISE_IT_PRO_INTEGRATION](docs/APPRAISE_IT_PRO_INTEGRATION.md) | Apex compatibility and Appraise-It integration |
| [DISTO](docs/DISTO.md), [SYMBOL_LIBRARY](docs/SYMBOL_LIBRARY.md) | Laser measure support, symbol library |
| [STATUS](docs/STATUS.md), [ACCEPTANCE](docs/ACCEPTANCE.md), [PARITY](docs/PARITY.md) | Change ledger, parity checks, feature audit |

All other notes are in [`docs/`](docs/).

## License

MIT, see [LICENSE](LICENSE). The CADdillo residential symbols are CC0 1.0, and the
Phosphor icons are MIT. See [docs/SYMBOL_LIBRARY.md](docs/SYMBOL_LIBRARY.md).
