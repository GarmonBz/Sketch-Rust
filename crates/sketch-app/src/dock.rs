//! Dock state models and area-palette helpers.

use crate::symlib::SymIndex;

/// `.sidebar-dock` default `--sidebar-w`.
#[cfg(test)]
pub const DEFAULT_W: f32 = 312.0;
/// SIDEBAR RESIZE LAYER: `MIN = 320, MAX = 400, CANVAS_MIN = 220`.
pub const MIN_W: f32 = 320.0;
pub const MAX_W: f32 = 400.0;
pub const CANVAS_MIN: f32 = 220.0;
/// `.sidebar-dock:not(.open)`.
#[cfg(test)]
pub const COLLAPSED_W: f32 = 30.0;

/// The `.layer-rename` field's text_input id.
pub const RENAME_ID: &str = "layer-rename-input";

/// ui-manager.js `defaultTextLibraryLabels`.
pub const DEFAULT_TEXT_LABELS: &[&str] = &[
    "½ Bath",
    "Atrium",
    "Bath",
    "Bed",
    "Breakfast",
    "Cabin",
    "Carport",
    "Den",
    "Dining",
    "Dining Room",
    "Entrance",
    "Family",
    "Foyer",
    "Garage",
    "Kitchen",
    "Laundry",
    "Living Room",
    "Master",
    "Office",
    "Patio",
    "Play Cabin",
    "Porch",
    "Suite",
    "Utilities",
    "Wood Deck",
];

/// `.alpha-filter-btn` `(label, data-range)`.
pub(crate) const ALPHA: &[(&str, &str)] = &[
    ("All", "all"),
    ("ABC", "abc"),
    ("DEF", "def"),
    ("GHI", "ghi"),
    ("JKL", "jkl"),
    ("MNO", "mno"),
    ("PQR", "pqr"),
    ("STU", "stu"),
    ("VWX", "vwx"),
    ("YZ", "yz"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockTab {
    Areas,
    Text,
    Symbols,
    Calcs,
    Layers,
}

impl DockTab {
    pub const ALL: [DockTab; 5] = [
        DockTab::Areas,
        DockTab::Text,
        DockTab::Symbols,
        DockTab::Calcs,
        DockTab::Layers,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DockTab::Areas => "Areas",
            DockTab::Text => "Text",
            DockTab::Symbols => "Symbols",
            DockTab::Calcs => "Calcs",
            DockTab::Layers => "Layers",
        }
    }

    /// `state.sidebar.active` key (the Areas panel keeps its legacy key).
    pub fn key(self) -> &'static str {
        match self {
            DockTab::Areas => "define",
            DockTab::Text => "text",
            DockTab::Symbols => "symbols",
            DockTab::Calcs => "calcs",
            DockTab::Layers => "layers",
        }
    }

    /// The inverse of `key` — restoring `state.sidebar.active`.
    pub fn from_key(text: &str) -> Option<Self> {
        match text {
            "define" => Some(DockTab::Areas),
            "text" => Some(DockTab::Text),
            "symbols" => Some(DockTab::Symbols),
            "calcs" => Some(DockTab::Calcs),
            "layers" => Some(DockTab::Layers),
            _ => None,
        }
    }
}

