//! Unified Area Type Editor for the Next shell (`docs/ui-redesign/05-area-types.md`).
//!
//! Replaces classic `codes_editor`, `def_editor`, and `new_code_dialog` in
//! `Shell::Next` with a two-column modal (`1080×740` clamped on Wide/Medium,
//! list -> `< Back` form on Compact).

use std::collections::{BTreeMap, BTreeSet};

use iced::widget::canvas::{self, Canvas, Frame, Path, Stroke};
use iced::widget::{
    button, checkbox, column, container, mouse_area, opaque, pick_list, row, scrollable, slider,
    stack, text, text_input, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Padding, Point, Rectangle, Size};
use sketch_core::geometry::{area_codes, AreaCode};

use super::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_MEDIUM};
use super::kit::{self, FieldProps, Segment};
use super::layout::SizeClass;
use super::tokens::{
    self, Density, RADIUS_CONTROL, RADIUS_DIALOG, SPACE_1, SPACE_2, SPACE_3, SPACE_4, TEXT_11,
    TEXT_12, TEXT_13, TEXT_14, TEXT_16,
};
use crate::dock::palette_group_label;
use crate::icons::mono;
use crate::new_code_dialog::{style_label, style_value, FILL_STYLES};
use crate::render::{parse_css_color, parse_hex};
use crate::Message;

const MODAL_MAX_W: f32 = 1080.0;
const MODAL_MAX_H: f32 = 740.0;
const KNOWN_GROUPS: [&str; 10] = [
    "GLA", "GBA", "GAR", "BSMT", "P/P", "OTH", "LAND", "SITE", "NCA", "UND",
];
const NEW_GROUP_LABEL: &str = "New group...";

#[derive(Debug, Clone, PartialEq)]
pub struct DraftEntry {
    /// Code this row started with when the editor opened (`None` for newly
    /// created, duplicated, or imported entries not matching an original row).
    pub origin_code: Option<String>,
    pub area: AreaCode,
    pub multiplier_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmKind {
    Delete(String),
    RestoreDefaults,
    Discard,
}

#[derive(Debug, Clone)]
pub enum RemapTarget {
    Deleted,
    Updated {
        new_code: String,
        new_name: String,
        old_name: String,
        new_color: String,
        code_changed: bool,
        name_changed: bool,
        color_changed: bool,
    },
}

#[derive(Debug, Clone)]
pub struct SaveOutcome {
    pub codes: Vec<AreaCode>,
    pub remaps: BTreeMap<String, RemapTarget>,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Search(String),
    Select(usize),
    CompactBack,
    MoveUp(usize),
    MoveDown(usize),
    NewType,
    Duplicate,
    RequestDelete,
    ConfirmDelete,
    CancelConfirm,
    Code(String),
    Name(String),
    Description(String),
    GroupSelected(String),
    CustomGroupInput(String),
    ApplyCustomGroup,
    Multiplier(String),
    Sign(bool),
    IncludeGla(bool),
    PickLineColor,
    LineHex(String),
    PickFillColor,
    FillHex(String),
    FillStyle(String),
    FillOpacity(f32),
    Import,
    Export,
    ImportChosen(u64, Option<std::path::PathBuf>),
    ExportChosen {
        session: u64,
        body: String,
        path: Option<std::path::PathBuf>,
    },
    RequestRestoreDefaults,
    ConfirmRestoreDefaults,
    RequestCancel,
    ConfirmDiscard,
    Save,
}

#[derive(Debug, Clone, Default)]
pub struct TypeEditor {
    pub open: bool,
    pub session: u64,
    pub drafts: Vec<DraftEntry>,
    pub initial: Vec<AreaCode>,
    pub selected: usize,
    pub search: String,
    pub compact_showing_form: bool,
    pub adding_group: bool,
    pub custom_group_input: String,
    pub confirm: Option<ConfirmKind>,
    pub usage_counts: BTreeMap<String, usize>,
    pub builtin_codes: BTreeSet<String>,
}

impl TypeEditor {
    pub fn open_list(&mut self, codes: &[AreaCode], counts: BTreeMap<String, usize>, select: Option<&str>) {
        self.reset_state(codes, counts);
        if let Some(target) = select {
            if let Some(idx) = self.drafts.iter().position(|d| d.area.code == target) {
                self.selected = idx;
                self.compact_showing_form = true;
            }
        }
    }

    pub fn open_new(&mut self, codes: &[AreaCode], counts: BTreeMap<String, usize>, group: &str) {
        self.reset_state(codes, counts);
        self.add_new_type(group);
        self.compact_showing_form = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.confirm = None;
        self.adding_group = false;
        self.custom_group_input.clear();
        self.session = self.session.wrapping_add(1);
    }

    fn reset_state(&mut self, codes: &[AreaCode], counts: BTreeMap<String, usize>) {
        self.session = self.session.wrapping_add(1);
        self.open = true;
        self.initial = codes.to_vec();
        self.drafts = codes.iter().cloned().map(DraftEntry::from_existing).collect();
        self.selected = 0;
        self.search.clear();
        self.compact_showing_form = false;
        self.adding_group = false;
        self.custom_group_input.clear();
        self.confirm = None;
        self.usage_counts = counts;
        self.builtin_codes = area_codes::default_area_codes()
            .into_iter()
            .map(|c| c.code)
            .collect();
    }

    pub fn is_dirty(&self) -> bool {
        if self.drafts.len() != self.initial.len() {
            return true;
        }
        self.drafts.iter().zip(&self.initial).any(|(d, orig)| {
            d.area != *orig || d.origin_code.as_deref() != Some(orig.code.as_str())
        })
    }

    pub fn set_line_color(&mut self, hex: String) {
        if let Some(d) = self.drafts.get_mut(self.selected) {
            d.area.color = normalize_hex_input(&hex);
        }
    }

    pub fn set_fill_color(&mut self, hex: String) {
        if let Some(d) = self.drafts.get_mut(self.selected) {
            d.area.fill_color = normalize_hex_input(&hex);
            sync_css_fill(&mut d.area);
        }
    }

    pub fn current_line_color(&self) -> String {
        self.drafts
            .get(self.selected)
            .map(|d| fallback_hex(&d.area.color, "#3182CE"))
            .unwrap_or_else(|| "#3182CE".to_string())
    }

    pub fn current_fill_color(&self) -> String {
        self.drafts
            .get(self.selected)
            .map(|d| fallback_hex(&d.area.fill_color, "#4299E1"))
            .unwrap_or_else(|| "#4299E1".to_string())
    }

