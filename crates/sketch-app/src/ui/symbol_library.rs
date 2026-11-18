//! Unified Symbol Library screen (`docs/ui-redesign/06-symbol-library.md`).
//!
//! Layout by size class:
//! - `Wide` (>= 1080px): 3 columns side by side (Categories 216px, Grid, Editor).
//! - `Medium` (700..1079px): Categories (216px) + Grid, with Editor as a right slide-over.
//! - `Compact` (< 700px): single-page navigation (`Categories` -> `Grid` -> `Editor`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke};
use iced::widget::{
    button, checkbox, column, container, mouse_area, opaque, pick_list, row, scrollable, stack,
    text, Space,
};
use iced::{
    alignment, Alignment, Background, Border, Color, Element, Length, Padding, Point, Rectangle,
    Renderer, Size, Theme,
};
use sketch_core::symbols::{Symbol, WallSnap};

use super::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_MEDIUM};
use super::kit::{self, FieldProps};
use super::layout::SizeClass;
use super::tokens::{
    self, Density, RADIUS_CONTROL, RADIUS_DIALOG, RADIUS_TILE, SPACE_1, SPACE_2, SPACE_3, SPACE_4,
    TEXT_11, TEXT_12, TEXT_13, TEXT_14, TEXT_16,
};
use crate::icons::mono;
use crate::symlib::{self, SymIndex};
use crate::{App, Message};

pub const ALL_KEY: &str = "__all__";
pub const FAVORITES_KEY: &str = "__favorites__";
pub const HIDDEN_KEY: &str = "__hidden__";

const MODAL_MAX_W: f32 = 1180.0;
const MODAL_MAX_H: f32 = 780.0;
const CAT_COL_W: f32 = 216.0;

fn m(msg: Msg) -> Message {
    Message::Ui(super::Msg::SymbolLibrary(msg))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompactPage {
    #[default]
    Categories,
    Grid,
    Editor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptMode {
    NewCategory(String),
    RenameCategory {
        key: String,
        name: String,
    },
    NewSymbol {
        name: String,
        category: String,
        from_selection: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteSymbols(BTreeSet<String>),
    DeleteCategory(String),
    MergeDuplicates,
    RestoreOriginal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapOption(pub WallSnap);

impl SnapOption {
    const ALL: [SnapOption; 3] = [
        SnapOption(WallSnap::None),
        SnapOption(WallSnap::Side),
        SnapOption(WallSnap::Center),
    ];
}

impl fmt::Display for SnapOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            WallSnap::None => "No",
            WallSnap::Side => "One side",
            WallSnap::Center => "Centre of wall",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryChoice {
    pub key: String,
    pub name: String,
}

impl fmt::Display for CategoryChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorDraft {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub wall_snap: WallSnap,
    pub wall_gap: String,
    pub wall_align: String,
    pub wall_cap: String,
    pub dirty: bool,
}

impl EditorDraft {
    pub fn from_symbol(s: &Symbol) -> Self {
        Self {
            id: s.id.clone(),
            name: s.name.clone(),
            category: s.category.clone(),
            description: s.description.clone(),
            wall_snap: s.wall_snap,
            wall_gap: fmt_num(s.wall_gap_width),
            wall_align: fmt_num(s.wall_gap_alignment),
            wall_cap: fmt_num(s.wall_cap_length),
            dirty: false,
        }
    }
}

fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.2}")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SymbolLibraryState {
    pub open: bool,
    pub category: String,
    pub search: String,
    pub selected: BTreeSet<String>,
    pub focused: Option<String>,
    pub editor: EditorDraft,
    pub compact_page: CompactPage,
    pub medium_editor_open: bool,
    pub row_menu_cat: Option<String>,
    pub move_target: String,
    pub prompt: Option<PromptMode>,
    pub confirm: Option<ConfirmAction>,
    pub ctrl_held: bool,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Open(Option<String>),
    OpenMakeFromSelection,
    Close,
    Escape,
    SelectCategory(String),
    CompactBack,
    CloseMediumEditor,
    Search(String),
    FocusSymbol(String),
    ToggleSelect(String),
    SelectAll,
    ClearSelection,
    ToggleCatMenu(String),
    StartNewCategory,
    StartRenameCategory(String),
    PromptText(String),
    PromptCategory(String),
    SubmitPrompt,
    CancelPrompt,
    MoveCategoryUp(String),
    MoveCategoryDown(String),
    RequestDeleteCategory(String),
    MoveSymbolEarlier(String),
    MoveSymbolLater(String),
    StartNewSymbol,
    StartFromSelection,
    ImportDxf,
    MoveTarget(String),
    MoveSelected,
    ToggleFavoriteSelected,
    HideSelected,
    UnhideSelected,
    RequestDeleteSelected,
    RequestMergeDuplicates,
    ExportSelected,
    EditName(String),
    EditCategory(String),
    EditDescription(String),
    EditWallSnap(WallSnap),
    EditWallGap(String),
    EditWallAlign(String),
    EditWallCap(String),
    SaveProperties,
    ToggleFavoriteFocused,
    DuplicateFocused,
    HideFocused,
    RequestDeleteFocused,
    PlaceFocused,
    EditShapeFocused,
    ExportLibrary,
    ImportLibrary,
    RequestRestoreOriginal,
    Confirm,
    CancelConfirm,
}

impl SymbolLibraryState {
    pub fn open_with(&mut self, ix: &SymIndex, initial_cat: &str, focus: Option<&str>) {
        self.open = true;
        self.search.clear();
        self.selected.clear();
        self.row_menu_cat = None;
        self.prompt = None;
        self.confirm = None;
        self.medium_editor_open = focus.is_some();

        if let Some(id) = focus.filter(|id| ix.library.symbols.contains_key(*id)) {
            if let Some(sym) = ix.library.symbols.get(id) {
                self.category = if !sym.category.is_empty() {
                    sym.category.clone()
                } else {
                    ALL_KEY.to_owned()
                };
                self.focused = Some(sym.id.clone());
                self.editor = EditorDraft::from_symbol(sym);
                self.compact_page = CompactPage::Editor;
            }
        } else {
            self.category = if !initial_cat.is_empty()
                && (initial_cat == ALL_KEY
                    || initial_cat == FAVORITES_KEY
                    || ix.categories.iter().any(|c| c.key == initial_cat))
            {
                initial_cat.to_owned()
            } else {
                ALL_KEY.to_owned()
            };
            self.focused = None;
            self.compact_page = CompactPage::Categories;
        }
        if self.move_target.is_empty() {
            if let Some(first) = ix.categories.first() {
                self.move_target = first.key.clone();
            }
        }
    }

    pub fn sync_focus(&mut self, ix: &SymIndex, favs: &[String], shown: &[&Symbol]) {
        let valid_focus = self
            .focused
            .as_deref()
            .filter(|id| shown.iter().any(|s| s.id == *id) || ix.library.symbols.contains_key(*id))
            .map(str::to_owned);
        let next_id = valid_focus.or_else(|| shown.first().map(|s| s.id.clone()));
        if next_id != self.focused {
            self.focused = next_id.clone();
            if let Some(id) = next_id.as_deref().and_then(|id| ix.symbol(id)) {
                self.editor = EditorDraft::from_symbol(id);
            } else {
                self.editor = EditorDraft::default();
            }
        } else if let Some(id) = self.focused.as_deref() {
            if !self.editor.dirty {
                if let Some(s) = ix.symbol(id) {
                    self.editor = EditorDraft::from_symbol(s);
                }
            }
        }
        self.selected.retain(|id| ix.library.symbols.contains_key(id));
        if self.move_target.is_empty()
            || !ix.categories.iter().any(|c| c.key == self.move_target)
        {
            if let Some(first) = ix.categories.first() {
                self.move_target = first.key.clone();
            }
        }
        let _ = favs;
    }

    pub fn refresh_editor_from_symbol(&mut self, sym: &Symbol) {
        self.focused = Some(sym.id.clone());
        self.editor = EditorDraft::from_symbol(sym);
    }

    pub fn handle_escape(&mut self) -> bool {
        if !self.open {
            return false;
        }
        if self.confirm.is_some() {
            self.confirm = None;
            return true;
        }
        if self.prompt.is_some() {
            self.prompt = None;
            return true;
        }
        if self.medium_editor_open {
            self.medium_editor_open = false;
            return true;
        }
        if self.compact_page != CompactPage::Categories {
            self.compact_page = match self.compact_page {
                CompactPage::Editor => CompactPage::Grid,
                CompactPage::Grid => CompactPage::Categories,
                CompactPage::Categories => CompactPage::Categories,
            };
            return true;
        }
        self.open = false;
        true
    }
}

pub fn visible_symbols<'a>(app: &'a App, ix: &'a SymIndex) -> Vec<&'a Symbol> {
    let fav_ids: BTreeSet<String> = app.favorite_types().into_iter().collect();
    let hidden = &app.symbol_changes.hidden;
    let st = &app.ui.symbol_library;
    let q = st.search.trim().to_lowercase();

    let mut ordered: Vec<&Symbol> = match st.category.as_str() {
        "" | ALL_KEY => {
            let mut out = Vec::new();
            let mut seen = BTreeSet::new();
            for c in &ix.categories {
                for item in &c.items {
                    let id = &item.id;
                    if !hidden.contains(id) && seen.insert(id.clone()) {
                        if let Some(s) = ix.symbol(id) {
                            out.push(s);
                        }
                    }
                }
            }
            for s in ix.library.symbols.values() {
                if !hidden.contains(&s.id) && seen.insert(s.id.clone()) {
                    out.push(s);
                }
            }
            out
        }
        FAVORITES_KEY => fav_ids
            .iter()
            .filter(|id| !hidden.contains(*id))
            .filter_map(|id| ix.symbol(id))
            .collect(),
        HIDDEN_KEY => hidden.iter().filter_map(|id| ix.symbol(id)).collect(),
        cat_key => {
            let mut out = Vec::new();
            let mut seen = BTreeSet::new();
            if let Some(cat) = ix.categories.iter().find(|c| c.key == cat_key) {
                for item in &cat.items {
                    let id = &item.id;
                    if !hidden.contains(id) && seen.insert(id.clone()) {
                        if let Some(s) = ix.symbol(id) {
                            out.push(s);
                        }
                    }
                }
            }
            for s in ix.library.symbols.values() {
                if s.category == cat_key
                    && !hidden.contains(&s.id)
                    && seen.insert(s.id.clone())
                {
                    out.push(s);
                }
            }
            out
        }
    };

    if !q.is_empty() {
        ordered.retain(|s| {
            s.name.to_lowercase().contains(&q)
                || s.label.to_lowercase().contains(&q)
                || s.id.to_lowercase().contains(&q)
                || s.description.to_lowercase().contains(&q)
        });
    }
    ordered
}

pub fn placed_count(app: &App, id: &str) -> usize {
    let active_idx = app.pages.current;
    let active_count = app
        .drawing
        .symbols
        .iter()
        .filter(|s| s.symbol_type == id)
        .count();
    let other_count: usize = app
        .pages
        .pages
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != active_idx)
        .flat_map(|(_, p)| p.symbols.iter())
        .filter(|s| s.symbol_type == id)
        .count();
    active_count + other_count
}

fn snap_badge(s: &Symbol) -> String {
    match s.wall_snap {
        WallSnap::None => "Free placement".into(),
        WallSnap::Side => format!("One side · {} in", fmt_num(s.wall_gap_width)),
        WallSnap::Center => format!("Centre of wall · {} in", fmt_num(s.wall_gap_width)),
    }
}

struct EditorPreview<'a> {
    symbol: &'a Symbol,
    stroke: Color,
    bg: Color,
    wall_ink: Color,
}

