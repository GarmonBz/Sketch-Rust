"""Reproduce the 2026-10-29 code-reviewed standalone feature ledger.

Classification below is an explicit audit decision, not inferred from test counts.
Grouped regression evidence is not individual pixel acceptance for every row.
"""
from collections import Counter
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
APP = "crates/sketch-app/src/"
CORE = "crates/sketch-core/src/"
GROUPS = {
    1: ("Chrome", ["titlebar.rs", "ribbon.rs", "rail.rs", "main.rs"], "window|title|ribbon|default"),
    2: ("Ribbon and themes", ["ribbon.rs", "backstage.rs", "popup_menu.rs", "canvas_theme.rs"], "ribbon|backstage|theme|popup"),
    3: ("Dock", ["dock.rs", "main.rs", "symlib.rs"], "dock|library|sidebar"),
    4: ("Status and navigation", ["statusbar.rs", "view_state.rs", "main.rs"], "status|zoom|filter|north|page|snap"),
    5: ("Drawing", ["drawing.rs", "render.rs", "input.rs", "color_dialog.rs", "shapes.rs"], "draw|wall|bow|keyboard|bay"),
    6: ("Selection and editing", ["selection.rs", "handles.rs", "transform.rs", "undo.rs", "clipboard.rs", "main.rs"], "select|undo|rotate|scale|drag|curve|paste"),
    7: ("Areas and calculations", ["area_workflow.rs", "area_props.rs", "area_dialog.rs", "codes_editor.rs", "calcs.rs", "breakdown_dialog.rs"], "area|workflow|calcs|breakdown|codes_json"),
    8: ("Symbols", ["placed_symbol.rs", "symlib.rs", "symbol_organizer.rs", "symbol_manager.rs", "management_tests.rs", "sketch_canvas.rs"], "symbol|opening|swing|placement"),
    9: ("Text, chains and measurement", ["placed_text.rs", "text_manager.rs", "chains.rs", "room_shapes.rs", "main.rs"], "text|label|chain|measure|room"),
    10: ("Pages, layers and reference images", ["pages.rs", "layers.rs", "layer_io.rs", "pages_nav.rs", "reference_image.rs", "main.rs"], "page|layer|photo|reference|geo|underlay"),
    11: ("Files, exports and options", ["exporter.rs", "print.rs", "options.rs", "options_dialog.rs", "page_layout.rs", "persist.rs", "autosave.rs", "main.rs"], "save|open|round_trip|export|report|print|option|layout|restart"),
    12: ("Utilities and dialogs", ["subject_info.rs", "command_palette.rs", "touch_panel.rs", "disto.rs", "disto_transport.rs", "dialog.rs", "shortcuts.rs"], "subject|palette|touch|disto|dialog|shortcut"),
}

EXCEPTIONS = {}
def classify(ids, status, note):
    for feature_id in ids:
        EXCEPTIONS[feature_id] = (status, note)

def ids(screen, *numbers):
    return [f"S{screen}-{n:03}" for n in numbers]

classify(ids(8, *range(3, 22), 36), "native-enhancement",
         "Requested native symbol management: context actions, category/search filters, previews, favorites, rename, hide/restore, double-confirmed permanent purge across pages/history, categories, DXF/JSON additions and persistent user overrides. DWG requires conversion to DXF. Bundled symbols remain unchanged.")
classify(ids(3, 12, 20) + ids(7, 41, 42, 43), "source-inactive",
         "Legacy panel/pin/classification controls are retired by the captured sidebar and optional area workflow redesign; current replacement controls are implemented.")
classify(ids(2, 19) + ids(12, 16, 17), "source-inactive",
         "Programmatic module/extended-data/layer-import hook has no standalone user entry point in the captured reference; excluded from the standalone UI scope.")
classify(ids(12, 4, 78, 79), "suite-only",
         "Requires the parent Appraisal Suite report/frame and its JavaScript bridge. This is a standalone native application; document and report export are provided through native files.")