    pub fn usage_for_entry(&self, entry: &DraftEntry) -> usize {
        entry
            .origin_code
            .as_deref()
            .and_then(|c| self.usage_counts.get(c).copied())
            .unwrap_or(0)
    }

    pub fn validation_error(&self) -> Option<String> {
        if self.drafts.is_empty() {
            return Some("At least one area type is required.".to_string());
        }
        let mut seen = BTreeSet::new();
        for (i, d) in self.drafts.iter().enumerate() {
            let code = d.area.code.trim();
            if code.is_empty() {
                return Some(format!("Row {}: Code cannot be empty.", i + 1));
            }
            if !is_valid_code(code) {
                return Some(format!(
                    "Type \"{code}\": use only uppercase letters, digits, hyphens, or underscores."
                ));
            }
            if !seen.insert(code.to_string()) {
                return Some(format!("Duplicate code \"{code}\"."));
            }
            match d.multiplier_text.trim().parse::<f64>() {
                Ok(m) if m.is_finite() && m >= 0.0 => {}
                _ => {
                    return Some(format!(
                        "Type \"{code}\": multiplier must be a non-negative number."
                    ));
                }
            }
        }
        if !seen.contains("UND") {
            return Some("Required fallback type \"UND\" must remain in the list.".to_string());
        }
        None
    }

    pub fn validated_codes(&self) -> Result<Vec<AreaCode>, String> {
        if let Some(err) = self.validation_error() {
            return Err(err);
        }
        let prepared: Vec<AreaCode> = self
            .drafts
            .iter()
            .map(|d| {
                let mut area = d.area.clone();
                area.code = area.code.trim().to_ascii_uppercase();
                area.name = area.name.trim().to_string();
                if area.name.is_empty() {
                    area.name = area.code.clone();
                }
                area.description = area.description.trim().to_string();
                area.group = area.group.trim().to_ascii_uppercase();
                if area.group.is_empty() {
                    area.group = "OTH".to_string();
                }
                area.multiplier = d.multiplier_text.trim().parse::<f64>().unwrap_or(1.0);
                area.color = fallback_hex(&area.color, "#718096");
                area.fill_color = fallback_hex(&area.fill_color, &area.color);
                sync_css_fill(&mut area);
                area
            })
            .collect();
        area_codes::validate_area_codes(prepared).map_err(|e| e.to_string())
    }

    pub fn build_save_outcome(&self) -> Result<SaveOutcome, String> {
        let codes = self.validated_codes()?;
        let initial_by_code: BTreeMap<&str, &AreaCode> = self
            .initial
            .iter()
            .map(|c| (c.code.as_str(), c))
            .collect();
        let mut by_origin: BTreeMap<String, &AreaCode> = BTreeMap::new();
        for (draft, clean) in self.drafts.iter().zip(&codes) {
            if let Some(orig) = &draft.origin_code {
                by_origin.insert(orig.clone(), clean);
            }
        }
        let mut remaps = BTreeMap::new();
        for (orig_code, orig) in initial_by_code {
            match by_origin.get(orig_code) {
                Some(next) => {
                    let code_changed = orig.code != next.code;
                    let name_changed = orig.name != next.name;
                    let color_changed = !orig.color.eq_ignore_ascii_case(&next.color);
                    if code_changed || name_changed || color_changed {
                        remaps.insert(
                            orig_code.to_string(),
                            RemapTarget::Updated {
                                new_code: next.code.clone(),
                                new_name: next.name.clone(),
                                old_name: orig.name.clone(),
                                new_color: next.color.clone(),
                                code_changed,
                                name_changed,
                                color_changed,
                            },
                        );
                    }
                }
                None => {
                    remaps.insert(orig_code.to_string(), RemapTarget::Deleted);
                }
            }
        }
        Ok(SaveOutcome { codes, remaps })
    }

    pub fn load_imported_codes(&mut self, imported: Vec<AreaCode>) {
        let orig_set: BTreeSet<String> = self.initial.iter().map(|c| c.code.clone()).collect();
        self.drafts = imported
            .into_iter()
            .map(|area| {
                let origin = orig_set.contains(&area.code).then(|| area.code.clone());
                DraftEntry::from_parts(origin, area)
            })
            .collect();
        self.selected = 0;
        self.adding_group = false;
        self.confirm = None;
        self.compact_showing_form = false;
    }

