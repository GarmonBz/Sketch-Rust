//! The Options dialog (Screen 11, S11-001..032): `UIManager.showOptions`
//! (refs/ui-manager.js:2570). A six-tab card — General / Display /
//! Hardware / System / Fonts / Drawing — over the flat `Options` model
//! (options.rs). OK applies the draft through `applyOptions`
//! (sketch-app.js:524), which clamps widths/thicknesses and mirrors
//! `gridEnabled`; Cancel discards; Restore Defaults applies the code
//! defaults + a toast; Save as Default stashes the draft (the browser keeps
//! it in `localStorage sketch.defaultOptions`; no persistence sidecar yet —
//! documented gap §D, so the notification says so).
//!
//! Layout note: the card chrome (title, tab rail, actions row) follows the
//! measured overlay style of the other Screen-11 dialogs; the panels are
//! scrollable rows rather than pinned absolute positions, since no measured
//! per-control offsets exist in the spec docs for this dialog.

use std::collections::BTreeMap;

use iced::widget::{
    button, checkbox, column, container, pick_list, radio, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length};

use crate::options::{BoolKey, FormatKey, NumKey, Options, StrKey, FONTS, FONT_SIZES};
use crate::theme::{bg_hover, bg_panel, border, muted};
use crate::theme::{hex, text_main, ARIAL};
use crate::Message;

/// A numeric `pick_list` entry (ink recognition, font sizes, pointer size):
/// displays its plain decimal, like `<option value="0.25">0.25</option>`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumChoice(pub f64);

impl std::fmt::Display for NumChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.fract() == 0.0 && self.0.abs() < 1e15 {
            write!(f, "{}", self.0 as i64)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

// Radio requires Eq; the f64 value never carries a NaN (options clamps).
impl Eq for NumChoice {}
impl Eq for Choice {}

/// A string `pick_list` entry: a stored value plus its display label
/// (`<option value="pen-ink">Pen as Ink</option>`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Choice {
    value: &'static str,
    label: &'static str,
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}

/// The six category tabs, in the nav rail's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    General,
    Display,
    Hardware,
    System,
    Fonts,
    Drawing,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::General,
        Tab::Display,
        Tab::Hardware,
        Tab::System,
        Tab::Fonts,
        Tab::Drawing,
    ];

    fn label(self) -> &'static str {
        match self {
            Tab::General => "General",
            Tab::Display => "Display",
            Tab::Hardware => "Hardware",
            Tab::System => "System",
            Tab::Fonts => "Fonts",
            Tab::Drawing => "Drawing",
        }
    }
}

/// The dialog's state: open, the visible tab, the draft being edited, and
/// the numeric text buffers (`text_input` text per field, parsed as typed).
#[derive(Debug, Clone)]
pub struct OptionsDialog {
    pub open: bool,
    pub tab: Tab,
    pub draft: Options,
    buffers: BTreeMap<NumKey, String>,
}

impl Default for OptionsDialog {
    fn default() -> Self {
        OptionsDialog {
            open: false,
            tab: Tab::General,
            draft: Options::default(),
            buffers: BTreeMap::new(),
        }
    }
}

