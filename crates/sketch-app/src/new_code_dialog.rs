//! The New Area Code dialog (screen 7, S7-009): `showAddAreaCode`.
//!
//! Measured from the live original at 1400x860: a 500x540 card at (450,160)
//! with `--bg-panel`, radius 5 and 12px padding. Ten rows put their label on
//! the left and the control at x 601, 336 wide: Code 206, Name 245, Description
//! 284, Group 323, Multiplier 362 (29px pitch), Include in GLA 401 (a 13px
//! checkbox), Line color 427, Fill color 467, Fill style 507 (a 31px select),
//! Fill opacity 548 (a 286px range). The preview is 474x62 at (463,588) and
//! Cancel (59.5) / Create Code (93.6) sit at y 660.
//!
//! The create logic is the original's form submit, including its exact notices.

use iced::widget::{button, checkbox, container, pick_list, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::snap::nudge;
use crate::theme::{bg_app, bg_hover, bg_panel, border, muted};
use crate::theme::{hex, text_main, ARIAL};
use crate::Message;

const CARD: (f32, f32, f32, f32) = (450.0, 160.0, 500.0, 540.0);
/// See `area_dialog::PIN_Y`.
const PIN_Y: f32 = 2.0;

/// The Fill style options, in the original's order.
/// One label + input row of the form: its top, label, value and setter.
type TextRow = (f32, &'static str, String, fn(String) -> NewCodeMsg);

pub const FILL_STYLES: [&str; 5] = ["solid", "none", "diagonal", "crosshatch", "dots"];
pub fn style_label(value: &str) -> &'static str {
    match value {
        "none" => "None",
        "diagonal" => "Diagonal",
        "crosshatch" => "Crosshatch",
        "dots" => "Dots",
        _ => "Solid",
    }
}
pub fn style_value(label: &str) -> &'static str {
    match label {
        "None" => "none",
        "Diagonal" => "diagonal",
        "Crosshatch" => "crosshatch",
        "Dots" => "dots",
        _ => "solid",
    }
}

/// The dialog's state.
#[derive(Debug, Clone)]
pub struct NewCodeDialog {
    pub open: bool,
    pub code: String,
    pub name: String,
    pub description: String,
    pub group: String,
    pub multiplier_text: String,
    pub include_gla: bool,
    pub color: String,
    pub fill_color: String,
    pub fill_style: String,
    pub fill_opacity: f32,
}

impl Default for NewCodeDialog {
    fn default() -> Self {
        NewCodeDialog {
            open: false,
            code: String::new(),
            name: String::new(),
            description: String::new(),
            group: "GLA".into(),
            multiplier_text: "1".into(),
            include_gla: false,
            color: "#718096".into(),
            fill_color: "#718096".into(),
            fill_style: "solid".into(),
            fill_opacity: 10.0,
        }
    }
}

/// A control in the dialog.
#[derive(Debug, Clone)]
pub enum NewCodeMsg {
    Code(String),
    Name(String),
    Description(String),
    Group(String),
    Multiplier(String),
    IncludeGla(bool),
    FillStyle(String),
    FillOpacity(f32),
    Create,
    Cancel,
}

impl NewCodeDialog {
    /// Open with the first group preselected (`preferredGroup || groups[0]`).
    pub fn open_with(group: &str) -> NewCodeDialog {
        NewCodeDialog {
            open: true,
            group: if group.is_empty() { "OTH" } else { group }.to_string(),
            ..NewCodeDialog::default()
        }
    }

    pub fn update(&mut self, msg: NewCodeMsg) {
        match msg {
            NewCodeMsg::Code(v) => self.code = v,
            NewCodeMsg::Name(v) => self.name = v,
            NewCodeMsg::Description(v) => self.description = v,
            NewCodeMsg::Group(v) => self.group = v,
            NewCodeMsg::Multiplier(v) => self.multiplier_text = v,
            NewCodeMsg::IncludeGla(v) => self.include_gla = v,
            NewCodeMsg::FillStyle(v) => self.fill_style = v,
            NewCodeMsg::FillOpacity(v) => self.fill_opacity = v,
            NewCodeMsg::Create | NewCodeMsg::Cancel => {}
        }
    }