    pub fn apply_local(&mut self, msg: Msg) {
        match msg {
            Msg::Search(v) => self.search = v,
            Msg::Select(idx) => {
                if idx < self.drafts.len() {
                    self.selected = idx;
                    self.adding_group = false;
                    self.confirm = None;
                    self.compact_showing_form = true;
                }
            }
            Msg::CompactBack => {
                self.compact_showing_form = false;
                self.adding_group = false;
                self.confirm = None;
            }
            Msg::MoveUp(idx) => self.move_within_group(idx, -1),
            Msg::MoveDown(idx) => self.move_within_group(idx, 1),
            Msg::NewType => {
                let group = self
                    .drafts
                    .get(self.selected)
                    .map(|d| d.area.group.clone())
                    .filter(|g| !g.is_empty())
                    .unwrap_or_else(|| "OTH".to_string());
                self.add_new_type(&group);
                self.compact_showing_form = true;
                self.adding_group = false;
                self.confirm = None;
            }
            Msg::Duplicate => {
                self.duplicate_selected();
                self.compact_showing_form = true;
                self.adding_group = false;
                self.confirm = None;
            }
            Msg::RequestDelete => {
                if let Some(d) = self.drafts.get(self.selected) {
                    if d.area.code == "UND" {
                        return;
                    }
                    let used = self.usage_for_entry(d);
                    if used > 0 {
                        self.confirm = Some(ConfirmKind::Delete(d.area.code.clone()));
                    } else {
                        self.delete_selected();
                    }
                }
            }
            Msg::ConfirmDelete => {
                self.delete_selected();
                self.confirm = None;
            }
            Msg::CancelConfirm => self.confirm = None,
            Msg::Code(v) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    if d.origin_code.as_deref() == Some("UND") {
                        return;
                    }
                    d.area.code = v.to_ascii_uppercase();
                }
            }
            Msg::Name(v) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.area.name = v;
                }
            }
            Msg::Description(v) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.area.description = v;
                }
            }
            Msg::GroupSelected(label) => {
                if label == NEW_GROUP_LABEL {
                    self.adding_group = true;
                    self.custom_group_input.clear();
                } else {
                    self.adding_group = false;
                    let group = group_key_from_option(&label);
                    if let Some(d) = self.drafts.get_mut(self.selected) {
                        d.area.group = group;
                    }
                }
            }
            Msg::CustomGroupInput(v) => {
                self.custom_group_input = v.to_ascii_uppercase();
            }
            Msg::ApplyCustomGroup => {
                let g = self.custom_group_input.trim().to_ascii_uppercase();
                if !g.is_empty() {
                    if let Some(d) = self.drafts.get_mut(self.selected) {
                        d.area.group = g;
                    }
                }
                self.adding_group = false;
                self.custom_group_input.clear();
            }
            Msg::Multiplier(v) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.multiplier_text = v.clone();
                    if let Ok(parsed) = v.trim().parse::<f64>() {
                        if parsed.is_finite() && parsed >= 0.0 {
                            d.area.multiplier = parsed;
                        }
                    }
                }
            }
            Msg::Sign(positive) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.area.positive = positive;
                }
            }
            Msg::IncludeGla(v) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.area.include_in_gla = v;
                }
            }
            Msg::LineHex(v) => self.set_line_color(v),
            Msg::FillHex(v) => self.set_fill_color(v),
            Msg::FillStyle(label) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.area.fill_style = style_value(&label).to_string();
                }
            }
            Msg::FillOpacity(pct) => {
                if let Some(d) = self.drafts.get_mut(self.selected) {
                    d.area.fill_opacity = f64::from(pct.clamp(0.0, 100.0)).round();
                    sync_css_fill(&mut d.area);
                }
            }
            Msg::RequestRestoreDefaults => {
                self.confirm = Some(ConfirmKind::RestoreDefaults);
            }
            Msg::ConfirmRestoreDefaults => {
                let orig_set: BTreeSet<String> = self.initial.iter().map(|c| c.code.clone()).collect();
                self.drafts = area_codes::default_area_codes()
                    .into_iter()
                    .map(|area| {
                        let origin = orig_set.contains(&area.code).then(|| area.code.clone());
                        DraftEntry::from_parts(origin, area)
                    })
                    .collect();
                self.selected = 0;
                self.adding_group = false;
                self.confirm = None;
                self.compact_showing_form = false;
            }
            Msg::RequestCancel | Msg::ConfirmDiscard => {}
            Msg::PickLineColor
            | Msg::PickFillColor
            | Msg::Import
            | Msg::Export
            | Msg::ImportChosen(_, _)
            | Msg::ExportChosen { .. }
            | Msg::Save => {}
        }
    }

    fn add_new_type(&mut self, group: &str) {
        let code = unique_code(&self.drafts, "NEW");
        let area = AreaCode {
            code: code.clone(),
            name: "New Type".to_string(),
            description: String::new(),
            group: group.to_string(),
            multiplier: 1.0,
            positive: true,
            include_in_gla: false,
            color: "#3182CE".to_string(),
            fill: "rgba(49,130,206,.10)".to_string(),
            fill_color: "#3182CE".to_string(),
            fill_style: "solid".to_string(),
            fill_opacity: 10.0,
        };
        let insert_at = self
            .drafts
            .iter()
            .rposition(|d| d.area.group == group)
            .map(|i| i + 1)
            .unwrap_or(self.drafts.len());
        self.drafts
            .insert(insert_at, DraftEntry::from_parts(None, area));
        self.selected = insert_at;
    }

    fn duplicate_selected(&mut self) {
        let Some(src) = self.drafts.get(self.selected).cloned() else {
            return;
        };
        let base = format!("{}_COPY", src.area.code);
        let code = unique_code(&self.drafts, &base);
        let mut area = src.area.clone();
        area.code = code;
        area.name = if src.area.name.is_empty() {
            area.code.clone()
        } else {
            format!("{} Copy", src.area.name)
        };
        let insert_at = self.selected + 1;
        self.drafts
            .insert(insert_at, DraftEntry::from_parts(None, area));
        self.selected = insert_at;
    }

    fn delete_selected(&mut self) {
        if self.selected >= self.drafts.len() {
            return;
        }
        if self.drafts[self.selected].area.code == "UND" {
            return;
        }
        self.drafts.remove(self.selected);
        if self.selected >= self.drafts.len() && !self.drafts.is_empty() {
            self.selected = self.drafts.len() - 1;
        }
        self.compact_showing_form = false;
    }

    fn move_within_group(&mut self, idx: usize, delta: isize) {
        let Some(group) = self.drafts.get(idx).map(|d| d.area.group.clone()) else {
            return;
        };
        let group_indices: Vec<usize> = self
            .drafts
            .iter()
            .enumerate()
            .filter_map(|(i, d)| (d.area.group == group).then_some(i))
            .collect();
        let Some(pos) = group_indices.iter().position(|&i| i == idx) else {
            return;
        };
        let target_pos = pos as isize + delta;
        if target_pos < 0 || target_pos as usize >= group_indices.len() {
            return;
        }
        let swap_idx = group_indices[target_pos as usize];
        self.drafts.swap(idx, swap_idx);
        if self.selected == idx {
            self.selected = swap_idx;
        } else if self.selected == swap_idx {
            self.selected = idx;
        }
    }
}

impl DraftEntry {
    fn from_existing(area: AreaCode) -> Self {
        let origin = area.code.clone();
        Self::from_parts(Some(origin), area)
    }

    fn from_parts(origin_code: Option<String>, mut area: AreaCode) -> Self {
        if area.fill_style.is_empty() {
            area.fill_style = "solid".to_string();
        }
        if area.fill_color.is_empty() {
            area.fill_color = fallback_hex(&area.color, "#4299E1");
        }
        let multiplier_text = format_multiplier(area.multiplier);
        Self {
            origin_code,
            area,
            multiplier_text,
        }
    }
}

