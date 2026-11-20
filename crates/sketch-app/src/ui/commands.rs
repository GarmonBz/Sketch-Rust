//! Complete Next-shell command registry across all 8 menu groups
//! (`docs/ui-redesign/02-architecture.md` §6, `04-feature-map.md` §2).

use crate::area_workflow;
use crate::canvas_theme::{self, CanvasTheme};
use crate::context_menu;
use crate::disto::DistoMsg;
use crate::drawing::WallShape;
use crate::format::Units;
use crate::guide::{self, GuideMsg};
use crate::input::KeyCmd;
use crate::options_dialog::OptMsg;
use crate::page_layout::LayoutMsg;
use crate::popup_menu::{self, PopupMsg};
use crate::select_menu::SelectKind;
use crate::shapes::ShapesMsg;
use crate::shortcuts::ShortcutsMsg;
use crate::statusbar::{PitchOption, StatusMsg};
use crate::theme::{self, InterfaceTheme};
use crate::transform::Axis;
use crate::ui::tokens::Density;
use crate::ui::Msg as UiMsg;
use crate::view_state::{Mode, RailTool, PITCHES};
use crate::{dock, App, Message};

/// Top-level command groups shown in the Menu and searchable via Ctrl+K.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Group {
    File,
    Edit,
    View,
    Areas,
    Insert,
    Image,
    Tools,
    Help,
}

impl Group {
    /// The group's name, shown beside each command in Menu search results.
    pub fn label(self) -> &'static str {
        match self {
            Group::File => "File",
            Group::Edit => "Edit",
            Group::View => "View",
            Group::Areas => "Areas",
            Group::Insert => "Insert",
            Group::Image => "Image",
            Group::Tools => "Tools",
            Group::Help => "Help",
        }
    }
}

/// A single registered command shared by the Menu and Command Search (`Ctrl+K`).
#[derive(Clone, Copy)]
pub struct Command {
    pub id: &'static str,
    pub label: &'static str,
    pub group: Group,
    pub shortcut: &'static str,
    pub enabled: fn(&App) -> bool,
    pub checked: Option<fn(&App) -> bool>,
    pub run: fn(&App) -> Message,
}

fn always(_app: &App) -> bool {
    true
}

fn has_selection(app: &App) -> bool {
    !app.selection.is_empty()
}

fn has_geometry(app: &App) -> bool {
    !app.drawing.lines.is_empty() || !app.drawing.polygons.is_empty()
}

fn has_photo(app: &App) -> bool {
    app.bg_image.bytes.is_some()
}

fn has_pages_multi(app: &App) -> bool {
    app.pages.len() > 1
}

fn has_recovery(app: &App) -> bool {
    app.autosave.recovery.is_some()
}

fn pitch_opt(index: usize) -> PitchOption {
    let (label, value) = PITCHES[index];
    PitchOption { label, value }
}

fn pitch_matches(app: &App, index: usize) -> bool {
    (app.view.area_snap_size - PITCHES[index].1).abs() < 1e-4
}

fn clone_units_with_metric(app: &App, metric: bool) -> Units {
    let mut u = app.drawing.options.units.clone();
    u.metric = metric;
    if metric && u.area_label_suffix == "sf" {
        u.area_label_suffix = "m²".to_owned();
    } else if !metric && u.area_label_suffix == "m²" {
        u.area_label_suffix = "sf".to_owned();
    }
    u
}