impl<M> canvas::Program<M> for EditorPreview<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let cw = bounds.width;
        let ch = bounds.height;

        if self.symbol.wall_snap != WallSnap::None {
            let b = &self.symbol.bounds;
            let (min_x, min_y, max_x, max_y) = if b.max_x > b.min_x || b.max_y > b.min_y {
                (b.min_x, b.min_y, b.max_x, b.max_y)
            } else {
                (-18.0, -18.0, 18.0, 18.0)
            };
            let width = (max_x - min_x).max(1.0);
            let height = (max_y - min_y).max(1.0);
            let scale = ((cw as f64 - 24.0).max(4.0) / width)
                .min((ch as f64 - 24.0).max(4.0) / height);
            let cx = (min_x + max_x) / 2.0;
            let cy = (min_y + max_y) / 2.0;
            let ox = cw as f64 / 2.0 - cx * scale;
            let wall_y = (ch as f64 / 2.0 + cy * scale) as f32;
            let wall_stroke = Stroke::default().with_color(self.wall_ink).with_width(2.0);
            let gap = self.symbol.wall_gap_width.max(0.0);
            if gap > 0.1 {
                let (gap_l, gap_r) = match self.symbol.wall_snap {
                    WallSnap::Side => (ox, ox + gap * scale),
                    _ => (
                        ox - (gap * 0.5 - self.symbol.wall_gap_alignment) * scale,
                        ox + (gap * 0.5 + self.symbol.wall_gap_alignment) * scale,
                    ),
                };
                if gap_l > 8.0 {
                    frame.stroke(
                        &Path::line(Point::new(8.0, wall_y), Point::new(gap_l as f32, wall_y)),
                        wall_stroke,
                    );
                }
                if (gap_r as f32) < cw - 8.0 {
                    frame.stroke(
                        &Path::line(
                            Point::new(gap_r as f32, wall_y),
                            Point::new(cw - 8.0, wall_y),
                        ),
                        wall_stroke,
                    );
                }
            } else {
                frame.stroke(
                    &Path::line(Point::new(8.0, wall_y), Point::new(cw - 8.0, wall_y)),
                    wall_stroke,
                );
            }
        }

        symlib::render_fitted(&mut frame, self.symbol, cw, ch, 24.0, self.stroke, self.bg);
        vec![frame.into_geometry()]
    }
}

struct TilePreview<'a> {
    symbol: &'a Symbol,
    stroke: Color,
    bg: Color,
}

impl<M> canvas::Program<M> for TilePreview<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        symlib::render_fitted(
            &mut frame,
            self.symbol,
            bounds.width,
            bounds.height,
            10.0,
            self.stroke,
            self.bg,
        );
        vec![frame.into_geometry()]
    }
}

pub fn layer<'a>(app: &'a App, size_class: SizeClass, d: Density) -> Option<Element<'a, Message>> {
    let st = &app.ui.symbol_library;
    if !st.open {
        return None;
    }
    let ix = match app.symbols.as_ref() {
        Ok(ix) => ix,
        Err(err) => {
            return Some(error_modal(app.window_size, size_class, d, err));
        }
    };

    let compact = matches!(size_class, SizeClass::Compact);
    let modal_w = if compact {
        app.window_size.width.max(300.0)
    } else {
        (app.window_size.width - 48.0).clamp(620.0, MODAL_MAX_W)
    };
    let modal_h = if compact {
        app.window_size.height.max(400.0)
    } else {
        (app.window_size.height - 48.0).clamp(480.0, MODAL_MAX_H)
    };

    let shown = visible_symbols(app, ix);
    let body: Element<'a, Message> = match size_class {
        SizeClass::Wide => row![
            container(categories_pane(app, ix, d))
                .width(Length::Fixed(CAT_COL_W))
                .height(Length::Fill),
            col_divider(),
            container(grid_pane(app, ix, &shown, 3, d))
                .width(Length::FillPortion(5))
                .height(Length::Fill),
            col_divider(),
            container(editor_pane(app, ix, d, false))
                .width(Length::FillPortion(4))
                .height(Length::Fill),
        ]
        .height(Length::Fill)
        .into(),
        SizeClass::Medium => {
            let base = row![
                container(categories_pane(app, ix, d))
                    .width(Length::Fixed(CAT_COL_W))
                    .height(Length::Fill),
                col_divider(),
                container(grid_pane(app, ix, &shown, 2, d))
                    .width(Length::Fill)
                    .height(Length::Fill),
            ]
            .height(Length::Fill);

            if st.medium_editor_open && st.focused.is_some() {
                let slide_over = container(
                    container(editor_pane(app, ix, d, true))
                        .width(Length::Fixed(370.0))
                        .height(Length::Fill)
                        .style(|_| container::Style {
                            background: Some(Background::Color(tokens::panel())),
                            border: Border {
                                color: tokens::rule(),
                                width: 1.0,
                                ..Border::default()
                            },
                            ..container::Style::default()
                        }),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::End);
                stack![base, slide_over].into()
            } else {
                base.into()
            }
        }
        SizeClass::Compact => match st.compact_page {
            CompactPage::Categories => categories_pane(app, ix, d),
            CompactPage::Grid => grid_pane(app, ix, &shown, 2, d),
            CompactPage::Editor => editor_pane(app, ix, d, false),
        },
    };

    let body = container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true);

    let header = modal_header(st, ix, size_class, d);
    let footer = modal_footer(app, compact, d);

    let card = container(
        column![header, kit::divider(), body, kit::divider(), footer]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fixed(modal_w))
    .height(Length::Fixed(modal_h))
    .clip(true)
    .style(move |_| container::Style {
        background: Some(Background::Color(tokens::panel())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: if compact { 0.0 } else { RADIUS_DIALOG }.into(),
        },
        ..container::Style::default()
    });

    let backdrop = mouse_area(
        container(opaque(card))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
                ..container::Style::default()
            }),
    )
    .on_press(m(Msg::Close));

    Some(backdrop.into())
}

fn error_modal<'a>(
    window: Size,
    size_class: SizeClass,
    d: Density,
    err: &str,
) -> Element<'a, Message> {
    let w = if matches!(size_class, SizeClass::Compact) {
        window.width - 24.0
    } else {
        460.0
    };
    let card = kit::panel_card(
        column![
            row![
                text("Symbol library")
                    .font(BARLOW_SEMIBOLD)
                    .size(TEXT_16)
                    .color(tokens::text()),
                Space::new().width(Length::Fill),
                kit::icon_button("x", false, Some(m(Msg::Close)), d),
            ]
            .align_y(Alignment::Center),
            text(err.to_owned())
                .font(BARLOW)
                .size(TEXT_13)
                .color(tokens::flag()),
            row![
                Space::new().width(Length::Fill),
                kit::text_button("Close", Some(m(Msg::Close)), d),
            ],
        ]
        .spacing(SPACE_3)
        .padding(SPACE_4)
        .width(Length::Fixed(w)),
    );
    container(opaque(card))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
            ..container::Style::default()
        })
        .into()
}

fn modal_header<'a>(
    st: &SymbolLibraryState,
    ix: &SymIndex,
    size_class: SizeClass,
    d: Density,
) -> Element<'a, Message> {
    let compact = matches!(size_class, SizeClass::Compact);
    let mut left = row![].spacing(SPACE_2).align_y(Alignment::Center);

    if compact && st.compact_page != CompactPage::Categories {
        let back_label = match st.compact_page {
            CompactPage::Grid => "< Categories",
            CompactPage::Editor => "< Grid",
            CompactPage::Categories => "",
        };
        left = left.push(kit::text_button(
            back_label,
            Some(m(Msg::CompactBack)),
            d,
        ));
    }

    let title = if compact && st.compact_page == CompactPage::Grid {
        category_display_name(ix, &st.category)
    } else if compact && st.compact_page == CompactPage::Editor {
        st.editor.name.clone()
    } else {
        "Symbol library".to_owned()
    };

    left = left.push(
        text(title)
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_16)
            .color(tokens::text()),
    );

    container(
        row![
            left,
            Space::new().width(Length::Fill),
            kit::icon_button("x", false, Some(m(Msg::Close)), d),
        ]
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([SPACE_2, SPACE_3])
    .into()
}

fn category_display_name(ix: &SymIndex, key: &str) -> String {
    match key {
        "" | ALL_KEY => "All symbols".into(),
        FAVORITES_KEY => "Favorites".into(),
        HIDDEN_KEY => "Hidden".into(),
        other => ix
            .categories
            .iter()
            .find(|c| c.key == other)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| other.to_owned()),
    }
}

