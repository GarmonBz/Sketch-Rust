# Feature audit — 2026-10-29

0.1.15 follow-up (2026-10-30): all 13 File/Home/View/Tools controls found to have
missing action mappings are repaired. Required guards cover 129 ribbon
button/field instances and 17 File-menu items; actual button regressions use
nonempty sketches. See BUILD_VALIDATION.md and CONTROL_WIRING_AUDIT.md in the
release package for verification and the original findings.

Captured standalone Appraisal Sketch 1.1 plus the requested area workflow, symbol management, layer JSON import, recovery autosave and native Leica BLE measurement feed.

Manual code review by implementation group and row exceptions; automated regression/visual evidence is recorded separately in ACCEPTANCE.md. Status is not inferred from test counts.

## Inventory coverage

All 634 inventory rows are classified. The old ~495/634 and ~88% estimates are retired.

| Classification | Rows |
|---|---:|
| hardware-deferred | 5 |
| implemented | 546 |
| native-adaptation | 43 |
| native-enhancement | 29 |
| source-inactive | 8 |
| suite-only | 3 |

`implemented`, `native-adaptation` and `native-enhancement` describe software behavior. `hardware-deferred` includes implemented Disto software whose physical connection is unverified. `reference-unavailable`, `source-inactive` and `suite-only` rows are explicitly not claimed implemented in this standalone build.

## Per-screen coverage

| Screen | Rows | Implemented / adapted / enhanced | Deferred or outside captured standalone scope |
|---|---:|---:|---:|
| 1. Chrome | 15 | 15 | 0 |
| 2. Ribbon and themes | 19 | 18 | 1 |
| 3. Dock | 20 | 18 | 2 |
| 4. Status and navigation | 36 | 35 | 1 |
| 5. Drawing | 64 | 64 | 0 |
| 6. Selection and editing | 88 | 88 | 0 |
| 7. Areas and calculations | 79 | 76 | 3 |
| 8. Symbols | 45 | 45 | 0 |
| 9. Text, chains and measurement | 46 | 46 | 0 |
| 10. Pages, layers and reference images | 57 | 57 | 0 |
| 11. Files, exports and options | 81 | 81 | 0 |
| 12. Utilities and dialogs | 84 | 75 | 9 |

## Software gaps closed in this audit

- Symbol manager now keeps a live preview beside the list, uses a separate enlarged viewer and fits footer actions at tested 800x600 and larger sizes.
- Persistent symbol manager with hide/restore, double-confirmed purge, categories, DXF/JSON import and custom symbols carried in SKX.
- Undoable layer JSON import and complete layer exports.
- Timed recovery autosave, startup/File recovery, atomic writes and successful-save cleanup.
- Bay-window mirror (B): reflected geometry, drawing chain and undo/redo.
- Calcs Calculation Details: invokes Area Breakdown.
- File commands in command search: Save As, New, Open, Import and PNG/SVG/DXF export.
- Selection dragging: absolute Area Grid Snap, relative ordinary geometry snap, free label movement, and correct activation threshold.
- Placement previews: actual wall-snapped symbol geometry and text instead of a placeholder.
- Newly placed symbols: selected with contextual editing controls.

## Remaining boundaries

- Disto physical-device validation is deferred at the user's request. See [DISTO.md](DISTO.md).
- Symbol additions support DXF and symbol JSON. Binary DWG requires conversion to DXF; no native DWG decoder is advertised.
- Parent-suite embedding/report-bridge hooks and hidden programmatic commands are outside the standalone UI; see the row ledger for exact IDs.
- Native pickers and lists, cursor artwork, dialog measurements, tooltips, animation and some swatch previews differ visually. Options and all other dialogs are not universally pixel-certified. Existing reference differences are recorded in ACCEPTANCE.md.
- Some source-inactive ink/addendum options remain stored-only. Autosave is now a native recovery feature; reference images remain session-only.

## Evidence and reproduction

- Full row ledger: [feature-audit.json](../refs/ui-spec/feature-audit.json), including implementation paths, exception reasons and grouped regression names.
- User feature list: [FEATURE_INVENTORY.md](FEATURE_INVENTORY.md).
- Actual test/build/screenshot results: [ACCEPTANCE.md](ACCEPTANCE.md); counts are validation evidence, not a feature-completion percentage.
- Reproduce the ledger: `python tools/audit_features.py`. Its classifications are explicit reviewed decisions, not automatic feature detection.


## 0.1.9 native Apex compatibility

This adds a development compatibility preview beyond the original standalone
parity scope. Genuine AXF editing, metric/transformed geometry, recursive native calculations,
unfinished drafts, host code-table URLs, independent multiple-file sessions,
SDK APIs and legacy file/XML/COM entry paths run inside the sketch EXE. It is not a completed universal Apex replacement.
See APEX-COMPATIBILITY.md in the package (docs/APEX_REPLACEMENT.md in the source)
for verified cases, unsupported native cases and host-validation gates.
