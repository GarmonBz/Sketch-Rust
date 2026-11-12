//! Page Layout settings (Screen 11, File > Page Layout, S11-033..037):
//! `state.pageLayout` (state-init.js:23) with `DEFAULT_PAGE_LAYOUT`
//! (ui-manager.js:3283), `normalizePageLayout` / `setPageLayout`
//! (sketch-app.js:177/192) and the `showPageLayout` card
//! (ui-manager.js:2681) that edits it.
//!
//! On disk the layout rides `settings.pageLayout` with the original's exact
//! keys: `paperSize` (`"legal"|"letter"|"a4"`), `currentForm`
//! (`"<form>-<paper>"`), the five addenda switches and the `captions` map.
//! A stored layout the port doesn't recognise clamps through
//! `normalizePageLayout` on load, exactly like `applySaveModel` does.

use serde::{Deserialize, Serialize};

use iced::widget::{
    button, checkbox, column, container, pick_list, radio, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Background, Border, Element, Length};

use crate::theme::{hex, ARIAL};
use crate::Message;

use crate::theme::{bg_hover, bg_panel, border, text_alt};
use sketch_core::print_layout::PaperSize;

// --- The model ---------------------------------------------------------------

/// The `currentForm` prefix (`'standard' | 'full-page' | 'sketch-only' |
/// 'with-calculations'); the paper is the suffix.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Form {
    #[default]
    Standard,
    FullPage,
    SketchOnly,
    WithCalculations,
}

impl Form {
    pub const ALL: &'static [Form] = &[
        Form::Standard,
        Form::FullPage,
        Form::SketchOnly,
        Form::WithCalculations,
    ];

    /// The `currentForm`'s part before the paper suffix.
    pub fn key(self) -> &'static str {
        match self {
            Form::Standard => "standard",
            Form::FullPage => "full-page",
            Form::SketchOnly => "sketch-only",
            Form::WithCalculations => "with-calculations",
        }
    }

    /// `String(formLabels[key](paperLabel(paper)))`.
    pub fn label(self, paper: PaperSize) -> String {
        let size = paper.label().to_string();
        match self {
            Form::Standard => format!("Standard {size}"),
            Form::FullPage => format!("Full Page {size}"),
            Form::SketchOnly => format!("{size} Size Sketch Only"),
            Form::WithCalculations => format!("{size} W/ Calculations"),
        }
    }

    /// The stored `currentForm`'s prefix, clamped to the allowed list:
    /// `String(layout.currentForm || '').split('-').slice(0, -1).join('-')
    /// || 'standard'` — a suffix-less value splits to nothing and lands on
    /// the standard form.
    pub fn from_current_form(current_form: Option<&str>) -> Self {
        let prefix = current_form
            .unwrap_or("")
            .rsplit_once('-')
            .map_or("", |(prefix, _)| prefix);
        Self::ALL
            .iter()
            .copied()
            .find(|form| form.key() == prefix)
            .unwrap_or(Form::Standard)
    }
}

/// The `paperSize` key strings.
pub fn paper_key(paper: PaperSize) -> &'static str {
    match paper {
        PaperSize::Legal => "legal",
        PaperSize::Letter => "letter",
        PaperSize::A4 => "a4",
    }
}

pub fn paper_from_key(key: &str) -> Option<PaperSize> {
    match key {
        "legal" => Some(PaperSize::Legal),
        "letter" => Some(PaperSize::Letter),
        "a4" => Some(PaperSize::A4),
        _ => None,
    }
}

/// `paperLabel`: `paper === 'a4' ? 'A4' : paper[0].toUpperCase() + slice`.
pub fn paper_label(paper: PaperSize) -> &'static str {
    match paper {
        PaperSize::Legal => "Legal",
        PaperSize::Letter => "Letter",
        PaperSize::A4 => "A4",
    }
}

/// All papers the dialog's radio row offers.
pub const PAPERS: [PaperSize; 3] = [PaperSize::Legal, PaperSize::Letter, PaperSize::A4];