fn format_multiplier(v: f64) -> String {
    if !v.is_finite() {
        return "1".to_string();
    }
    if (v.fract()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.4}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn unique_code(drafts: &[DraftEntry], prefix: &str) -> String {
    let existing: BTreeSet<&str> = drafts.iter().map(|d| d.area.code.as_str()).collect();
    if !existing.contains(prefix) {
        return prefix.to_string();
    }
    let mut n = 1usize;
    loop {
        let candidate = format!("{prefix}{n}");
        if !existing.contains(candidate.as_str()) {
            return candidate;
        }
        n += 1;
    }
}

fn is_valid_code(code: &str) -> bool {
    !code.is_empty()
        && code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn normalize_hex_input(raw: &str) -> String {
    let trimmed = raw.trim().to_ascii_uppercase();
    if trimmed.is_empty() {
        String::new()
    } else if trimmed.starts_with('#') {
        trimmed
    } else {
        format!("#{trimmed}")
    }
}

fn fallback_hex(candidate: &str, default: &str) -> String {
    if parse_hex(candidate.trim()).is_some() {
        candidate.trim().to_ascii_uppercase()
    } else {
        default.to_string()
    }
}

fn sync_css_fill(area: &mut AreaCode) {
    let base = parse_hex(area.fill_color.trim())
        .or_else(|| parse_hex(area.color.trim()))
        .unwrap_or_else(|| Color::from_rgb8(0x42, 0x99, 0xE1));
    let r = (base.r * 255.0).round() as u8;
    let g = (base.g * 255.0).round() as u8;
    let b = (base.b * 255.0).round() as u8;
    let a = (area.fill_opacity / 100.0).clamp(0.0, 1.0);
    let alpha = format!("{a:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string();
    area.fill = format!(
        "rgba({r},{g},{b},{})",
        if alpha == "0" { "0" } else { &alpha }
    );
}

fn group_option_label(key: &str) -> String {
    let label = palette_group_label(key);
    if KNOWN_GROUPS.contains(&key) {
        format!("{key} - {label}")
    } else {
        key.to_string()
    }
}

fn group_key_from_option(opt: &str) -> String {
    opt.split(" - ")
        .next()
        .unwrap_or(opt)
        .trim()
        .to_ascii_uppercase()
}

fn group_options(drafts: &[DraftEntry]) -> Vec<String> {
    let mut keys: Vec<String> = KNOWN_GROUPS.iter().map(|s| (*s).to_string()).collect();
    let mut custom: BTreeSet<String> = BTreeSet::new();
    for d in drafts {
        let g = d.area.group.trim().to_ascii_uppercase();
        if !g.is_empty() && !KNOWN_GROUPS.contains(&g.as_str()) {
            custom.insert(g);
        }
    }
    keys.extend(custom);
    let mut out: Vec<String> = keys.iter().map(|k| group_option_label(k)).collect();
    out.push(NEW_GROUP_LABEL.to_string());
    out
}

fn send(m: Msg) -> Message {
    Message::Ui(super::Msg::TypeEditor(m))
}

pub fn layer<'a>(
    editor: &'a TypeEditor,
    viewport: Size,
    size_class: SizeClass,
    d: Density,
) -> Option<Element<'a, Message>> {
    if !editor.open {
        return None;
    }
    let compact = size_class == SizeClass::Compact;

    let card: Element<'a, Message> = if compact {
        compact_view(editor, d)
    } else {
        modal_view(editor, size_class, d)
    };

    if compact {
        return Some(opaque(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(tokens::panel())),
                    ..container::Style::default()
                }),
        ));
    }

    let modal_w = MODAL_MAX_W.min((viewport.width - 32.0).max(320.0));
    let modal_h = MODAL_MAX_H.min((viewport.height - 32.0).max(320.0));

    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::scrim())),
                ..container::Style::default()
            }),
    )
    .on_press(send(Msg::RequestCancel));

    let centered = container(
        opaque(
            container(card)
                .width(Length::Fixed(modal_w))
                .height(Length::Fixed(modal_h))
                .style(|_| container::Style {
                    background: Some(Background::Color(tokens::panel())),
                    border: Border {
                        color: tokens::rule(),
                        width: 1.0,
                        radius: RADIUS_DIALOG.into(),
                    },
                    shadow: tokens::floating_shadow(),
                    ..container::Style::default()
                }),
        ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center);

    Some(stack![backdrop, centered].into())
}

fn modal_view<'a>(editor: &'a TypeEditor, size_class: SizeClass, d: Density) -> Element<'a, Message> {
    let left_w = if size_class == SizeClass::Wide {
        296.0
    } else {
        252.0
    };
    let header = dialog_header(d, false);
    let v_sep = container(Space::new().width(1.0).height(Length::Fill)).style(|_| container::Style {
        background: Some(Background::Color(tokens::rule())),
        ..container::Style::default()
    });
    let body = row![
        container(list_pane(editor, d))
            .width(Length::Fixed(left_w))
            .height(Length::Fill),
        v_sep,
        container(form_pane(editor, d, false))
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .height(Length::Fill);

    column![
        header,
        kit::divider(),
        container(body).width(Length::Fill).height(Length::Fill),
        kit::divider(),
        footer_bar(editor, d, false),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn compact_view<'a>(editor: &'a TypeEditor, d: Density) -> Element<'a, Message> {
    let header = dialog_header(d, editor.compact_showing_form);
    let body: Element<'a, Message> = if editor.compact_showing_form {
        form_pane(editor, d, true)
    } else {
        list_pane(editor, d)
    };
    column![
        header,
        kit::divider(),
        container(body).width(Length::Fill).height(Length::Fill),
        kit::divider(),
        footer_bar(editor, d, true),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn dialog_header<'a>(d: Density, compact_form: bool) -> Element<'a, Message> {
    let left: Element<'a, Message> = if compact_form {
        kit::text_button("< Back", Some(send(Msg::CompactBack)), d)
    } else {
        text("Area types")
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_16)
            .color(tokens::text())
            .into()
    };

    let close_btn = kit::tooltip(
        kit::icon_button("x", false, Some(send(Msg::RequestCancel)), d),
        "Close",
        iced::widget::tooltip::Position::Bottom,
    );

    let r = if compact_form {
        row![
            left,
            Space::new().width(SPACE_2),
            text("Edit area type")
                .font(BARLOW_SEMIBOLD)
                .size(TEXT_16)
                .color(tokens::text()),
            Space::new().width(Length::Fill),
            close_btn,
        ]
    } else {
        row![left, Space::new().width(Length::Fill), close_btn]
    };
    r.align_y(Alignment::Center)
        .padding([SPACE_2 + 2.0, SPACE_4])
        .into()
}

