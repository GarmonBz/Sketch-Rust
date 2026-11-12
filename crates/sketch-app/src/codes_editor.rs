//! The area-code table editor (screen 7, S7-004 / S7-010): `showAreaCodeEditor`.
//!
//! Measured from the live original at 1400x860: a full-screen
//! `.breakdown-overlay` at `rgba(0,0,0,.62)` holding a 720x554.8 dialog at
//! (340,152.59) with `--bg-panel`, a 1px `--border` and NO radius; a 44px title
//! row, a 12px `<p>` at (355,207.6), a 690x412.8 monospace textarea at
//! (355,233.59) on `#111111`, and four `#3E3E42` buttons at y 667.4 — Apply
//! (59.4), Import JSON (103), Export JSON (103.8), Restore defaults (123.1).
//!
//! Apply runs the already-ported `AreaCodes::from_json`, so its error messages
//! are the original's verbatim. Import stages file text without applying it;
//! Export validates and normalizes the draft before the native save dialog.

use iced::widget::{button, container, text, text_editor, Space};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::snap::nudge;
use crate::theme::{bg_canvas, bg_hover, bg_panel, border, muted};
use crate::theme::{text_main, ARIAL, SEGOE};
use crate::Message;

/// The dialog's card: (x, y, w, h).
const CARD: (f32, f32, f32, f32) = (340.0, 152.59, 720.0, 554.8);
/// A pinned child inside the card lands 2px above its measured position (see
/// `area_dialog::PIN_Y`).
const PIN_Y: f32 = 2.0;

/// The editor's own state: whether it is open plus the text being edited.
#[derive(Debug, Clone, Default)]
pub struct CodesEditor {
    pub open: bool,
    /// The JSON text. `None` until the dialog opens (a `text_editor::Content`
    /// cannot be built without its starting text).
    pub content: Option<text_editor::Content>,
    /// Invalidates file-dialog results when the editor closes or reopens.
    pub session: u64,
}

/// A control in the editor.
#[derive(Debug, Clone)]
pub enum CodesMsg {
    /// A `text_editor` action (typing, selecting, scrolling).
    Edit(text_editor::Action),
    Apply,
    Import,
    Export,
    ImportChosen(u64, Option<std::path::PathBuf>),
    ExportChosen {
        session: u64,
        body: String,
        path: Option<std::path::PathBuf>,
    },
    RestoreDefaults,
    Close,
}

impl CodesEditor {
    /// Open with the current table's JSON (`AreaCodes.toJSON`).
    pub fn open_with(json: String) -> CodesEditor {
        CodesEditor {
            open: true,
            content: Some(text_editor::Content::with_text(&json)),
            session: 0,
        }
    }

    pub fn open(&mut self, json: String) {
        let session = self.session.wrapping_add(1);
        *self = Self::open_with(json);
        self.session = session;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.content = None;
        self.session = self.session.wrapping_add(1);
    }

    /// The textarea's current text.
    pub fn text(&self) -> String {
        self.content.as_ref().map(|c| c.text()).unwrap_or_default()
    }

    pub fn export_json(&self) -> Result<String, sketch_core::geometry::AreaCodesError> {
        use sketch_core::geometry::area_codes::{from_json, to_json};
        to_json(from_json(&self.text())?)
    }

    pub fn update(&mut self, msg: CodesMsg) {
        match msg {
            CodesMsg::Edit(action) => {
                if let Some(content) = self.content.as_mut() {
                    content.perform(action);
                }
            }
            CodesMsg::RestoreDefaults => {
                let json = sketch_core::geometry::area_codes::to_json(
                    sketch_core::geometry::area_codes::default_area_codes(),
                )
                .unwrap_or_default();
                self.content = Some(text_editor::Content::with_text(&json));
            }
            _ => {}
        }
    }
}

fn place<'a>(x: f32, y: f32, el: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    crate::theme::at(x - CARD.0, y - CARD.1 + PIN_Y, el)
}