/// The `captions` map (`DEFAULT_PAGE_LAYOUT.captions`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Captions {
    pub sketch_page_title: String,
    pub subject_label: String,
    pub sketch_label: String,
    pub summary_label: String,
    pub comment_table1_label: String,
    pub comment_table2_label: String,
    pub comment_table3_label: String,
}

impl Default for Captions {
    fn default() -> Self {
        Captions {
            sketch_page_title: "SKETCH/AREA TABLE ADDENDUM".to_string(),
            subject_label: "SUBJECT INFO".to_string(),
            sketch_label: "SKETCH".to_string(),
            summary_label: "AREA CALCULATIONS SUMMARY".to_string(),
            comment_table1_label: "COMMENT TABLE 1".to_string(),
            comment_table2_label: "COMMENT TABLE 2".to_string(),
            comment_table3_label: "COMMENT TABLE 3".to_string(),
        }
    }
}

/// The dialog's caption fields, in the form's order.
pub const CAPTIONS: [(&str, Key); 7] = [
    ("Sketch Page Title:", Key::SketchPageTitle),
    ("Subject Label:", Key::SubjectLabel),
    ("Sketch Label:", Key::SketchLabel),
    ("Summary Label:", Key::SummaryLabel),
    ("Comment Table 1 Label:", Key::CommentTable1Label),
    ("Comment Table 2 Label:", Key::CommentTable2Label),
    ("Comment Table 3 Label:", Key::CommentTable3Label),
];

/// A caption key (the dialog's field index; matches `captionFields`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    SketchPageTitle,
    SubjectLabel,
    SketchLabel,
    SummaryLabel,
    CommentTable1Label,
    CommentTable2Label,
    CommentTable3Label,
}

/// The five addenda switches, in the form's order.
pub const OPTIONS: [(Key2, &str); 5] = [
    (Key2::ShowSketchComments, "Show Sketch Comments"),
    (Key2::ShowCommentTables, "Show Comment Tables"),
    (Key2::ShowPageNumbers, "Show Page Numbers"),
    (Key2::CalculationAddendum, "Calculation Addendum"),
    (Key2::DeedCallsAddendum, "Deed Calls Addendum"),
];

/// An addenda switch key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key2 {
    ShowSketchComments,
    ShowCommentTables,
    ShowPageNumbers,
    CalculationAddendum,
    DeedCallsAddendum,
}

/// `state.pageLayout`. The dialog edits a draft, and
/// `setPageLayout(apply)` runs the stored layout.
#[derive(Debug, Clone, PartialEq)]
pub struct PageLayout {
    pub paper_size: PaperSize,
    /// `currentForm` = `form.key() + "-" + paper_key`.
    pub form: Form,
    pub show_sketch_comments: bool,
    pub show_comment_tables: bool,
    pub show_page_numbers: bool,
    pub calculation_addendum: bool,
    pub deed_calls_addendum: bool,
    pub captions: Captions,
}

impl Default for PageLayout {
    /// `DEFAULT_PAGE_LAYOUT` (ui-manager.js:3283).
    fn default() -> Self {
        PageLayout {
            paper_size: PaperSize::Legal,
            form: Form::Standard,
            show_sketch_comments: true,
            show_comment_tables: true,
            show_page_numbers: true,
            calculation_addendum: false,
            deed_calls_addendum: false,
            captions: Captions::default(),
        }
    }
}

impl PageLayout {
    /// The stored `currentForm.
    pub fn current_form(&self) -> String {
        format!("{}-{}", self.form.key(), paper_key(self.paper_size))
    }

    pub fn get_bool(&self, key: Key2) -> bool {
        match key {
            Key2::ShowSketchComments => self.show_sketch_comments,
            Key2::ShowCommentTables => self.show_comment_tables,
            Key2::ShowPageNumbers => self.show_page_numbers,
            Key2::CalculationAddendum => self.calculation_addendum,
            Key2::DeedCallsAddendum => self.deed_calls_addendum,
        }
    }