fn list_pane<'a>(editor: &'a TypeEditor, d: Density) -> Element<'a, Message> {
    let search_bar = container(
        row![
            mono("magnifying-glass", 14.0, tokens::muted()),
            text_input("Search types by name or code", &editor.search)
                .on_input(|v| send(Msg::Search(v)))
                .font(BARLOW)
                .size(TEXT_13)
                .padding([4, 6])
                .width(Length::Fill)
                .style(|_, _| text_input::Style {
                    background: Background::Color(Color::TRANSPARENT),
                    border: Border::default(),
                    icon: tokens::muted(),
                    placeholder: tokens::muted(),
                    value: tokens::text(),
                    selection: tokens::accent_soft(),
                }),
        ]
        .spacing(SPACE_1)
        .align_y(Alignment::Center)
        .padding([2, 8]),
    )
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::input_bg())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    });

    let q = editor.search.trim().to_ascii_lowercase();
    let mut grouped: Vec<(String, Vec<(usize, &DraftEntry)>)> = Vec::new();
    for (idx, draft) in editor.drafts.iter().enumerate() {
        if !q.is_empty() {
            let code_m = draft.area.code.to_ascii_lowercase().contains(&q);
            let name_m = draft.area.name.to_ascii_lowercase().contains(&q);
            let group_m = draft.area.group.to_ascii_lowercase().contains(&q);
            if !code_m && !name_m && !group_m {
                continue;
            }
        }
        let gkey = if draft.area.group.is_empty() {
            "OTH".to_string()
        } else {
            draft.area.group.to_ascii_uppercase()
        };
        if let Some((_, items)) = grouped.iter_mut().find(|(g, _)| *g == gkey) {
            items.push((idx, draft));
        } else {
            grouped.push((gkey, vec![(idx, draft)]));
        }
    }

    let mut list_col = column![].spacing(2).width(Length::Fill);
    if grouped.is_empty() {
        list_col = list_col.push(
            container(
                text("No matching area types")
                    .font(BARLOW)
                    .size(TEXT_13)
                    .color(tokens::muted()),
            )
            .padding(SPACE_3),
        );
    } else {
        for (gkey, items) in grouped {
            let glabel = palette_group_label(&gkey);
            let group_heading = if glabel.is_empty() {
                gkey.clone()
            } else {
                glabel.to_string()
            };
            list_col = list_col.push(
                container(
                    text(group_heading)
                        .font(BARLOW_SEMIBOLD)
                        .size(TEXT_11)
                        .color(tokens::muted()),
                )
                .padding([SPACE_2, SPACE_2]),
            );
            let group_len = items.len();
            for (pos, (idx, draft)) in items.into_iter().enumerate() {
                list_col = list_col.push(type_list_row(
                    editor,
                    idx,
                    draft,
                    pos > 0,
                    pos + 1 < group_len,
                    d,
                ));
            }
        }
    }

    column![
        container(search_bar).padding([SPACE_2, SPACE_3]),
        kit::divider(),
        scrollable(container(list_col).padding([SPACE_1, SPACE_2]).width(Length::Fill)).id("area-types-list")
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn type_list_row<'a>(
    editor: &'a TypeEditor,
    idx: usize,
    draft: &'a DraftEntry,
    can_up: bool,
    can_down: bool,
    d: Density,
) -> Element<'a, Message> {
    let selected = idx == editor.selected;
    let swatch_col = parse_css_color(&draft.area.color)
        .or_else(|| parse_css_color(&draft.area.fill))
        .unwrap_or_else(tokens::accent);
    let used = editor.usage_for_entry(draft);
    let title = if draft.area.name.trim().is_empty() {
        draft.area.code.clone()
    } else {
        draft.area.name.clone()
    };

    let mut row_content = row![
        container(Space::new().width(10.0).height(10.0)).style(move |_| container::Style {
            background: Some(Background::Color(swatch_col)),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: 3.0.into(),
            },
            ..container::Style::default()
        }),
        column![
            text(title)
                .font(if selected { BARLOW_SEMIBOLD } else { BARLOW_MEDIUM })
                .size(TEXT_13)
                .color(tokens::text()),
            text(if used > 0 {
                format!("{} · {} used", draft.area.code, used)
            } else {
                draft.area.code.clone()
            })
            .font(BSC_MEDIUM)
            .size(TEXT_11)
            .color(tokens::muted()),
        ]
        .spacing(1)
        .width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    if selected {
        let up = kit::tooltip(
            kit::icon_button("arrow-up", false, can_up.then(|| send(Msg::MoveUp(idx))), d),
            "Move up in group",
            iced::widget::tooltip::Position::Top,
        );
        let down = kit::tooltip(
            kit::icon_button(
                "arrow-down",
                false,
                can_down.then(|| send(Msg::MoveDown(idx))),
                d,
            ),
            "Move down in group",
            iced::widget::tooltip::Position::Top,
        );
        row_content = row_content.push(row![up, down].spacing(2).align_y(Alignment::Center));
    }

    button(row_content)
        .on_press(send(Msg::Select(idx)))
        .width(Length::Fill)
        .padding([SPACE_1 + 2.0, SPACE_2])
        .style(move |_, status| {
            let bg = if selected {
                tokens::accent_soft()
            } else if matches!(status, button::Status::Hovered) {
                tokens::hover()
            } else {
                Color::TRANSPARENT
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: tokens::text(),
                border: Border {
                    color: if selected { tokens::accent() } else { Color::TRANSPARENT },
                    width: if selected { 1.0 } else { 0.0 },
                    radius: RADIUS_CONTROL.into(),
                },
                ..button::Style::default()
            }
        })
        .into()
}