/// What a dock item carries when it is dragged onto the canvas
/// (`application/x-sketch-item`: `kind` 'symbol' / 'text').
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragItem {
    Symbol(String),
    Text(String),
    /// An Areas-palette tile (`kind: 'area'`): the code, and its name when
    /// the tile shows one (the "Undefined · clear" tile does not).
    Area {
        code: String,
        name: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum DockMsg {
    Tab(DockTab),
    /// `showPanel(id)` from outside the tab strip (e.g. the totals chip):
    /// activates the panel and always opens the dock.
    Show(DockTab),
    Collapse,
    Expand,
    TogglePin,
    ResizeStart,
    ResizeReset,
    HandleHover(bool),
    TextSearch(String),
    TextSubmit,
    TextRange(&'static str),
    TextPick(String),
    SymbolCategory(CategoryOption),
    SymbolSearch(String),
    SymbolPick(String),
    /// The two library managers' openers (the Edit / Organize... buttons).
    EditTextLibrary,
    OrganizeSymbols,
    /// Buttons whose behavior belongs to a later screen (dialogs, Calcs).
    Noop,
}

/// A `#symbol-category-select` option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryOption {
    pub key: String,
    pub name: String,
}

impl std::fmt::Display for CategoryOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

/// `state.sidebar` plus the dock's UI-manager state.
#[derive(Debug, Clone)]
pub struct DockState {
    pub active: DockTab,
    pub open: bool,
    pub pinned: bool,
    /// `state.sidebar.width`; None = the CSS default (312px).
    pub width: Option<f32>,
    pub resizing: bool,
    pub handle_hover: bool,
    pub text_labels: Vec<String>,
    pub text_range: &'static str,
    pub text_search: String,
    /// `app.areaCodes` — the live table the Areas tree renders from.
    pub area_codes: Vec<sketch_core::geometry::AreaCode>,
    /// The `.lib-item.active` label (cleared whenever the list re-renders).
    pub text_active: Option<String>,
    pub symbol_category: String,
    pub symbol_search: String,
    /// The symbol shown in `#symbol-preview`.
    pub preview: Option<String>,
    /// Calcs groups the user collapsed (empty = all expanded, as the original
    /// opens them).
    pub calcs_collapsed: std::collections::HashSet<String>,
    /// Calcs areas whose details are open (the original starts them closed).
    pub calcs_expanded: std::collections::HashSet<i64>,
    // --- The Layers tree (S10-025..034 + S10-031) ---
    /// The row armed by a press (the drag source); `updateLayersPanel`'s
    /// rows are HTML5 `draggable` — the port arms on press and commits on
    /// release (drop-before/after/inside markers follow).
    pub layer_drag: Option<String>,
    /// The hovered drop target + ratio zone while a drag is armed.
    pub layer_over: Option<(String, crate::layers::DragPos)>,
    /// `.layer-row:hover` — the cursor's row.
    pub layer_hover: Option<String>,
    /// The inline rename input: (row id, live buffer).
    pub layer_rename: Option<(String, String)>,
    /// The right-click popup's row (`showLayerContextMenu`).
    pub layer_menu: Option<String>,
}

impl Default for DockState {
    fn default() -> Self {
        DockState {
            active: DockTab::Areas,
            open: true,
            pinned: true,
            width: None,
            resizing: false,
            handle_hover: false,
            text_labels: DEFAULT_TEXT_LABELS.iter().map(|s| s.to_string()).collect(),
            text_range: "all",
            text_search: String::new(),
            area_codes: Vec::new(),
            text_active: None,
            symbol_category: "favorites".into(),
            symbol_search: String::new(),
            preview: None,
            calcs_collapsed: Default::default(),
            calcs_expanded: Default::default(),
            layer_drag: None,
            layer_over: None,
            layer_hover: None,
            layer_rename: None,
            layer_menu: None,
        }
    }
}

/// What a dock message asks the app to do besides updating dock state.
#[derive(Debug, PartialEq)]
pub enum Effect {
    None,
    Toast(String),
}

/// SIDEBAR RESIZE LAYER `clamp(w)`.
pub fn clamp_width(w: f32, viewport_w: f32) -> f32 {
    let avail = MIN_W.max(viewport_w - CANVAS_MIN);
    MIN_W.max(MAX_W.min(w.round()).min(avail))
}

/// `String.prototype.localeCompare` order for the label sets involved:
/// case-insensitive, with symbols before digits/number forms (`½`) before
/// letters, as ICU collates them.
fn collate_key(s: &str) -> Vec<(u8, String)> {
    s.chars()
        .map(|c| {
            let class = if c.is_alphabetic() {
                2
            } else if c.is_numeric() {
                1
            } else {
                0
            };
            (class, c.to_lowercase().collect())
        })
        .collect()
}

impl DockState {
    /// The dock's laid-out width.
    #[cfg(test)]
    pub fn width(&self) -> f32 {
        if self.open {
            self.width.unwrap_or(DEFAULT_W)
        } else {
            COLLAPSED_W
        }
    }

    /// Pointer moved to `x` while dragging the handle (`right - clientX`,
    /// the dock's right edge being the viewport's).
    pub fn drag_to(&mut self, x: f32, viewport_w: f32) {
        if self.resizing {
            self.width = Some(clamp_width(viewport_w - x, viewport_w));
        }
    }

    pub fn end_drag(&mut self) {
        self.resizing = false;
    }

    /// `populateTextLibrary(filter)` — the labels on display, sorted.
    pub fn text_items(&self) -> Vec<&str> {
        let q = self.text_search.trim().to_lowercase();
        let letters = if self.text_range == "all" {
            ""
        } else {
            self.text_range
        };
        let mut items: Vec<&str> = self
            .text_labels
            .iter()
            .map(String::as_str)
            .filter(|l| q.is_empty() || l.to_lowercase().contains(&q))
            .filter(|l| {
                letters.is_empty()
                    || l.trim()
                        .chars()
                        .next()
                        .map(|c| c.to_lowercase().any(|lc| letters.contains(lc)))
                        .unwrap_or(false)
            })
            .collect();
        items.sort_by_key(|l| collate_key(l));
        items
    }

    /// Symbol items filtered by active category, custom favorites, and search query.
    pub fn symbol_items(
        &self,
        ix: &SymIndex,
        favorites: Option<&[String]>,
    ) -> Vec<crate::symlib::SymItem> {
        let q = self.symbol_search.trim().to_lowercase();
        let matches_query = |item: &crate::symlib::SymItem| {
            q.is_empty()
                || item.label.to_lowercase().contains(&q)
                || item.id.to_lowercase().contains(&q)
        };
        match (self.symbol_category.as_str(), favorites) {
            ("favorites", Some(list)) => list
                .iter()
                .filter_map(|id| ix.by_id(id))
                .filter(matches_query)
                .collect(),
            ("favorites", None) => ix
                .favorites
                .iter()
                .filter(|item| matches_query(item))
                .cloned()
                .collect(),
            ("__all__", _) => ix
                .categories
                .iter()
                .flat_map(|c| c.items.iter())
                .filter(|item| matches_query(item))
                .cloned()
                .collect(),
            (key, _) => ix
                .items(key)
                .iter()
                .filter(|item| matches_query(item))
                .cloned()
                .collect(),
        }
    }

    pub fn update(&mut self, msg: DockMsg) -> Effect {
        match msg {
            // showPanel(): tab clicks only act on inactive tabs and open the dock.
            DockMsg::Tab(tab) => {
                if self.active != tab {
                    self.active = tab;
                    self.open = true;
                }
            }
            DockMsg::Show(tab) => {
                self.active = tab;
                self.open = true;
            }
            DockMsg::Collapse => {
                self.open = false;
                self.handle_hover = false;
            }
            DockMsg::Expand => self.open = true,
            DockMsg::TogglePin => {
                self.pinned = !self.pinned;
                return Effect::Toast(
                    if self.pinned {
                        "Dock stays open after placing."
                    } else {
                        "Dock hides after placing."
                    }
                    .into(),
                );
            }
            DockMsg::ResizeStart => {
                if self.open {
                    self.resizing = true;
                }
            }
            DockMsg::ResizeReset => self.width = None,
            DockMsg::HandleHover(h) => self.handle_hover = h,
            DockMsg::TextSearch(q) => {
                self.text_search = q;
                self.text_active = None;
            }
            DockMsg::TextSubmit => {
                let value = self.text_search.trim().to_string();
                if !value.is_empty() {
                    self.text_active = Some(value.clone());
                    return Effect::Toast(format!("Click the drawing to place {}.", value));
                    // placeholder “{value}”."));
                }
            }
            DockMsg::TextRange(r) => {
                self.text_range = r;
                self.text_active = None;
            }
            // startTextPlacement(label) arrives with screen 9.
            DockMsg::TextPick(label) => self.text_active = Some(label),
            DockMsg::SymbolCategory(opt) => self.symbol_category = opt.key,
            DockMsg::SymbolSearch(q) => self.symbol_search = q,
            // startSymbolPlacement(id) arrives with screen 8; it re-renders the
            // list, so no row keeps `.active` — only the preview stays.
            DockMsg::SymbolPick(id) => self.preview = Some(id),
            // The App opens the two library managers before the dock sees
            // these.
            DockMsg::EditTextLibrary | DockMsg::OrganizeSymbols => {}
            DockMsg::Noop => {}
        }
        Effect::None
    }
}

/// `paintPalette`'s `labels` — the short group display names the live palette
/// uses (not the long `groupNames` map from `populateAreaCodes`, which targets
/// a `.area-tree` DOM that does not exist in the original).
pub fn palette_group_label(group: &str) -> &'static str {
    match group {
        "GLA" => "Living space",
        "GBA" => "Building area",
        "GAR" => "Garage / carport",
        "BSMT" => "Basement",
        "P/P" => "Outdoor",
        "OTH" => "Storage",
        "LAND" => "Land",
        "SITE" => "Site",
        "NCA" => "Non-calculated",
        "UND" => "Undefined",
        _ => "",
    }
}

/// One `.area-tile-group` of the live palette.
#[derive(Debug, Clone, PartialEq)]
pub struct AreaGroup {
    /// The raw group code (GLA, GAR, OTH…).
    pub group: String,
    /// Display label — `groupLabel(group)`, falling back to the group code.
    pub label: String,
    /// The codes in it, in the table's order — `(name, code, colour)`.
    pub tiles: Vec<(String, String, u32)>,
}

/// The palette model: the group rows plus the trailing "Undefined · clear"
/// tile. Empty groups are skipped, as `paintPalette` does.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub groups: Vec<AreaGroup>,
    /// The trailing clear tile — `(name, code, colour)`. `None` when the UND
    /// entry is missing from the table and the fallback provides it.
    pub clear: Option<(String, String, u32)>,
}