/// The editor's layer, or None when closed.
pub fn layer<'a>(d: &'a CodesEditor) -> Option<Element<'a, Message>> {
    if !d.open {
        return None;
    }
    let content = d.content.as_ref()?;
    let mut body = iced::widget::stack![]
        .width(Length::Fill)
        .height(Length::Fill);

    body = body.push(place(
        355.0,
        164.0,
        nudge(
            0.5,
            0.0,
            text("Area code table")
                .font(SEGOE)
                .size(16.0)
                .color(text_main()),
        ),
    ));
    body = body.push(place(
        1042.0,
        157.0,
        button(text("\u{d7}").font(ARIAL).size(16.0).color(muted()))
            .on_press(Message::CodesEditor(CodesMsg::Close))
            .padding(0)
            .width(14.0)
            .height(24.0)
            .style(|_, _| button::Style {
                background: None,
                border: Border::default(),
                text_color: muted(),
                ..button::Style::default()
            }),
    ));
    body = body.push(place(
        355.0,
        207.0,
        nudge(
            0.5,
            0.0,
            text("Edit the JSON table, or import/export a reusable code set.")
                .font(ARIAL)
                .size(12.0)
                .color(muted()),
        ),
    ));
    body = body.push(place(
        355.0,
        236.5,
        text_editor(content)
            .on_action(|a| Message::CodesEditor(CodesMsg::Edit(a)))
            .padding(8)
            .height(412.8)
            .width(690.0)
            .font(CONSOLAS_FONT)
            .size(12.0)
            .line_height(iced::widget::text::LineHeight::Absolute(iced::Pixels(16.5)))
            .style(|_, _| text_editor::Style {
                background: Background::Color(bg_canvas()),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                placeholder: muted(),
                value: text_main(),
                selection: crate::dialog::SELECTION,
            }),
    ));
    // The four actions, at their measured x and widths.
    let actions: [(&str, f32, f32, CodesMsg); 4] = [
        ("Apply", 355.0, 59.4, CodesMsg::Apply),
        ("Import JSON", 422.4, 103.0, CodesMsg::Import),
        ("Export JSON", 533.4, 103.8, CodesMsg::Export),
        ("Restore defaults", 645.2, 123.1, CodesMsg::RestoreDefaults),
    ];
    for (label, x, w, msg) in actions {
        body = body.push(place(x, 667.4, editor_button(label, w, msg)));
    }

    let card = container(body)
        .width(CARD.2)
        .height(CARD.3)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.62))),
            ..container::Style::default()
        });
    Some(
        iced::widget::stack![
            iced::widget::opaque(
                iced::widget::mouse_area(backdrop).on_press(Message::CodesEditor(CodesMsg::Close))
            ),
            crate::theme::at(CARD.0, CARD.1, card)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

fn editor_button<'a>(label: &'a str, width: f32, msg: CodesMsg) -> Element<'a, Message> {
    button(text(label).font(ARIAL).size(13.33).color(text_main()))
        .on_press(Message::CodesEditor(msg))
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 10.0,
            right: 10.0,
        })
        .width(width)
        .height(29.0)
        .style(|_, _| button::Style {
            background: Some(Background::Color(bg_hover())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            text_color: text_main(),
            ..button::Style::default()
        })
        .into()
}

/// `Consolas` for the JSON body, as the original's textarea uses.
const CONSOLAS_FONT: iced::Font = iced::Font::with_name("Consolas");

/// Test builds inject picker results without opening native dialogs.
#[cfg(not(test))]
pub async fn pick_import() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Area code JSON", &["json"])
        .pick_file()
        .await
        .map(|file| file.path().to_path_buf())
}

#[cfg(not(test))]
pub async fn pick_export() -> Option<std::path::PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Area code JSON", &["json"])
        .set_file_name("sketch-area-codes.json")
        .save_file()
        .await
        .map(|file| file.path().to_path_buf())
}

#[cfg(test)]
pub async fn pick_import() -> Option<std::path::PathBuf> {
    std::future::pending().await
}

#[cfg(test)]
pub async fn pick_export() -> Option<std::path::PathBuf> {
    std::future::pending().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_editor_opens_with_the_table_json() {
        let json = sketch_core::geometry::area_codes::to_json(
            sketch_core::geometry::area_codes::default_area_codes(),
        )
        .expect("json");
        let d = CodesEditor::open_with(json.clone());
        assert!(d.open);
        assert_eq!(d.text(), json);
        assert!(d.text().contains("\"includeInGLA\": true"));
    }

    #[test]
    fn restore_defaults_rewrites_the_text() {
        let mut d = CodesEditor::open_with("[]".to_string());
        assert_eq!(d.text(), "[]");
        d.update(CodesMsg::RestoreDefaults);
        let restored = d.text();
        assert!(restored.contains("GLA1"), "the default catalogue is back");
        // And it parses back into the 32 defaults.
        let codes = sketch_core::geometry::area_codes::from_json(&restored).expect("parse");
        assert_eq!(codes.len(), 32);
    }

    #[test]
    fn apply_reads_the_edited_text_through_the_catalogue() {
        // The editor's text is what Apply validates; a broken table yields the
        // original's message.
        let mut d = CodesEditor::open_with("[{\"code\":\"bad code\"}]".to_string());
        d.update(CodesMsg::RestoreDefaults);
        d.content = Some(text_editor::Content::with_text("[{\"code\":\"bad code\"}]"));
        let err = sketch_core::geometry::area_codes::from_json(&d.text()).unwrap_err();
        assert_eq!(err.to_string(), "Area code 1 has an invalid code.");
        d.content = Some(text_editor::Content::with_text(
            "[{\"code\":\"GLA1\",\"name\":\"First Floor\"}]",
        ));
        let codes = sketch_core::geometry::area_codes::from_json(&d.text()).expect("valid");
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].name, "First Floor");
    }

    #[test]
    fn the_card_and_buttons_are_the_measured_ones() {
        assert_eq!(CARD, (340.0, 152.59, 720.0, 554.8));
        // The four action buttons share y 667.4.
        let xs = [355.0_f32, 422.4, 533.4, 645.2];
        assert_eq!(xs.len(), 4);
    }
}