    /// The entry this form would create, or the original's notice.
    pub fn create(
        &self,
        existing: &[sketch_core::geometry::AreaCode],
    ) -> Result<sketch_core::geometry::AreaCode, String> {
        let code = self.code.trim().to_uppercase();
        let name = self.name.trim().to_string();
        if code.is_empty() {
            return Err("Code is required.".into());
        }
        if name.is_empty() {
            return Err("Name is required.".into());
        }
        if existing.iter().any(|c| c.code == code) {
            return Err(format!("Area code \"{code}\" already exists."));
        }
        let description = {
            let d = self.description.trim();
            if d.is_empty() {
                name.clone()
            } else {
                d.to_string()
            }
        };
        let group = {
            let g = self.group.trim().to_uppercase();
            if g.is_empty() {
                "OTH".to_string()
            } else {
                g
            }
        };
        // `Math.max(0, Number(x) || 1)`.
        let multiplier = self
            .multiplier_text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| *v != 0.0 && v.is_finite())
            .unwrap_or(1.0)
            .max(0.0);
        let entry = sketch_core::geometry::AreaCode {
            code,
            name,
            description,
            group,
            multiplier,
            positive: true,
            include_in_gla: self.include_gla,
            color: self.color.clone(),
            fill: format!("rgba(113,128,150,{:.2})", self.fill_opacity / 100.0),
            fill_color: self.fill_color.clone(),
            fill_style: self.fill_style.clone(),
            fill_opacity: self.fill_opacity as f64,
        };
        Ok(sketch_core::geometry::area_codes::normalize_area_code(
            entry,
        ))
    }
}

fn place<'a>(x: f32, y: f32, el: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    crate::theme::at(x - CARD.0, y - CARD.1 + PIN_Y, el)
}

