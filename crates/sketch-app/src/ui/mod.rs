//! Next-generation Sketch Rust interface shell (`SKETCH_UI=next`).

pub mod chooser;
pub mod commands;
pub mod controls;
pub mod dialogs;
pub mod fonts;
pub mod gallery;
pub mod insert;
pub mod inspector;
pub mod kit;
pub mod layout;
pub mod ledger;
pub mod menu;
pub mod pad;
pub mod start;
pub mod symbol_library;
pub mod tape;
pub mod tokens;
pub mod topbar;
pub mod tray;
pub mod type_editor;

use iced::widget::Stack;
use iced::{Element, Length, Point};
use sketch_core::model::WallType;

use crate::drawing::WallShape;
use crate::ui::layout::SizeClass;
use crate::ui::ledger::LedgerView;
use crate::ui::menu::MenuSub;
use crate::ui::tokens::{Density, SPACE_2, SPACE_3};
use crate::ui::tray::{DrawerTab, TrayTool};
use crate::view_state::CurveMode;
use crate::{App, Message};

/// Presentational messages specific to the Next shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlsPopover {
    Zoom,
    Fit,
    Snap,
    North,
    Underlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadPopover {
    Diagonals,
    More,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Noop,
    SetDensity(Density),
    SetGallery(bool),
    GalleryInput(String),
    ToggleLedger,
    ToggleInspector,
    OpenLedger(bool),
    OpenInspector(bool),
    CloseOverlays,
    TouchDetected,
    ToggleMenu,
    CloseMenu,
    OpenSelectByType,
    MenuSub(Option<MenuSub>),
    MenuQuery(String),
    MenuMove(i32),
    /// Left (-1) closes an open fly-out; Right (+1) opens the focused row's.
    MenuSwitchGroup(i32),
    MenuEnter,
    MenuRunId(&'static str),
    MenuRunRecent(usize),
    MenuClearRecent,
    PromptRenameDocument,
    PromptRenamePage(usize),
    TogglePageMenu(Option<usize>),
    PageMoveFirst(usize),
    PageMoveLast(usize),
    ToggleExportMenu,
    TogglePreferExportPrimary,
    ToggleTrayFlyout(Option<TrayTool>),
    TrayWallShape(WallType, WallShape),
    OpenDrawer(Option<DrawerTab>),
    ToggleDrawerPin,
    ToggleControlsPopover(ControlsPopover),
    NorthEdit(String),
    NorthSubmit,
    SetAngleStep(f64),
    MeasureFinish,
    MeasureClose,
    MeasureUndo,
    MeasureClear,
    MeasureCopy,
    SetLedgerView(LedgerView),
    ToggleLedgerMenu,
    ToggleLayersSection,
    TypeFilterChanged(String),
    LedgerResizeStart,
    LedgerResizeReset,
    LedgerHandleHover(bool),
    InspectorAdjustImage,
    SetWallLineStyle(&'static str),
    SetWallCurveMode(CurveMode),
    WallBowStep(f64),
    ToggleSelectedDimensions,
    SetAreaSubtractTarget(Option<i64>),
    SelectWall(i64),
    /// Show page `n` of the inspector.
    InspectorTab(u8),
    ChainSideFlip,
    ChainOffsetStep(f64),
    ChainToggleOverall,
    TouchTooltip(String),
    ToggleSelectMore,
    PadArrow(u8),
    PadTurn(bool),
    PadPen,
    PadLaser,
    PadPopover(Option<PadPopover>),
    PadDragStart,
    HardwareKeyPressed,
    TypeEditor(type_editor::Msg),
    SymbolLibrary(symbol_library::Msg),
    Start(start::Msg),
}

/// Presentational state for the Next shell.
#[derive(Debug)]
pub struct UiState {
    pub density: Density,
    pub gallery: bool,
    pub gallery_input: String,
    pub ledger_open: bool,
    pub ledger_view: LedgerView,
    pub ledger_width: Option<f32>,
    pub ledger_resizing: bool,
    pub ledger_handle_hover: bool,
    pub ledger_menu: bool,
    pub layers_expanded: bool,
    pub type_filter: String,
    pub inspector_open: bool,
    pub inspector_image_adjust: bool,
    /// Which page of the inspector is showing.
    pub inspector_tab: u8,
    pub inspector_area_id: Option<i64>,
    pub touch_seen: bool,
    pub menu_open: bool,
    pub menu_sub: Option<MenuSub>,
    /// The Menu's search text; non-empty lists matching commands instead.
    pub menu_query: String,
    pub menu_row: usize,
    pub page_menu: Option<usize>,
    pub export_menu: bool,
    pub prefer_export_primary: bool,
    pub tray_flyout: Option<TrayTool>,
    pub drawer: Option<DrawerTab>,
    pub drawer_pinned: bool,
    pub controls_popover: Option<ControlsPopover>,
    pub north_draft: String,
    pub select_more: bool,
    pub pad_pos: Option<Point>,
    pub pad_dragging: bool,
    pub pad_drag_last: Option<Point>,
    pub pad_popover: Option<PadPopover>,
    pub pad_user_hidden: bool,
    pub pad_hidden_by_key: bool,
    pub pad_manual: bool,
    pub type_editor: type_editor::TypeEditor,
    pub symbol_library: symbol_library::SymbolLibraryState,
    pub start: start::StartState,
}

impl Default for UiState {
    fn default() -> Self {
        let gallery = std::env::var("SKETCH_UI_GALLERY")
            .map(|v| matches!(v.trim(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        Self {
            density: Density::Comfortable,
            gallery,
            gallery_input: "24'-6\"".to_owned(),
            ledger_open: false,
            ledger_view: LedgerView::Drawn,
            ledger_width: None,
            ledger_resizing: false,
            ledger_handle_hover: false,
            ledger_menu: false,
            layers_expanded: true,
            type_filter: String::new(),
            inspector_open: false,
            inspector_image_adjust: false,
            inspector_tab: 0,
            inspector_area_id: None,
            touch_seen: false,
            menu_open: false,
            menu_sub: None,
            menu_query: String::new(),
            menu_row: 0,
            page_menu: None,
            export_menu: false,
            prefer_export_primary: false,
            tray_flyout: None,
            drawer: None,
            drawer_pinned: false,
            controls_popover: None,
            north_draft: String::new(),
            select_more: false,
            pad_pos: None,
            pad_dragging: false,
            pad_drag_last: None,
            pad_popover: None,
            pad_user_hidden: false,
            pad_hidden_by_key: false,
            pad_manual: false,
            type_editor: type_editor::TypeEditor::default(),
            symbol_library: symbol_library::SymbolLibraryState::default(),
            start: start::StartState::default(),
        }
    }
}

impl UiState {
    /// Active width of the Wide docked ledger in logical pixels.
    pub fn effective_ledger_width(&self) -> f32 {
        self.ledger_width.unwrap_or_else(|| self.density.ledger_w())
    }

    /// True when any Next shell overlay (menu, slide-over, or sheet) is open.
    pub fn has_open_overlays(&self) -> bool {
        self.ledger_open
            || self.ledger_menu
            || self.inspector_open
            || self.menu_open
            || self.page_menu.is_some()
            || self.export_menu
            || self.tray_flyout.is_some()
            || self.drawer.is_some()
            || self.controls_popover.is_some()
            || self.pad_popover.is_some()
            || self.type_editor.open
            || self.symbol_library.open
            || self.start.open
    }

    /// Closes all transient Next shell overlays (menus, popovers, sheets, modals).
    pub fn close_all(&mut self) {
        self.ledger_open = false;
        self.ledger_menu = false;
        self.inspector_open = false;
        self.inspector_image_adjust = false;
        self.menu_open = false;
        self.menu_sub = None;
        self.page_menu = None;
        self.export_menu = false;
        self.tray_flyout = None;
        self.drawer = None;
        self.controls_popover = None;
        self.pad_popover = None;
        self.type_editor.close();
        self.symbol_library.open = false;
        self.symbol_library.medium_editor_open = false;
        self.start.open = false;
    }

    /// True when the bottom sheet adds height in Compact (`< 700px`).
    pub fn compact_bottom_open(&self) -> bool {
        self.ledger_open || self.inspector_open
    }
}

fn drawing_overlays<'a>(
    app: &'a App,
    tray: Option<Element<'a, Message>>,
) -> Vec<Element<'a, Message>> {
    let mut out = vec![controls::bars_layer(app)];
    if let Some(strip) = tape::layer(app) {
        out.push(strip);
    }
    if let Some(t) = tray {
        out.push(t);
    }
    if let Some(pop) = controls::popover_layer(app) {
        out.push(pop);
    }
    out
}

/// Root view for `Shell::Next`.
pub fn view(app: &App) -> Element<'_, Message> {
    if app.ui.gallery {
        return gallery::view(app);
    }

    let size_class = SizeClass::of(app.window_size.width);
    let d = app.ui.density;
    let has_inspector = inspector::is_available(app);

    let editing_symbol = app.symbol_edit_session.is_some();

    let chrome: Element<'_, Message> = match size_class {
        SizeClass::Wide => {
            let insp = if has_inspector {
                Some(inspector::panel(app, d, false))
            } else {
                None
            };
            let ledger_el = if editing_symbol {
                iced::widget::Space::new().width(0).height(0).into()
            } else {
                ledger::panel(app, size_class, d, false)
            };
            let resize_el = if editing_symbol {
                iced::widget::Space::new().width(0).height(0).into()
            } else {
                ledger::wide_resize_handle(app)
            };
            layout::wide_layout(
                topbar::top_bar(app, size_class, d, has_inspector),
                ledger_el,
                resize_el,
                layout::drawing_stack(app.canvas_element(), drawing_overlays(app, Some(tray::floating_tray(app, d)))),
                insp,
            )
        }
        SizeClass::Medium => {
            let ledger_slide = if app.ui.ledger_open && !editing_symbol {
                Some(ledger::panel(app, size_class, d, true))
            } else {
                None
            };
            let inspector_slide = if app.ui.inspector_open && has_inspector {
                Some(inspector::panel(app, d, true))
            } else {
                None
            };
            let tray_clearance = d.tray_button().1 + SPACE_3 * 2.0 + SPACE_2;
            layout::medium_layout(
                topbar::top_bar(app, size_class, d, has_inspector),
                topbar::page_bar(app, size_class, d, has_inspector),
                layout::drawing_stack(app.canvas_element(), drawing_overlays(app, Some(tray::floating_tray(app, d)))),
                ledger_slide,
                inspector_slide,
                tray_clearance,
            )
        }
        SizeClass::Compact => {
            let sheet = if app.ui.ledger_open && !editing_symbol {
                Some(ledger::panel(app, size_class, d, true))
            } else if app.ui.inspector_open && has_inspector {
                Some(inspector::panel(app, d, true))
            } else if app.touch_panel_open {
                Some(pad::compact_sheet(app))
            } else {
                None
            };
            layout::compact_layout(
                topbar::top_bar(app, size_class, d, has_inspector),
                topbar::page_bar(app, size_class, d, has_inspector),
                layout::drawing_stack(app.canvas_element(), drawing_overlays(app, None)),
                tray::compact_bar(app, d),
                sheet,
            )
        }
    };

    let root: Element<'_, Message> = Stack::with_children(
        std::iter::once(kit::isolate_operate(chrome))
            .chain(menu::layer(app).map(kit::isolate_operate))
            .chain(topbar::layer(app).map(kit::isolate_operate))
            .chain(ledger::layer(app).map(kit::isolate_operate))
            .chain(tray::layer(app).map(kit::isolate_operate))
            .chain(pad::layer(app).map(kit::isolate_operate))
            .chain(insert::layer(app).map(kit::isolate_operate))
            .chain(
                type_editor::layer(&app.ui.type_editor, app.window_size, size_class, d)
                    .map(kit::isolate_operate),
            )
            .chain(symbol_library::layer(app, size_class, d).map(kit::isolate_operate))
            .chain(start::layer(app, size_class, d).map(kit::isolate_operate))
            .chain(
                dialogs::layers(app, size_class, d)
                    .into_iter()
                    .map(kit::isolate_operate),
            )
            .chain(app.overlay_layers().into_iter().map(kit::isolate_operate)),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    app.wrap_event_tap(root)
}
