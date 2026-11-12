//! The undo history popup (screen 6c, S6-021): `showUndoHistory`
//! (refs/ui-manager.js:2847-2858).
//!
//! It is the same command popup the wall-shape menu uses
//! (`showCommandPopup(anchor, items, 'command-popup undo-history-popup')`),
//! anchored under the QAT button that opened it. Entries are newest first and
//! each one rewinds to the state before that action, so the display order is
//! the reverse of the stack's own order.

use iced::widget::{container, stack};
use iced::{Background, Border, Color, Element, Length, Vector};

use crate::shapes::menu_row_sized;
use crate::theme::at;
use crate::theme::{bg_panel, border_alt};

/// `--row-h` in the command popup.
const ROW_H: f32 = 29.5;
const WIDTH: f32 = 190.0;
/// `.undo-history-popup button span { font-size: 11.5px }`.
const LABEL_SIZE: f32 = 11.5;
/// The popup's own padding, top and bottom.
const PAD: f32 = 5.0;
/// See `shapes::ROW_Y0`.
const ROW_Y0: f32 = crate::shapes::ROW_Y0;

/// Where the popup is open and under which anchor.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HistoryView {
    /// The anchor's top-left; the popup opens 2px under its bottom edge.
    pub open: Option<(f32, f32)>,
}

impl HistoryView {
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.open = Some((x, y));
    }

    pub fn close(&mut self) {
        self.open = None;
    }
}

/// The popup's layer, given the stack's labels newest-first.
pub fn layer<'a>(v: &HistoryView, labels: &[String]) -> Option<Element<'a, crate::Message>> {
    let (x, y) = v.open?;
    if labels.is_empty() {
        return None;
    }
    let mut rows = stack![].width(Length::Fill).height(Length::Fill);
    for (i, text) in labels.iter().enumerate() {
        // Row i is the (len-1-i)th entry in the stack's own order; rewinding to
        // that index replays the right number of steps.
        let index = labels.len() - 1 - i;
        let numbered = format!("{}. {}", index + 1, text);
        rows = rows.push(at(
            0.0,
            ROW_Y0 + ROW_H * i as f32,
            menu_row_sized(
                &numbered,
                "arrow-u-up-left",
                LABEL_SIZE,
                crate::Message::UndoTo(index),
            ),
        ));
    }
    let height = ROW_H * labels.len() as f32 + PAD * 2.0 + 2.0;
    let panel = container(rows)
        .width(WIDTH)
        .height(height)
        .padding(PAD as u16)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border_alt(),
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
    // `showCommandPopup` places the popup 2px under the anchor's bottom edge.
    Some(at(x, y + 2.0, panel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closed_popup_draws_nothing() {
        let v = HistoryView::default();
        assert!(layer(&v, &["Draw".to_string()]).is_none());
    }

    #[test]
    fn an_empty_stack_draws_nothing() {
        let mut v = HistoryView::default();
        v.open_at(104.0, 29.0);
        assert!(layer(&v, &[]).is_none());
    }

    /// The row -> stack-index mapping the popup relies on: row 0 (the newest
    /// label, numbered "1.") rewinds the newest action, so it addresses the
    /// last stack entry.
    fn index_for_row(labels: &[String], row: usize) -> usize {
        labels.len() - 1 - row
    }

    #[test]
    fn entries_are_numbered_newest_first_and_map_back_to_the_stack() {
        let mut v = HistoryView::default();
        v.open_at(104.0, 29.0);
        let labels = vec!["Set Length".to_string(), "Draw".to_string()];
        assert!(layer(&v, &labels).is_some());
        assert_eq!(index_for_row(&labels, 0), 1, "row 0 is the newest entry");
        assert_eq!(index_for_row(&labels, 1), 0, "row 1 is the oldest");
        // Numbering is index + 1 over the stack's own order.
        assert_eq!(
            format!("{}. {}", index_for_row(&labels, 0) + 1, labels[0]),
            "2. Set Length"
        );
        assert_eq!(
            format!("{}. {}", index_for_row(&labels, 1) + 1, labels[1]),
            "1. Draw"
        );
    }
}