/// The dialog's layer, or None when closed.
pub fn layer<'a>(d: &'a NewCodeDialog) -> Option<Element<'a, Message>> {
    if !d.open {
        return None;
    }
    let send = |m: NewCodeMsg| Message::NewCode(m);
    let mut body = iced::widget::stack![]
        .width(Length::Fill)
        .height(Length::Fill);

    body = body.push(place(
        463.0,
        172.0,
        nudge(
            0.5,
            0.0,
            text("New Area Code")
                .font(ARIAL)
                .size(16.0)
                .color(text_main()),
        ),
    ));
    body = body.push(place(
        928.0,
        170.0,
        button(text("\u{d7}").font(ARIAL).size(16.0).color(muted()))
            .on_press(send(NewCodeMsg::Cancel))
            .padding(0)
            .width(12.0)
            .height(23.0)
            .style(|_, _| button::Style {
                background: None,
                border: Border::default(),
                text_color: muted(),
                ..button::Style::default()
            }),
    ));

    // The text rows: label left, a 336-wide control at x 601.
    let rows: [TextRow; 4] = [
        (206.0, "Code", d.code.clone(), NewCodeMsg::Code),
        (245.0, "Name", d.name.clone(), NewCodeMsg::Name),
        (
            284.0,
            "Description",
            d.description.clone(),
            NewCodeMsg::Description,
        ),
        (323.0, "Group", d.group.clone(), NewCodeMsg::Group),
    ];
    for (y, label, value, make) in rows {
        body = body.push(place(
            463.0,
            y + 7.0,
            nudge(
                0.5,
                0.0,
                text(label).font(ARIAL).size(11.0).color(text_main()),
            ),
        ));
        body = body.push(place(
            601.0,
            y,
            text_input("", &value)
                .on_input(move |v| Message::NewCode(make(v)))
                .font(ARIAL)
                .size(13.33)
                .padding(Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 8.0,
                    right: 8.0,
                })
                .width(336.0)
                .style(|_, _| iced::widget::text_input::Style {
                    background: Background::Color(bg_app()),
                    border: Border {
                        color: border(),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    icon: muted(),
                    placeholder: muted(),
                    value: text_main(),
                    selection: crate::dialog::SELECTION,
                }),
        ));
    }
    // Multiplier.
    body = body.push(place(
        463.0,
        369.0,
        nudge(
            0.5,
            0.0,
            text("Multiplier").font(ARIAL).size(11.0).color(text_main()),
        ),
    ));
    body = body.push(place(
        601.0,
        362.0,
        text_input("", &d.multiplier_text)
            .on_input(|v| Message::NewCode(NewCodeMsg::Multiplier(v)))
            .font(ARIAL)
            .size(13.33)
            .padding(Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 8.0,
            })
            .width(336.0)
            .style(|_, _| iced::widget::text_input::Style {
                background: Background::Color(bg_app()),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 2.0.into(),
                },
                icon: muted(),
                placeholder: muted(),
                value: text_main(),
                selection: crate::dialog::SELECTION,
            }),
    ));
    // Include in GLA.
    body = body.push(place(
        463.0,
        400.0,
        nudge(
            0.5,
            0.0,
            text("Include in GLA")
                .font(ARIAL)
                .size(11.0)
                .color(text_main()),
        ),
    ));
    body = body.push(place(
        601.0,
        400.0,
        checkbox(d.include_gla)
            .on_toggle(|v| Message::NewCode(NewCodeMsg::IncludeGla(v)))
            .size(13.0),
    ));
    // The two colour swatches.
    for (label, y, color) in [
        ("Line color", 427.0, &d.color),
        ("Fill color", 467.0, &d.fill_color),
    ] {
        body = body.push(place(
            463.0,
            y + 6.0,
            nudge(
                0.5,
                0.0,
                text(label).font(ARIAL).size(11.0).color(text_main()),
            ),
        ));
        body = body.push(place(
            601.0,
            y - 1.0,
            container(Space::new())
                .width(32.0)
                .height(24.0)
                .style(move |_| container::Style {
                    background: Some(Background::Color(
                        crate::render::parse_hex(color).unwrap_or(hex(0x718096)),
                    )),
                    border: Border {
                        color: border(),
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..container::Style::default()
                }),
        ));
    }
    // Fill style.
    body = body.push(place(
        463.0,
        517.0,
        nudge(
            0.5,
            0.0,
            text("Fill style").font(ARIAL).size(11.0).color(text_main()),
        ),
    ));
    body = body.push(place(
        601.0,
        507.0,
        pick_list(
            FILL_STYLES
                .iter()
                .map(|s| style_label(s).to_string())
                .collect::<Vec<_>>(),
            Some(style_label(&d.fill_style).to_string()),
            |v: String| Message::NewCode(NewCodeMsg::FillStyle(style_value(&v).to_string())),
        )
        .width(336.0)
        .text_size(13.0)
        .font(ARIAL)
        .style(|_, _| pick_list::Style {
            text_color: text_main(),
            placeholder_color: muted(),
            handle_color: muted(),
            background: Background::Color(bg_app()),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
        }),
    ));
    // Fill opacity.
    body = body.push(place(
        463.0,
        558.0,
        nudge(
            0.5,
            0.0,
            text("Fill opacity")
                .font(ARIAL)
                .size(11.0)
                .color(text_main()),
        ),
    ));
    body = body.push(place(
        601.0,
        553.0,
        iced::widget::slider(0.0..=100.0, d.fill_opacity, |v| {
            Message::NewCode(NewCodeMsg::FillOpacity(v))
        })
        .width(286.0)
        .step(1.0_f32),
    ));
    // The style preview.
    body = body.push(place(
        463.0,
        588.0,
        container(
            container(nudge(
                0.5,
                0.0,
                text("Area style preview")
                    .font(ARIAL)
                    .size(11.0)
                    .color(muted()),
            ))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        )
        .width(474.0)
        .height(62.0)
        .style(move |_| container::Style {
            background: Some(Background::Color(
                crate::render::parse_hex(&d.fill_color)
                    .map(|c| Color {
                        a: d.fill_opacity / 100.0,
                        ..c
                    })
                    .unwrap_or(hex(0x718096)),
            )),
            border: Border {
                color: crate::render::parse_hex(&d.color).unwrap_or(border()),
                width: 1.0,
                radius: 2.0.into(),
            },
            ..container::Style::default()
        }),
    ));
    // Actions.
    body = body.push(place(
        775.9,
        660.0,
        dialog_button("Cancel", 59.5, NewCodeMsg::Cancel),
    ));
    body = body.push(place(
        843.4,
        660.0,
        dialog_button("Create Code", 93.6, NewCodeMsg::Create),
    ));

    let card = container(body)
        .width(CARD.2)
        .height(CARD.3)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 5.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: iced::Vector::new(0.0, 16.0),
                blur_radius: 23.0,
            },
            ..container::Style::default()
        });
    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.46))),
            ..container::Style::default()
        });
    Some(
        iced::widget::stack![
            iced::widget::opaque(
                iced::widget::mouse_area(backdrop).on_press(send(NewCodeMsg::Cancel))
            ),
            crate::theme::at(CARD.0, CARD.1, card)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

fn dialog_button<'a>(label: &'a str, width: f32, msg: NewCodeMsg) -> Element<'a, Message> {
    button(text(label).font(ARIAL).size(13.33).color(text_main()))
        .on_press(Message::NewCode(msg))
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 10.0,
            right: 10.0,
        })
        .width(width)
        .height(27.0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sketch_core::geometry::area_codes::default_area_codes;

    #[test]
    fn a_new_code_is_normalized_and_appended() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("GLA");
        d.update(NewCodeMsg::Code("  gla5 ".into()));
        d.update(NewCodeMsg::Name(" Fifth Floor ".into()));
        d.update(NewCodeMsg::IncludeGla(true));
        let entry = d.create(&existing).expect("created");
        assert_eq!(entry.code, "GLA5", "upper-cased and trimmed");
        assert_eq!(entry.name, "Fifth Floor");
        assert_eq!(
            entry.description, "Fifth Floor",
            "description defaults to the name"
        );
        assert_eq!(entry.group, "GLA");
        assert_eq!(entry.multiplier, 1.0);
        assert!(entry.include_in_gla);
    }

    #[test]
    fn the_originals_notices_come_back_verbatim() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("GLA");
        d.update(NewCodeMsg::Name("Fifth Floor".into()));
        assert_eq!(d.create(&existing).unwrap_err(), "Code is required.");
        d.update(NewCodeMsg::Code("GLA5".into()));
        d.update(NewCodeMsg::Name("  ".into()));
        assert_eq!(d.create(&existing).unwrap_err(), "Name is required.");
        d.update(NewCodeMsg::Name("Fifth".into()));
        d.update(NewCodeMsg::Code("gla1".into()));
        assert_eq!(
            d.create(&existing).unwrap_err(),
            "Area code \"GLA1\" already exists."
        );
    }

    #[test]
    fn the_multiplier_follows_the_js_coercion() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("OTH");
        d.update(NewCodeMsg::Code("X1".into()));
        d.update(NewCodeMsg::Name("Test".into()));
        // `Number(x) || 1`, so junk and 0 become 1, and negatives clamp to 0.
        for (text, want) in [
            ("2.5", 2.5),
            ("", 1.0),
            ("abc", 1.0),
            ("0", 1.0),
            ("-3", 0.0),
        ] {
            d.update(NewCodeMsg::Multiplier(text.into()));
            let entry = d.create(&existing).expect("created");
            assert_eq!(entry.multiplier, want, "multiplier {text:?}");
        }
    }

    #[test]
    fn an_empty_group_becomes_oth() {
        let existing = default_area_codes();
        let mut d = NewCodeDialog::open_with("");
        d.update(NewCodeMsg::Code("X2".into()));
        d.update(NewCodeMsg::Name("Test".into()));
        d.update(NewCodeMsg::Group("  ".into()));
        let entry = d.create(&existing).expect("created");
        assert_eq!(entry.group, "OTH");
    }
}