fn form_pane<'a>(editor: &'a TypeEditor, d: Density, compact: bool) -> Element<'a, Message> {
    let Some(entry) = editor.drafts.get(editor.selected) else {
        return container(
            text("Select an area type to edit")
                .font(BARLOW)
                .size(TEXT_14)
                .color(tokens::muted()),
        )
        .padding(SPACE_4)
        .into();
    };

    let area = &entry.area;
    let is_und = area.code == "UND" || entry.origin_code.as_deref() == Some("UND");
    let is_builtin = entry
        .origin_code
        .as_deref()
        .is_some_and(|c| editor.builtin_codes.contains(c))
        || editor.builtin_codes.contains(&area.code);
    let used = editor.usage_for_entry(entry);

    let origin_badge = container(
        text(if is_builtin { "Built in" } else { "Added by you" })
            .font(BARLOW_MEDIUM)
            .size(TEXT_11)
            .color(tokens::text()),
    )
    .padding([2, 8])
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::field())),
        border: Border {
            color: tokens::rule(),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    });

    let usage_label = match used {
        0 => "Not used in this sketch".to_string(),
        1 => "Used by 1 area in this sketch".to_string(),
        n => format!("Used by {n} areas in this sketch"),
    };

    let delete_btn: Element<'a, Message> = if is_und {
        row![
            kit::text_button("Delete", None, d),
            text("Required fallback type")
                .font(BARLOW)
                .size(TEXT_11)
                .color(tokens::muted()),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .into()
    } else {
        kit::text_button("Delete", Some(send(Msg::RequestDelete)), d)
    };

    let meta_row = row![
        origin_badge,
        text(usage_label)
            .font(BARLOW)
            .size(TEXT_12)
            .color(tokens::muted()),
        Space::new().width(Length::Fill),
        delete_btn,
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center);

    let mut form = column![meta_row].spacing(SPACE_3).width(Length::Fill);

    if let Some(confirm) = &editor.confirm {
        form = form.push(confirm_banner(editor, confirm, d));
    }

    let code_row = form_field(
        "Code",
        if is_und {
            container(
                text("UND (required fallback)")
                    .font(BARLOW_MEDIUM)
                    .size(TEXT_13)
                    .color(tokens::muted()),
            )
            .padding([6, 8])
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(tokens::field())),
                border: Border {
                    color: tokens::rule(),
                    width: 1.0,
                    radius: RADIUS_CONTROL.into(),
                },
                ..container::Style::default()
            })
            .into()
        } else {
            kit::field(
                FieldProps {
                    label: "",
                    value: &area.code,
                    placeholder: "e.g. GAR",
                    on_input: Some(Box::new(|v| send(Msg::Code(v)))),
                    on_submit: None,
                    focused: false,
                },
                d,
            )
        },
        compact,
    );

    let name_row = form_field(
        "Name",
        kit::field(
            FieldProps {
                label: "",
                value: &area.name,
                placeholder: "Area display name",
                on_input: Some(Box::new(|v| send(Msg::Name(v)))),
                on_submit: None,
                focused: false,
            },
            d,
        ),
        compact,
    );

    let desc_row = form_field(
        "Description",
        kit::field(
            FieldProps {
                label: "",
                value: &area.description,
                placeholder: "Short note shown under the name",
                on_input: Some(Box::new(|v| send(Msg::Description(v)))),
                on_submit: None,
                focused: false,
            },
            d,
        ),
        compact,
    );

    let g_options = group_options(&editor.drafts);
    let selected_group = group_option_label(if area.group.is_empty() {
        "OTH"
    } else {
        &area.group
    });
    let group_pick: Element<'a, Message> = styled_pick_list(
        g_options,
        if editor.adding_group {
            Some(NEW_GROUP_LABEL.to_string())
        } else {
            Some(selected_group)
        },
        |v| send(Msg::GroupSelected(v)),
        d,
    );
    let group_control: Element<'a, Message> = if editor.adding_group {
        column![
            group_pick,
            row![
                kit::field(
                    FieldProps {
                        label: "New group code",
                        value: &editor.custom_group_input,
                        placeholder: "e.g. CUSTOM",
                        on_input: Some(Box::new(|v| send(Msg::CustomGroupInput(v)))),
                        on_submit: Some(send(Msg::ApplyCustomGroup)),
                        focused: false,
                    },
                    d,
                ),
                kit::text_button("Use group", Some(send(Msg::ApplyCustomGroup)), d),
            ]
            .spacing(SPACE_2)
            .align_y(Alignment::Center),
        ]
        .spacing(SPACE_2)
        .into()
    } else {
        group_pick
    };
    let group_row = form_field("Group", group_control, compact);

    let mult_row = form_field(
        "Multiplier",
        kit::field(
            FieldProps {
                label: "",
                value: &entry.multiplier_text,
                placeholder: "1 or 0.5",
                on_input: Some(Box::new(|v| send(Msg::Multiplier(v)))),
                on_submit: None,
                focused: false,
            },
            d,
        ),
        compact,
    );

    let sign_row = form_field(
        "Sign",
        kit::segmented(
            vec![
                Segment {
                    label: "Adds",
                    selected: area.positive,
                    focused: false,
                    on_press: Some(send(Msg::Sign(true))),
                },
                Segment {
                    label: "Subtracts",
                    selected: !area.positive,
                    focused: false,
                    on_press: Some(send(Msg::Sign(false))),
                },
            ],
            d,
        ),
        compact,
    );

    let gla_row = form_field(
        "Living area",
        styled_checkbox("Count in gross living area", area.include_in_gla, |v| {
            send(Msg::IncludeGla(v))
        }),
        compact,
    );

    let line_col = parse_hex(area.color.trim()).unwrap_or_else(|| Color::from_rgb8(0x31, 0x82, 0xCE));
    let line_color_row = form_field(
        "Line colour",
        color_picker_row("Pick line colour", line_col, &area.color, Msg::PickLineColor, Msg::LineHex, d),
        compact,
    );

    let fill_col = parse_hex(area.fill_color.trim())
        .or_else(|| parse_css_color(&area.fill))
        .unwrap_or(line_col);
    let fill_color_row = form_field(
        "Fill colour",
        color_picker_row(
            "Pick fill colour",
            fill_col,
            &area.fill_color,
            Msg::PickFillColor,
            Msg::FillHex,
            d,
        ),
        compact,
    );

    let style_options: Vec<String> = FILL_STYLES
        .iter()
        .map(|v| style_label(v).to_string())
        .collect();
    let style_row = form_field(
        "Fill style",
        styled_pick_list(
            style_options,
            Some(style_label(&area.fill_style).to_string()),
            |v| send(Msg::FillStyle(v)),
            d,
        ),
        compact,
    );

    let opacity_pct = (area.fill_opacity as f32).clamp(0.0, 100.0);
    let opacity_row = form_field(
        "Fill opacity",
        row![
            container(
                slider(0.0..=100.0, opacity_pct, |v| send(Msg::FillOpacity(v)))
                    .step(1.0_f32)
                    .style(kit::slider_style)
                    .width(Length::Fill),
            )
            .width(Length::Fill),
            text(format!("{opacity_pct:.0}%"))
                .font(BSC_MEDIUM)
                .size(TEXT_12)
                .color(tokens::text())
                .width(Length::Fixed(42.0))
                .align_x(Alignment::End),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .into(),
        compact,
    );

    let preview_row = form_field(
        "Preview",
        container(
            Canvas::new(TypePreview {
                code: area.code.clone(),
                name: if area.name.trim().is_empty() {
                    area.code.clone()
                } else {
                    area.name.clone()
                },
                line_color: line_col,
                fill_color: fill_col,
                fill_style: area.fill_style.clone(),
                fill_opacity: opacity_pct,
            })
            .width(Length::Fill)
            .height(Length::Fixed(86.0)),
        )
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::canvas())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..container::Style::default()
        })
        .into(),
        compact,
    );

    form = form
        .push(code_row)
        .push(name_row)
        .push(desc_row)
        .push(group_row)
        .push(mult_row)
        .push(sign_row)
        .push(gla_row)
        .push(line_color_row)
        .push(fill_color_row)
        .push(style_row)
        .push(opacity_row)
        .push(preview_row);

    scrollable(container(form).padding(SPACE_4).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn confirm_banner<'a>(
    editor: &'a TypeEditor,
    confirm: &'a ConfirmKind,
    d: Density,
) -> Element<'a, Message> {
    let (message, action_label, action_msg) = match confirm {
        ConfirmKind::Delete(code) => {
            let count = editor.usage_counts.get(code).copied().unwrap_or(0);
            let noun = if count == 1 { "area uses" } else { "areas use" };
            (
                format!("{count} {noun} this type and will become Undefined."),
                "Delete type",
                Msg::ConfirmDelete,
            )
        }
        ConfirmKind::RestoreDefaults => (
            "Restore the 32 built-in default area types? Unsaved edits in this dialog will be replaced.".to_string(),
            "Confirm restore",
            Msg::ConfirmRestoreDefaults,
        ),
        ConfirmKind::Discard => (
            "Discard your unsaved area type changes?".to_string(),
            "Discard changes",
            Msg::ConfirmDiscard,
        ),
    };

    container(
        row![
            text(message)
                .font(BARLOW_MEDIUM)
                .size(TEXT_13)
                .color(tokens::flag())
                .width(Length::Fill),
            kit::text_button("Keep editing", Some(send(Msg::CancelConfirm)), d),
            kit::primary_button(action_label, Some(send(action_msg)), d),
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .wrap(),
    )
    .padding(SPACE_2)
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(tokens::field())),
        border: Border {
            color: tokens::flag(),
            width: 1.0,
            radius: RADIUS_CONTROL.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn form_field<'a>(
    label: &'a str,
    control: Element<'a, Message>,
    compact: bool,
) -> Element<'a, Message> {
    if compact {
        column![
            text(label)
                .font(BARLOW_MEDIUM)
                .size(TEXT_12)
                .color(tokens::muted()),
            control,
        ]
        .spacing(SPACE_1)
        .width(Length::Fill)
        .into()
    } else {
        row![
            text(label)
                .font(BARLOW_MEDIUM)
                .size(TEXT_12)
                .color(tokens::muted())
                .width(Length::Fixed(128.0)),
            container(control).width(Length::Fill),
        ]
        .spacing(SPACE_3)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
    }
}