/// `codeColor`: `entry.color || entry.fillColor || '#858585'`.
pub(crate) fn code_color(entry: &sketch_core::geometry::AreaCode) -> u32 {
    let hex_or = |s: &str| u32::from_str_radix(s.trim_start_matches('#'), 16).ok();
    hex_or(&entry.color)
        .or_else(|| hex_or(&entry.fill_color))
        .unwrap_or(0x85_85_85)
}

/// `paintPalette` — the palette's model: groups (UND excluded) plus the
/// trailing "Undefined · clear" tile.
pub fn paint_palette(codes: &[sketch_core::geometry::AreaCode]) -> Palette {
    let mut model = Palette {
        groups: Vec::new(),
        clear: None,
    };
    // areaCodes is not group-contiguous (OTH sits either side of LAND/SITE),
    // so collect each group rather than heading on adjacency.
    let mut seen: Vec<&str> = Vec::new();
    for entry in codes.iter().filter(|e| e.group != "UND") {
        if seen.iter().any(|s| *s == entry.group) {
            continue;
        }
        seen.push(&entry.group);
    }
    for group in seen {
        let tiles: Vec<(String, String, u32)> = codes
            .iter()
            .filter(|e| e.code != "UND" && e.group == group)
            .map(|e| {
                (
                    if e.name.is_empty() {
                        e.code.clone()
                    } else {
                        e.name.clone()
                    },
                    e.code.clone(),
                    code_color(e),
                )
            })
            .collect();
        if tiles.is_empty() {
            continue;
        }
        // `groupLabel(group) { return labels[group] || group; }`
        let label = match palette_group_label(group) {
            "" => group.to_string(),
            known => known.to_string(),
        };
        model.groups.push(AreaGroup {
            group: group.to_string(),
            label,
            tiles,
        });
    }
    // The trailing clear tile: the UND entry or the fallback.
    let undefined_entry = codes.iter().find(|c| c.code == "UND").cloned().unwrap_or(
        sketch_core::geometry::AreaCode {
            code: "UND".into(),
            name: "Undefined Area".into(),
            color: "#E53E3E".into(),
            ..Default::default()
        },
    );
    let clear_color = code_color(&undefined_entry);
    model.clear = Some((
        "Undefined · clear".to_string(),
        "UND".to_string(),
        clear_color,
    ));
    model
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_library_sorts_and_filters_like_ui_manager() {
        let mut st = DockState::default();
        let all = st.text_items();
        // The live original's order (localeCompare): ½ Bath first.
        assert_eq!(all.len(), 25);
        assert_eq!(&all[..4], &["½ Bath", "Atrium", "Bath", "Bed"]);
        assert_eq!(all.last(), Some(&"Wood Deck"));
        st.update(DockMsg::TextRange("abc"));
        assert_eq!(
            st.text_items(),
            ["Atrium", "Bath", "Bed", "Breakfast", "Cabin", "Carport"]
        );
        st.update(DockMsg::TextSearch("ba".into()));
        assert_eq!(st.text_items(), ["Bath"]);
        st.update(DockMsg::TextRange("all"));
        assert_eq!(st.text_items(), ["½ Bath", "Bath"]);
        st.update(DockMsg::TextSearch("zzz".into()));
        assert!(st.text_items().is_empty());
    }

    #[test]
    fn resize_clamps_like_the_resize_layer() {
        assert_eq!(clamp_width(390.0, 1400.0), 390.0);
        assert_eq!(clamp_width(410.0, 1400.0), 400.0);
        assert_eq!(clamp_width(100.0, 1400.0), 320.0);
        // Canvas keeps >= 220px: at 560px the dock may not exceed 340.
        assert_eq!(clamp_width(400.0, 560.0), 340.0);
        assert_eq!(clamp_width(400.0, 400.0), 320.0);
        let mut st = DockState::default();
        st.drag_to(990.0, 1400.0);
        assert_eq!(st.width, None, "moves only count after ResizeStart");
        st.update(DockMsg::ResizeStart);
        st.drag_to(990.0, 1400.0);
        assert_eq!(st.width(), 400.0);
        st.drag_to(1050.0, 1400.0);
        assert_eq!(st.width(), 350.0);
        st.end_drag();
        st.drag_to(990.0, 1400.0);
        assert_eq!(st.width(), 350.0);
        st.update(DockMsg::ResizeReset);
        assert_eq!(st.width(), DEFAULT_W);
    }

    #[test]
    fn collapse_pin_and_tabs_follow_show_panel() {
        let mut st = DockState::default();
        st.update(DockMsg::Collapse);
        assert_eq!(st.width(), COLLAPSED_W);
        st.update(DockMsg::ResizeStart);
        assert!(!st.resizing, "no resize while collapsed");
        st.update(DockMsg::Expand);
        assert_eq!(st.width(), DEFAULT_W);
        st.update(DockMsg::Collapse);
        st.update(DockMsg::Tab(DockTab::Calcs));
        assert!(st.open, "a tab click opens the dock");
        assert_eq!(st.active.key(), "calcs");
        assert_eq!(
            st.update(DockMsg::TogglePin),
            Effect::Toast("Dock hides after placing.".into())
        );
        assert!(!st.pinned);
        assert_eq!(
            st.update(DockMsg::TogglePin),
            Effect::Toast("Dock stays open after placing.".into())
        );
    }
}

