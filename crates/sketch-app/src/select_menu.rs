//! Select by type (screen 6e): `selectByType` and its command popup
//! (refs/sketch-app.js:842-869, refs/ui-manager.js:1421-1426).
//!
//! The menu is the same command popup the wall-shape and undo-history menus
//! use: 190 wide, 29px rows, 13.33px Arial labels with a 13.34px icon, opened
//! 2px under the Edit Tools › Select button.

use iced::widget::{container, stack};
use iced::{Background, Border, Color, Element, Length, Vector};

use crate::shapes::menu_row_sized;
use crate::theme::at;
use crate::theme::{bg_panel, border};

const ROW_H: f32 = 29.0;
const PAD: f32 = 5.0;
/// See `shapes::ROW_Y0`.
const ROW_Y0: f32 = crate::shapes::ROW_Y0;
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

    fn label(self) -> &'static str {
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
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.open = Some((x, y));
    }

    pub fn close(&mut self) {
        self.open = None;
    }
}

/// The popup's layer, or None when closed.
pub fn layer<'a>(v: &SelectMenu) -> Option<Element<'a, crate::Message>> {
    let (x, y) = v.open?;
    let mut rows = stack![].width(Length::Fill).height(Length::Fill);
    for (i, kind) in SelectKind::ALL.into_iter().enumerate() {
        rows = rows.push(at(
            PAD,
            ROW_Y0 + ROW_H * i as f32,
            menu_row_sized(
                kind.label(),
                kind.icon(),
                LABEL_SIZE,
                crate::Message::SelectByType(kind),
            ),
        ));
    }
    // iced draws the border inside the box, so the CSS height (content +
    // padding + 2 borders) needs them added explicitly.
    let height = ROW_H * SelectKind::ALL.len() as f32 + PAD * 2.0 + 2.0;
    let panel = container(rows)
        .width(WIDTH)
        .height(height)
        .style(|_| container::Style {
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
            ..container::Style::default()
        });
    Some(at(x, y + 2.0, panel))
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