    pub fn set_bool(&mut self, key: Key2, value: bool) {
        match key {
            Key2::ShowSketchComments => self.show_sketch_comments = value,
            Key2::ShowCommentTables => self.show_comment_tables = value,
            Key2::ShowPageNumbers => self.show_page_numbers = value,
            Key2::CalculationAddendum => self.calculation_addendum = value,
            Key2::DeedCallsAddendum => self.deed_calls_addendum = value,
        }
    }

    pub fn get_caption(&self, key: Key) -> &str {
        match key {
            Key::SketchPageTitle => &self.captions.sketch_page_title,
            Key::SubjectLabel => &self.captions.subject_label,
            Key::SketchLabel => &self.captions.sketch_label,
            Key::SummaryLabel => &self.captions.summary_label,
            Key::CommentTable1Label => &self.captions.comment_table1_label,
            Key::CommentTable2Label => &self.captions.comment_table2_label,
            Key::CommentTable3Label => &self.captions.comment_table3_label,
        }
    }

    pub fn set_caption(&mut self, key: Key, value: String) {
        let label = match key {
            Key::SketchPageTitle => &mut self.captions.sketch_page_title,
            Key::SubjectLabel => &mut self.captions.subject_label,
            Key::SketchLabel => &mut self.captions.sketch_label,
            Key::SummaryLabel => &mut self.captions.summary_label,
            Key::CommentTable1Label => &mut self.captions.comment_table1_label,
            Key::CommentTable2Label => &mut self.captions.comment_table2_label,
            Key::CommentTable3Label => &mut self.captions.comment_table3_label,
        };
        *label = value;
    }
}

// --- The save-model serde ------------------------------------------------------

impl Serialize for PageLayout {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(8))?;
        map.serialize_entry("paperSize", paper_key(self.paper_size))?;
        map.serialize_entry("currentForm", &self.current_form())?;
        map.serialize_entry("showSketchComments", &self.show_sketch_comments)?;
        map.serialize_entry("showCommentTables", &self.show_comment_tables)?;
        map.serialize_entry("showPageNumbers", &self.show_page_numbers)?;
        map.serialize_entry("calculationAddendum", &self.calculation_addendum)?;
        map.serialize_entry("deedCallsAddendum", &self.deed_calls_addendum)?;
        map.serialize_entry("captions", &self.captions)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for PageLayout {
    /// `normalizePageLayout`: an unrecognised paper or form prefix clamps to
    /// the defaults, and missing keys arrive through `Captions::default()`.
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize, Default)]
        #[serde(default, rename_all = "camelCase")]
        struct Raw {
            #[serde(default)]
            paper_size: Option<String>,
            #[serde(default)]
            current_form: Option<String>,
            // The five switches keep the original's DEFAULT_TABLE defaults
            // (true³; only the addenda default off).
            #[serde(default = "yes")]
            show_sketch_comments: bool,
            #[serde(default = "yes")]
            show_comment_tables: bool,
            #[serde(default = "yes")]
            show_page_numbers: bool,
            #[serde(default)]
            calculation_addendum: bool,
            #[serde(default)]
            deed_calls_addendum: bool,
            #[serde(default)]
            captions: Captions,
        }
        fn yes() -> bool {
            true
        }
        let raw = Raw::deserialize(d)?;
        Ok(PageLayout {
            paper_size: raw
                .paper_size
                .as_deref()
                .and_then(paper_from_key)
                .unwrap_or(PaperSize::Legal),
            form: Form::from_current_form(raw.current_form.as_deref()),
            show_sketch_comments: raw.show_sketch_comments,
            show_comment_tables: raw.show_comment_tables,
            show_page_numbers: raw.show_page_numbers,
            calculation_addendum: raw.calculation_addendum,
            deed_calls_addendum: raw.deed_calls_addendum,
            captions: raw.captions,
        })
    }
}

// --- The dialog (S11-033..037) ---------------------------------------------------

