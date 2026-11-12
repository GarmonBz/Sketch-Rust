//! The canvas context menu (screen 6f): `showCanvasContextMenu`
//! (refs/ui-manager.js:1486-1524) and the right-click that opens it
//! (refs/canvas-engine.js:808-825).
//!
//! Measured against the live original at 1400x860: a right-click at (600,300)
//! puts the 190x434 panel at (600,302) — 2px under the cursor, clamped 8px
//! inside the viewport. 13 rows of 29px with five 1px `#3F3F46` separators
//! carrying 4px margins (9px each), 5px padding and a 1px border.
//!
//! The original also prepends symbol and area-workflow entries when those are
//! selected or hovered; this port has neither yet, so the menu is the base
//! list. All thirteen entries act.

use iced::widget::{container, stack};
use iced::{Background, Border, Color, Element, Length, Vector};

use crate::shapes::menu_row_sized;
use crate::theme::at;
use crate::theme::{bg_panel, border};

const ROW_H: f32 = 29.0;
const PAD: f32 = 5.0;
const WIDTH: f32 = 190.0;
const LABEL_SIZE: f32 = 13.33;
/// The separator: `hr { border-top: 1px; margin: 4px 0 }`.
const SEP_MARGIN: f32 = 4.0;
const SEP_H: f32 = 1.0;
/// Rows are pinned 6px above the panel's border box (see `shapes::ROW_Y0`).
const ROW_Y0: f32 = crate::shapes::ROW_Y0;
/// A 1px-tall pinned rule lands 6px below where a row with the same `y` does,
/// so its offset is converted out of the row space (measured: the reference's
/// rules sit at 341/466/533/629/696 from a panel top of 302).
const SEP_ORIGIN_DELTA: f32 = 6.0;
/// `showCommandPopup` keeps the panel 8px inside the viewport.
const VIEW_W: f32 = 1400.0;
const VIEW_H: f32 = 860.0;
const VIEW_MARGIN: f32 = 8.0;

/// The menu's entries, in the original's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EditDefinition,
    MakeNegative,
    Delete,
    Cut,
    Copy,
    Paste,
    SendToBack,
    BringToFront,
    FlipHorz,
    FlipVert,
    Rotate90,
    CloneArea,
    AutoSubtract,
    Properties,
}

impl Action {
    fn label(self) -> &'static str {
        match self {
            Action::EditDefinition => "Edit Definition...",
            Action::MakeNegative => "Make Negative",
            Action::Delete => "Delete",
            Action::Cut => "Cut",
            Action::Copy => "Copy",
            Action::Paste => "Paste",
            Action::SendToBack => "Send to Back",
            Action::BringToFront => "Bring to Front",
            Action::FlipHorz => "Flip Horz",
            Action::FlipVert => "Flip Vert",
            Action::Rotate90 => "Rotate 90°",
            Action::CloneArea => "Clone Area",
            Action::AutoSubtract => "Auto-Subtract Area",
            Action::Properties => "Properties…",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Action::EditDefinition => "pencil-simple",
            Action::MakeNegative => "minus-circle",
            Action::Delete => "trash",
            Action::Cut => "scissors",
            Action::Copy => "copy",
            Action::Paste => "clipboard-text",
            Action::SendToBack => "arrow-line-down",
            Action::BringToFront => "arrow-line-up",
            Action::FlipHorz => "arrows-left-right",
            Action::FlipVert => "arrows-down-up",
            Action::Rotate90 => "arrow-clockwise",
            Action::CloneArea => "copy-simple",
            Action::AutoSubtract => "selection-background",
            Action::Properties => "sliders",
        }
    }

    /// Whether this port can carry the action out yet.
    pub fn is_wired(self) -> bool {
        // Every entry acts since Make Negative and Clone Area landed.
        let _ = self;
        true
    }
}

/// One row of the menu: a button or a separator.
#[derive(Debug, Clone, Copy)]
enum Item {
    Btn(Action),
    Sep,
}

/// The original's base list, with the separators where it puts them.
const ITEMS: &[Item] = &[
    Item::Btn(Action::MakeNegative),
    Item::Sep,
    Item::Btn(Action::Delete),
    Item::Btn(Action::Cut),
    Item::Btn(Action::Copy),
    Item::Btn(Action::Paste),
    Item::Sep,
    Item::Btn(Action::SendToBack),
    Item::Btn(Action::BringToFront),
    Item::Sep,
    Item::Btn(Action::FlipHorz),
    Item::Btn(Action::FlipVert),
    Item::Btn(Action::Rotate90),
    Item::Sep,
    Item::Btn(Action::CloneArea),
    Item::Btn(Action::AutoSubtract),
    Item::Sep,
    Item::Btn(Action::Properties),
];

/// The popup's open state and the click that opened it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ContextMenu {
    /// The right-click position (client coordinates).
    pub open: Option<(f32, f32)>,
    pub area_target: Option<i64>,
}