fn color_picker_row<'a>(
    aria: &'a str,
    color: Color,
    hex_val: &'a str,
    pick_msg: Msg,
    on_hex: fn(String) -> Msg,
    d: Density,
) -> Element<'a, Message> {
    let swatch = button(Space::new().width(24.0).height(20.0))
        .on_press(send(pick_msg))
        .padding(0)
        .style(move |_, _| button::Style {
            background: Some(Background::Color(color)),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        });
    row![
        kit::tooltip(swatch, aria, iced::widget::tooltip::Position::Top),
        kit::field(
            FieldProps {
                label: "",
                value: hex_val,
                placeholder: "#3182CE",
                on_input: Some(Box::new(move |v| send(on_hex(v)))),
                on_submit: None,
                focused: false,
            },
            d,
        ),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .into()
}

fn footer_bar<'a>(editor: &'a TypeEditor, d: Density, compact: bool) -> Element<'a, Message> {
    let err = editor.validation_error();
    let can_save = err.is_none();

    let mut actions_left = row![
        kit::text_button("New type", Some(send(Msg::NewType)), d),
        kit::text_button(
            "Duplicate",
            (!editor.drafts.is_empty()).then(|| send(Msg::Duplicate)),
            d,
        ),
    ]
    .spacing(SPACE_1);

    if !compact || !editor.compact_showing_form {
        actions_left = actions_left
            .push(kit::text_button("Import", Some(send(Msg::Import)), d))
            .push(kit::text_button(
                "Export",
                can_save.then(|| send(Msg::Export)),
                d,
            ))
            .push(kit::text_button(
                "Restore defaults",
                Some(send(Msg::RequestRestoreDefaults)),
                d,
            ));
    }
    let actions_left = actions_left.wrap();

    let actions_right = row![
        kit::text_button("Cancel", Some(send(Msg::RequestCancel)), d),
        kit::primary_button("Save", can_save.then(|| send(Msg::Save)), d),
    ]
    .spacing(SPACE_2);

    let bar: Element<'a, Message> = if compact {
        column![
            actions_left,
            row![Space::new().width(Length::Fill), actions_right].width(Length::Fill),
        ]
        .spacing(SPACE_2)
        .into()
    } else {
        row![
            actions_left,
            Space::new().width(Length::Fill),
            actions_right,
        ]
        .spacing(SPACE_2)
        .align_y(Alignment::Center)
        .into()
    };

    let mut wrap_col = column![].spacing(SPACE_1);
    if let Some(msg) = err {
        wrap_col = wrap_col.push(
            text(msg)
                .font(BARLOW_MEDIUM)
                .size(TEXT_12)
                .color(tokens::flag()),
        );
    }
    wrap_col = wrap_col.push(bar);
    container(wrap_col)
        .padding([SPACE_2 + 2.0, SPACE_4])
        .width(Length::Fill)
        .into()
}

fn styled_pick_list<'a>(
    options: Vec<String>,
    selected: Option<String>,
    on_select: impl Fn(String) -> Message + 'a,
    d: Density,
) -> Element<'a, Message> {
    pick_list(options, selected, on_select)
        .width(Length::Fill)
        .text_size(TEXT_13)
        .font(BARLOW)
        .padding(Padding {
            top: ((d.field_h() - 18.0).max(4.0)) * 0.5,
            bottom: ((d.field_h() - 18.0).max(4.0)) * 0.5,
            left: 8.0,
            right: 8.0,
        })
        .menu_style(|_| iced::overlay::menu::Style {
            background: Background::Color(tokens::panel()),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            text_color: tokens::text(),
            selected_text_color: tokens::on_accent(),
            selected_background: Background::Color(tokens::accent()),
            shadow: tokens::floating_shadow(),
        })
        .style(|_, _| pick_list::Style {
            text_color: tokens::text(),
            placeholder_color: tokens::muted(),
            handle_color: tokens::muted(),
            background: Background::Color(tokens::input_bg()),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
        })
        .into()
}