fn categories_pane<'a>(
    app: &'a App,
    ix: &'a SymIndex,
    d: Density,
) -> Element<'a, Message> {
    let st = &app.ui.symbol_library;
    let hidden = &app.symbol_changes.hidden;
    let total_visible = ix
        .library
        .symbols
        .values()
        .filter(|s| !hidden.contains(&s.id))
        .count();
    let fav_count = app
        .favorite_types()
        .into_iter()
        .filter(|id| !hidden.contains(id) && ix.library.symbols.contains_key(id))
        .count();

    let mut list = column![
        cat_row("All symbols", total_visible, st.category == ALL_KEY || st.category.is_empty(), ALL_KEY, false, false, false, 0, 0, d),
        cat_row("Favorites", fav_count, st.category == FAVORITES_KEY, FAVORITES_KEY, false, false, false, 0, 0, d),
    ]
    .spacing(2)
    .width(Length::Fill);

    if !hidden.is_empty() {
        list = list.push(cat_row(
            "Hidden",
            hidden.len(),
            st.category == HIDDEN_KEY,
            HIDDEN_KEY,
            false,
            false,
            false,
            0,
            0,
            d,
        ));
    }

    list = list.push(container(kit::divider()).padding([SPACE_1, 0.0]));

    let cat_count = ix.categories.len();
    for (idx, cat) in ix.categories.iter().enumerate() {
        let active = st.category == cat.key;
        let menu_open = st.row_menu_cat.as_deref() == Some(cat.key.as_str());
        let visible_count = cat.items.len();
        list = list.push(cat_row(
            &cat.name,
            visible_count,
            active,
            &cat.key,
            true,
            menu_open,
            true,
            idx,
            cat_count,
            d,
        ));
        if menu_open {
            let k = cat.key.clone();
            let can_up = idx > 0;
            let can_down = idx + 1 < cat_count;
            let strip = container(
                row![
                    kit::text_button(
                        "Rename",
                        Some(m(Msg::StartRenameCategory(k.clone()))),
                        d,
                    ),
                    kit::text_button(
                        "Delete",
                        Some(m(Msg::RequestDeleteCategory(k.clone()))),
                        d,
                    ),
                    kit::icon_button(
                        "arrow-up",
                        false,
                        can_up.then(|| m(Msg::MoveCategoryUp(k.clone()))),
                        d,
                    ),
                    kit::icon_button(
                        "arrow-down",
                        false,
                        can_down.then(|| m(Msg::MoveCategoryDown(k))),
                        d,
                    ),
                ]
                .spacing(SPACE_1)
                .wrap(),
            )
            .padding([SPACE_1, SPACE_2]);
            list = list.push(strip);
        }
    }

    let mut footer = column![].spacing(SPACE_1).width(Length::Fill);
    match &st.prompt {
        Some(PromptMode::NewCategory(buf)) => {
            footer = footer.push(
                column![
                    kit::field(
                        FieldProps {
                            label: "New category",
                            placeholder: "Category name",
                            value: buf,
                            on_input: Some(Box::new(|v| m(Msg::PromptText(v)))),
                            on_submit: Some(m(Msg::SubmitPrompt)),
                            focused: false,
                        },
                        d,
                    ),
                    row![
                        kit::primary_button(
                            "Create",
                            Some(m(Msg::SubmitPrompt)),
                            d,
                        ),
                        kit::text_button(
                            "Cancel",
                            Some(m(Msg::CancelPrompt)),
                            d,
                        ),
                    ]
                    .spacing(SPACE_1),
                ]
                .spacing(SPACE_1),
            );
        }
        Some(PromptMode::RenameCategory { key: _, name }) => {
            footer = footer.push(
                column![
                    kit::field(
                        FieldProps {
                            label: "Rename category",
                            placeholder: "New name",
                            value: name,
                            on_input: Some(Box::new(|v| m(Msg::PromptText(v)))),
                            on_submit: Some(m(Msg::SubmitPrompt)),
                            focused: false,
                        },
                        d,
                    ),
                    row![
                        kit::primary_button(
                            "Rename",
                            Some(m(Msg::SubmitPrompt)),
                            d,
                        ),
                        kit::text_button(
                            "Cancel",
                            Some(m(Msg::CancelPrompt)),
                            d,
                        ),
                    ]
                    .spacing(SPACE_1),
                ]
                .spacing(SPACE_1),
            );
        }
        _ => {
            footer = footer.push(kit::text_button(
                "New category",
                Some(m(Msg::StartNewCategory)),
                d,
            ));
        }
    }

    column![
        scrollable(container(list).padding(SPACE_2).width(Length::Fill)).id("symbol-categories-list")
            .width(Length::Fill)
            .height(Length::Fill),
        kit::divider(),
        container(footer).padding(SPACE_2).width(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn cat_row<'a>(
    label: &str,
    count: usize,
    active: bool,
    key: &str,
    has_menu: bool,
    menu_open: bool,
    _real: bool,
    _idx: usize,
    _total: usize,
    d: Density,
) -> Element<'a, Message> {
    let k = key.to_owned();
    let fg = if active { tokens::accent() } else { tokens::text() };
    let count_str = count.to_string();
    let label_btn = button(
        row![
            text(label.to_owned())
                .font(if active { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
                .size(TEXT_13)
                .color(fg)
                .width(Length::Fill),
            text(count_str)
                .font(BSC_MEDIUM)
                .size(TEXT_11)
                .color(tokens::muted()),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([6, 8])
    .on_press(m(Msg::SelectCategory(k.clone())))
    .style(move |_, status| {
        let hov = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if active {
                tokens::accent_soft()
            } else if hov {
                tokens::hover()
            } else {
                Color::TRANSPARENT
            })),
            border: Border {
                color: if active { tokens::accent() } else { Color::TRANSPARENT },
                width: if active { 1.0 } else { 0.0 },
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        }
    });

    if !has_menu {
        return label_btn.into();
    }
    row![
        label_btn,
        kit::icon_button(
            if menu_open { "caret-up" } else { "dots-three" },
            menu_open,
            Some(m(Msg::ToggleCatMenu(k))),
            d,
        ),
    ]
    .spacing(2)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}

fn grid_pane<'a>(
    app: &'a App,
    ix: &'a SymIndex,
    shown: &[&'a Symbol],
    cols: usize,
    d: Density,
) -> Element<'a, Message> {
    let st = &app.ui.symbol_library;
    let fav_ids: BTreeSet<String> = app.favorite_types().into_iter().collect();
    let has_selection = !st.selected.is_empty();
    let dups_count = app.merge_pairs().len();
    let is_hidden_view = st.category == HIDDEN_KEY;
    let can_make_from_sel = crate::symbol_shape::selection_convertible(&app.drawing, &app.selection);

    let search = kit::field(
        FieldProps {
            label: "",
            placeholder: "Search symbols in category",
            value: &st.search,
            on_input: Some(Box::new(|v| m(Msg::Search(v)))),
            on_submit: None,
            focused: false,
        },
        d,
    );

    let mut top_bar = row![
        container(search).width(Length::Fill),
        kit::text_button("New symbol", Some(m(Msg::StartNewSymbol)), d),
        kit::text_button("Import DXF", Some(m(Msg::ImportDxf)), d),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    if can_make_from_sel {
        top_bar = top_bar.push(kit::text_button(
            "From selection",
            Some(m(Msg::StartFromSelection)),
            d,
        ));
    }

    let mut col_items = column![container(top_bar).padding([SPACE_2, SPACE_3])];

    if let Some(PromptMode::NewSymbol {
        name,
        category,
        from_selection,
    }) = &st.prompt
    {
        let cat_opts: Vec<CategoryChoice> = ix
            .categories
            .iter()
            .map(|c| CategoryChoice {
                key: c.key.clone(),
                name: c.name.clone(),
            })
            .collect();
        let selected_cat = cat_opts
            .iter()
            .find(|c| c.key == *category)
            .cloned()
            .or_else(|| cat_opts.first().cloned());
        let title_str = if *from_selection {
            "New symbol from selection"
        } else {
            "Create new symbol"
        };
        let prompt_card = container(
            column![
                text(title_str)
                    .font(BARLOW_SEMIBOLD)
                    .size(TEXT_13)
                    .color(tokens::text()),
                kit::field(
                    FieldProps {
                        label: "Name",
                        placeholder: "Symbol name",
                        value: name,
                        on_input: Some(Box::new(|v| m(Msg::PromptText(v)))),
                        on_submit: Some(m(Msg::SubmitPrompt)),
                        focused: false,
                    },
                    d,
                ),
                row![
                    text("Category")
                        .font(BARLOW)
                        .size(TEXT_12)
                        .color(tokens::muted()),
                    pick_list(cat_opts, selected_cat, |c: CategoryChoice| {
                        m(Msg::PromptCategory(c.key))
                    })
            .style(kit::pick_list_style)
            .menu_style(kit::pick_menu_style)
                    .width(Length::Fill),
                ]
                .spacing(SPACE_2)
                .align_y(Alignment::Center),
                row![
                    kit::primary_button("Create", Some(m(Msg::SubmitPrompt)), d),
                    kit::text_button("Cancel", Some(m(Msg::CancelPrompt)), d),
                ]
                .spacing(SPACE_2),
            ]
            .spacing(SPACE_2),
        )
        .padding(SPACE_2)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::field())),
            border: Border {
                color: tokens::accent(),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        });
        col_items = col_items.push(container(prompt_card).padding(Padding {
            top: 0.0,
            right: SPACE_3,
            bottom: SPACE_2,
            left: SPACE_3,
        }));
    }

    // Multi-select + library tools bar
    let cat_opts: Vec<CategoryChoice> = ix
        .categories
        .iter()
        .map(|c| CategoryChoice {
            key: c.key.clone(),
            name: c.name.clone(),
        })
        .collect();
    let move_sel = cat_opts
        .iter()
        .find(|c| c.key == st.move_target)
        .cloned()
        .or_else(|| cat_opts.first().cloned());

    let mut sel_bar = row![
        text(format!("{} shown · {} selected", shown.len(), st.selected.len()))
            .font(BSC_MEDIUM)
            .size(TEXT_11)
            .color(tokens::muted()),
        kit::text_button("Select all", Some(m(Msg::SelectAll)), d),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    if has_selection {
        sel_bar = sel_bar
            .push(kit::text_button(
                "Clear",
                Some(m(Msg::ClearSelection)),
                d,
            ))
            .push(
                pick_list(cat_opts, move_sel, |c: CategoryChoice| {
                    m(Msg::MoveTarget(c.key))
                })
            .style(kit::pick_list_style)
            .menu_style(kit::pick_menu_style)
                .width(Length::Fixed(130.0)),
            )
            .push(kit::text_button(
                "Move to category",
                Some(m(Msg::MoveSelected)),
                d,
            ))
            .push(kit::text_button(
                "Add to favorites",
                Some(m(Msg::ToggleFavoriteSelected)),
                d,
            ));
        if is_hidden_view {
            sel_bar = sel_bar.push(kit::text_button(
                "Unhide",
                Some(m(Msg::UnhideSelected)),
                d,
            ));
        } else {
            sel_bar = sel_bar.push(kit::text_button(
                "Hide",
                Some(m(Msg::HideSelected)),
                d,
            ));
        }
        sel_bar = sel_bar
            .push(kit::text_button(
                "Delete",
                Some(m(Msg::RequestDeleteSelected)),
                d,
            ))
            .push(kit::text_button(
                "Export selected",
                Some(m(Msg::ExportSelected)),
                d,
            ));
    }
    if dups_count > 0 {
        sel_bar = sel_bar.push(kit::text_button(
            format!("Merge duplicates ({dups_count})"),
            Some(m(Msg::RequestMergeDuplicates)),
            d,
        ));
    }

    col_items = col_items
        .push(container(sel_bar.wrap()).padding(Padding {
            top: 0.0,
            right: SPACE_3,
            bottom: SPACE_2,
            left: SPACE_3,
        }))
        .push(kit::divider());

    // Tile grid
    let grid_el: Element<'a, Message> = if shown.is_empty() {
        container(
            column![
                text("No symbols in this view.")
                    .font(BARLOW_MEDIUM)
                    .size(TEXT_13)
                    .color(tokens::muted()),
                kit::text_button(
                    "Show all symbols",
                    Some(m(Msg::SelectCategory(ALL_KEY.into()))),
                    d,
                ),
            ]
            .spacing(SPACE_2)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(SPACE_4)
        .align_x(Alignment::Center)
        .into()
    } else {
        let mut grid = column![].spacing(SPACE_2).width(Length::Fill);
        let cols = cols.max(1);
        for chunk in shown.chunks(cols) {
            let mut r = row![].spacing(SPACE_2).width(Length::Fill);
            for sym in chunk {
                let is_focused = st.focused.as_deref() == Some(sym.id.as_str());
                let is_checked = st.selected.contains(&sym.id);
                let is_fav = fav_ids.contains(&sym.id);
                r = r.push(symbol_grid_tile(sym, is_focused, is_checked, is_fav));
            }
            for _ in chunk.len()..cols {
                r = r.push(Space::new().width(Length::FillPortion(1)));
            }
            grid = grid.push(r);
        }
        grid.into()
    };

    col_items = col_items.push(
        scrollable(container(grid_el).padding(SPACE_3).width(Length::Fill)).id("symbol-grid-list")
            .width(Length::Fill)
            .height(Length::Fill),
    );

    col_items.width(Length::Fill).height(Length::Fill).into()
}