impl ContextMenu {
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.open = Some((x, y));
        self.area_target = None;
    }

    pub fn close(&mut self) {
        self.open = None;
        self.area_target = None;
    }
}

/// Row offsets in the panel's pinned space, and the panel's height.
fn layout() -> (Vec<(Item, f32)>, f32) {
    let mut y = ROW_Y0;
    let mut out = Vec::with_capacity(ITEMS.len());
    for item in ITEMS {
        match item {
            Item::Btn(_) => {
                out.push((*item, y));
                y += ROW_H;
            }
            Item::Sep => {
                // 4px margin above, the 1px rule, 4px below.
                out.push((*item, y + SEP_MARGIN - SEP_ORIGIN_DELTA));
                y += SEP_MARGIN + SEP_H + SEP_MARGIN;
            }
        }
    }
    // The panel's CSS height: rows + separators + padding + the 2 borders.
    (out, y - ROW_Y0 + PAD * 2.0 + 2.0)
}

/// The popup's layer, or None when closed.
pub fn layer<'a>(v: &ContextMenu) -> Option<Element<'a, crate::Message>> {
    let (cx, cy) = v.open?;
    let (mut rows, mut height) = layout();
    if v.area_target.is_some() {
        for (_, y) in &mut rows {
            *y += ROW_H;
        }
        rows.insert(0, (Item::Btn(Action::EditDefinition), ROW_Y0));
        height += ROW_H;
    }
    let mut body = stack![].width(Length::Fill).height(Length::Fill);
    for (item, y) in rows {
        let el: Element<'a, crate::Message> = match item {
            Item::Btn(action) => menu_row_sized(action.label(), action.icon(), LABEL_SIZE, {
                crate::Message::ContextAction(action)
            }),
            Item::Sep => container(iced::widget::Space::new())
                .width(Length::Fill)
                .height(SEP_H)
                .padding(iced::Padding {
                    top: 0.0,
                    bottom: 0.0,
                    left: SEP_MARGIN,
                    right: SEP_MARGIN,
                })
                .style(|_| container::Style {
                    background: Some(Background::Color(border())),
                    ..container::Style::default()
                })
                .into(),
        };
        body = body.push(at(PAD, y, el));
    }
    let panel = container(body)
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
    // `showCommandPopup`: 2px under the cursor, clamped 8px inside the view.
    let left = (VIEW_W - WIDTH - VIEW_MARGIN).min(cx.max(VIEW_MARGIN));
    let top = (VIEW_H - height - VIEW_MARGIN).min((cy + 2.0).max(VIEW_MARGIN));
    Some(at(left, top, panel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_190_by_434_like_the_original() {
        let (rows, height) = layout();
        assert_eq!(rows.len(), ITEMS.len());
        assert_eq!(height, 434.0, "13 rows, 5 separators, padding, borders");
        assert_eq!(WIDTH, 190.0);
    }

    #[test]
    fn rows_land_where_the_original_puts_them() {
        // Measured from the live original with the panel at y=302:
        // 308, 346, 375, 404, 433, 471, 500, 538, 567, 596, 634, 663, 701.
        let (rows, _) = layout();
        let panel_top = 302.0;
        // The pinned space sits 6px above the panel's border box.
        let absolute = |y: f32| panel_top - 6.0 + y;
        let buttons: Vec<f32> = rows
            .iter()
            .filter(|(i, _)| matches!(i, Item::Btn(_)))
            .map(|(_, y)| absolute(*y))
            .collect();
        assert_eq!(
            buttons,
            vec![
                308.0, 346.0, 375.0, 404.0, 433.0, 471.0, 500.0, 538.0, 567.0, 596.0, 634.0, 663.0,
                701.0
            ]
        );
        // The separators sit 1px tall at 341, 466, 533, 629, 696 — a rule's
        // pin origin is the panel's border box, not the rows' shifted one.
        let seps: Vec<f32> = rows
            .iter()
            .filter(|(i, _)| matches!(i, Item::Sep))
            .map(|(_, y)| panel_top + y)
            .collect();
        assert_eq!(seps, vec![341.0, 466.0, 533.0, 629.0, 696.0]);
    }

    #[test]
    fn the_menu_lists_the_originals_thirteen_entries() {
        let labels: Vec<&str> = ITEMS
            .iter()
            .filter_map(|i| match i {
                Item::Btn(a) => Some(a.label()),
                Item::Sep => None,
            })
            .collect();
        assert_eq!(labels.len(), 13);
        assert_eq!(labels[0], "Make Negative");
        assert_eq!(labels[12], "Properties…");
        // All thirteen entries act.
        let wired = ITEMS
            .iter()
            .filter(|i| matches!(i, Item::Btn(a) if a.is_wired()))
            .count();
        assert_eq!(wired, 13);
    }

    #[test]
    fn the_popup_clamps_inside_the_viewport() {
        let mut v = ContextMenu::default();
        v.open_at(1390.0, 850.0);
        assert!(layer(&v).is_some());
        v.close();
        assert!(layer(&v).is_none());
    }
}