static COMMANDS: &[Command] = &[
    // --- File ---
    Command {
        id: "start-screen",
        label: "Start screen...",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Ui(crate::ui::Msg::Start(crate::ui::start::Msg::Open)),
    },
    Command {
        id: "recent",
        label: "Recent sketches...",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Ui(crate::ui::Msg::Start(crate::ui::start::Msg::Open)),
    },
    Command {
        id: "nudge-up",
        label: "Nudge selection up",
        group: Group::Edit,
        shortcut: "Up",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Key(KeyCmd::Nudge(0.0, 1.0)),
    },
    Command {
        id: "nudge-down",
        label: "Nudge selection down",
        group: Group::Edit,
        shortcut: "Down",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Key(KeyCmd::Nudge(0.0, -1.0)),
    },
    Command {
        id: "nudge-left",
        label: "Nudge selection left",
        group: Group::Edit,
        shortcut: "Left",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Key(KeyCmd::Nudge(-1.0, 0.0)),
    },
    Command {
        id: "nudge-right",
        label: "Nudge selection right",
        group: Group::Edit,
        shortcut: "Right",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Key(KeyCmd::Nudge(1.0, 0.0)),
    },
    Command {
        id: "underlay-previous",
        label: "Use previous page as underlay",
        group: Group::View,
        shortcut: "",
        enabled: |a| a.pages.current > 0,
        checked: None,
        run: |a| {
            Message::PagesNav(crate::pages_nav::NavMsg::Ghost(
                a.pages.current.saturating_sub(1),
            ))
        },
    },
    Command {
        id: "underlay-clear",
        label: "Clear underlay",
        group: Group::View,
        shortcut: "",
        enabled: |a| a.pages.underlay.is_some(),
        checked: None,
        run: |_| Message::PagesNav(crate::pages_nav::NavMsg::Underlay(None)),
    },
    Command {
        id: "label-defaults",
        label: "Label defaults and area tools...",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| {
            Message::AreaWorkflow(crate::area_workflow::Msg::Advanced(true))
        },
    },
    Command {
        id: "new",
        label: "New sketch",
        group: Group::File,
        shortcut: "Ctrl+N",
        enabled: always,
        checked: None,
        run: |_| Message::New,
    },
    Command {
        id: "open",
        label: "Open...",
        group: Group::File,
        shortcut: "Ctrl+O",
        enabled: always,
        checked: None,
        run: |_| Message::OpenFile,
    },
    Command {
        id: "save",
        label: "Save",
        group: Group::File,
        shortcut: "F2",
        enabled: always,
        checked: None,
        run: |_| Message::SaveFile,
    },
    Command {
        id: "save-as",
        label: "Save As...",
        group: Group::File,
        shortcut: "Ctrl+Shift+S",
        enabled: always,
        checked: None,
        run: |_| Message::SaveFileAs,
    },
    Command {
        id: "import",
        label: "Import sketch...",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ImportSketch,
    },
    Command {
        id: "import-cad",
        label: "Import CAD...",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ImportCad,
    },
    Command {
        id: "import-layer",
        label: "Import layer...",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ImportLayer,
    },
    Command {
        id: "export-png",
        label: "Export as PNG",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ExportPng,
    },
    Command {
        id: "export-svg",
        label: "Export as SVG",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ExportSvg,
    },
    Command {
        id: "export-dxf",
        label: "Export as DXF",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ExportDxf,
    },
    Command {
        id: "export-pdf",
        label: "Export as PDF",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::ExportPdf,
    },
    Command {
        id: "print",
        label: "Print",
        group: Group::File,
        shortcut: "Ctrl+P",
        enabled: always,
        checked: None,
        run: |_| Message::Print,
    },
    Command {
        id: "print-preview",
        label: "Print preview",
        group: Group::File,
        shortcut: "Ctrl+Shift+P",
        enabled: always,
        checked: None,
        run: |_| Message::PrintPreviewOpen,
    },
    Command {
        id: "page-layout",
        label: "Page layout",
        group: Group::File,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::PageLayout(LayoutMsg::Open),
    },
    Command {
        id: "recover-autosave",
        label: "Recover autosave...",
        group: Group::File,
        shortcut: "",
        enabled: has_recovery,
        checked: None,
        run: |_| Message::RecoveryOpen,
    },
    Command {
        id: "close-window",
        label: "Close window",
        group: Group::File,
        shortcut: "Ctrl+W",
        enabled: always,
        checked: None,
        run: |_| Message::Close,
    },
    // --- Edit ---
    Command {
        id: "undo",
        label: "Undo",
        group: Group::Edit,
        shortcut: "Ctrl+Z",
        enabled: |a| a.undo.can_undo(),
        checked: None,
        run: |_| Message::Undo,
    },
    Command {
        id: "redo",
        label: "Redo",
        group: Group::Edit,
        shortcut: "Ctrl+Y",
        enabled: |a| a.undo.can_redo(),
        checked: None,
        run: |_| Message::Redo,
    },
    Command {
        id: "undo-history",
        label: "Undo history",
        group: Group::Edit,
        shortcut: "",
        enabled: |a| a.undo.can_undo() || a.undo.can_redo(),
        checked: Some(|a| a.history.open.is_some()),
        run: |_| Message::UndoHistory,
    },
    Command {
        id: "cut",
        label: "Cut",
        group: Group::Edit,
        shortcut: "Ctrl+X",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Cut,
    },
    Command {
        id: "copy",
        label: "Copy",
        group: Group::Edit,
        shortcut: "Ctrl+C",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Copy,
    },
    Command {
        id: "paste",
        label: "Paste",
        group: Group::Edit,
        shortcut: "Ctrl+V",
        enabled: |a| a.clipboard.is_some(),
        checked: None,
        run: |_| Message::Paste,
    },
    Command {
        id: "delete-selected",
        label: "Delete selection",
        group: Group::Edit,
        shortcut: "Del",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Delete,
    },
    Command {
        id: "select-all",
        label: "Select all",
        group: Group::Edit,
        shortcut: "Ctrl+A",
        enabled: always,
        checked: None,
        run: |_| Message::SelectAll,
    },
    Command {
        id: "select-by-type",
        label: "Select by type",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Ui(crate::ui::Msg::OpenSelectByType),
    },
    Command {
        id: "select-areas",
        label: "Select all areas",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::SelectByType(SelectKind::Areas),
    },
    Command {
        id: "select-walls",
        label: "Select freeform lines",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::SelectByType(SelectKind::Freeform),
    },
    Command {
        id: "select-text",
        label: "Select all text",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::SelectByType(SelectKind::Text),
    },
    Command {
        id: "select-dimensions",
        label: "Select all dimensions",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::SelectByType(SelectKind::Dimensions),
    },
    Command {
        id: "select-symbols",
        label: "Select all symbols",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::SelectByType(SelectKind::Symbols),
    },
    Command {
        id: "rotate-selected-cw",
        label: "Rotate selection +90°",
        group: Group::Edit,
        shortcut: "Shift+R",
        enabled: has_selection,
        checked: None,
        run: |_| Message::RotateSelected(90.0),
    },
    Command {
        id: "rotate-selected-ccw",
        label: "Rotate selection -90°",
        group: Group::Edit,
        shortcut: "Ctrl+Shift+R",
        enabled: has_selection,
        checked: None,
        run: |_| Message::RotateSelected(-90.0),
    },
    Command {
        id: "orient-h",
        label: "Orient selection horizontal",
        group: Group::Edit,
        shortcut: "H",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Key(KeyCmd::OrientH),
    },
    Command {
        id: "orient-v",
        label: "Orient selection vertical",
        group: Group::Edit,
        shortcut: "V",
        enabled: has_selection,
        checked: None,
        run: |_| Message::Key(KeyCmd::OrientV),
    },
    Command {
        id: "scale-up",
        label: "Scale selection up",
        group: Group::Edit,
        shortcut: "+",
        enabled: has_selection,
        checked: None,
        run: |_| Message::ScaleSelected(1.1),
    },
    Command {
        id: "scale-down",
        label: "Scale selection down",
        group: Group::Edit,
        shortcut: "-",
        enabled: has_selection,
        checked: None,
        run: |_| Message::ScaleSelected(0.9),
    },
    Command {
        id: "flip-sketch",
        label: "Flip sketch",
        group: Group::Edit,
        shortcut: "",
        enabled: has_geometry,
        checked: None,
        run: |_| Message::PaletteMenu(popup_menu::Kind::Flip),
    },
    Command {
        id: "flip-horizontal",
        label: "Flip sketch horizontally",
        group: Group::Edit,
        shortcut: "Ctrl+H",
        enabled: has_geometry,
        checked: None,
        run: |_| Message::PopupMenu(PopupMsg::Flip(Axis::Horizontal)),
    },
    Command {
        id: "flip-vertical",
        label: "Flip sketch vertically",
        group: Group::Edit,
        shortcut: "Ctrl+Shift+H",
        enabled: has_geometry,
        checked: None,
        run: |_| Message::PopupMenu(PopupMsg::Flip(Axis::Vertical)),
    },
    Command {
        id: "rotate-sketch",
        label: "Rotate sketch +90°",
        group: Group::Edit,
        shortcut: "R",
        enabled: always,
        checked: None,
        run: |_| Message::RotateSketch,
    },
    Command {
        id: "rotate-sketch-ccw",
        label: "Rotate sketch -90°",
        group: Group::Edit,
        shortcut: "L",
        enabled: always,
        checked: None,
        run: |_| Message::RotateSketchBy(-90.0),
    },
    Command {
        id: "layer-bring-front",
        label: "Bring selection to front",
        group: Group::Edit,
        shortcut: "",
        enabled: has_selection,
        checked: None,
        run: |_| Message::LayerBringToFront,
    },
    Command {
        id: "layer-send-back",
        label: "Send selection to back",
        group: Group::Edit,
        shortcut: "",
        enabled: has_selection,
        checked: None,
        run: |_| Message::LayerSendToBack,
    },
    Command {
        id: "curve-mode",
        label: "Toggle curve step mode",
        group: Group::Edit,
        shortcut: "C",
        enabled: always,
        checked: None,
        run: |_| Message::Key(KeyCmd::CurveMode),
    },
    Command {
        id: "auto-close-loop",
        label: "Auto-close wall loop",
        group: Group::Edit,
        shortcut: "Ctrl+F3",
        enabled: |a| a.drawing.pen_down && a.view.mode.is_drawing(),
        checked: None,
        run: |_| Message::Key(KeyCmd::AutoClose),
    },
    Command {
        id: "bay-mirror",
        label: "Mirror bay / bump-out",
        group: Group::Edit,
        shortcut: "M",
        enabled: always,
        checked: None,
        run: |_| Message::Key(KeyCmd::BayMirror),
    },
    Command {
        id: "jump-endpoint",
        label: "Jump to line endpoint",
        group: Group::Edit,
        shortcut: "J",
        enabled: always,
        checked: None,
        run: |_| Message::Key(KeyCmd::JumpEndpoint),
    },
    // --- View ---
    Command {
        id: "zoom-in",
        label: "Zoom in",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Status(StatusMsg::ZoomIn),
    },
    Command {
        id: "zoom-out",
        label: "Zoom out",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Status(StatusMsg::ZoomOut),
    },
    Command {
        id: "center-sketch",
        label: "Center sketch",
        group: Group::View,
        shortcut: "Home",
        enabled: always,
        checked: None,
        run: |_| Message::Status(StatusMsg::CenterSketch),
    },
    Command {
        id: "center-fit",
        label: "Zoom to fit",
        group: Group::View,
        shortcut: "Ctrl+0",
        enabled: always,
        checked: None,
        run: |_| Message::Status(StatusMsg::CenterFit),
    },
    Command {
        id: "zoom-fence",
        label: "Zoom fence",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::ZoomFence),
        run: |_| Message::Status(StatusMsg::ZoomFence),
    },
    Command {
        id: "toggle-grid",
        label: "Show grid",
        group: Group::View,
        shortcut: "G",
        enabled: always,
        checked: Some(|a| a.view.grid),
        run: |_| Message::Status(StatusMsg::ToggleGrid),
    },
    Command {
        id: "toggle-snap",
        label: "Grid & angle snap",
        group: Group::View,
        shortcut: "S",
        enabled: always,
        checked: Some(|a| a.view.snap),
        run: |_| Message::Status(StatusMsg::ToggleSnap),
    },
    Command {
        id: "toggle-area-snap",
        label: "Area grid snap",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.area_snap),
        run: |_| Message::Status(StatusMsg::ToggleAreaSnap),
    },
    Command {
        id: "pitch-1ft",
        label: "Snap pitch: 1'",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| pitch_matches(a, 0)),
        run: |_| Message::Status(StatusMsg::Pitch(pitch_opt(0))),
    },
    Command {
        id: "pitch-half",
        label: "Snap pitch: 1/2'",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| pitch_matches(a, 1)),
        run: |_| Message::Status(StatusMsg::Pitch(pitch_opt(1))),
    },
    Command {
        id: "pitch-quarter",
        label: "Snap pitch: 1/4'",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| pitch_matches(a, 2)),
        run: |_| Message::Status(StatusMsg::Pitch(pitch_opt(2))),
    },
    Command {
        id: "pitch-1in",
        label: "Snap pitch: 1\"",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| pitch_matches(a, 3)),
        run: |_| Message::Status(StatusMsg::Pitch(pitch_opt(3))),
    },
    Command {
        id: "cycle-dimensions",
        label: "Cycle dimension labels",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.dimensions),
        run: |_| Message::Status(StatusMsg::CycleDimensions),
    },
    Command {
        id: "filter",
        label: "Show in sketch filter",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filter_open),
        run: |_| Message::Status(StatusMsg::ToggleFilter),
    },
    Command {
        id: "filter-areas",
        label: "Filter: Area fills",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filters[0]),
        run: |_| Message::Status(StatusMsg::Filter(0)),
    },
    Command {
        id: "filter-exterior",
        label: "Filter: Exterior walls",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filters[1]),
        run: |_| Message::Status(StatusMsg::Filter(1)),
    },
    Command {
        id: "filter-interior",
        label: "Filter: Interior lines",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filters[2]),
        run: |_| Message::Status(StatusMsg::Filter(2)),
    },
    Command {
        id: "filter-dimensions",
        label: "Filter: Dimensions",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filters[3]),
        run: |_| Message::Status(StatusMsg::Filter(3)),
    },
    Command {
        id: "filter-labels",
        label: "Filter: Text labels",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filters[4]),
        run: |_| Message::Status(StatusMsg::Filter(4)),
    },
    Command {
        id: "filter-symbols",
        label: "Filter: Symbols",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.filters[5]),
        run: |_| Message::Status(StatusMsg::Filter(5)),
    },
    Command {
        id: "toggle-north",
        label: "Edit north angle",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Status(StatusMsg::North),
    },
    Command {
        id: "mini-map",
        label: "Mini map",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.mini_map.is_some()),
        run: |_| Message::ToggleMiniMap,
    },
    Command {
        id: "toggle-calcs",
        label: "Show totals",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.ui.ledger_open),
        run: |_| Message::ToggleCalcs,
    },
    Command {
        id: "page-prev",
        label: "Previous page",
        group: Group::View,
        shortcut: "",
        enabled: |a| a.pages.can_go_prev(),
        checked: None,
        run: |_| Message::Status(StatusMsg::PagePrev),
    },
    Command {
        id: "page-next",
        label: "Next page",
        group: Group::View,
        shortcut: "",
        enabled: |a| !a.pages.is_last(),
        checked: None,
        run: |_| Message::Status(StatusMsg::PageNext),
    },
    Command {
        id: "page-first",
        label: "First page",
        group: Group::View,
        shortcut: "",
        enabled: |a| a.pages.can_go_prev(),
        checked: None,
        run: |_| Message::Status(StatusMsg::PageFirst),
    },
    Command {
        id: "page-last",
        label: "Last page",
        group: Group::View,
        shortcut: "",
        enabled: |a| !a.pages.is_last(),
        checked: None,
        run: |_| Message::Status(StatusMsg::PageLast),
    },
    Command {
        id: "page-add",
        label: "Add page",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Status(StatusMsg::PageAdd),
    },
    Command {
        id: "page-delete",
        label: "Delete page",
        group: Group::View,
        shortcut: "",
        enabled: has_pages_multi,
        checked: None,
        run: |_| Message::Status(StatusMsg::PageDelete),
    },
    Command {
        id: "toggle-canvas-background",
        label: "Canvas background",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::PaletteMenu(popup_menu::Kind::CanvasBackground),
    },
    Command {
        id: "canvas-bg-light",
        label: "Drawing background: White",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|_| canvas_theme::current() == CanvasTheme::Light),
        run: |_| Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Light)),
    },
    Command {
        id: "canvas-bg-dark",
        label: "Drawing background: Dark",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|_| canvas_theme::current() == CanvasTheme::Dark),
        run: |_| Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Dark)),
    },
    Command {
        id: "canvas-bg-blue",
        label: "Drawing background: Blue",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|_| canvas_theme::current() == CanvasTheme::Blue),
        run: |_| Message::PopupMenu(PopupMsg::Canvas(CanvasTheme::Blue)),
    },
    Command {
        id: "interface-theme",
        label: "Interface theme",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::PaletteMenu(popup_menu::Kind::InterfaceTheme),
    },
    Command {
        id: "theme-dark",
        label: "Interface theme: Dark",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|_| theme::interface_theme() == InterfaceTheme::Dark),
        run: |_| Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Dark)),
    },
    Command {
        id: "theme-blue",
        label: "Interface theme: Blue",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|_| theme::interface_theme() == InterfaceTheme::Blue),
        run: |_| Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Blue)),
    },
    Command {
        id: "theme-light",
        label: "Interface theme: Light",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|_| theme::interface_theme() == InterfaceTheme::Light),
        run: |_| Message::PopupMenu(PopupMsg::Interface(InterfaceTheme::Light)),
    },
    Command {
        id: "units-imperial",
        label: "Units: Imperial (ft / sf)",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| !a.drawing.options.units.metric),
        run: |a| Message::Units(clone_units_with_metric(a, false)),
    },
    Command {
        id: "units-metric",
        label: "Units: Metric (m / m²)",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.drawing.options.units.metric),
        run: |a| Message::Units(clone_units_with_metric(a, true)),
    },
    Command {
        id: "density-comfortable",
        label: "Density: Comfortable (mouse)",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.ui.density == Density::Comfortable),
        run: |_| Message::Ui(UiMsg::SetDensity(Density::Comfortable)),
    },
    Command {
        id: "density-touch",
        label: "Density: Touch (tablet)",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.ui.density == Density::Touch),
        run: |_| Message::Ui(UiMsg::SetDensity(Density::Touch)),
    },
    Command {
        id: "touch-panel",
        label: "Pad",
        group: Group::View,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.touch_panel_open),
        run: |_| Message::TouchPanelToggle,
    },
    // --- Areas ---
    Command {
        id: "define-area",
        label: "Area tool",
        group: Group::Areas,
        shortcut: "A",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::DefineArea),
        run: |_| Message::Rail(RailTool::DefineArea),
    },
    Command {
        id: "area-find-undefined",
        label: "Find undefined areas",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::AreaWorkflow(area_workflow::Msg::FindUndefined),
    },
    Command {
        id: "area-detect",
        label: "Auto-detect closed areas",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::AreaWorkflow(area_workflow::Msg::Detect),
    },
    Command {
        id: "area-redefine",
        label: "Redefine selected area",
        group: Group::Areas,
        shortcut: "",
        enabled: |a| a.selection.polys.len() == 1,
        checked: None,
        run: |_| Message::AreaWorkflow(area_workflow::Msg::Redefine),
    },
    Command {
        id: "area-clone",
        label: "Clone area definition",
        group: Group::Areas,
        shortcut: "",
        enabled: |a| a.selection.polys.len() == 1,
        checked: None,
        run: |_| Message::AreaWorkflow(area_workflow::Msg::Clone),
    },
    Command {
        id: "area-reopen",
        label: "Reopen area workflow menu",
        group: Group::Areas,
        shortcut: "",
        enabled: |a| a.selection.polys.len() == 1,
        checked: None,
        run: |_| Message::AreaWorkflow(area_workflow::Msg::Reopen),
    },
    Command {
        id: "area-properties",
        label: "Area properties",
        group: Group::Areas,
        shortcut: "",
        enabled: |a| !a.selection.polys.is_empty(),
        checked: None,
        run: |_| Message::AreaProps,
    },
    Command {
        id: "area-attributes",
        label: "Area attributes",
        group: Group::Areas,
        shortcut: "",
        enabled: |a| !a.selection.polys.is_empty(),
        checked: None,
        run: |_| Message::OpenAreaAttributes(true),
    },
    Command {
        id: "area-make-negative",
        label: "Subtract / toggle negative area",
        group: Group::Areas,
        shortcut: "",
        enabled: |a| !a.selection.polys.is_empty(),
        checked: None,
        run: |_| Message::ContextAction(context_menu::Action::MakeNegative),
    },
    Command {
        id: "area-auto-subtract",
        label: "Auto-subtract overlapping areas",
        group: Group::Areas,
        shortcut: "",
        enabled: has_geometry,
        checked: None,
        run: |_| Message::ContextAction(context_menu::Action::AutoSubtract),
    },
    Command {
        id: "area-breakdown",
        label: "Area breakdown",
        group: Group::Areas,
        shortcut: "B",
        enabled: always,
        checked: None,
        run: |_| Message::OpenBreakdown,
    },
    Command {
        id: "area-code-settings",
        label: "Area types",
        group: Group::Areas,
        shortcut: "F4",
        enabled: always,
        checked: None,
        run: |_| Message::OpenCodesEditor,
    },
    Command {
        id: "area-new-code",
        label: "New area type...",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::OpenNewCode,
    },
    Command {
        id: "area-edit-definitions",
        label: "Edit area types...",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::OpenDefEditor,
    },
    Command {
        id: "wall-solids",
        label: "Draw walls as solids",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.drawing.options.wall_solids),
        run: |a| Message::WallSolids(!a.drawing.options.wall_solids),
    },
    Command {
        id: "define-first",
        label: "Require area type before drawing",
        group: Group::Areas,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.drawing.options.define_first),
        run: |a| Message::DefineFirst(!a.drawing.options.define_first),
    },
    // --- Insert ---
    Command {
        id: "tool-select",
        label: "Select tool",
        group: Group::Insert,
        shortcut: "V / Esc",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::Select),
        run: |_| Message::Rail(RailTool::Select),
    },
    Command {
        id: "tool-pan",
        label: "Pan tool",
        group: Group::Insert,
        shortcut: "Space",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::Pan),
        run: |_| Message::Rail(RailTool::Pan),
    },
    Command {
        id: "tool-draw-exterior",
        label: "Exterior wall tool",
        group: Group::Insert,
        shortcut: "W / E",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::DrawExterior),
        run: |_| Message::Rail(RailTool::DrawExterior),
    },
    Command {
        id: "tool-draw-interior",
        label: "Interior wall tool",
        group: Group::Insert,
        shortcut: "I",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::DrawInterior),
        run: |_| Message::Rail(RailTool::DrawInterior),
    },
    Command {
        id: "tool-insert-vertex",
        label: "Insert vertex tool",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::InsertVertex),
        run: |_| Message::Rail(RailTool::InsertVertex),
    },
    Command {
        id: "tool-freeform",
        label: "Freeform draw tool",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::Freeform),
        run: |_| Message::Rail(RailTool::Freeform),
    },
    Command {
        id: "tool-erase",
        label: "Erase tool",
        group: Group::Edit,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::Erase),
        run: |_| Message::Rail(RailTool::Erase),
    },
    Command {
        id: "tool-curve",
        label: "Curve wall tool",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::Curve),
        run: |_| Message::Rail(RailTool::Curve),
    },
    Command {
        id: "tool-measure",
        label: "Measure tool",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::Measure),
        run: |_| Message::Rail(RailTool::Measure),
    },
    Command {
        id: "tool-dimension-chain",
        label: "Dimension chain tool",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.view.mode == Mode::ChainDim),
        run: |_| Message::Rail(RailTool::ChainDim),
    },
    Command {
        id: "tool-symbols",
        label: "Insert symbols",
        group: Group::Insert,
        shortcut: "F6",
        enabled: always,
        checked: Some(|a| a.dock.open && a.dock.active == dock::DockTab::Symbols),
        run: |_| Message::Rail(RailTool::Symbols),
    },
    Command {
        id: "tool-text",
        label: "Insert room labels",
        group: Group::Insert,
        shortcut: "F5",
        enabled: always,
        checked: Some(|a| a.dock.open && a.dock.active == dock::DockTab::Text),
        run: |_| Message::Rail(RailTool::Text),
    },
    Command {
        id: "tool-room-shapes",
        label: "Room shapes...",
        group: Group::Insert,
        shortcut: "F7",
        enabled: always,
        checked: None,
        run: |_| Message::RoomShapeOpen,
    },
    Command {
        id: "shape-rectangle",
        label: "Insert rectangle shape...",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Shapes(ShapesMsg::Choose(WallShape::Rectangle)),
    },
    Command {
        id: "shape-circle",
        label: "Insert circle shape...",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Shapes(ShapesMsg::Choose(WallShape::Circle)),
    },
    Command {
        id: "shape-polygon",
        label: "Insert polygon shape...",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Shapes(ShapesMsg::Choose(WallShape::Polygon)),
    },
    Command {
        id: "draw-color",
        label: "Draw color",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::OpenColorDialog,
    },
    Command {
        id: "edit-text-library",
        label: "Room labels list...",
        group: Group::Insert,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Dock(dock::DockMsg::EditTextLibrary),
    },
    // --- Image ---
    Command {
        id: "photo-load",
        label: "Load trace image...",
        group: Group::Image,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::PhotoLoad,
    },
    Command {
        id: "photo-show",
        label: "Show trace image",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: Some(|a| has_photo(a) && a.bg_image.visible),
        run: |_| Message::PhotoShow,
    },
    Command {
        id: "photo-hide",
        label: "Hide trace image",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoHide,
    },
    Command {
        id: "photo-remove",
        label: "Remove trace image",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoRemove,
    },
    Command {
        id: "photo-scale",
        label: "Set image scale",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: Some(|a| a.bg_image.scale_mode),
        run: |_| Message::PhotoScaleStart,
    },
    Command {
        id: "photo-smaller",
        label: "Shrink trace image",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoSmaller,
    },
    Command {
        id: "photo-larger",
        label: "Enlarge trace image",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoLarger,
    },
    Command {
        id: "photo-rotate-left",
        label: "Rotate image left",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoRotateLeft,
    },
    Command {
        id: "photo-rotate-right",
        label: "Rotate image right",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoRotateRight,
    },
    Command {
        id: "photo-flip-h",
        label: "Flip image horizontally",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoFlipH,
    },
    Command {
        id: "photo-flip-v",
        label: "Flip image vertically",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: None,
        run: |_| Message::PhotoFlipV,
    },
    Command {
        id: "photo-edit-mode",
        label: "Image edit mode",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: Some(|a| a.bg_image.edit_mode),
        run: |_| Message::PhotoEditModeToggle,
    },
    Command {
        id: "photo-adjust",
        label: "Adjust trace image (intensity, size, rotation)",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: Some(|a| a.ui.inspector_image_adjust || a.bg_image.edit_mode),
        run: |_| Message::Ui(crate::ui::Msg::InspectorAdjustImage),
    },
    Command {
        id: "photo-orient",
        label: "Orient guide",
        group: Group::Image,
        shortcut: "",
        enabled: has_photo,
        checked: Some(|a| a.bg_image.orient_mode),
        run: |_| Message::PhotoOrientStart,
    },
    Command {
        id: "geo-load-map",
        label: "Geo: Load map area",
        group: Group::Image,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::GeoLoadMap,
    },
    Command {
        id: "geo-coordinate",
        label: "Geo: Jump to coordinates...",
        group: Group::Image,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::GeoCoordinate,
    },
    Command {
        id: "geo-address",
        label: "Geo: Search address...",
        group: Group::Image,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::GeoAddress,
    },
    // --- Tools ---
    Command {
        id: "disto-connect",
        label: "Connect Bluetooth device",
        group: Group::Tools,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Disto(DistoMsg::Connect),
    },
    Command {
        id: "subject-info",
        label: "Property details",
        group: Group::Tools,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::OpenSubjectInfo,
    },
    Command {
        id: "options",
        label: "Options",
        group: Group::Tools,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Options(OptMsg::Open),
    },
    Command {
        id: "symbol-library",
        label: "Symbol library...",
        group: Group::Tools,
        shortcut: "",
        enabled: always,
        checked: Some(|a| a.ui.symbol_library.open),
        run: |_| {
            Message::Ui(crate::ui::Msg::SymbolLibrary(
                crate::ui::symbol_library::Msg::Open(None),
            ))
        },
    },
    Command {
        id: "make-symbol-from-selection",
        label: "Make symbol from selection",
        group: Group::Tools,
        shortcut: "",
        enabled: |a| {
            a.symbol_edit_session.is_none()
                && crate::symbol_shape::selection_convertible(&a.drawing, &a.selection)
        },
        checked: None,
        run: |_| {
            Message::Ui(crate::ui::Msg::SymbolLibrary(
                crate::ui::symbol_library::Msg::OpenMakeFromSelection,
            ))
        },
    },
    // --- Help ---
    Command {
        id: "shortcuts",
        label: "Keyboard shortcuts",
        group: Group::Help,
        shortcut: "F1",
        enabled: always,
        checked: Some(|a| a.shortcuts.open),
        run: |_| Message::Shortcuts(ShortcutsMsg::Open),
    },
    Command {
        id: "user-guide",
        label: "User guide",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Guide(GuideMsg::Open(0)),
    },
    Command {
        id: "appraise-it-reports",
        label: "Appraise-It reports",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Guide(GuideMsg::Open(guide::INTEGRATION_TOPIC)),
    },
    #[cfg(windows)]
    Command {
        id: "appraise-it-check",
        label: "Check Appraise-It report",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Guide(GuideMsg::CheckReport),
    },
    #[cfg(windows)]
    Command {
        id: "appraise-it-preview",
        label: "Preview sketch for Appraise-It",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Guide(GuideMsg::PreviewSketch),
    },
    #[cfg(windows)]
    Command {
        id: "appraise-it-send",
        label: "Send sketch to Appraise-It",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Guide(GuideMsg::SendSketch),
    },
    Command {
        id: "email",
        label: "Email",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::Email,
    },
    Command {
        id: "about",
        label: "About",
        group: Group::Help,
        shortcut: "",
        enabled: always,
        checked: None,
        run: |_| Message::About,
    },
];