fn styled_checkbox<'a>(
    label: &'a str,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(on_toggle)
        .font(BARLOW)
        .text_size(TEXT_13)
        .size(16.0)
        .style(|_, status| {
            let is_checked = matches!(
                status,
                checkbox::Status::Active { is_checked: true }
                    | checkbox::Status::Hovered { is_checked: true }
            );
            checkbox::Style {
                background: Background::Color(if is_checked {
                    tokens::accent()
                } else {
                    tokens::input_bg()
                }),
                icon_color: tokens::on_accent(),
                border: Border {
                    color: if is_checked {
                        tokens::accent()
                    } else {
                        tokens::rule()
                    },
                    width: 1.0,
                    radius: 3.0.into(),
                },
                text_color: Some(tokens::text()),
            }
        })
        .into()
}

struct TypePreview {
    code: String,
    name: String,
    line_color: Color,
    fill_color: Color,
    fill_style: String,
    fill_opacity: f32,
}

impl canvas::Program<Message> for TypePreview {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let pad_x = 18.0_f32;
        let pad_y = 12.0_f32;
        let w = (bounds.width - pad_x * 2.0).max(20.0);
        let h = (bounds.height - pad_y * 2.0).max(20.0);
        let rect_tl = Point::new(pad_x, pad_y);
        let rect_size = Size::new(w, h);
        let rect_path = Path::rectangle(rect_tl, rect_size);

        if self.fill_style != "none" {
            let alpha = (self.fill_opacity / 100.0).clamp(0.0, 1.0);
            let fill = Color {
                a: alpha,
                ..self.fill_color
            };
            frame.fill(&rect_path, fill);

            let pattern_ink = Color {
                a: 0.45,
                ..self.line_color
            };
            let p_stroke = Stroke::default().with_color(pattern_ink).with_width(1.0);
            if self.fill_style == "diagonal" || self.fill_style == "crosshatch" {
                frame.with_clip(Rectangle::new(rect_tl, rect_size), |f| {
                    let mut x = -h;
                    while x < w + h {
                        f.stroke(
                            &Path::line(
                                Point::new(pad_x + x, pad_y + h),
                                Point::new(pad_x + x + h, pad_y),
                            ),
                            p_stroke,
                        );
                        if self.fill_style == "crosshatch" {
                            f.stroke(
                                &Path::line(
                                    Point::new(pad_x + x, pad_y),
                                    Point::new(pad_x + x + h, pad_y + h),
                                ),
                                p_stroke,
                            );
                        }
                        x += 10.0;
                    }
                });
            } else if self.fill_style == "dots" {
                let mut x = pad_x + 8.0;
                while x < pad_x + w - 4.0 {
                    let mut y = pad_y + 8.0;
                    while y < pad_y + h - 4.0 {
                        frame.fill(&Path::circle(Point::new(x, y), 1.3), pattern_ink);
                        y += 10.0;
                    }
                    x += 10.0;
                }
            }
        }

        frame.stroke(
            &rect_path,
            Stroke::default().with_color(self.line_color).with_width(2.0),
        );

        let center = Point::new(bounds.width * 0.5, bounds.height * 0.5);
        frame.fill_text(canvas::Text {
            content: format!("{} ({})", self.name, self.code),
            position: Point::new(center.x, center.y - 8.0),
            color: tokens::text(),
            size: 13.0.into(),
            font: BARLOW_SEMIBOLD,
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Default::default()
        });
        frame.fill_text(canvas::Text {
            content: "100.0 sf".to_string(),
            position: Point::new(center.x, center.y + 9.0),
            color: tokens::muted(),
            size: 11.0.into(),
            font: BSC_MEDIUM,
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Default::default()
        });

        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_and_new_type_create_unique_codes() {
        let mut ed = TypeEditor::default();
        ed.open_list(&area_codes::default_area_codes(), BTreeMap::new(), Some("GAR"));
        let orig_len = ed.drafts.len();
        ed.apply_local(Msg::Duplicate);
        assert_eq!(ed.drafts.len(), orig_len + 1);
        assert_eq!(ed.drafts[ed.selected].area.code, "GAR_COPY");
        ed.apply_local(Msg::Duplicate);
        assert_eq!(ed.drafts[ed.selected].area.code, "GAR_COPY_COPY");
        ed.apply_local(Msg::NewType);
        assert_eq!(ed.drafts[ed.selected].area.code, "NEW");
        ed.apply_local(Msg::NewType);
        assert_eq!(ed.drafts[ed.selected].area.code, "NEW1");
        assert!(ed.validation_error().is_none());
    }

    #[test]
    fn rename_and_delete_produce_accurate_remaps() {
        let codes = area_codes::default_area_codes();
        let mut counts = BTreeMap::new();
        counts.insert("GAR".to_string(), 2);
        counts.insert("PAT".to_string(), 1);
        let mut ed = TypeEditor::default();
        ed.open_list(&codes, counts, Some("GAR"));
        ed.apply_local(Msg::Code("garage_x".to_string()));
        assert_eq!(ed.drafts[ed.selected].area.code, "GARAGE_X");

        let pat_idx = ed.drafts.iter().position(|d| d.area.code == "PAT").unwrap();
        ed.apply_local(Msg::Select(pat_idx));
        ed.apply_local(Msg::RequestDelete);
        assert_eq!(ed.confirm, Some(ConfirmKind::Delete("PAT".to_string())));
        ed.apply_local(Msg::ConfirmDelete);

        let out = ed.build_save_outcome().expect("valid save outcome");
        assert!(matches!(
            out.remaps.get("GAR"),
            Some(RemapTarget::Updated {
                new_code,
                code_changed: true,
                ..
            }) if new_code == "GARAGE_X"
        ));
        assert!(matches!(out.remaps.get("PAT"), Some(RemapTarget::Deleted)));
    }

    #[test]
    fn und_cannot_be_deleted_and_move_up_down_swaps_within_group() {
        let codes = area_codes::default_area_codes();
        let mut ed = TypeEditor::default();
        ed.open_list(&codes, BTreeMap::new(), Some("UND"));
        ed.apply_local(Msg::RequestDelete);
        assert!(ed.drafts.iter().any(|d| d.area.code == "UND"));

        let gla2 = ed.drafts.iter().position(|d| d.area.code == "GLA2").unwrap();
        ed.apply_local(Msg::Select(gla2));
        ed.apply_local(Msg::MoveUp(gla2));
        assert_eq!(ed.drafts[0].area.code, "GLA2");
        assert_eq!(ed.drafts[1].area.code, "GLA1");
        ed.apply_local(Msg::MoveDown(0));
        assert_eq!(ed.drafts[0].area.code, "GLA1");
    }
}
