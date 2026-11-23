//! Select by type options and helpers.

use iced::{Background, Border, Color, Element, Length, Vector};

use crate::icons::mono;
use crate::theme::{bg_hover, bg_panel, border, text_main, ARIAL};

const ROW_H: f32 = 29.0;
const PAD: f32 = 5.0;
const WIDTH: f32 = 190.0;
const LABEL_SIZE: f32 = 13.33;

/// The six menu entries, in the original's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectKind {
    All,
    Areas,
    Text,
    Dimensions,
    Symbols,
    Freeform,
}

impl SelectKind {
    pub const ALL: [SelectKind; 6] = [
        SelectKind::All,
        SelectKind::Areas,
        SelectKind::Text,
        SelectKind::Dimensions,
        SelectKind::Symbols,
        SelectKind::Freeform,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SelectKind::All => "Select All",
            SelectKind::Areas => "Select Areas",
            SelectKind::Text => "Select Text",
            SelectKind::Dimensions => "Select Dimensions",
            SelectKind::Symbols => "Select Symbols",
            SelectKind::Freeform => "Select Freeform Lines",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            SelectKind::All => "selection-all",
            SelectKind::Areas => "polygon",
            SelectKind::Text => "text-t",
            SelectKind::Dimensions => "ruler",
            SelectKind::Symbols => "couch",
            SelectKind::Freeform => "line-segment",
        }
    }
}

/// The popup's open state and anchor.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SelectMenu {
    /// The anchor's top-left; the popup opens 2px under its bottom edge.
    pub open: Option<(f32, f32)>,
}

impl SelectMenu {
    #[cfg(test)]
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.open = Some((x, y));
    }

    pub fn close(&mut self) {
        self.open = None;
    }
}

fn row_item<'a>(label_s: &str, glyph: &str, kind: SelectKind) -> Element<'a, crate::Message> {
    let body = iced::widget::row![
        mono(glyph, 13.34, crate::theme::muted()),
        iced::widget::text(label_s.to_string())
            .size(LABEL_SIZE)
            .font(ARIAL)
            .color(text_main()),
    ]
    .spacing(9)
    .align_y(iced::alignment::Vertical::Center);
    iced::widget::button(body)
        .padding(iced::Padding {
            top: 0.0,
            bottom: 0.0,
            left: 10.0,
            right: 10.0,
        })
        .width(Length::Fill)
        .height(ROW_H)
        .on_press(crate::Message::SelectByType(kind))
        .style(|_, status| {
            let hovered = matches!(
                status,
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
            );
            iced::widget::button::Style {
                background: hovered.then_some(Background::Color(bg_hover())),
                border: Border::default(),
                text_color: text_main(),
                ..iced::widget::button::Style::default()
            }
        })
        .into()
}

/// The popup's layer, or None when closed.
pub fn layer<'a>(v: &SelectMenu) -> Option<Element<'a, crate::Message>> {
    let (x, y) = v.open?;
    let mut rows = iced::widget::Column::new().width(Length::Fill);
    for kind in SelectKind::ALL {
        rows = rows.push(row_item(kind.label(), kind.icon(), kind));
    }
    let height = ROW_H * SelectKind::ALL.len() as f32 + PAD * 2.0 + 2.0;
    let panel = iced::widget::container(rows)
        .padding(PAD)
        .width(WIDTH)
        .height(height)
        .style(|_| iced::widget::container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.42),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 11.0,
            },
            ..iced::widget::container::Style::default()
        });
    Some(iced::widget::pin(panel).x(x).y(y + 2.0).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_lists_the_originals_six_entries_in_order() {
        let labels: Vec<&str> = SelectKind::ALL.iter().map(|k| k.label()).collect();
        assert_eq!(
            labels,
            vec![
                "Select All",
                "Select Areas",
                "Select Text",
                "Select Dimensions",
                "Select Symbols",
                "Select Freeform Lines",
            ]
        );
        // The popup is 190 x (6 rows of 29 + 2*5) = 190 x 184.
        assert_eq!(
            ROW_H * 6.0 + PAD * 2.0 + 2.0,
            186.0,
            "the CSS popup is 186 tall"
        );
    }

    #[test]
    fn a_closed_menu_draws_nothing() {
        let v = SelectMenu::default();
        assert!(layer(&v).is_none());
    }

    #[test]
    fn an_open_menu_draws_the_panel() {
        let mut v = SelectMenu::default();
        v.open_at(213.0, 117.0);
        assert!(layer(&v).is_some());
        v.close();
        assert!(layer(&v).is_none());
    }
}
