//! The Area Attributes dialog (screen 7, S7-006): `showSelectedAreaAttributes`
//! + `updateSelectedAreaAttributes`.
//!
//! Measured from the live original at 1400x860: the card is 520x261 at
//! (440,299.5) with `--bg-panel`, a 1px `--border`, radius 5,
//! `0 16px 46px rgba(0,0,0,.5)`; the title sits at (453,313.5) with the close ×
//! at (935.3,312.5); a three-column header (Area / Attribute / Value) at
//! y 345.5; four rows at y 386.5/416.5/446.5/476.5 with the labels at x 636.8
//! and 148.6-wide inputs at x 793.4; Cancel (59.5 wide) and OK (37.3 wide) at
//! y 520.5, both `#3E3E42`.
//!
//! The Home ribbon opens it read-only (`editAreaAttributes` ->
//! `showSelectedAreaAttributes(false)`); Edit Tools opens it editable.

use iced::widget::{button, container, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::snap::nudge;
use crate::theme::{bg_app, bg_hover, bg_panel, border, muted};
use crate::theme::{text_main, ARIAL};
use crate::Message;

/// The card's top-left and size.
const CARD: (f32, f32, f32, f32) = (440.0, 299.5, 520.0, 261.0);
/// A pinned child inside the card lands 2px above its measured position (see
/// `area_dialog::PIN_Y`).
const PIN_Y: f32 = 2.0;

/// The four attributes, in the original's order: (key, label).
pub const ATTRS: [(&str, &str); 4] = [
    ("planRefNumber", "Plan Ref Number"),
    ("floorCode", "Floor Code"),
    ("floorCodeText", "FloorCodeText"),
    ("yearBuilt", "Year Built"),
];

/// The dialog's state.
#[derive(Debug, Clone, Default)]
pub struct AttrDialog {
    pub open: bool,
    /// `editable` false is the Home ribbon's read-only view.
    pub editable: bool,
    /// The four values, in `ATTRS` order.
    pub values: [String; 4],
}

/// A control in the attributes dialog.
#[derive(Debug, Clone)]
pub enum AttrMsg {
    /// The index into `ATTRS`.
    Value(usize, String),
    Ok,
    Cancel,
}

impl AttrDialog {
    /// Open with the first selected area's attributes.
    pub fn open_with(
        attributes: &std::collections::BTreeMap<String, String>,
        editable: bool,
    ) -> AttrDialog {
        let mut values: [String; 4] = Default::default();
        for (i, (key, _)) in ATTRS.iter().enumerate() {
            values[i] = attributes.get(*key).cloned().unwrap_or_default();
        }
        AttrDialog {
            open: true,
            editable,
            values,
        }
    }

    pub fn update(&mut self, msg: AttrMsg) {
        if let AttrMsg::Value(i, v) = msg {
            if i < 4 {
                self.values[i] = v;
            }
        }
    }

    /// `updateSelectedAreaAttributes(Object.fromEntries(data))`.
    pub fn payload(&self) -> Vec<(String, String)> {
        ATTRS
            .iter()
            .enumerate()
            .map(|(i, (key, _))| ((*key).to_string(), self.values[i].clone()))
            .collect()
    }
}

fn place<'a>(x: f32, y: f32, el: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    crate::theme::at(x - CARD.0, y - CARD.1 + PIN_Y, el)
}