/// The dialog's state: open + the draft being edited
/// (`showPageLayout` seeds it from `normalizePageLayout(state.pageLayout)`).
#[derive(Debug, Clone, Default)]
pub struct PageLayoutDialog {
    pub open: bool,
    pub draft: PageLayout,
    /// The caption text buffers (`text_input` text per field).
    pub buffers: std::collections::BTreeMap<Key, String>,
}

impl PageLayoutDialog {
    pub fn open_with(layout: PageLayout) -> Self {
        let mut buffers = std::collections::BTreeMap::new();
        for (label, key) in CAPTIONS {
            let _ = label;
            buffers.insert(key, layout.get_caption(key).to_string());
        }
        PageLayoutDialog {
            open: true,
            draft: layout,
            buffers,
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.buffers.clear();
    }

    pub fn update(&mut self, msg: LayoutMsg) {
        match msg {
            LayoutMsg::RestoreDefaults => {
                // `data-defaults`: the DOM form gets the code defaults; the
                // buffers reflect the reset captions.
                self.draft = PageLayout::default();
                for (label, key) in CAPTIONS {
                    let _ = label;
                    self.buffers
                        .insert(key, self.draft.get_caption(key).to_string());
                }
            }
            LayoutMsg::SetPaper(paper) => {
                // `refreshForms(paper)`: switching the paper resets the form
                // pick to `standard-<paper>`.
                self.draft.paper_size = paper;
                self.draft.form = Form::Standard;
            }
            LayoutMsg::SetForm(form) => self.draft.form = form,
            LayoutMsg::Toggle(key) => {
                let value = !self.draft.get_bool(key);
                self.draft.set_bool(key, value);
            }
            LayoutMsg::Caption(key, text) => {
                // The original commits on submit (Apply); committing as
                // typed keeps the Apply handler simple.
                self.buffers.insert(key, text.clone());
                self.draft.set_caption(key, text);
            }
            LayoutMsg::Open | LayoutMsg::Cancel | LayoutMsg::Apply => {}
        }
    }

    /// The live caption text (the buffer while typing, else the draft).
    pub fn caption_text(&self, key: Key) -> &str {
        self.buffers.get(&key).map(|s| s.as_str()).unwrap_or("")
    }
}

/// The dialog's own messages (a `Message::PageLayout` arm routes them).
/// Not `Copy`: the caption fields carry their typed text.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMsg {
    Open,
    Cancel,
    Apply,
    RestoreDefaults,
    SetPaper(PaperSize),
    SetForm(Form),
    Toggle(Key2),
    Caption(Key, String),
}

// --- The layer ---------------------------------------------------------------

fn spacer() -> Space {
    Space::new().width(10.0)
}

