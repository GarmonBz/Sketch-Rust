//! The Options dialog (Screen 11, S11-001..032): `UIManager.showOptions`
//! (refs/ui-manager.js:2570). A six-tab card â€” General / Display /
//! Hardware / System / Fonts / Drawing â€” over the flat `Options` model
//! (options.rs). OK applies the draft through `applyOptions`
//! (sketch-app.js:524), which clamps widths/thicknesses and mirrors
//! `gridEnabled`; Cancel discards; Restore Defaults applies the code
//! defaults + a toast; Save as Default stashes the draft (the browser keeps
//! it in `localStorage sketch.defaultOptions`; no persistence sidecar yet â€”
//! documented gap Â§D, so the notification says so).
//!
//! Layout note: the card chrome (title, tab rail, actions row) follows the
//! measured overlay style of the other Screen-11 dialogs; the panels are
//! scrollable rows rather than pinned absolute positions, since no measured
//! per-control offsets exist in the spec docs for this dialog.

use std::collections::BTreeMap;

use crate::options::{BoolKey, FormatKey, NumKey, Options, StrKey};

pub(crate) const INPUT_MODES: &[(&str, &str)] = &[
    ("pen-ink", "Pen as Ink"),
    ("pen-mouse", "Pen as Mouse"),
    ("touch", "Touch"),
];
pub(crate) const ALIGNMENT_STYLES: &[(&str, &str)] = &[("dash", "Dash"), ("solid", "Solid")];
pub(crate) const ROUNDING_MODES: &[(&str, &str)] = &[
    ("none", "None"),
    ("nearestHalf", "Nearest Half"),
    ("nearestHundredth", "Nearest Hundredth"),
    ("nearestTenth", "Nearest Tenth"),
    ("nearestWhole", "Nearest Whole"),
];
pub(crate) const DRAWING_MODES: &[(&str, &str)] =
    &[("standard", "Standard"), ("continuous", "Continuous")];
pub(crate) const POINTERS: &[f64] = &[128.0, 64.0, 32.0];
pub(crate) const PRECISIONS: &[f64] = &[1.0, 0.1, 0.01, 0.001, 0.0001];
pub(crate) const INK_RECOGNITIONS: &[f64] = &[0.1, 0.25, 0.5];

pub(crate) fn size_list() -> &'static [(&'static str, &'static str)] {
    static LIST: &[(&str, &str)] = &[
        ("4800x4800", "4800x4800"),
        ("3200x3200", "3200x3200"),
        ("2048x1536", "2048x1536"),
        ("1920x1080", "1920x1080"),
        ("1280x720", "1280x720"),
    ];
    LIST
}

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

    pub(crate) fn label(self) -> &'static str {
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
    /// defineFirst, gridEnabled}` â€” all already live in the flat model).
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
    pub(crate) fn num_text(&self, key: NumKey) -> String {
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
    /// Unit of Measure radios â†’ `Units.metric`.
    UnitMetric(bool),
    /// Strip Trailing Zeroes (the `Units` group's checkbox).
    ToggleStripZeroes,
}

/// `12.0` â†’ `"12"`, `0.1` â†’ `"0.1"` â€” what `Number` inputs display.
pub(crate) fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}