impl OptionsDialog {
    /// Open over the current options (`{defaultOptions, ...state.options,
    /// defineFirst, gridEnabled}` — all already live in the flat model).
    pub fn open_with(options: Options) -> Self {
        OptionsDialog {
            open: true,
            tab: Tab::General,
            draft: options,
            buffers: BTreeMap::new(),
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.buffers.clear();
    }

    /// The dialog-local controls (tabs, toggles, typed/picked values).
    /// The App-level actions (OK / Cancel / Restore / Save-as-Default) are
    /// `OptMsg` variants the handler consumes directly.
    pub fn update(&mut self, msg: OptMsg) {
        match msg {
            OptMsg::Tab(tab) => self.tab = tab,
            OptMsg::Toggle(key) => {
                let value = !self.draft.get_bool(key);
                self.draft.set_bool(key, value);
            }
            OptMsg::ToggleFormat(key) => {
                self.draft.set_format(key, !self.draft.format_enabled(key));
            }
            OptMsg::NumTyped(key, raw) => {
                self.buffers.insert(key, raw.clone());
                if let Ok(n) = raw.trim().parse::<f64>() {
                    if n.is_finite() {
                        self.draft.set_num(key, n);
                    }
                }
            }
            OptMsg::NumPicked(key, NumChoice(n)) => {
                self.buffers.remove(&key);
                self.draft.set_num(key, n);
            }
            OptMsg::Str(key, value) => self.draft.set_str(key, value),
            OptMsg::UnitMetric(metric) => self.draft.units.metric = metric,
            OptMsg::ToggleStripZeroes => {
                self.draft.units.strip_trailing_zeroes = !self.draft.units.strip_trailing_zeroes;
            }
            OptMsg::Open | OptMsg::Ok | OptMsg::Cancel | OptMsg::SaveDefaults => {}
            OptMsg::RestoreDefaults => {
                self.draft = Options::default();
                self.buffers.clear();
            }
        }
    }

    /// A numeric field's display text: the typed buffer while it exists,
    /// otherwise the value.
    fn num_text(&self, key: NumKey) -> String {
        self.buffers
            .get(&key)
            .cloned()
            .unwrap_or_else(|| fmt_num(self.draft.get_num(key)))
    }
}

/// A dialog control.
#[derive(Debug, Clone)]
pub enum OptMsg {
    Open,
    Tab(Tab),
    /// OK (submit): the handler collects the draft.
    Ok,
    /// `data-close` / Cancel.
    Cancel,
    /// `data-defaults`.
    RestoreDefaults,
    /// `data-save-defaults`.
    SaveDefaults,
    Toggle(BoolKey),
    ToggleFormat(FormatKey),
    /// A numeric text input: the raw text (of the parseable prefix).
    NumTyped(NumKey, String),
    /// A numeric radio / pick: an exact value.
    NumPicked(NumKey, NumChoice),
    /// A string radio / pick / text input.
    Str(StrKey, String),
    /// Unit of Measure radios → `Units.metric`.
    UnitMetric(bool),
    /// Strip Trailing Zeroes (the `Units` group's checkbox).
    ToggleStripZeroes,
}

/// `12.0` → `"12"`, `0.1` → `"0.1"` — what `Number` inputs display.
fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

// --- Fieldset + row helpers --------------------------------------------------

fn group<'a>(title: &'static str, rows: Vec<Element<'a, Message>>) -> Element<'a, Message> {
    let mut col = column![text(title).font(ARIAL).size(12.0).color(muted())]
        .spacing(6)
        .padding(10);
    for r in rows {
        col = col.push(r);
    }
    container(col)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn spacer() -> Space {
    Space::new().width(10.0)
}

fn check(label: &'static str, checked: bool, key: BoolKey) -> Element<'static, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(move |_| Message::Options(OptMsg::Toggle(key)))
        .text_size(12.0)
        .spacing(6)
        .into()
}

fn format_check(label: &'static str, checked: bool, key: FormatKey) -> Element<'static, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(move |_| Message::Options(OptMsg::ToggleFormat(key)))
        .text_size(12.0)
        .spacing(6)
        .into()
}

/// A numeric radio group (`<input type="radio">` family).
fn num_radios(
    key: NumKey,
    values: &'static [f64],
    current: f64,
    label: impl Fn(f64) -> String,
) -> Element<'static, Message> {
    row(values
        .iter()
        .map(|v| {
            let choice = NumChoice(*v);
            radio(
                label(*v),
                choice,
                if choice == NumChoice(current) {
                    Some(choice)
                } else {
                    None
                },
                move |picked| Message::Options(OptMsg::NumPicked(key, picked)),
            )
            .text_size(12.0)
            .spacing(6)
            .into()
        })
        .collect::<Vec<Element<'static, Message>>>())
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// A numeric pick list (selects in the original).
fn num_pick(key: NumKey, values: &'static [f64], current: f64) -> Element<'static, Message> {
    let options: Vec<NumChoice> = values.iter().map(|v| NumChoice(*v)).collect();
    let selected = options.iter().find(|c| c.0 == current).copied();
    pick_list(options, selected, move |c| {
        Message::Options(OptMsg::NumPicked(key, c))
    })
    .text_size(12.0)
    .padding([3, 8])
    .into()
}

/// A labelled numeric pick: `Font  [12 v]`.
fn labeled_num_pick(
    label: &'static str,
    key: NumKey,
    values: &'static [f64],
    current: f64,
) -> Element<'static, Message> {
    row![
        text(label).font(ARIAL).size(12.0),
        spacer(),
        num_pick(key, values, current)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// A string pick (selects in the original).
fn str_pick(
    key: StrKey,
    list: &'static [(&'static str, &'static str)],
    current: &str,
) -> Element<'static, Message> {
    let options: Vec<Choice> = list
        .iter()
        .map(|(value, label)| Choice { value, label })
        .collect();
    let selected = options.iter().find(|c| c.value == current).copied();
    pick_list(options, selected, move |c| {
        Message::Options(OptMsg::Str(key, c.value.to_string()))
    })
    .text_size(12.0)
    .padding([3, 8])
    .into()
}

fn labeled_str_pick(
    label: &'static str,
    key: StrKey,
    list: &'static [(&'static str, &'static str)],
    current: &str,
) -> Element<'static, Message> {
    row![
        text(label).font(ARIAL).size(12.0),
        spacer(),
        str_pick(key, list, current)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// Two-state radio (Imperial/Metric, Automatic/Manual, 8/24-bit).
fn bool_radios(
    current: bool,
    yes: (&'static str, Message),
    no: (&'static str, Message),
) -> Element<'static, Message> {
    let (yes_label, yes_msg) = yes;
    let (no_label, no_msg) = no;
    row![
        radio(yes_label, true, Some(current), |_| yes_msg.clone())
            .text_size(12.0)
            .spacing(6),
        radio(no_label, false, Some(!current), |_| no_msg.clone())
            .text_size(12.0)
            .spacing(6),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// `Label [input]` for a typed numeric field.
fn num_field(d: &OptionsDialog, label: &'static str, key: NumKey) -> Element<'static, Message> {
    row![
        text(label).font(ARIAL).size(12.0),
        spacer(),
        text_input("", &d.num_text(key))
            .on_input(move |s| Message::Options(OptMsg::NumTyped(key, s)))
            .width(Length::Fixed(90.0))
            .size(12.0)
            .padding([3, 6]),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// `Label [input]` for a typed string field (suffix, COM port).
fn str_field(d: &OptionsDialog, label: &'static str, key: StrKey) -> Element<'static, Message> {
    let value = d.draft.get_str(key);
    row![
        text(label).font(ARIAL).size(12.0),
        spacer(),
        text_input("", &value)
            .on_input(move |s| Message::Options(OptMsg::Str(key, s)))
            .width(Length::Fixed(120.0))
            .size(12.0)
            .padding([3, 6]),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

/// A colour input as text + a static swatch (the port's documented
/// `<input type="color">` stand-in; §C: swatches static, hex text edits).
fn color_field(d: &OptionsDialog, label: &'static str, key: StrKey) -> Element<'static, Message> {
    let value = d.draft.get_str(key);
    let swatch = crate::render::parse_hex(&value).unwrap_or(Color::TRANSPARENT);
    row![
        text(label).font(ARIAL).size(12.0),
        spacer(),
        container(Space::new().width(14.0))
            .height(Length::Fixed(14.0))
            .style(move |_| container::Style {
                background: Some(Background::Color(swatch)),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }),
        text_input("", &value)
            .on_input(move |s| Message::Options(OptMsg::Str(key, s)))
            .width(Length::Fixed(96.0))
            .size(12.0)
            .padding([3, 6]),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

// --- The six panels ----------------------------------------------------------

const INPUT_MODES: &[(&str, &str)] = &[
    ("pen-ink", "Pen as Ink"),
    ("pen-mouse", "Pen as Mouse"),
    ("touch", "Touch"),
];
const ALIGNMENT_STYLES: &[(&str, &str)] = &[("dash", "Dash"), ("solid", "Solid")];
const ROUNDING_MODES: &[(&str, &str)] = &[
    ("none", "None"),
    ("nearestHalf", "Nearest Half"),
    ("nearestHundredth", "Nearest Hundredth"),
    ("nearestTenth", "Nearest Tenth"),
    ("nearestWhole", "Nearest Whole"),
];
const DRAWING_MODES: &[(&str, &str)] = &[("standard", "Standard"), ("continuous", "Continuous")];
const POINTERS: &[f64] = &[128.0, 64.0, 32.0];
const PRECISIONS: &[f64] = &[1.0, 0.1, 0.01, 0.001, 0.0001];
const INK_RECOGNITIONS: &[f64] = &[0.1, 0.25, 0.5];

fn font_pick(key: StrKey, current: &str) -> Element<'static, Message> {
    let list: Vec<(&'static str, &'static str)> = FONTS.iter().map(|f| (*f, *f)).collect();
    // `&'static [..]` from a local build: FONTS itself is static; rebuild the
    // pair list as a value for pick_list to own.
    let options: Vec<Choice> = list
        .iter()
        .map(|(value, label)| Choice { value, label })
        .collect();
    let selected = options.iter().find(|c| c.value == current).copied();
    pick_list(options, selected, move |c| {
        Message::Options(OptMsg::Str(key, c.value.to_string()))
    })
    .text_size(12.0)
    .padding([3, 8])
    .into()
}

fn labeled_font_pick(label: &'static str, key: StrKey, current: &str) -> Element<'static, Message> {
    row![
        text(label).font(ARIAL).size(12.0),
        spacer(),
        font_pick(key, current)
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn font_group(
    d: &OptionsDialog,
    title: &'static str,
    font: StrKey,
    size: NumKey,
    bold: BoolKey,
    italic: BoolKey,
    underline: BoolKey,
    extras: Vec<Element<'static, Message>>,
) -> Element<'static, Message> {
    let mut rows = vec![
        labeled_font_pick("Font", font, &d.draft.get_str(font)),
        labeled_num_pick("Size", size, FONT_SIZES, d.draft.get_num(size)),
        check("Bold", d.draft.get_bool(bold), bold),
        check("Italic", d.draft.get_bool(italic), italic),
        check("Underline", d.draft.get_bool(underline), underline),
    ];
    rows.extend(extras);
    group(title, rows)
}

/// The currently visible tab's controls.
fn panel(d: &OptionsDialog) -> Element<'static, Message> {
    let draft = &d.draft;
    match d.tab {
        Tab::General => column![
            group(
                "Input Options",
                vec![
                    labeled_str_pick(
                        "Input mode",
                        StrKey::InputMode,
                        INPUT_MODES,
                        &draft.input_mode
                    ),
                    check(
                        "Prompt on Deletion",
                        draft.prompt_on_deletion,
                        BoolKey::PromptOnDeletion,
                    ),
                ],
            ),
            group(
                "Ink Options",
                vec![
                    check(
                        "Ink Confirmation",
                        draft.ink_confirmation,
                        BoolKey::InkConfirmation
                    ),
                    num_radios(
                        NumKey::InkRecognition,
                        INK_RECOGNITIONS,
                        draft.ink_recognition,
                        fmt_num,
                    ),
                ],
            ),
            group(
                "Unit of Measure",
                vec![bool_radios(
                    draft.units.metric,
                    ("Metric", Message::Options(OptMsg::UnitMetric(true))),
                    ("Imperial", Message::Options(OptMsg::UnitMetric(false))),
                )],
            ),
            group(
                "Decimal Precision",
                vec![num_radios(
                    NumKey::DecimalPrecision,
                    PRECISIONS,
                    draft.units.decimal_precision,
                    fmt_num,
                )],
            ),
            group(
                "Step Unit",
                vec![bool_radios(
                    draft.step_unit == "whole",
                    (
                        "Whole",
                        Message::Options(OptMsg::Str(StrKey::StepUnit, "whole".into()))
                    ),
                    (
                        "Half",
                        Message::Options(OptMsg::Str(StrKey::StepUnit, "half".into()))
                    ),
                )],
            ),
            group(
                "Dimension Options",
                vec![
                    num_field(d, "Dimension", NumKey::DimensionOption),
                    num_field(d, "Ortho (°)", NumKey::OrthoAngle),
                    check(
                        "Auto Placement",
                        draft.auto_placement,
                        BoolKey::AutoPlacement
                    ),
                    route_strip_check(d),
                    check(
                        "Same Color As Line",
                        draft.same_color_as_line,
                        BoolKey::SameColorAsLine,
                    ),
                    str_field(d, "Area Label Suffix", StrKey::AreaLabelSuffix),
                ],
            ),
        ]
        .spacing(10)
        .into(),
        Tab::Display => column![
            group(
                "Alignment Indicators",
                vec![
                    check(
                        "Lines Visible",
                        draft.alignment_lines,
                        BoolKey::AlignmentLines
                    ),
                    check(
                        "Points Visible",
                        draft.alignment_points,
                        BoolKey::AlignmentPoints
                    ),
                    check(
                        "Test Close Indicators",
                        draft.test_close_indicators,
                        BoolKey::TestCloseIndicators,
                    ),
                    color_field(d, "Color", StrKey::AlignmentColor),
                    labeled_str_pick(
                        "Style",
                        StrKey::AlignmentStyle,
                        ALIGNMENT_STYLES,
                        &draft.alignment_style,
                    ),
                ],
            ),
            group(
                "Grid",
                vec![check("Enabled", draft.grid_enabled, BoolKey::GridEnabled)],
            ),
            group(
                "Minimum Dimension Properties",
                vec![
                    check(
                        "Enabled",
                        draft.minimum_dimension_enabled,
                        BoolKey::MinimumDimensionEnabled,
                    ),
                    num_field(d, "Minimum length", NumKey::MinimumDimension),
                ],
            ),
            group(
                "Auto-Shade",
                vec![
                    check("Enabled", draft.auto_shade, BoolKey::AutoShade),
                    num_field(d, "Percent", NumKey::AutoShadePercent),
                    check(
                        "Selection Highlight",
                        draft.selection_highlight,
                        BoolKey::SelectionHighlight,
                    ),
                ],
            ),
        ]
        .spacing(10)
        .into(),
        Tab::Hardware => column![
            group(
                "DISTO Discovery",
                vec![
                    bool_radios(
                        draft.disto_discovery == "automatic",
                        (
                            "Automatic",
                            Message::Options(OptMsg::Str(
                                StrKey::DistoDiscovery,
                                "automatic".into()
                            ))
                        ),
                        (
                            "Manual",
                            Message::Options(OptMsg::Str(StrKey::DistoDiscovery, "manual".into()))
                        ),
                    ),
                    str_field(d, "COM Port", StrKey::DistoComPort),
                ],
            ),
            group(
                "Configuration",
                vec![
                    labeled_str_pick(
                        "Rounding",
                        StrKey::DistoRounding,
                        ROUNDING_MODES,
                        &draft.disto_rounding,
                    ),
                    labeled_str_pick(
                        "Drawing Mode",
                        StrKey::DistoDrawingMode,
                        DRAWING_MODES,
                        &draft.disto_drawing_mode,
                    ),
                ],
            ),
        ]
        .spacing(10)
        .into(),
        Tab::System => column![
            group(
                "Auto-Save",
                vec![
                    check("Enabled", draft.auto_save_enabled, BoolKey::AutoSaveEnabled),
                    num_field(d, "Interval (minutes)", NumKey::AutoSaveInterval),
                    text("Auto-save data is kept in this browser.")
                        .font(ARIAL)
                        .size(11.0)
                        .color(muted())
                        .into(),
                ],
            ),
            group(
                "Image Files",
                vec![
                    labeled_str_pick("Size", StrKey::ImageSize, size_list(), &draft.image_size),
                    bool_radios(
                        draft.image_depth == 24.0,
                        (
                            "24-Bit",
                            Message::Options(OptMsg::NumPicked(
                                NumKey::ImageDepth,
                                NumChoice(24.0),
                            )),
                        ),
                        (
                            "8-Bit",
                            Message::Options(
                                OptMsg::NumPicked(NumKey::ImageDepth, NumChoice(8.0),)
                            ),
                        ),
                    ),
                    check(
                        "Append Page Number",
                        draft.append_page_number,
                        BoolKey::AppendPageNumber,
                    ),
                    format_check("PNG", draft.format_enabled(FormatKey::Png), FormatKey::Png),
                    format_check("JPG", draft.format_enabled(FormatKey::Jpg), FormatKey::Jpg),
                    format_check("BMP", draft.format_enabled(FormatKey::Bmp), FormatKey::Bmp),
                ],
            ),
            group(
                "Application",
                vec![
                    check(
                        "Disable Video Tutorial",
                        draft.disable_video_tutorial,
                        BoolKey::DisableVideoTutorial,
                    ),
                    check(
                        "Auto Heal Files",
                        draft.auto_heal_files,
                        BoolKey::AutoHealFiles
                    ),
                ],
            ),
        ]
        .spacing(10)
        .into(),
        Tab::Fonts => column![
            font_group(
                d,
                "Dimension Properties",
                StrKey::DimensionFont,
                NumKey::DimensionFontSize,
                BoolKey::DimensionBold,
                BoolKey::DimensionItalic,
                BoolKey::DimensionUnderline,
                vec![],
            ),
            font_group(
                d,
                "Text Label Properties",
                StrKey::TextFont,
                NumKey::TextFontSize,
                BoolKey::TextBold,
                BoolKey::TextItalic,
                BoolKey::TextUnderline,
                vec![color_field(d, "Color", StrKey::TextColor)],
            ),
            font_group(
                d,
                "Automatic labels Properties",
                StrKey::AutoPostFont,
                NumKey::AutoPostFontSize,
                BoolKey::AutoPostBold,
                BoolKey::AutoPostItalic,
                BoolKey::AutoPostUnderline,
                vec![check(
                    "Circle Area Calcs",
                    draft.circle_area_calcs,
                    BoolKey::CircleAreaCalcs,
                )],
            ),
            group(
                "Text Background",
                vec![
                    color_field(d, "Color", StrKey::TextBackground),
                    num_field(d, "Opacity", NumKey::TextBackgroundOpacity),
                ],
            ),
        ]
        .spacing(10)
        .into(),
        Tab::Drawing => column![
            group(
                "Area Line Properties",
                vec![
                    color_field(d, "Drawing Color", StrKey::ExteriorColor),
                    num_field(d, "Thickness", NumKey::ExteriorWidth),
                ],
            ),
            group(
                "Interior Line Properties",
                vec![
                    color_field(d, "Drawing Color", StrKey::InteriorColor),
                    num_field(d, "Thickness", NumKey::InteriorWidth),
                ],
            ),
            group(
                "Wall Solids",
                vec![
                    check("Enable wall solids", draft.wall_solids, BoolKey::WallSolids,),
                    num_field(d, "Exterior thickness (in)", NumKey::ExteriorThickness),
                    num_field(d, "Interior thickness (in)", NumKey::InteriorThickness),
                    text("When disabled, walls render as legacy single-line strokes.")
                        .font(ARIAL)
                        .size(11.0)
                        .color(muted())
                        .into(),
                ],
            ),
            group(
                "Witness Line Properties",
                vec![
                    color_field(d, "Base", StrKey::WitnessBaseColor),
                    color_field(d, "Interior", StrKey::WitnessInteriorColor),
                    color_field(d, "Curve", StrKey::WitnessCurveColor),
                    num_field(d, "Label Opacity", NumKey::LabelOpacity),
                ],
            ),
            group(
                "Define Area Properties",
                vec![
                    check(
                        "Display on Close",
                        draft.display_on_close,
                        BoolKey::DisplayOnClose
                    ),
                    check("Define First", draft.define_first, BoolKey::DefineFirst),
                    check(
                        "Auto-Subtract Overlapped Areas",
                        draft.auto_subtract_overlapped,
                        BoolKey::AutoSubtractOverlapped,
                    ),
                    check(
                        "Quick-Shape Define",
                        draft.quick_shape_define,
                        BoolKey::QuickShapeDefine,
                    ),
                    check("Re-Define", draft.redefine, BoolKey::Redefine),
                    check(
                        "Split wall dimensions at area junctions",
                        draft.split_walls_at_junctions,
                        BoolKey::SplitWallsAtJunctions,
                    ),
                ],
            ),
            group(
                "Keyboard Modifiers",
                vec![check(
                    "Pan-Zoom / Delete",
                    draft.pan_zoom_delete,
                    BoolKey::PanZoomDelete
                )],
            ),
            group(
                "Drawing Pointer",
                vec![labeled_num_pick(
                    "Size",
                    NumKey::PointerSize,
                    POINTERS,
                    draft.pointer_size,
                )],
            ),
            group(
                "Reference image",
                vec![
                    check(
                        "Save Image With Sketch",
                        draft.save_image_with_sketch,
                        BoolKey::SaveImageWithSketch,
                    ),
                    num_field(d, "Image Quality", NumKey::ImageQuality),
                ],
            ),
        ]
        .spacing(10)
        .into(),
    }
}

/// Strip Trailing Zeroes lives in the `Units` group; routed via its own
/// toggle message so the checkboxes stay in `Options`' flat keys too.
fn route_strip_check(d: &OptionsDialog) -> Element<'static, Message> {
    checkbox(d.draft.units.strip_trailing_zeroes)
        .label("Strip Trailing Zeroes")
        .on_toggle(|_| Message::Options(OptMsg::ToggleStripZeroes))
        .text_size(12.0)
        .spacing(6)
        .into()
}

fn size_list() -> &'static [(&'static str, &'static str)] {
    static LIST: &[(&str, &str)] = &[
        ("4800x4800", "4800x4800"),
        ("3200x3200", "3200x3200"),
        ("2048x1536", "2048x1536"),
        ("1920x1080", "1920x1080"),
        ("1280x720", "1280x720"),
    ];
    LIST
}

// --- The layer ---------------------------------------------------------------

/// The dialog's card size (the Screen-11 measured overlay family).
const CARD: (f32, f32) = (760.0, 600.0);

/// A card button (`#3E3E42`, like the other dialogs').
fn btn(label: &'static str, msg: Message) -> Element<'static, Message> {
    button(container(text(label).font(ARIAL).size(12.0).color(text_main())).padding([5, 12]))
        .on_press(msg)
        .style(move |_, status| button::Style {
            background: Some(Background::Color(match status {
                button::Status::Hovered | button::Status::Pressed => hex(0x4a4a4e),
                _ => bg_hover(),
            })),
            ..button::Style::default()
        })
        .into()
}

/// The dialog overlay, or None when closed.
pub fn layer(d: &OptionsDialog) -> Option<Element<'static, Message>> {
    if !d.open {
        return None;
    }
    // Title row
    let title = row![
        text("Options").font(ARIAL).size(15.0).color(text_main()),
        Space::new().width(Length::Fill),
        button(text("×").font(ARIAL).size(14.0).color(text_main()))
            .on_press(Message::Options(OptMsg::Cancel)),
    ]
    .padding(8)
    .spacing(6);

    // Tab rail
    let tabs = row(Tab::ALL
        .iter()
        .map(|tab| {
            let active = *tab == d.tab;
            button(
                container(text(tab.label()).font(ARIAL).size(12.0).color(text_main()))
                    .padding([4, 10]),
            )
            .on_press(Message::Options(OptMsg::Tab(*tab)))
            .style(move |_, status| button::Style {
                background: if active
                    || status == button::Status::Hovered
                    || status == button::Status::Pressed
                {
                    Some(Background::Color(bg_hover()))
                } else {
                    Some(Background::Color(Color::TRANSPARENT))
                },
                ..button::Style::default()
            })
            .into()
        })
        .collect::<Vec<Element<'static, Message>>>())
    .spacing(2)
    .padding([6, 8]);

    // Panel (scrollable like cavity `.options-panels`).
    let body = scrollable(container(panel(d)).width(Length::Fill).padding([0, 8]))
        .height(Length::FillPortion(1));

    // Actions: Restore Defaults | Save as Default | ... | Cancel | OK
    let actions = row![
        btn(
            "Restore Defaults",
            Message::Options(OptMsg::RestoreDefaults)
        ),
        spacer(),
        btn("Save as Default", Message::Options(OptMsg::SaveDefaults),),
        Space::new().width(Length::Fill),
        btn("Cancel", Message::Options(OptMsg::Cancel)),
        spacer(),
        btn("OK", Message::Options(OptMsg::Ok)),
    ]
    .padding(8)
    .spacing(8)
    .align_y(Alignment::Center);

    let card = container(
        column![title, tabs, body, actions]
            .width(Length::Fixed(CARD.0))
            .height(Length::Fixed(CARD.1)),
    )
    .style(|_| container::Style {
        background: Some(Background::Color(bg_panel())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 2.0.into(),
        },
        ..container::Style::default()
    });

    let overlay = container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .center(Length::Fill);
    Some(overlay.into())
}
