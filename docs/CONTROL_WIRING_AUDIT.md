# Control wiring audit — repaired in 0.1.15, 2026-10-30

All 13 findings below are repaired. The current audit executes 130 ribbon
button/field instances and 17 File-menu items and requires zero missing actions.
It now runs as a normal regression guard, alongside real-widget behavior tests.

The button tests exercise Copy/Cut/Paste/Delete on a selected 10-foot wall,
Select All on a defined four-wall area, Rotate Sketch through a 90-degree turn,
the read-only Area Attributes viewer, Select menu, Calcs panel and Area Codes
editor. File Save/Save As were originally clicked through their naming prompts;
0.1.16 removes those prompts and tests direct native-picker calls, SKX writes,
filename consistency, cancellation and write failure through the same buttons.
0.1.17 additionally verifies direct File/QAT Save with no picker, file binding
after Open and Save As, JSON preservation, and clearing the target on New.
The optimized Windows EXE was additionally tested through actual File Open,
Save and Save As buttons and native file dialogs. Both saved SKX files were
read back and retained all page data exactly; the new Save As name was correct.

Reproduce the repaired mappings and real-widget regressions:

```powershell
cargo test -p sketch-app --offline control_wiring_audit -- --nocapture
cargo test -p sketch-app --offline fixed_controls -- --test-threads=1
```

0.1.20 adds the Help > Appraise-It Integration action. Its actual Enable,
Restore and Close buttons have headless widget checks, including permission-
error feedback and Escape dismissal.

Current machine-readable results are under `target/control-audit/`. These are
development-only artifacts. See BUILD_VALIDATION.md for release verification.

## Original 0.1.14 findings

**13 enabled controls resolved directly to `Message::Noop`.** The underlying
operations already had native handlers; these entry points never invoked them.

| Location | Unwired controls | Working alternate entry points |
|---|---|---|
| File menu | Open…, Save, Save As… | Ctrl+O / F3; Ctrl+S / F2 / title-bar Save; F12 / command search |
| Home ribbon | Cut, Copy, Paste, Select All, Select, Delete | Ctrl+X, Ctrl+C, Ctrl+V, Ctrl+A, command search "Select by type", Delete; contextual Edit Tools buttons |
| Home ribbon | Rotate Sketch | Command search "Rotate sketch"; L / R |
| Home ribbon | Area Attributes | Command search "Area attributes"; contextual Edit Tools → Area Attributes, with an area selected |
| View ribbon | View Calcs | Status-bar totals chip; command search "View calcs" |
| Tools ribbon | Area Codes | Command search "Area codes" |

## Original causes and source locations

- `crates/sketch-app/src/backstage.rs::item_action` has no cases for Open…, Save,
  or Save As…. The visible menu creates enabled buttons for all three, and the
  wildcard returns `Message::Noop`. Root pointer handling only closes the menu;
  it does not dispatch file commands.
- `crates/sketch-app/src/ribbon.rs::command` maps clipboard, selection, and Delete
  for `Body::EditTools` only. Home creates separate enabled buttons without
  explicit actions, so its six corresponding controls reach the wildcard.
- Home's Rotate Sketch also has no command case.
- Home Area Attributes, View Calcs, and Tools Area Codes use two-line captions
  containing `\n`; their command cases expect space-separated captions. The
  exact-string lookup misses all three. Tools Area Breakdown uses a matching
  two-line command case and is wired.
- Real handlers exist in `main.rs`, and the 42-entry command-search catalog maps
  to native handlers. This finding concerns entry-point wiring, not absent
  clipboard, file, area-code, calculation, or attribute functionality.

## Original discovery verification and scope

- Executed the actual ribbon constructors and final command resolver for all
  five main tabs plus contextual wall, area+wall, area-only, text, free-symbol,
  and hosted-symbol selections: **129 button/field instances**, ten enabled
  no-op buttons. No further no-op fields or contextual buttons were found.
- Executed all **17 File-menu item mappings**: three no-op entries.
- Real headless widget clicks reproduced Home → Select, View → View Calcs,
  and Tools → Area Codes: their expected menu/panel/dialog never opened.
- The existing command-catalog coverage test passed for all 42 entries, and
  workspace/all-target Clippy passed with warnings denied after adding the
  opt-in diagnostics. These checks do not certify every underlying operation.
- Opt-in diagnostic output is saved under `target/control-audit/`:
  `ribbon.json`, `file-menu.json`, and `gui-clicks.json`.
- Reviewed status controls and `App::status`, title-bar actions, tool rail,
  keyboard map, command catalog, dock drag/click routes, and explicit no-op
  wrappers. No additional unwired entry point was identified in that review.
- Dock library buttons use an inner press/release handler; their outer no-op
  buttons supply hover styling. Blank backstage space intentionally absorbs
  clicks. Empty-stack Undo/Redo and the Bluetooth status chip are intentionally
  disabled. These are excluded from the 13 findings.
- Stale comments saying F1 / Ctrl+F1 are unwired are incorrect: the current map
  routes them to shortcuts / email. Stored-only source options and deferred
  DISTO hardware verification are separate from this wiring audit.

The original opt-in diagnostics recorded observations without behavior
assertions. They have now been replaced by required action and behavior checks.
During the original discovery, the full GUI suite, native file
dialogs, external email handler, and hardware were not exhaustively retested.
No Appraise-It reports or running user documents were changed.

The earlier 634-row feature ledger predated these discoveries. Wiring guards
and nonempty-sketch button tests now cover the repaired entry points;
empty-state tests had missed the former Center / Center & Fit placeholder.