/// Every registered command across all 8 groups.
pub fn all() -> &'static [Command] {
    COMMANDS
}

/// The icon a command shows in the Menu and in search: `(Phosphor name,
/// fill weight)`. The names follow the classic ribbon where it had one (Cut
/// scissors, Delete x, Subject Info house, Area tool bounding box), and every
/// name must be embedded in `icons.rs` (checked by
/// `every_command_has_an_embedded_icon`).
pub fn icon_for(id: &str) -> Option<(&'static str, bool)> {
    Some(match id {
        // File
        "start-screen" | "page-layout" => ("layout", false),
        "recent" | "undo-history" => ("clock-counter-clockwise", true),
        "new" | "page-add" => ("file-plus", false),
        "open" => ("folder-open", true),
        "save" | "save-as" => ("floppy-disk", true),
        "import" => ("arrow-square-in", true),
        "import-layer" => ("arrow-square-in", false),
        "import-cad" | "export-dxf" => ("blueprint", true),
        "export-png" => ("file-image", false),
        "export-svg" => ("arrow-square-out", true),
        "export-pdf" => ("file-pdf", true),
        "print" => ("printer", true),
        "print-preview" => ("printer", false),
        "recover-autosave" => ("clock-counter-clockwise", false),
        "close-window" => ("power", true),
        // Edit
        "nudge-up" => ("arrow-up", false),
        "nudge-down" => ("arrow-down", false),
        "nudge-left" => ("arrow-left", false),
        "nudge-right" => ("arrow-right", false),
        "undo" => ("arrow-u-up-left", false),
        "redo" => ("arrow-u-up-right", false),
        "cut" => ("scissors", false),
        "copy" => ("copy", false),
        "paste" | "appraise-it-reports" => ("clipboard-text", false),
        "delete-selected" => ("trash", false),
        "select-all" => ("selection-all", false),
        "select-by-type" | "area-detect" => ("intersect", true),
        "select-areas" | "define-area" => ("bounding-box", false),
        "select-walls" | "tool-insert-vertex" => ("line-segments", false),
        "select-text" | "tool-text" => ("text-t", false),
        "select-dimensions" | "cycle-dimensions" | "tool-dimension-chain" => {
            ("arrows-horizontal", false)
        }
        "select-symbols" | "tool-symbols" | "symbol-library" => ("couch", false),
        "rotate-selected-cw" | "photo-rotate-right" => ("arrow-clockwise", false),
        "rotate-selected-ccw" | "photo-rotate-left" => ("arrow-counter-clockwise", false),
        "orient-h" | "flip-sketch" | "flip-horizontal" | "photo-flip-h" => {
            ("arrows-left-right", false)
        }
        "orient-v" | "flip-vertical" | "photo-flip-v" => ("arrows-down-up", false),
        "scale-up" | "photo-larger" => ("arrows-out", false),
        "scale-down" | "photo-smaller" => ("arrows-in", false),
        "rotate-sketch" => ("arrow-clockwise", true),
        "rotate-sketch-ccw" => ("arrow-counter-clockwise", true),
        "layer-bring-front" => ("arrow-line-up", false),
        "layer-send-back" => ("arrow-line-down", false),
        "curve-mode" | "tool-curve" => ("bezier-curve", false),
        "auto-close-loop" | "tool-draw-exterior" | "shape-polygon" => ("polygon", false),
        "bay-mirror" => ("vector-two", true),
        "jump-endpoint" | "center-sketch" => ("crosshair", false),
        // View
        "underlay-previous" => ("files", false),
        "underlay-clear" | "photo-remove" => ("file-x", false),
        "zoom-in" | "zoom-out" => ("magnifying-glass", false),
        "center-fit" => ("corners-out", false),
        "zoom-fence" => ("selection-background", false),
        "toggle-grid" => ("grid-four", false),
        "toggle-snap" | "toggle-area-snap" => ("magnet", false),
        "pitch-1ft" | "pitch-half" | "pitch-quarter" | "pitch-1in" | "units-imperial" | "units-metric"
        | "tool-measure" | "photo-scale" => ("ruler", false),
        "filter" | "filter-areas" | "filter-exterior" | "filter-interior"
        | "filter-dimensions" | "filter-labels" | "filter-symbols" => ("funnel", false),
        "toggle-north" | "photo-orient" => ("compass", false),
        "mini-map" | "geo-load-map" => ("map-trifold", false),
        "toggle-calcs" | "area-breakdown" => ("calculator", true),
        "page-prev" => ("caret-line-left", true),
        "page-next" => ("caret-line-right", true),
        "page-first" => ("caret-left", true),
        "page-last" => ("caret-right", true),
        "page-delete" => ("file-minus", false),
        "toggle-canvas-background" | "canvas-bg-light" | "canvas-bg-dark" | "canvas-bg-blue"
        | "theme-blue" => ("drop", false),
        "interface-theme" => ("circle-half-tilt", false),
        "theme-dark" => ("moon", false),
        "theme-light" => ("sun", false),
        "density-comfortable" | "tool-select" => ("cursor", false),
        "density-touch" | "touch-panel" => ("hand-pointing", false),
        // Areas
        "label-defaults" => ("text-aa", false),
        "area-find-undefined" => ("warning", false),
        "area-redefine" | "photo-edit-mode" => ("pencil-simple", false),
        "area-clone" => ("copy-simple", false),
        "area-reopen" | "edit-text-library" => ("list-bullets", false),
        "area-properties" | "photo-adjust" => ("sliders", false),
        "area-attributes" => ("list-dashes", false),
        "area-make-negative" => ("minus-circle", false),
        "area-auto-subtract" => ("intersect", false),
        "area-code-settings" => ("list-numbers", false),
        "area-new-code" => ("plus-circle", false),
        "area-edit-definitions" | "draw-color" => ("pencil-simple", true),
        "wall-solids" | "shape-rectangle" => ("rectangle", false),
        "define-first" | "appraise-it-check" => ("check-circle", false),
        // Insert
        "tool-pan" => ("arrows-out-cardinal", false),
        "tool-draw-interior" => ("square-split-vertical", false),
        "tool-freeform" => ("pencil-simple", false),
        "tool-erase" => ("eraser", false),
        "tool-room-shapes" => ("shapes", false),
        "shape-circle" => ("circle", false),
        // Image
        "photo-load" => ("image", false),
        "photo-show" => ("eye", true),
        "photo-hide" => ("eye-slash", true),
        "geo-coordinate" => ("map-pin", true),
        "geo-address" => ("globe-hemisphere-west", false),
        // Tools and Help
        "disto-connect" => ("bluetooth", true),
        "subject-info" => ("house-line", false),
        "options" => ("gear", true),
        "make-symbol-from-selection" => ("star", false),
        "shortcuts" => ("keyboard", false),
        "user-guide" | "about" => ("info", false),
        "appraise-it-preview" => ("eye", false),
        "appraise-it-send" => ("export", false),
        "email" => ("envelope-simple", true),
        _ => return None,
    })
}