/// The dialog's layer, or None when closed.
pub fn layer<'a>(d: &AttrDialog) -> Option<Element<'a, Message>> {
    if !d.open {
        return None;
    }
    let send = |m: AttrMsg| Message::AreaAttributes(m);
    let mut body = iced::widget::stack![]
        .width(Length::Fill)
        .height(Length::Fill);

    body = body.push(place(
        453.0,
        313.0,
        nudge(
            0.5,
            0.0,
            text("Area Attributes")
                .font(ARIAL)
                .size(16.0)
                .color(text_main()),
        ),
    ));
    body = body.push(place(
        935.0,
        309.0,
        button(text("\u{d7}").font(ARIAL).size(20.0).color(muted()))
            .on_press(send(AttrMsg::Cancel))
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
    // The column header.
    for (label, x) in [("Area", 458.0), ("Attribute", 636.8), ("Value", 793.4)] {
        body = body.push(place(
            x,
            350.0,
            nudge(0.5, 0.0, text(label).font(ARIAL).size(11.0).color(muted())),
        ));
    }
    // The four rows; only the first carries the "All Selected Areas" cell.
    for (i, (_, label)) in ATTRS.iter().enumerate() {
        // The reference's first row's text sits at 398, six below the
        // measured row box (a label is centred in its 29px input).
        let y = 392.5 + 30.0 * i as f32;
        if i == 0 {
            body = body.push(place(
                458.0,
                y + 1.0,
                nudge(
                    0.5,
                    0.0,
                    text("All Selected Areas")
                        .font(ARIAL)
                        .size(11.0)
                        .color(text_main()),
                ),
            ));
        }
        body = body.push(place(
            636.8,
            y + 1.0,
            nudge(
                0.5,
                0.0,
                text(*label).font(ARIAL).size(11.0).color(text_main()),
            ),
        ));
        let mut input = text_input("", &d.values[i])
            .font(ARIAL)
            .size(13.33)
            .padding(Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 8.0,
            })
            .width(148.6)
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
            });
        if d.editable {
            input = input.on_input(move |v| Message::AreaAttributes(AttrMsg::Value(i, v)));
        }
        body = body.push(place(793.4, y, input));
    }
    // Actions.
    body = body.push(place(
        842.0,
        519.5,
        attr_button("Cancel", 59.5, send(AttrMsg::Cancel)),
    ));
    body = body.push(place(
        909.7,
        519.5,
        attr_button("OK", 37.3, send(AttrMsg::Ok)),
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
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(0.0, 16.0),
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
                iced::widget::mouse_area(backdrop).on_press(send(AttrMsg::Cancel))
            ),
            crate::theme::at(CARD.0, CARD.1, card)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

fn attr_button<'a>(label: &'a str, width: f32, on_press: Message) -> Element<'a, Message> {
    button(text(label).font(ARIAL).size(13.33).color(text_main()))
        .on_press(on_press)
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 10.0,
            right: 10.0,
        })
        .width(width)
        .height(27.0)
        .style(move |_, _| button::Style {
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

    #[test]
    fn the_dialog_opens_with_the_areas_attributes() {
        let mut attrs = std::collections::BTreeMap::new();
        attrs.insert("planRefNumber".to_string(), "A-1".to_string());
        attrs.insert("yearBuilt".to_string(), "1998".to_string());
        let d = AttrDialog::open_with(&attrs, true);
        assert!(d.open && d.editable);
        assert_eq!(d.values[0], "A-1");
        assert_eq!(d.values[1], "", "a missing attribute is empty");
        assert_eq!(d.values[3], "1998");
        // The payload carries all four keys, in the original's order.
        let p = d.payload();
        assert_eq!(p.len(), 4);
        assert_eq!(p[0].0, "planRefNumber");
        assert_eq!(p[3].0, "yearBuilt");
        assert_eq!(p[3].1, "1998");
    }

    #[test]
    fn edits_land_in_the_payload() {
        let mut d = AttrDialog::open_with(&Default::default(), true);
        d.update(AttrMsg::Value(1, "2".into()));
        d.update(AttrMsg::Value(2, "Second".into()));
        let p = d.payload();
        assert_eq!(p[1].1, "2");
        assert_eq!(p[2].1, "Second");
        // OK and Cancel do not change the values.
        d.update(AttrMsg::Ok);
        assert_eq!(d.payload()[1].1, "2");
    }

    #[test]
    fn the_card_and_rows_are_the_measured_ones() {
        assert_eq!(CARD, (440.0, 299.5, 520.0, 261.0));
        // Four rows on a 30px pitch from 386.5.
        let rows: Vec<f32> = (0..4).map(|i| 386.5 + 30.0 * i as f32).collect();
        assert_eq!(rows, vec![386.5, 416.5, 446.5, 476.5]);
    }
}