#[cfg(test)]
mod area_tree_tests {
    use super::*;
    use sketch_core::geometry::area_codes::default_area_codes;

    #[test]
    fn the_palette_uses_the_short_group_labels() {
        assert_eq!(palette_group_label("GLA"), "Living space");
        assert_eq!(palette_group_label("GBA"), "Building area");
        assert_eq!(palette_group_label("GAR"), "Garage / carport");
        assert_eq!(palette_group_label("P/P"), "Outdoor");
        assert_eq!(palette_group_label("NCA"), "Non-calculated");
        // An unknown group falls back to its code, as groupLabel does.
        assert_eq!(palette_group_label("XX"), "");
    }

    #[test]
    fn the_default_table_groups_in_first_seen_order_without_und() {
        let model = paint_palette(&default_area_codes());
        let groups: Vec<&str> = model.groups.iter().map(|g| g.group.as_str()).collect();
        assert_eq!(
            groups,
            vec!["GLA", "GBA", "GAR", "BSMT", "P/P", "OTH", "LAND", "SITE", "NCA"]
        );
        let gla = &model.groups[0];
        assert_eq!(gla.label, "Living space");
        assert_eq!(gla.tiles.len(), 4);
        assert_eq!(gla.tiles[0].0, "First Floor");
        assert_eq!(gla.tiles[0].1, "GLA1");
    }