/// Finds a registered command by its stable string id.
pub fn find(id: &str) -> Option<&'static Command> {
    all().iter().find(|c| c.id == id)
}

/// Filters `all()` by case-insensitive substring on label (or group label).
pub fn matching(query: &str) -> Vec<&'static Command> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return all().iter().collect();
    }
    let mut by_label = Vec::new();
    let mut by_group = Vec::new();
    for cmd in all() {
        if cmd.label.to_lowercase().contains(&needle) {
            by_label.push(cmd);
        } else if cmd.group.label().to_lowercase().contains(&needle) {
            by_group.push(cmd);
        }
    }
    by_label.extend(by_group);
    by_label
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e_support::{harness_next_at, state, VIEWPORT};

    fn skips_update_in_headless_test(id: &str) -> bool {
        // Native OS file/mail/exit dialogs and WinRT PdfDocument (`print-preview`,
        // which loads Windows.Data.Pdf MTA threads that stall headless test teardown).
        matches!(
            id,
            "open"
                | "save"
                | "save-as"
                | "import"
                | "import-cad"
                | "import-layer"
                | "export-png"
                | "export-svg"
                | "export-dxf"
                | "export-pdf"
                | "print"
                | "print-preview"
                | "photo-load"
                | "geo-load-map"
                | "email"
                | "close-window"
        )
    }

    #[test]
    fn command_ids_are_unique() {
        let mut ids: Vec<&str> = all().iter().map(|c| c.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "command ids must be unique");
    }

    #[test]
    fn matching_covers_every_group_and_label() {
        assert_eq!(matching("").len(), all().len());
        let mini = matching("  MINI ");
        assert!(mini.iter().any(|c| c.id == "mini-map"));
        let save = matching("Save");
        assert!(save.iter().any(|c| c.id == "save"));
        assert!(matching("zzzzzzzzz").is_empty());
    }

    #[test]
    fn every_non_dialog_command_runs_without_panic_on_booted_app() {
        let harness = harness_next_at(VIEWPORT);
        let mut app = state(harness);
        for cmd in all() {
            let _ = (cmd.enabled)(&app);
            if let Some(check) = cmd.checked {
                let _ = check(&app);
            }
            let msg = (cmd.run)(&app);
            if skips_update_in_headless_test(cmd.id) {
                continue;
            }
            let _ = app.update(msg);
            // Reset overlays/dialogs back to a safe baseline if a tool/modal opened.
            app.dialog = None;
            app.shortcuts.open = false;
            app.guide.open = false;
            app.popup_menu.close();
            app.select_menu.close();
            app.shapes.dismiss();
            app.room_shapes.open = false;
            app.options_dialog.open = false;
            app.page_layout_dialog.open = false;
            app.text_manager.open = false;
            app.symbol_manager.open = false;
            app.ui.symbol_library.open = false;
            app.ui.start.open = false;
            app.symbol_edit_session = None;
            app.area_dialog.open = false;
            app.attr_dialog.open = false;
            app.breakdown.open = false;
            app.geo_menu.open = false;
            app.print_preview.open = false;
            app.mini_map = None;
            app.touch_panel_open = false;
        }
    }

    fn parse_csv_line(line: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = String::new();
        let mut chars = line.chars().peekable();
        let mut in_quotes = false;
        while let Some(c) = chars.next() {
            if in_quotes {
                if c == '"' {
                    if chars.peek() == Some(&'"') {
                        cur.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    cur.push(c);
                }
            } else if c == '"' {
                in_quotes = true;
            } else if c == ',' {
                out.push(std::mem::take(&mut cur));
            } else {
                cur.push(c);
            }
        }
        out.push(cur);
        out
    }

    #[test]
    fn coverage_csv_rows_and_commands_are_valid() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("ui-redesign")
            .join("coverage.csv");
        let raw = std::fs::read_to_string(&path).expect("coverage.csv readable");
        let mut lines = raw.lines().filter(|l| !l.trim().is_empty());
        let header = parse_csv_line(lines.next().expect("CSV header"));
        assert_eq!(
            header,
            [
                "id",
                "screen",
                "feature",
                "classic_location",
                "home",
                "disposition",
                "command_or_message",
                "test",
                "notes"
            ]
        );
        let mut count = 0usize;
        for line in lines {
            let cols = parse_csv_line(line);
            assert_eq!(cols.len(), 9, "bad column count in row: {line}");
            let [ref id, _, _, _, ref home, ref disp, ref cmd_msg, ref test_ref, _] =
                cols.as_slice()
            else {
                unreachable!()
            };
            assert!(!id.trim().is_empty(), "empty id");
            assert!(!home.trim().is_empty(), "empty home for {id}");
            assert!(
                matches!(
                    disp.as_str(),
                    "KEEP" | "MOVED" | "REPLACED" | "SHARED" | "RETIRED"
                ),
                "invalid disposition {disp:?} for {id}"
            );
            if matches!(disp.as_str(), "KEEP" | "MOVED") {
                assert!(
                    !cmd_msg.trim().is_empty(),
                    "{id} ({disp}) missing command_or_message"
                );
                assert!(!test_ref.trim().is_empty(), "{id} ({disp}) missing test");
            }
            for token in cmd_msg.split_whitespace() {
                if let Some(cmd_id) = token.strip_prefix("cmd:") {
                    let clean = cmd_id.trim_end_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
                    assert!(
                        find(clean).is_some(),
                        "{id} cites unknown command id {clean:?} in {cmd_msg:?}"
                    );
                }
            }
            count += 1;
        }
        assert_eq!(count, 634, "expected 634 inventory rows");
    }

    /// Every command shows an icon in the Menu, and each one is embedded:
    /// building the icon element panics in a debug build when it is not.
    #[test]
    fn every_command_has_an_embedded_icon() {
        for cmd in all() {
            let (name, fill) = icon_for(cmd.id)
                .unwrap_or_else(|| panic!("command {} has no icon", cmd.id));
            let _ = crate::icons::mono_icon(name, fill, 14.0, iced::Color::WHITE);
        }
    }

    /// Icons that name an action carry a tint; structural glyphs do not.
    #[test]
    fn action_icons_are_tinted_and_structural_glyphs_are_not() {
        use crate::ui::tokens::IconTint;
        assert_eq!(crate::icons::tint_of("trash"), Some(IconTint::Red));
        assert_eq!(crate::icons::tint_of("scissors"), Some(IconTint::Blue));
        assert_eq!(crate::icons::tint_of("ruler"), Some(IconTint::Orange));
        assert_eq!(crate::icons::tint_of("floppy-disk"), Some(IconTint::Green));
        assert_eq!(crate::icons::tint_of("bluetooth"), Some(IconTint::Slate));
        for glyph in ["caret-down", "dots-three", "list", "x", "check", "plus", "minus"] {
            assert_eq!(crate::icons::tint_of(glyph), None, "{glyph} should stay neutral");
        }
    }
}