fn btn(label: &'static str, msg: Message) -> Element<'static, Message> {
    button(container(text(label).font(ARIAL).size(12.0).color(text_alt())).padding([5, 12]))
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

fn group(title: &'static str, rows: Vec<Element<'static, Message>>) -> Element<'static, Message> {
    let mut col = column![text(title).font(ARIAL).size(12.0).color(text_alt())]
        .spacing(6)
        .padding(10);
    for r in rows {
        col = col.push(r);
    }
    container(col)
        .width(Length::Fill)
        .style(|_| container::Style {
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

fn row_of(items: Vec<Element<'static, Message>>) -> Element<'static, Message> {
    row(items).spacing(10).align_y(Alignment::Center).into()
}

fn check(label: &'static str, checked: bool, key: Key2) -> Element<'static, Message> {
    checkbox(checked)
        .label(label)
        .on_toggle(move |_| Message::PageLayout(LayoutMsg::Toggle(key)))
        .text_size(12.0)
        .spacing(6)
        .into()
}

/// `Caption Label: [input]`.
fn caption_field(d: &PageLayoutDialog, label: &'static str, key: Key) -> Element<'static, Message> {
    row![
        text(label).font(ARIAL).size(12.0).color(text_alt()),
        spacer(),
        text_input("", d.caption_text(key))
            .on_input(move |s| Message::PageLayout(LayoutMsg::Caption(key, s)))
            .width(Length::Fixed(280.0))
            .size(12.0)
            .padding([3, 6]),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn panel(d: &PageLayoutDialog) -> Element<'static, Message> {
    let draft = &d.draft;
    // Form Selection: the paper radios + the form pick (labels follow the
    // draft's paper, like `refreshForms`).
    let paper_radios = PAPERS
        .iter()
        .map(|paper| {
            radio(paper_label(*paper), *paper, Some(draft.paper_size), |p| {
                Message::PageLayout(LayoutMsg::SetPaper(p))
            })
            .text_size(12.0)
            .spacing(6)
            .into()
        })
        .collect::<Vec<Element<'static, Message>>>();
    let form_options: Vec<FormChoice> = Form::ALL
        .iter()
        .map(|form| FormChoice {
            form: *form,
            label: form.label(draft.paper_size),
        })
        .collect();
    let paper = draft.paper_size;
    let form_pick = pick_list(
        form_options,
        Some(FormChoice {
            form: draft.form,
            label: draft.form.label(paper),
        }),
        move |choice: FormChoice| Message::PageLayout(LayoutMsg::SetForm(choice.form)),
    )
    .text_size(12.0)
    .width(Length::Fixed(240.0));
    let form_selection = group(
        "Form Selection",
        vec![
            row_of(vec![
                text("Paper Size:")
                    .font(ARIAL)
                    .size(12.0)
                    .color(text_alt())
                    .into(),
                spacer().into(),
                row(paper_radios).spacing(12).into(),
            ]),
            row_of(vec![
                text("Current Form:")
                    .font(ARIAL)
                    .size(12.0)
                    .color(text_alt())
                    .into(),
                spacer().into(),
                form_pick.into(),
            ]),
        ],
    );
    // Additional Options: the five switches.
    let additional = group(
        "Additional Options",
        OPTIONS
            .iter()
            .map(|(key, label)| check(label, draft.get_bool(*key), *key))
            .collect(),
    );
    // Form Captions: the seven text fields.
    let captions = group(
        "Form Captions",
        CAPTIONS
            .iter()
            .map(|(label, key)| caption_field(d, label, *key))
            .collect(),
    );
    column![form_selection, additional, captions]
        .spacing(10)
        .width(Length::Fill)
        .into()
}

/// A `pick_list` option: the form plus its paper-specific label.
#[derive(Debug, Clone, PartialEq, Eq)]
struct FormChoice {
    form: Form,
    label: String,
}

impl std::fmt::Display for FormChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// The dialog's card size (the Screen-11 measured overlay family).
const CARD: (f32, f32) = (560.0, 520.0);

/// The dialog overlay, or None when closed.
pub fn layer(d: &PageLayoutDialog) -> Option<Element<'static, Message>> {
    if !d.open {
        return None;
    }
    let title = row![
        text("Page Layout Settings")
            .font(ARIAL)
            .size(15.0)
            .color(text_alt()),
        Space::new().width(Length::Fill),
        button(text("×").font(ARIAL).size(14.0).color(text_alt()))
            .on_press(Message::PageLayout(LayoutMsg::Cancel)),
    ]
    .padding(8)
    .spacing(6);

    let body = scrollable(container(panel(d)).width(Length::Fill).padding([0, 8]))
        .height(Length::FillPortion(1));

    // Actions: Restore Defaults | ... | Apply | Cancel (the form's order).
    let actions = row![
        btn(
            "Restore Defaults",
            Message::PageLayout(LayoutMsg::RestoreDefaults)
        ),
        Space::new().width(Length::Fill),
        btn("Apply", Message::PageLayout(LayoutMsg::Apply)),
        spacer(),
        btn("Cancel", Message::PageLayout(LayoutMsg::Cancel)),
    ]
    .padding(8)
    .spacing(8)
    .align_y(Alignment::Center);

    let card = container(
        column![title, body, actions]
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

// --- Unit tests -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_default_page_layout() {
        let layout = PageLayout::default();
        let json = serde_json::to_value(&layout).unwrap();
        assert_eq!(json["paperSize"], "legal");
        assert_eq!(json["currentForm"], "standard-legal");
        assert_eq!(json["showSketchComments"], true);
        assert_eq!(json["showCommentTables"], true);
        assert_eq!(json["showPageNumbers"], true);
        assert_eq!(json["calculationAddendum"], false);
        assert_eq!(json["deedCallsAddendum"], false);
        assert_eq!(
            json["captions"]["sketchPageTitle"],
            "SKETCH/AREA TABLE ADDENDUM"
        );
        assert_eq!(json["captions"]["commentTable3Label"], "COMMENT TABLE 3");
    }

    #[test]
    fn normalize_clamps_unknown_paper_and_form() {
        // applySaveModel: normalizePageLayout(data.settings.pageLayout).
        let raw = serde_json::json!({
            "paperSize": "tabloid",
            "currentForm": "ledger-letter",
            "showSketchComments": false,
            "showCommentTables": true,
            "showPageNumbers": false,
            "calculationAddendum": true,
            "deedCallsAddendum": true,
            "captions": {"sketchLabel": "DRAWING"}
        });
        let layout: PageLayout = serde_json::from_value(raw).unwrap();
        assert_eq!(layout.paper_size, PaperSize::Legal);
        // The ` || 'standard'` pass, then the allowed-list clamp.
        assert_eq!(layout.form, Form::Standard);
        assert!(!layout.show_sketch_comments);
        assert!(!layout.show_page_numbers);
        assert!(layout.calculation_addendum);
        assert_eq!(layout.get_caption(Key::SketchLabel), "DRAWING");
        assert_eq!(
            layout.get_caption(Key::SummaryLabel),
            "AREA CALCULATIONS SUMMARY"
        );
        // A suffix-less currentForm splits to nothing and lands on standard.
        assert_eq!(
            serde_json::from_value::<PageLayout>(serde_json::json!({
                "currentForm": "sketch-only"
            }))
            .unwrap()
            .form,
            Form::Standard
        );
    }

    #[test]
    fn a_full_layout_round_trips_and_composes_the_form() {
        let layout = PageLayout {
            paper_size: PaperSize::A4,
            form: Form::WithCalculations,
            ..PageLayout::default()
        };
        let json = serde_json::to_value(&layout).unwrap();
        assert_eq!(json["currentForm"], "with-calculations-a4");
        let back: PageLayout = serde_json::from_value(json).unwrap();
        assert_eq!(back, layout);
        // The form picker's labels follow the paper.
        assert_eq!(
            Form::SketchOnly.label(PaperSize::Letter),
            "Letter Size Sketch Only"
        );
        assert_eq!(Form::FullPage.label(PaperSize::A4), "Full Page A4");
    }

    #[test]
    fn the_dialog_edits_a_draft_and_applies_the_form_composition() {
        let mut d = PageLayoutDialog::open_with(PageLayout::default());
        assert!(d.open);
        d.update(LayoutMsg::SetPaper(PaperSize::A4));
        // `refreshForms` resets the form pick with the paper.
        assert_eq!(d.draft.form, Form::Standard);
        d.update(LayoutMsg::SetForm(Form::SketchOnly));
        d.update(LayoutMsg::Toggle(Key2::CalculationAddendum));
        d.update(LayoutMsg::Caption(Key::SketchLabel, "PLAN".to_string()));
        // The sketch-label buffer carries the typed value.
        assert_eq!(d.caption_text(Key::SketchLabel), "PLAN");
        assert_eq!(d.draft.current_form(), "sketch-only-a4");
        assert!(d.draft.calculation_addendum);
        // Restore Defaults returns everything to the code defaults.
        d.update(LayoutMsg::RestoreDefaults);
        assert_eq!(d.draft, PageLayout::default());
        assert_eq!(d.caption_text(Key::SketchLabel), "SKETCH");
        d.close();
        assert!(!d.open);
    }
}