    #[test]
    fn und_is_the_trailing_clear_tile_not_a_group() {
        let model = paint_palette(&default_area_codes());
        let (name, code, color) = model.clear.expect("clear tile");
        assert_eq!(name, "Undefined · clear");
        assert_eq!(code, "UND");
        assert_eq!(color, 0xe53e3e, "the default table's UND colour");
        assert!(!model.groups.iter().any(|g| g.group == "UND"));
    }

    #[test]
    fn an_und_entry_missing_from_the_table_falls_back() {
        let codes: Vec<_> = default_area_codes()
            .into_iter()
            .filter(|c| c.code != "UND")
            .collect();
        let model = paint_palette(&codes);
        let (name, code, color) = model.clear.expect("clear tile");
        assert_eq!(name, "Undefined · clear");
        assert_eq!(code, "UND");
        assert_eq!(color, 0xe53e3e, "the fallback's colour");
    }

    #[test]
    fn code_color_falls_back_through_the_fill_to_the_default() {
        let mut entry = default_area_codes()
            .into_iter()
            .find(|c| c.code == "OTH")
            .expect("OTH");
        assert_eq!(code_color(&entry), 0xa0aec0);
        entry.color = String::new();
        // Falls back to fillColor when color is empty.
        let via_fill = if entry.fill_color.is_empty() {
            0x85_85_85
        } else {
            u32::from_str_radix(entry.fill_color.trim_start_matches('#'), 16).expect("fill")
        };
        assert_eq!(code_color(&entry), via_fill);
        entry.fill_color = String::new();
        assert_eq!(code_color(&entry), 0x85_85_85, "then to #858585");
        // A full hex string is parsed; the '#' is optional.
        entry.color = "#2B6CB0".into();
        assert_eq!(code_color(&entry), 0x2b6cb0);
    }

    #[test]
    fn empty_groups_are_skipped_and_labels_fall_back_to_the_code() {
        let codes: Vec<_> = default_area_codes()
            .into_iter()
            .filter(|c| c.group != "GAR")
            .collect();
        let model = paint_palette(&codes);
        assert!(!model.groups.iter().any(|g| g.group == "GAR"));
        // A group the label map doesn't know shows just its code.
        let mut entry = default_area_codes()
            .into_iter()
            .find(|c| c.code == "OTH")
            .expect("OTH");
        entry.group = "XX".into();
        entry.code = "XX1".into();
        let model = paint_palette(std::slice::from_ref(&entry));
        assert_eq!(model.groups[0].label, "XX");
        assert_eq!(model.groups[0].tiles[0].0, "Storage");
    }
}