classify(ids(4, 30) + ids(12, 13, 14, 65, 83), "hardware-deferred",
         "Native chooser, Leica GATT distance subscription, rounding and keyboard feed are implemented and simulated tests pass. Physical radio/device acceptance is deferred at the user's request; not claimed verified.")
classify(ids(5, 50, 51), "native-adaptation",
         "Keyboard-pointer behavior and configured sizes are implemented with a native arrow marker rather than the browser cursor bitmap.")
classify(ids(4, 15, 21) + ids(7, 5, 7, 9, 54, 59, 60) + ids(9, 14) + ids(10, 25, 40), "native-adaptation",
         "Functional native controls are implemented. Pick lists/cycling controls, text clipping, hover/transition behavior or swatch/pattern presentation differ from browser styling; not claimed pixel-identical.")
classify(ids(11, *range(2, 33)), "native-adaptation",
         "All option fields and restore/save/apply/cancel controls are modeled; behavior keys used by the captured original are consumed. The native dialog follows CSS rather than measured control captures. Several source options are stored-only in both apps.")
classify(ids(12, *range(30, 37)), "native-enhancement",
         "File command-search entries now invoke the corresponding native file handlers. The older browser command map left these entries unavailable.")

classify(ids(10, 36), "native-enhancement", "File > Import Layer accepts validated Sketch-Layer JSON, preserves hierarchy and geometry, remaps IDs/hosts/chains and is undoable. Layer export now includes text, symbols and custom definitions.")
classify(ids(11, 15), "native-enhancement", "Autosave writes changed full-document SKX recovery snapshots at the configured interval; startup and File > Recover Autosave restore them. Atomic writes, deduplication, error preservation and manual-save cleanup are tested.")

SPECIAL = {
    "S5-030": "0.1.4: redraw restores original area identity, name, code, full properties and layer/group; repair records survive Undo/Redo, SKX, autosave and layer import/export. Typed Enter commits share finishCommittedWall with mouse/touch/bay input. A replacement exterior boundary recreates its area and ends the run; Undo/Redo preserve the edit.",
    "S5-048": "0.1.3: exterior boundary repair detects closed rings for all wall-commit paths; unrelated closed loops do not stop the active run. Core/drawing ID-counter translation prevents a later wall from reusing the new area ID. Nine closure and twelve metadata-preservation regressions pass, including curved walls and neighboring-room isolation.",
    "S8-011": "Bounded native two-pane manager with category/search filters and independently scrolling library; footer stays visible down to tested 800x600.",
    "S8-012": "Symbol rows retain checkbox selection separately from preview focus; focused row, size/category, View and shown/selected counts are rendered.",
    "S8-013": "Always-visible live preview beside the list updates on row-name clicks, independently of bulk selection, with type/category/size/wall-snap/placed metadata.",
    "S8-014": "Row View and Enlarge open a separate responsive Symbol Viewer; Escape returns to the manager. Live preview and viewer UI regressions pass.",
    "S5-032": "Bay mirror implemented from refs/sketch-app.js::bayWindowMirror; same reflected endpoint, run state and undo/redo.",
    "S5-058": "Bay mirror implemented from refs/sketch-app.js::bayWindowMirror; B no longer displays a placeholder.",
    "S6-066": "Ordinary geometry snap uses relative steps; Area Grid Snap uses absolute selection reference/pitch, including initially off-grid selections.",
    "S6-079": "Area Grid Snap is consumed by the drag path. Label-only movement remains free under ordinary snap.",
    "S7-018": "Calcs Calculation Details button now invokes the existing Area Breakdown handler.",
    "S7-011": "Native JSON picker stages file text without applying the code library; BOM, cancellation and stale callbacks covered.",
    "S7-012": "Native JSON export validates and normalizes the draft; Apply persists the library for restart.",
    "S8-002": "Actual library artwork follows the pointer with the same wall snapping/rotation resolver as placement; blue placeholder removed.",
    "S8-034": "Actual wall-snapped artwork preview and placement are implemented.",
    "S8-044": "Placement now selects the new symbol and activates contextual controls; Symbols-layer assignment is retained.",
    "S9-006": "Dock text drag/drop is implemented; preview follows the pointer.",
    "S10-051": "Image load/render/export is implemented. Source recheck: saveImageWithSketch controls clean export inclusion, not embedding image bytes in SKX; the image is session-only in the original too.",
    "S11-055": "All pages of the cached report PDF are rendered through Windows PDF; Print/Save uses those exact cached bytes.",
    "S11-075": "SVG consumes actual drawing styles, labels, symbols, dimensions, fills, solids and hosted opening visibility.",
}