fn symbol_grid_tile<'a>(
    sym: &'a Symbol,
    focused: bool,
    checked: bool,
    is_fav: bool,
) -> Element<'a, Message> {
    let id = sym.id.clone();
    let toggle_id = sym.id.clone();
    let bg = if focused {
        tokens::accent_soft()
    } else {
        tokens::field()
    };
    let border_col = if focused {
        tokens::accent()
    } else if checked {
        tokens::text()
    } else {
        tokens::rule()
    };

    let preview = Canvas::new(TilePreview {
        symbol: sym,
        stroke: tokens::text(),
        bg,
    })
    .width(Length::Fill)
    .height(Length::Fixed(52.0));

    let name_el = text(sym.name.clone())
        .font(if focused { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
        .size(TEXT_12)
        .color(tokens::text())
        .align_x(alignment::Horizontal::Center);

    let behavior_el = text(snap_badge(sym))
        .font(BSC_MEDIUM)
        .size(TEXT_11)
        .color(tokens::muted())
        .align_x(alignment::Horizontal::Center);

    let top_row = row![
        checkbox(checked)
            .size(15)
            .on_toggle(move |_| m(Msg::ToggleSelect(toggle_id.clone()))),
        Space::new().width(Length::Fill),
        if is_fav {
            mono("star", 11.0, tokens::accent())
        } else {
            Space::new().width(Length::Fixed(11.0)).into()
        },
    ]
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let tile_body = container(
        column![top_row, preview, name_el, behavior_el]
            .spacing(4.0)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::FillPortion(1))
    .padding(8)
    .style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_col,
            width: if focused { 1.5 } else { 1.0 },
            radius: RADIUS_TILE.into(),
        },
        ..container::Style::default()
    });

    mouse_area(tile_body)
        .on_press(m(Msg::FocusSymbol(id)))
        .into()
}

fn editor_pane<'a>(
    app: &'a App,
    ix: &'a SymIndex,
    d: Density,
    slide_over: bool,
) -> Element<'a, Message> {
    let st = &app.ui.symbol_library;
    let Some(id) = st.focused.as_deref() else {
        return container(
            text("Select a symbol to inspect or edit.")
                .font(BARLOW_MEDIUM)
                .size(TEXT_13)
                .color(tokens::muted()),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into();
    };
    let Some(sym) = ix.symbol(id) else {
        return Space::new().into();
    };

    let favs = app.favorite_types();
    let is_fav = favs.iter().any(|x| x == id);
    let is_hidden = app.symbol_changes.hidden.contains(id);
    let placed = placed_count(app, id);

    let preview_box = container(
        Canvas::new(EditorPreview {
            symbol: sym,
            stroke: tokens::text(),
            bg: tokens::field(),
            wall_ink: tokens::accent(),
        })
        .width(Length::Fill)
        .height(Length::Fixed(110.0)),
    )
    .width(Length::Fill)
    .padding(SPACE_2)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::field())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    });

    let cat_opts: Vec<CategoryChoice> = ix
        .categories
        .iter()
        .map(|c| CategoryChoice {
            key: c.key.clone(),
            name: c.name.clone(),
        })
        .collect();
    let selected_cat = cat_opts
        .iter()
        .find(|c| c.key == st.editor.category)
        .cloned();
    let selected_snap = Some(SnapOption(st.editor.wall_snap));

    let placed_label = match placed {
        0 => "Not placed in this sketch".into(),
        1 => "Placed 1 time in this sketch".into(),
        n => format!("Placed {n} times in this sketch"),
    };

    let in_real_cat = ix.categories.iter().find(|c| c.key == st.category);
    let cat_pos = in_real_cat.and_then(|c| c.items.iter().position(|i| i.id == id));

    let mut action_row = row![
        kit::text_button(
            if is_fav { "Remove favorite" } else { "Add to favorites" },
            Some(m(Msg::ToggleFavoriteFocused)),
            d,
        ),
        kit::text_button(
            "Duplicate",
            Some(m(Msg::DuplicateFocused)),
            d,
        ),
        kit::text_button(
            if is_hidden { "Unhide" } else { "Hide" },
            Some(m(Msg::HideFocused)),
            d,
        ),
        kit::text_button(
            "Delete",
            Some(m(Msg::RequestDeleteFocused)),
            d,
        ),
        kit::primary_button(
            "Place",
            Some(m(Msg::PlaceFocused)),
            d,
        ),
    ]
    .spacing(SPACE_1);

    if let (Some(cat), Some(pos)) = (in_real_cat, cat_pos) {
        if pos > 0 {
            action_row = action_row.push(kit::icon_button(
                "arrow-up",
                false,
                Some(m(Msg::MoveSymbolEarlier(id.to_owned()))),
                d,
            ));
        }
        if pos + 1 < cat.items.len() {
            action_row = action_row.push(kit::icon_button(
                "arrow-down",
                false,
                Some(m(Msg::MoveSymbolLater(id.to_owned()))),
                d,
            ));
        }
    }
    let action_row = action_row.wrap();

    let mut form = column![].spacing(SPACE_2).width(Length::Fill);

    if slide_over {
        form = form.push(
            row![
                text("Edit symbol")
                    .font(BARLOW_SEMIBOLD)
                    .size(TEXT_14)
                    .color(tokens::text()),
                Space::new().width(Length::Fill),
                kit::text_button(
                    "Back",
                    Some(m(Msg::CloseMediumEditor)),
                    d,
                ),
            ]
            .align_y(Alignment::Center),
        );
    }

    form = form
        .push(preview_box)
        .push(
            row![
                text(placed_label)
                    .font(BARLOW)
                    .size(TEXT_11)
                    .color(tokens::muted()),
                Space::new().width(Length::Fill),
                kit::text_button(
                    "Edit shape",
                    Some(m(Msg::EditShapeFocused)),
                    d,
                ),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .push(kit::field(
            FieldProps {
                label: "Name",
                placeholder: "Symbol name",
                value: &st.editor.name,
                on_input: Some(Box::new(|v| m(Msg::EditName(v)))),
                on_submit: Some(m(Msg::SaveProperties)),
                focused: false,
            },
            d,
        ))
        .push(labeled_pick(
            "Category",
            pick_list(cat_opts, selected_cat, |c: CategoryChoice| {
                m(Msg::EditCategory(c.key))
            })
            .style(kit::pick_list_style)
            .menu_style(kit::pick_menu_style)
            .width(Length::Fill),
        ))
        .push(kit::field(
            FieldProps {
                label: "Description",
                placeholder: "Optional description",
                value: &st.editor.description,
                on_input: Some(Box::new(|v| m(Msg::EditDescription(v)))),
                on_submit: Some(m(Msg::SaveProperties)),
                focused: false,
            },
            d,
        ))
        .push(labeled_pick(
            "Snaps to wall",
            pick_list(SnapOption::ALL, selected_snap, |opt: SnapOption| {
                m(Msg::EditWallSnap(opt.0))
            })
            .style(kit::pick_list_style)
            .menu_style(kit::pick_menu_style)
            .width(Length::Fill),
        ))
        .push(
            row![
                container(kit::field(
                    FieldProps {
                        label: "Wall gap (in)",
                        placeholder: "0",
                        value: &st.editor.wall_gap,
                        on_input: Some(Box::new(|v| m(Msg::EditWallGap(v)))),
                        on_submit: Some(m(Msg::SaveProperties)),
                        focused: false,
                    },
                    d,
                ))
                .width(Length::Fill),
                container(kit::field(
                    FieldProps {
                        label: "Gap align (in)",
                        placeholder: "0",
                        value: &st.editor.wall_align,
                        on_input: Some(Box::new(|v| m(Msg::EditWallAlign(v)))),
                        on_submit: Some(m(Msg::SaveProperties)),
                        focused: false,
                    },
                    d,
                ))
                .width(Length::Fill),
                container(kit::field(
                    FieldProps {
                        label: "End cap (in)",
                        placeholder: "0",
                        value: &st.editor.wall_cap,
                        on_input: Some(Box::new(|v| m(Msg::EditWallCap(v)))),
                        on_submit: Some(m(Msg::SaveProperties)),
                        focused: false,
                    },
                    d,
                ))
                .width(Length::Fill),
            ]
            .spacing(SPACE_2)
            .width(Length::Fill),
        )
        .push(
            row![
                text("Units")
                    .font(BARLOW)
                    .size(TEXT_12)
                    .color(tokens::muted()),
                Space::new().width(Length::Fill),
                text("Inches")
                    .font(BARLOW_MEDIUM)
                    .size(TEXT_12)
                    .color(tokens::text()),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .push(
            row![
                text(format!("ID: {}", sym.id))
                    .font(BSC_MEDIUM)
                    .size(TEXT_11)
                    .color(tokens::muted()),
                Space::new().width(Length::Fill),
                kit::primary_button(
                    if st.editor.dirty { "Save changes" } else { "Saved" },
                    Some(m(Msg::SaveProperties)),
                    d,
                ),
            ]
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .push(kit::divider())
        .push(action_row);

    scrollable(container(form).padding(SPACE_3).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn labeled_pick<'a>(
    label: &'a str,
    picker: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    column![
        text(label).font(BARLOW).size(TEXT_12).color(tokens::muted()),
        picker.into(),
    ]
    .spacing(3.0)
    .width(Length::Fill)
    .into()
}

fn modal_footer<'a>(app: &'a App, compact: bool, d: Density) -> Element<'a, Message> {
    let st = &app.ui.symbol_library;
    let has_original = symlib::original_library_path().is_some();

    if let Some(confirm) = &st.confirm {
        let prompt = match confirm {
            ConfirmAction::DeleteSymbols(ids) => format!(
                "Delete {} symbol{} permanently? Placed instances will be removed and the previous file saved as symbols.json.bak.",
                ids.len(),
                if ids.len() == 1 { "" } else { "s" }
            ),
            ConfirmAction::DeleteCategory(key) => format!(
                "Delete category '{key}' and all its symbols permanently? Previous library saved as symbols.json.bak."
            ),
            ConfirmAction::MergeDuplicates => {
                "Merge every group of identical symbols into its keeper? Placed instances will be remapped."
                    .to_owned()
            }
            ConfirmAction::RestoreOriginal => {
                "Restore the original shipped symbol library? Current library will be saved as symbols.json.bak and user overrides cleared."
                    .to_owned()
            }
        };
        return container(
            row![
                text(prompt)
                    .font(BARLOW_MEDIUM)
                    .size(TEXT_12)
                    .color(tokens::flag())
                    .width(Length::Fill),
                kit::text_button("Cancel", Some(m(Msg::CancelConfirm)), d),
                kit::primary_button("Confirm", Some(m(Msg::Confirm)), d),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center)
            .width(Length::Fill),
        )
        .padding([SPACE_2, SPACE_3])
        .into();
    }

    let restore_btn = kit::text_button(
        "Restore original library",
        has_original.then_some(m(Msg::RequestRestoreOriginal)),
        d,
    );

    let mut left = row![
        kit::text_button(
            "Export library",
            Some(m(Msg::ExportLibrary)),
            d,
        ),
        kit::text_button(
            "Import library",
            Some(m(Msg::ImportLibrary)),
            d,
        ),
        restore_btn,
    ]
    .spacing(SPACE_1);

    if !has_original && !compact {
        left = left.push(
            text("(symbols.original.json not found)")
                .font(BARLOW)
                .size(TEXT_11)
                .color(tokens::muted()),
        );
    }

    container(
        row![
            container(left.wrap()).width(Length::Fill),
            kit::text_button("Close", Some(m(Msg::Close)), d),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([SPACE_2, SPACE_3])
    .into()
}

fn col_divider<'a>() -> Element<'a, Message> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::rule())),
            ..container::Style::default()
        })
        .into()
}