def main():
    inventory = ROOT / "refs/ui-spec/feature-inventory.md"
    lines = inventory.read_text(encoding="utf-8").splitlines()
    test_source = (ROOT / APP / "e2e.rs").read_text(encoding="utf-8")
    test_names = re.findall(r"^fn (e2e_\w+)\(", test_source, re.M)
    records = []
    revised = []
    management_names = re.findall(r"^fn (\w+)\(", (ROOT / APP / "management_tests.rs").read_text(encoding="utf-8"), re.M)
    for line in lines:
        match = re.match(r"\| (S(\d+)-\d+) \|", line)
        if not match:
            if line.startswith("Status column is "):
                line = "Status column refreshed 2026-10-29. See PARITY.md and feature-audit.json for classification, scope and evidence; implementation status is distinct from individual visual acceptance."
            revised.append(line)
            continue
        feature_id, screen = match.group(1), int(match.group(2))
        parts = re.split(r"(?<!\\)\|", line)
        title, modules, pattern = GROUPS[screen]
        paths = [APP + name for name in modules]
        if screen in (5, 7):
            paths += [CORE + "geometry.rs", CORE + "area_calc.rs"]
        if screen == 11:
            paths += [CORE + name for name in ("skx.rs", "cad_import.rs", "export_pdf.rs", "export_svg.rs", "export_dxf.rs", "export_png.rs")]
        for path in paths:
            assert (ROOT / path).is_file(), path
        status, note = EXCEPTIONS.get(feature_id, ("implemented", "Code reviewed against the captured standalone reference; regression evidence is grouped by subsystem, not an independent acceptance claim for this row."))
        note = SPECIAL.get(feature_id, note)
        records.append({"id": feature_id, "screen": screen, "feature": parts[2].strip(),
                        "status": status, "note": note, "implementation": paths,
                        "regression_file": APP + "e2e.rs",
                        "group_regressions": [name for name in test_names if re.search(pattern, name)][:8]})
        if feature_id in set(ids(8, *range(3, 22), 36) + ids(10, 36) + ids(11, 15)):
            records[-1]["regression_file"] = APP + "management_tests.rs"
            pattern = "library|purge|category|custom" if screen == 8 else "layer" if screen == 10 else "autosave|recovery"
            records[-1]["group_regressions"] = [name for name in management_names if re.search(pattern, name)]
        if screen == 8 and feature_id in EXCEPTIONS:
            records[-1]["ui_regressions"] = [name for name in test_names if "symbol_manager" in name]
        revised.append(line.rsplit("|", 2)[0] + f"| {status} |")
    assert len(records) == 634, len(records)
    assert len({r["id"] for r in records}) == 634
    assert set(EXCEPTIONS).issubset({r["id"] for r in records})
    inventory.write_text("\n".join(revised) + "\n", encoding="utf-8")
    counts = dict(Counter(r["status"] for r in records))
    result = {"audit_date": "2026-10-29", "scope": "Captured standalone Appraisal Sketch 1.1 plus the requested area workflow, symbol management, layer JSON import, recovery autosave and native Leica BLE measurement feed", "method": "Manual code review by implementation group and row exceptions; automated regression/visual evidence is recorded separately in ACCEPTANCE.md. Status is not inferred from test counts.", "counts": counts, "features": records}
    (ROOT / "refs/ui-spec/feature-audit.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    summary = ["# Feature audit — 2026-10-29", "", result["scope"] + ".", "", result["method"], "", "## Inventory coverage", "", "All 634 inventory rows are classified. The old ~495/634 and ~88% estimates are retired.", "", "| Classification | Rows |", "|---|---:|"]
    summary += [f"| {status} | {count} |" for status, count in sorted(counts.items())]
    summary += ["", "`implemented`, `native-adaptation` and `native-enhancement` describe software behavior. `hardware-deferred` includes implemented Disto software whose physical connection is unverified. `reference-unavailable`, `source-inactive` and `suite-only` rows are explicitly not claimed implemented in this standalone build.", "", "## Per-screen coverage", "", "| Screen | Rows | Implemented / adapted / enhanced | Deferred or outside captured standalone scope |", "|---|---:|---:|---:|"]
    for screen, (title, _, _) in GROUPS.items():
        rows = [r for r in records if r["screen"] == screen]
        live = sum(r["status"] in ("implemented", "native-adaptation", "native-enhancement") for r in rows)
        summary.append(f"| {screen}. {title} | {len(rows)} | {live} | {len(rows)-live} |")
    summary += ["", "## Software gaps closed in this audit", "", "- Symbol manager now keeps a live preview beside the list, uses a separate enlarged viewer and fits footer actions at tested 800x600 and larger sizes.", "- Persistent symbol manager with hide/restore, double-confirmed purge, categories, DXF/JSON import and custom symbols carried in SKX.", "- Undoable layer JSON import and complete layer exports.", "- Timed recovery autosave, startup/File recovery, atomic writes and successful-save cleanup.", "- Bay-window mirror (B): reflected geometry, drawing chain and undo/redo.", "- Calcs Calculation Details: invokes Area Breakdown.", "- File commands in command search: Save As, New, Open, Import and PNG/SVG/DXF export.", "- Selection dragging: absolute Area Grid Snap, relative ordinary geometry snap, free label movement, and correct activation threshold.", "- Placement previews: actual wall-snapped symbol geometry and text instead of a placeholder.", "- Newly placed symbols: selected with contextual editing controls.", "", "## Remaining boundaries", "", "- Disto physical-device validation is deferred at the user's request. See docs/DISTO.md.", "- Symbol additions support DXF and symbol JSON. Binary DWG requires conversion to DXF; no native DWG decoder is advertised.", "- Parent-suite embedding/report-bridge hooks and hidden programmatic commands are outside the standalone UI; see the row ledger for exact IDs.", "- Native pickers and lists, cursor artwork, dialog measurements, tooltips, animation and some swatch previews differ visually. Options and all other dialogs are not universally pixel-certified. Existing reference differences are recorded in ACCEPTANCE.md.", "- Some source-inactive ink/addendum options remain stored-only. Autosave is now a native recovery feature; reference images remain session-only.", "", "## Evidence and reproduction", "", "- Full row ledger: [feature-audit.json](feature-audit.json), including implementation paths, exception reasons and grouped regression names.", "- User feature list: [feature-inventory.md](feature-inventory.md).", "- Actual test/build/screenshot results: ../../ACCEPTANCE.md; counts are validation evidence, not a feature-completion percentage.", "- Reproduce the ledger: `python tools/audit_features.py`. Its classifications are explicit reviewed decisions, not automatic feature detection.", ""]
    (ROOT / "refs/ui-spec/PARITY.md").write_text("\n".join(summary), encoding="utf-8")
    print(json.dumps(counts, sort_keys=True))

if __name__ == "__main__":
    main()