/// App-level handler wiring `symbol_library` to `sketch_core::symbol_edit` with
/// per-user `Changes.custom` fallback on read-only library destinations (§7).
pub fn update(app: &mut App, msg: Msg) -> iced::Task<Message> {
    match msg {
        Msg::Open(focus) => {
            app.ui.close_all();
            let initial_cat = if app.dock.symbol_category == "favorites" {
                FAVORITES_KEY.to_owned()
            } else if app.dock.symbol_category == "__all__" {
                ALL_KEY.to_owned()
            } else {
                app.dock.symbol_category.clone()
            };
            let arc = app.symbols_arc();
            let Ok(ix) = arc.as_ref().as_ref() else {
                app.ui.symbol_library.open = true;
                return iced::Task::none();
            };
            app.ui
                .symbol_library
                .open_with(ix, &initial_cat, focus.as_deref());
            let shown_ids: Vec<String> = visible_symbols(app, ix)
                .into_iter()
                .map(|s| s.id.clone())
                .collect();
            let favs = app.favorite_types();
            let shown: Vec<Symbol> = shown_ids
                .iter()
                .filter_map(|id| ix.symbol(id).cloned())
                .collect();
            let refs: Vec<&Symbol> = shown.iter().collect();
            app.ui.symbol_library.sync_focus(ix, &favs, &refs);
            iced::Task::none()
        }
        Msg::OpenMakeFromSelection => {
            if !crate::symbol_shape::selection_convertible(&app.drawing, &app.selection) {
                return app
                    .toasts
                    .show("Select one or more walls, areas, or labels first.".into());
            }
            app.ui.close_all();
            let arc = app.symbols_arc();
            let Ok(ix) = arc.as_ref().as_ref() else {
                app.ui.symbol_library.open = true;
                return iced::Task::none();
            };
            let first_cat = ix
                .categories
                .first()
                .map(|c| c.key.clone())
                .unwrap_or_else(|| "custom".into());
            app.ui.symbol_library.open_with(ix, &first_cat, None);
            app.ui.symbol_library.compact_page = CompactPage::Grid;
            app.ui.symbol_library.prompt = Some(PromptMode::NewSymbol {
                name: String::new(),
                category: first_cat,
                from_selection: true,
            });
            iced::Task::none()
        }
        Msg::Close => {
            app.ui.symbol_library.open = false;
            app.ui.symbol_library.medium_editor_open = false;
            iced::Task::none()
        }
        Msg::Escape => {
            app.ui.symbol_library.handle_escape();
            iced::Task::none()
        }
        Msg::SelectCategory(cat) => {
            app.ui.symbol_library.category = cat;
            app.ui.symbol_library.row_menu_cat = None;
            app.ui.symbol_library.compact_page = CompactPage::Grid;
            app.ui.symbol_library.medium_editor_open = false;
            sync_library_focus(app);
            iced::Task::none()
        }
        Msg::CompactBack => {
            let st = &mut app.ui.symbol_library;
            st.compact_page = match st.compact_page {
                CompactPage::Editor => CompactPage::Grid,
                CompactPage::Grid => CompactPage::Categories,
                CompactPage::Categories => CompactPage::Categories,
            };
            iced::Task::none()
        }
        Msg::CloseMediumEditor => {
            app.ui.symbol_library.medium_editor_open = false;
            iced::Task::none()
        }
        Msg::Search(q) => {
            app.ui.symbol_library.search = q;
            sync_library_focus(app);
            iced::Task::none()
        }
        Msg::FocusSymbol(id) => {
            let ctrl = app.ui.symbol_library.ctrl_held;
            if ctrl && !app.ui.symbol_library.selected.remove(&id) {
                app.ui.symbol_library.selected.insert(id.clone());
            }
            if let Some(sym) = app.symbols_lib().symbol(&id).cloned() {
                app.ui.symbol_library.refresh_editor_from_symbol(&sym);
                app.ui.symbol_library.compact_page = CompactPage::Editor;
                app.ui.symbol_library.medium_editor_open = true;
            }
            iced::Task::none()
        }
        Msg::ToggleSelect(id) => {
            if !app.ui.symbol_library.selected.remove(&id) {
                app.ui.symbol_library.selected.insert(id.clone());
            }
            if let Some(sym) = app.symbols_lib().symbol(&id).cloned() {
                app.ui.symbol_library.refresh_editor_from_symbol(&sym);
            }
            iced::Task::none()
        }
        Msg::SelectAll => {
            let ix = app.symbols_lib();
            let ids: BTreeSet<String> = visible_symbols(app, ix)
                .into_iter()
                .map(|s| s.id.clone())
                .collect();
            app.ui.symbol_library.selected = ids;
            iced::Task::none()
        }
        Msg::ClearSelection => {
            app.ui.symbol_library.selected.clear();
            iced::Task::none()
        }
        Msg::ToggleCatMenu(k) => {
            if app.ui.symbol_library.row_menu_cat.as_deref() == Some(k.as_str()) {
                app.ui.symbol_library.row_menu_cat = None;
            } else {
                app.ui.symbol_library.row_menu_cat = Some(k);
            }
            iced::Task::none()
        }
        Msg::StartNewCategory => {
            app.ui.symbol_library.prompt = Some(PromptMode::NewCategory(String::new()));
            iced::Task::none()
        }
        Msg::StartRenameCategory(key) => {
            let current = app
                .symbols_lib()
                .categories
                .iter()
                .find(|c| c.key == key)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            app.ui.symbol_library.prompt = Some(PromptMode::RenameCategory { key, name: current });
            app.ui.symbol_library.row_menu_cat = None;
            iced::Task::none()
        }
        Msg::PromptText(val) => {
            match &mut app.ui.symbol_library.prompt {
                Some(PromptMode::NewCategory(buf)) => *buf = val,
                Some(PromptMode::RenameCategory { name, .. }) => *name = val,
                Some(PromptMode::NewSymbol { name, .. }) => *name = val,
                None => {}
            }
            iced::Task::none()
        }
        Msg::PromptCategory(cat) => {
            if let Some(PromptMode::NewSymbol { category, .. }) = &mut app.ui.symbol_library.prompt {
                *category = cat;
            }
            iced::Task::none()
        }
        Msg::CancelPrompt => {
            app.ui.symbol_library.prompt = None;
            iced::Task::none()
        }
        Msg::SubmitPrompt => submit_prompt(app),
        Msg::MoveCategoryUp(key) => move_category(app, &key, -1),
        Msg::MoveCategoryDown(key) => move_category(app, &key, 1),
        Msg::RequestDeleteCategory(key) => {
            app.ui.symbol_library.confirm = Some(ConfirmAction::DeleteCategory(key));
            app.ui.symbol_library.row_menu_cat = None;
            iced::Task::none()
        }
        Msg::MoveSymbolEarlier(id) => move_symbol_in_cat(app, &id, -1),
        Msg::MoveSymbolLater(id) => move_symbol_in_cat(app, &id, 1),
        Msg::StartNewSymbol => start_new_symbol_prompt(app, false),
        Msg::StartFromSelection => start_new_symbol_prompt(app, true),
        Msg::ImportDxf => import_dxf_dialog(app),
        Msg::MoveTarget(k) => {
            app.ui.symbol_library.move_target = k;
            iced::Task::none()
        }
        Msg::MoveSelected => move_selected(app),
        Msg::ToggleFavoriteSelected => {
            let ids: Vec<String> = app.ui.symbol_library.selected.iter().cloned().collect();
            toggle_favorites(app, &ids)
        }
        Msg::HideSelected => {
            let ids = app.ui.symbol_library.selected.clone();
            app.symbol_changes.hidden.extend(ids);
            let task = app.store_symbols();
            sync_library_focus(app);
            task
        }
        Msg::UnhideSelected => {
            let ids = app.ui.symbol_library.selected.clone();
            for id in &ids {
                app.symbol_changes.hidden.remove(id);
            }
            let task = app.store_symbols();
            sync_library_focus(app);
            task
        }
        Msg::RequestDeleteSelected => {
            if !app.ui.symbol_library.selected.is_empty() {
                app.ui.symbol_library.confirm = Some(ConfirmAction::DeleteSymbols(
                    app.ui.symbol_library.selected.clone(),
                ));
            }
            iced::Task::none()
        }
        Msg::RequestMergeDuplicates => {
            if !app.merge_pairs().is_empty() {
                app.ui.symbol_library.confirm = Some(ConfirmAction::MergeDuplicates);
            }
            iced::Task::none()
        }
        Msg::ExportSelected => export_library_dialog(app, Some(app.ui.symbol_library.selected.clone())),
        Msg::EditName(v) => {
            app.ui.symbol_library.editor.name = v;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::EditCategory(v) => {
            app.ui.symbol_library.editor.category = v;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::EditDescription(v) => {
            app.ui.symbol_library.editor.description = v;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::EditWallSnap(s) => {
            app.ui.symbol_library.editor.wall_snap = s;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::EditWallGap(v) => {
            app.ui.symbol_library.editor.wall_gap = v;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::EditWallAlign(v) => {
            app.ui.symbol_library.editor.wall_align = v;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::EditWallCap(v) => {
            app.ui.symbol_library.editor.wall_cap = v;
            app.ui.symbol_library.editor.dirty = true;
            iced::Task::none()
        }
        Msg::SaveProperties => save_editor_properties(app),
        Msg::ToggleFavoriteFocused => {
            if let Some(id) = app.ui.symbol_library.focused.clone() {
                toggle_favorites(app, &[id])
            } else {
                iced::Task::none()
            }
        }
        Msg::DuplicateFocused => duplicate_focused(app),
        Msg::HideFocused => {
            if let Some(id) = app.ui.symbol_library.focused.clone() {
                if !app.symbol_changes.hidden.remove(&id) {
                    app.symbol_changes.hidden.insert(id);
                }
                let task = app.store_symbols();
                sync_library_focus(app);
                return task;
            }
            iced::Task::none()
        }
        Msg::RequestDeleteFocused => {
            if let Some(id) = app.ui.symbol_library.focused.clone() {
                app.ui.symbol_library.confirm =
                    Some(ConfirmAction::DeleteSymbols(BTreeSet::from([id])));
            }
            iced::Task::none()
        }
        Msg::PlaceFocused => {
            if let Some(id) = app.ui.symbol_library.focused.clone() {
                app.ui.symbol_library.open = false;
                app.ui.symbol_library.medium_editor_open = false;
                app.placing_symbol = Some(id.clone());
                app.dock.preview = Some(id);
                if !app.ui.drawer_pinned {
                    app.ui.drawer = None;
                }
                return app.set_mode(crate::view_state::Mode::PlaceSymbol);
            }
            iced::Task::none()
        }
        Msg::EditShapeFocused => {
            if let Some(id) = app.ui.symbol_library.focused.clone() {
                return app.start_symbol_shape_edit(&id);
            }
            iced::Task::none()
        }
        Msg::ExportLibrary => export_library_dialog(app, None),
        Msg::ImportLibrary => import_library_dialog(app),
        Msg::RequestRestoreOriginal => {
            app.ui.symbol_library.confirm = Some(ConfirmAction::RestoreOriginal);
            iced::Task::none()
        }
        Msg::CancelConfirm => {
            app.ui.symbol_library.confirm = None;
            iced::Task::none()
        }
        Msg::Confirm => execute_confirm(app),
    }
}

fn sync_library_focus(app: &mut App) {
    let arc = app.symbols_arc();
    let Ok(ix) = arc.as_ref().as_ref() else {
        return;
    };
    let shown: Vec<Symbol> = visible_symbols(app, ix).into_iter().cloned().collect();
    let refs: Vec<&Symbol> = shown.iter().collect();
    let favs = app.favorite_types();
    app.ui.symbol_library.sync_focus(ix, &favs, &refs);
}

fn kebab(s: &str) -> String {
    let k = crate::symbol_manager::category_key(s);
    if k.is_empty() {
        "symbol".into()
    } else {
        k
    }
}

pub fn unique_symbol_id(ix: &SymIndex, category_key: &str, name: &str) -> String {
    let base = format!("{}/{}", category_key, kebab(name));
    if !ix.library.symbols.contains_key(&base) {
        return base;
    }
    let mut n = 2usize;
    loop {
        let cand = format!("{base}-{n}");
        if !ix.library.symbols.contains_key(&cand) {
            return cand;
        }
        n += 1;
    }
}

fn category_name_for(ix: &SymIndex, key: &str) -> String {
    ix.categories
        .iter()
        .find(|c| c.key == key)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| key.to_owned())
}

fn toggle_favorites(app: &mut App, ids: &[String]) -> iced::Task<Message> {
    let mut favs = app.favorite_types();
    for id in ids {
        if let Some(p) = favs.iter().position(|x| x == id) {
            favs.remove(p);
        } else {
            favs.push(id.clone());
        }
    }
    app.symbol_favorites = Some(favs);
    app.persist_library();
    iced::Task::none()
}

fn start_new_symbol_prompt(app: &mut App, from_selection: bool) -> iced::Task<Message> {
    let cat = {
        let ix = app.symbols_lib();
        let cur = &app.ui.symbol_library.category;
        if !matches!(cur.as_str(), "" | ALL_KEY | FAVORITES_KEY | HIDDEN_KEY)
            && ix.categories.iter().any(|c| c.key == *cur)
        {
            cur.clone()
        } else {
            ix.categories
                .first()
                .map(|c| c.key.clone())
                .unwrap_or_else(|| "custom".into())
        }
    };
    app.ui.symbol_library.prompt = Some(PromptMode::NewSymbol {
        name: String::new(),
        category: cat,
        from_selection,
    });
    iced::Task::none()
}

fn submit_prompt(app: &mut App) -> iced::Task<Message> {
    let Some(prompt) = app.ui.symbol_library.prompt.take() else {
        return iced::Task::none();
    };
    match prompt {
        PromptMode::NewCategory(raw) => {
            let name = raw.trim().to_owned();
            if name.is_empty() {
                return app.toasts.show("Category name cannot be empty.".into());
            }
            let base_key = crate::symbol_manager::category_key(&name);
            let mut key = if base_key.is_empty() {
                "category".into()
            } else {
                base_key.clone()
            };
            let ix = app.symbols_lib();
            let mut n = 2usize;
            while ix.categories.iter().any(|c| c.key == key) {
                key = format!("{base_key}-{n}");
                n += 1;
            }
            match symlib::try_edit_library(|p| sketch_core::symbol_edit::add_category(p, &key, &name)) {
                Ok(_) => {
                    app.rebuild_symbols();
                    app.ui.symbol_library.category = key;
                    app.ui.symbol_library.compact_page = CompactPage::Grid;
                    sync_library_focus(app);
                    app.toasts.show(format!("Created category {name}."))
                }
                Err((err, true)) => {
                    let _ = err;
                    app.toasts.show("Could not write new category to shared library.".into())
                }
                Err((err, false)) => app.toasts.show(err),
            }
        }
        PromptMode::RenameCategory { key, name } => {
            let trimmed = name.trim().to_owned();
            if trimmed.is_empty() {
                return app.toasts.show("Category name cannot be empty.".into());
            }
            match symlib::try_edit_library(|p| {
                sketch_core::symbol_edit::rename_category(p, &key, &trimmed)
            }) {
                Ok(_) => {
                    for s in app.symbol_changes.custom.values_mut() {
                        if s.category == key {
                            s.category_name = trimmed.clone();
                        }
                    }
                    let task = app.store_symbols();
                    sync_library_focus(app);
                    task.chain(app.toasts.show(format!("Renamed category to {trimmed}.")))
                }
                Err((_, true)) => {
                    let updates: Vec<Symbol> = app
                        .symbols_lib()
                        .library
                        .symbols
                        .values()
                        .filter(|s| s.category == key)
                        .cloned()
                        .map(|mut s| {
                            s.category_name = trimmed.clone();
                            s
                        })
                        .collect();
                    for s in updates {
                        app.symbol_changes.custom.insert(s.id.clone(), s);
                    }
                    let task = app.store_symbols();
                    sync_library_focus(app);
                    task.chain(app.toasts.show("Saved for this user only".into()))
                }
                Err((err, false)) => app.toasts.show(err),
            }
        }
        PromptMode::NewSymbol {
            name,
            category,
            from_selection,
        } => {
            let trimmed = name.trim().to_owned();
            if trimmed.is_empty() {
                return app.toasts.show("Symbol name cannot be empty.".into());
            }
            let ix = app.symbols_lib();
            let id = unique_symbol_id(ix, &category, &trimmed);
            let cat_name = category_name_for(ix, &category);
            let (geometry, bounds) = if from_selection {
                match crate::symbol_shape::from_selection(&app.drawing, &app.selection) {
                    Ok(pair) => pair,
                    Err(e) => return app.toasts.show(e),
                }
            } else {
                // Default 24"x24" square in inches
                let geom = vec![sketch_core::symbols::Entity {
                    entity_type: sketch_core::symbols::EntityType::Polyline,
                    closed: Some(true),
                    vertices: Some(vec![
                        sketch_core::symbols::Vertex { x: -12.0, y: -12.0, b: None },
                        sketch_core::symbols::Vertex { x: 12.0, y: -12.0, b: None },
                        sketch_core::symbols::Vertex { x: 12.0, y: 12.0, b: None },
                        sketch_core::symbols::Vertex { x: -12.0, y: 12.0, b: None },
                    ]),
                    ..Default::default()
                }];
                let bounds = sketch_core::symbols::Bounds {
                    min_x: -12.0,
                    min_y: -12.0,
                    max_x: 12.0,
                    max_y: 12.0,
                };
                (geom, bounds)
            };
            let sym = Symbol {
                id: id.clone(),
                category: category.clone(),
                category_name: cat_name.clone(),
                name: trimmed.clone(),
                label: if cat_name.is_empty() {
                    trimmed.clone()
                } else {
                    format!("{cat_name} - {trimmed}")
                },
                description: trimmed.clone(),
                units: "INCHES".into(),
                wall_snap: WallSnap::None,
                wall_gap_width: 0.0,
                wall_gap_alignment: 0.0,
                wall_cap_length: 0.0,
                bounds,
                geometry,
                native_cad: None,
            };
            add_symbol_to_library(app, sym, format!("Created symbol {trimmed}."))
        }
    }
}

pub fn add_symbol_to_library(app: &mut App, sym: Symbol, ok_toast: String) -> iced::Task<Message> {
    let id = sym.id.clone();
    let cat = sym.category.clone();
    match symlib::try_edit_library(|p| sketch_core::symbol_edit::add(p, &sym)) {
        Ok(_) => {
            app.rebuild_symbols();
            app.ui.symbol_library.category = cat;
            app.ui.symbol_library.refresh_editor_from_symbol(&sym);
            app.ui.symbol_library.selected = BTreeSet::from([id]);
            app.ui.symbol_library.medium_editor_open = true;
            app.ui.symbol_library.compact_page = CompactPage::Editor;
            app.toasts.show(ok_toast)
        }
        Err((_, true)) => {
            app.symbol_changes.custom.insert(id.clone(), sym.clone());
            let task = app.store_symbols();
            app.ui.symbol_library.category = cat;
            app.ui.symbol_library.refresh_editor_from_symbol(&sym);
            app.ui.symbol_library.selected = BTreeSet::from([id]);
            app.ui.symbol_library.medium_editor_open = true;
            app.ui.symbol_library.compact_page = CompactPage::Editor;
            task.chain(app.toasts.show("Saved for this user only".into()))
        }
        Err((err, false)) => app.toasts.show(err),
    }
}

fn move_category(app: &mut App, key: &str, delta: isize) -> iced::Task<Message> {
    let ix = app.symbols_lib();
    let mut keys: Vec<String> = ix.categories.iter().map(|c| c.key.clone()).collect();
    let Some(pos) = keys.iter().position(|k| k == key) else {
        return iced::Task::none();
    };
    let next = (pos as isize + delta).clamp(0, keys.len().saturating_sub(1) as isize) as usize;
    if next == pos {
        return iced::Task::none();
    }
    keys.swap(pos, next);
    match symlib::try_edit_library(|p| sketch_core::symbol_edit::reorder_categories(p, &keys)) {
        Ok(_) => {
            app.rebuild_symbols();
            iced::Task::none()
        }
        Err((err, _)) => app.toasts.show(format!("Could not reorder categories ({err}).")),
    }
}

fn move_symbol_in_cat(app: &mut App, id: &str, delta: isize) -> iced::Task<Message> {
    let cat_key = app.ui.symbol_library.category.clone();
    let ix = app.symbols_lib();
    let Some(cat) = ix.categories.iter().find(|c| c.key == cat_key) else {
        return iced::Task::none();
    };
    let mut ids: Vec<String> = cat.items.iter().map(|i| i.id.clone()).collect();
    let Some(pos) = ids.iter().position(|x| x == id) else {
        return iced::Task::none();
    };
    let next = (pos as isize + delta).clamp(0, ids.len().saturating_sub(1) as isize) as usize;
    if next == pos {
        return iced::Task::none();
    }
    ids.swap(pos, next);
    match symlib::try_edit_library(|p| sketch_core::symbol_edit::reorder_symbols(p, &cat_key, &ids)) {
        Ok(_) => {
            app.rebuild_symbols();
            iced::Task::none()
        }
        Err((err, _)) => app.toasts.show(format!("Could not reorder symbols ({err}).")),
    }
}

fn move_selected(app: &mut App) -> iced::Task<Message> {
    let ids = app.ui.symbol_library.selected.clone();
    let target = app.ui.symbol_library.move_target.clone();
    if ids.is_empty() || target.is_empty() {
        return iced::Task::none();
    }
    let ix = app.symbols_lib();
    let cat_name = category_name_for(ix, &target);
    match symlib::try_edit_library(|p| {
        sketch_core::symbol_edit::move_to_category(p, &ids, &target, &cat_name)
    }) {
        Ok(_) => {
            for id in &ids {
                if let Some(s) = app.symbol_changes.custom.get_mut(id) {
                    s.category = target.clone();
                    s.category_name = cat_name.clone();
                }
            }
            let task = app.store_symbols();
            sync_library_focus(app);
            task.chain(app.toasts.show(format!("Moved {} symbol(s) to {cat_name}.", ids.len())))
        }
        Err((_, true)) => {
            let mut moved = Vec::new();
            {
                let ix = app.symbols_lib();
                for id in &ids {
                    if let Some(mut s) = ix.symbol(id).cloned() {
                        s.category = target.clone();
                        s.category_name = cat_name.clone();
                        moved.push((id.clone(), s));
                    }
                }
            }
            for (id, s) in moved {
                app.symbol_changes.custom.insert(id, s);
            }
            let task = app.store_symbols();
            sync_library_focus(app);
            task.chain(app.toasts.show("Saved for this user only".into()))
        }
        Err((err, false)) => app.toasts.show(err),
    }
}

fn save_editor_properties(app: &mut App) -> iced::Task<Message> {
    let draft = app.ui.symbol_library.editor.clone();
    let name = draft.name.trim().to_owned();
    if draft.id.is_empty() || name.is_empty() {
        return app.toasts.show("Symbol name cannot be empty.".into());
    }
    let gap = draft.wall_gap.trim().parse::<f64>().unwrap_or(0.0).max(0.0);
    let align = draft.wall_align.trim().parse::<f64>().unwrap_or(0.0);
    let cap = draft.wall_cap.trim().parse::<f64>().unwrap_or(0.0).max(0.0);
    let desc = draft.description.clone();
    let snap = draft.wall_snap;
    let target_cat = draft.category.clone();

    let ix = app.symbols_lib();
    let Some(old_sym) = ix.symbol(&draft.id).cloned() else {
        return iced::Task::none();
    };
    let cat_name = category_name_for(ix, &target_cat);

    let file_res = symlib::try_edit_library(|path| {
        let mut count = 0;
        if !target_cat.is_empty() && target_cat != old_sym.category {
            let one = BTreeSet::from([draft.id.clone()]);
            count += sketch_core::symbol_edit::move_to_category(path, &one, &target_cat, &cat_name)?;
        }
        let props = sketch_core::symbol_edit::SymbolProps {
            name: Some(name.clone()),
            description: Some(desc.clone()),
            wall_snap: Some(snap),
            wall_gap_width: Some(gap),
            wall_gap_alignment: Some(align),
            wall_cap_length: Some(cap),
        };
        count += sketch_core::symbol_edit::update_properties(path, &draft.id, &props)?;
        Ok(count)
    });

    match file_res {
        Ok(_) => {
            app.symbol_changes.renamed.remove(&draft.id);
            if let Some(c) = app.symbol_changes.custom.get_mut(&draft.id) {
                c.name = name.clone();
                c.label = if cat_name.is_empty() {
                    name.clone()
                } else {
                    format!("{cat_name} - {name}")
                };
                c.category = target_cat.clone();
                c.category_name = cat_name;
                c.description = desc;
                c.wall_snap = snap;
                c.wall_gap_width = gap;
                c.wall_gap_alignment = align;
                c.wall_cap_length = cap;
            }
            let task = app.store_symbols();
            if let Some(updated) = app.symbols_lib().symbol(&draft.id).cloned() {
                app.ui.symbol_library.refresh_editor_from_symbol(&updated);
            }
            task.chain(app.toasts.show(format!("Saved {name}.")))
        }
        Err((_, true)) => {
            let mut updated = old_sym;
            updated.name = name.clone();
            updated.category = target_cat.clone();
            updated.category_name = cat_name.clone();
            updated.label = if cat_name.is_empty() {
                name
            } else {
                format!("{cat_name} - {name}")
            };
            updated.description = desc;
            updated.wall_snap = snap;
            updated.wall_gap_width = gap;
            updated.wall_gap_alignment = align;
            updated.wall_cap_length = cap;
            app.symbol_changes.custom.insert(updated.id.clone(), updated.clone());
            let task = app.store_symbols();
            app.ui.symbol_library.refresh_editor_from_symbol(&updated);
            task.chain(app.toasts.show("Saved for this user only".into()))
        }
        Err((err, false)) => app.toasts.show(err),
    }
}

fn duplicate_focused(app: &mut App) -> iced::Task<Message> {
    let Some(id) = app.ui.symbol_library.focused.clone() else {
        return iced::Task::none();
    };
    let ix = app.symbols_lib();
    let Some(orig) = ix.symbol(&id).cloned() else {
        return iced::Task::none();
    };
    let new_name = format!("{} Copy", orig.name);
    let new_id = unique_symbol_id(ix, &orig.category, &new_name);
    match symlib::try_edit_library(|p| {
        sketch_core::symbol_edit::duplicate(p, &id, &new_id, &new_name)
    }) {
        Ok(_) => {
            app.rebuild_symbols();
            if let Some(dup) = app.symbols_lib().symbol(&new_id).cloned() {
                app.ui.symbol_library.refresh_editor_from_symbol(&dup);
            }
            app.toasts.show(format!("Duplicated as {new_name}."))
        }
        Err((_, true)) => {
            let mut copy = orig;
            copy.id = new_id.clone();
            copy.name = new_name.clone();
            copy.label = if copy.category_name.is_empty() {
                new_name
            } else {
                format!("{} - {new_name}", copy.category_name)
            };
            app.symbol_changes.custom.insert(new_id, copy.clone());
            let task = app.store_symbols();
            app.ui.symbol_library.refresh_editor_from_symbol(&copy);
            task.chain(app.toasts.show("Saved for this user only".into()))
        }
        Err((err, false)) => app.toasts.show(err),
    }
}

fn execute_confirm(app: &mut App) -> iced::Task<Message> {
    let Some(confirm) = app.ui.symbol_library.confirm.take() else {
        return iced::Task::none();
    };
    match confirm {
        ConfirmAction::DeleteSymbols(ids) => {
            let base_ids: BTreeSet<String> = match symlib::library().as_ref() {
                Ok(base) => ids
                    .iter()
                    .filter(|id| base.library.symbols.contains_key(*id))
                    .cloned()
                    .collect(),
                Err(_) => BTreeSet::new(),
            };
            let mut notice = format!(
                "Deleted {} symbol{}.",
                ids.len(),
                if ids.len() == 1 { "" } else { "s" }
            );
            if !base_ids.is_empty() {
                match symlib::try_edit_library(|p| sketch_core::symbol_edit::delete(p, &base_ids)) {
                    Ok(_) => notice.push_str(" Previous library saved as symbols.json.bak."),
                    Err((_, true)) => {
                        app.symbol_changes.deleted.extend(ids.clone());
                        notice = "Saved for this user only".into();
                    }
                    Err((err, false)) => return app.toasts.show(err),
                }
            }
            app.forget_symbols(&ids);
            app.ui.symbol_library.selected.retain(|x| !ids.contains(x));
            let task = app.store_symbols();
            sync_library_focus(app);
            task.chain(app.toasts.show(notice))
        }
        ConfirmAction::DeleteCategory(key) => {
            let ids: BTreeSet<String> = app
                .symbols_lib()
                .library
                .symbols
                .values()
                .filter(|s| s.category == key)
                .map(|s| s.id.clone())
                .collect();
            let mut notice = format!("Deleted category {key}.");
            match symlib::try_edit_library(|p| sketch_core::symbol_edit::delete_category(p, &key)) {
                Ok(_) => notice.push_str(" Previous library saved as symbols.json.bak."),
                Err((_, true)) => {
                    app.symbol_changes.deleted.extend(ids.clone());
                    app.symbol_changes.deleted_categories.insert(key.clone());
                    notice = "Saved for this user only".into();
                }
                Err((err, false)) => return app.toasts.show(err),
            }
            app.forget_symbols(&ids);
            if app.ui.symbol_library.category == key {
                app.ui.symbol_library.category = ALL_KEY.into();
            }
            let task = app.store_symbols();
            sync_library_focus(app);
            task.chain(app.toasts.show(notice))
        }
        ConfirmAction::MergeDuplicates => {
            let pairs = app.merge_pairs();
            if pairs.is_empty() {
                return iced::Task::none();
            }
            match symlib::try_edit_library(|p| sketch_core::symbol_edit::merge(p, &pairs)) {
                Ok(_) => {
                    let map: BTreeMap<String, String> = pairs.iter().cloned().collect();
                    app.remap_symbols(&map);
                    let kept: BTreeSet<&String> = map.values().collect();
                    let task = app.store_symbols();
                    sync_library_focus(app);
                    task.chain(app.toasts.show(format!(
                        "Merged {} duplicates into {} symbols.",
                        map.len(),
                        kept.len()
                    )))
                }
                Err((err, _)) => app.toasts.show(format!("Could not merge duplicates ({err}).")),
            }
        }
        ConfirmAction::RestoreOriginal => restore_original_library(app),
    }
}

pub fn restore_original_library(app: &mut App) -> iced::Task<Message> {
    let Some(orig) = symlib::original_library_path() else {
        return app
            .toasts
            .show("Original library file (symbols.original.json) was not found.".into());
    };
    let dest = symlib::library_path();
    let orig_bytes = match std::fs::read(&orig) {
        Ok(b) => b,
        Err(e) => return app.toasts.show(format!("Could not read {}: {e}", orig.display())),
    };
    if let Ok(cur_bytes) = std::fs::read(&dest) {
        let bak = dest.with_file_name("symbols.json.bak");
        if let Err(e) = std::fs::write(&bak, &cur_bytes) {
            return app.toasts.show(format!("Could not write backup ({}): {e}", bak.display()));
        }
    }
    let tmp = dest.with_file_name("symbols.json.tmp");
    if let Err(e) = std::fs::write(&tmp, &orig_bytes) {
        return app.toasts.show(format!("Could not restore library ({}): {e}", dest.display()));
    }
    if let Err(e) = std::fs::rename(&tmp, &dest) {
        let _ = std::fs::remove_file(&tmp);
        return app.toasts.show(format!("Could not replace library ({}): {e}", dest.display()));
    }
    symlib::invalidate_cache();
    app.symbol_changes.hidden.clear();
    app.symbol_changes.renamed.clear();
    app.symbol_changes.deleted.clear();
    app.symbol_changes.deleted_categories.clear();
    app.ui.symbol_library.category = ALL_KEY.into();
    app.ui.symbol_library.selected.clear();
    app.ui.symbol_library.editor.dirty = false;
    let task = app.store_symbols();
    sync_library_focus(app);
    task.chain(app.toasts.show(
        "Restored original symbol library. Previous library saved as symbols.json.bak.".into(),
    ))
}

fn import_dxf_dialog(app: &mut App) -> iced::Task<Message> {
    #[cfg(not(test))]
    if let Some(paths) = crate::platform::pick_files("DXF or Symbol JSON", &["dxf", "json", "dwg"])
    {
        return import_dxf_paths(app, &paths);
    }
    let _ = app;
    iced::Task::none()
}

pub fn import_dxf_paths(app: &mut App, paths: &[std::path::PathBuf]) -> iced::Task<Message> {
    let mut added = 0usize;
    let mut user_only = false;
    let mut tasks = Vec::new();
    let mut errors = Vec::new();
    let (cat_key, cat_name) = {
        let ix = app.symbols_lib();
        let cat_key = if !matches!(
            app.ui.symbol_library.category.as_str(),
            "" | ALL_KEY | FAVORITES_KEY | HIDDEN_KEY
        ) && ix.categories.iter().any(|c| c.key == app.ui.symbol_library.category)
        {
            app.ui.symbol_library.category.clone()
        } else {
            "imported".into()
        };
        let cat_name = category_name_for(ix, &cat_key);
        (cat_key, cat_name)
    };

    for path in paths {
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("dwg")) {
            errors.push("DWG requires conversion to DXF.".into());
            continue;
        }
        let Ok(body) = std::fs::read_to_string(path) else {
            errors.push(format!("{}: read error", path.display()));
            continue;
        };
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported");
        let parsed: Result<Symbol, String> = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dxf"))
        {
            sketch_core::symbol_import::from_dxf(&body, stem)
        } else {
            serde_json::from_str(&body).map_err(|e| format!("Invalid JSON: {e}"))
        };
        match parsed {
            Ok(mut s) => {
                if s.name.trim().is_empty() {
                    s.name = stem.to_owned();
                }
                s.category = cat_key.clone();
                s.category_name = cat_name.clone();
                s.id = unique_symbol_id(app.symbols_lib(), &cat_key, &s.name);
                s.label = if cat_name.is_empty() {
                    s.name.clone()
                } else {
                    format!("{cat_name} - {}", s.name)
                };
                if s.units.trim().is_empty() {
                    s.units = "INCHES".into();
                }
                match symlib::try_edit_library(|p| sketch_core::symbol_edit::add(p, &s)) {
                    Ok(_) => {
                        app.rebuild_symbols();
                        app.ui.symbol_library.refresh_editor_from_symbol(&s);
                        added += 1;
                    }
                    Err((_, true)) => {
                        app.symbol_changes.custom.insert(s.id.clone(), s.clone());
                        tasks.push(app.store_symbols());
                        app.ui.symbol_library.refresh_editor_from_symbol(&s);
                        user_only = true;
                        added += 1;
                    }
                    Err((e, false)) => errors.push(e),
                }
            }
            Err(e) => errors.push(e),
        }
    }
    sync_library_focus(app);
    let toast = if errors.is_empty() {
        let mut msg = format!("Imported {added} symbol(s).");
        if user_only {
            msg.push_str(" Saved for this user only.");
        }
        app.toasts.show(msg)
    } else {
        app.toasts
            .show(format!("Imported {added} symbol(s). {}", errors.join("; ")))
    };
    tasks.push(toast);
    iced::Task::batch(tasks)
}

fn export_library_dialog(app: &mut App, subset: Option<BTreeSet<String>>) -> iced::Task<Message> {
    #[cfg(not(test))]
    if let Some(dest) = crate::platform::save_file("Symbol Library JSON", &["json"], "symbols-export.json")
    {
        return match sketch_core::symbol_edit::export(&symlib::library_path(), &dest, subset.as_ref()) {
            Ok(n) => app.toasts.show(format!("Exported {n} symbol(s).")),
            Err(e) => app.toasts.show(format!("Export failed: {e}")),
        };
    }
    let _ = (app, subset);
    iced::Task::none()
}

fn import_library_dialog(app: &mut App) -> iced::Task<Message> {
    #[cfg(not(test))]
    if let Some(src) = crate::platform::pick_file("Symbol Library JSON", &["json"])
    {
        return import_library_file(app, &src);
    }
    let _ = app;
    iced::Task::none()
}

pub fn import_library_file(app: &mut App, src: &std::path::Path) -> iced::Task<Message> {
    let mut conflicts = 0usize;
    match symlib::try_edit_library(|p| {
        let rep = sketch_core::symbol_edit::import_with_report(p, src)?;
        conflicts = rep.conflicts.len();
        Ok(rep.added)
    }) {
        Ok(added) => {
            app.rebuild_symbols();
            sync_library_focus(app);
            if conflicts > 0 {
                app.toasts.show(format!(
                    "Imported {added} symbol(s); skipped {conflicts} existing."
                ))
            } else {
                app.toasts.show(format!("Imported {added} symbol(s)."))
            }
        }
        Err((err, _)) => app.toasts.show(format!("Import failed ({err}).")),
    }
}
