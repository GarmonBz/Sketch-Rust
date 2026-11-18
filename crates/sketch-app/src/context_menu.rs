//! The canvas context menu opened on right-click.
//!
//! Redesigned on 2026-11-17 to match the rest of the interface: the widget
//! kit's surfaces, Barlow type, tinted icons and sentence-case labels; rows
//! that never wrap; placement against the real window size; and only the
//! entries that apply to what was clicked and selected.

use iced::widget::{button, container, pin, row, text, Column, Space};
use iced::{alignment, Background, Border, Element, Length, Padding, Size};

use crate::icons;
use crate::ui::fonts::{BARLOW_MEDIUM, BSC_MEDIUM};
use crate::ui::tokens::{self, Density, RADIUS_CONTROL, RADIUS_DIALOG, TEXT_12, TEXT_13};

const PAD: f32 = 6.0;
const WIDTH: f32 = 264.0;
const SEP_H: f32 = 9.0;
const VIEW_MARGIN: f32 = 8.0;

/// Every entry the menu can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    EditDefinition,
    MakeSymbol,
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
            Action::EditDefinition => "Change area type…",
            Action::MakeSymbol => "Make symbol from selection",
            Action::MakeNegative => "Subtract from totals",
            Action::Delete => "Delete",
            Action::Cut => "Cut",
            Action::Copy => "Copy",
            Action::Paste => "Paste",
            Action::SendToBack => "Send to back",
            Action::BringToFront => "Bring to front",
            Action::FlipHorz => "Flip horizontally",
            Action::FlipVert => "Flip vertically",
            Action::Rotate90 => "Rotate 90°",
            Action::CloneArea => "Clone area",
            Action::AutoSubtract => "Auto-subtract overlaps",
            Action::Properties => "Area properties…",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Action::EditDefinition => "bounding-box",
            Action::MakeSymbol => "star",
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
            Action::AutoSubtract => "intersect",
            Action::Properties => "sliders",
        }
    }

    /// The key that does the same thing, where there is a fixed one.
    fn shortcut(self) -> &'static str {
        match self {
            Action::Cut => "Ctrl X",
            Action::Copy => "Ctrl C",
            Action::Paste => "Ctrl V",
            Action::Delete => "Del",
            _ => "",
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Btn(Action),
    Sep,
}

/// The popup's open state and what it was opened on.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ContextMenu {
    /// The right-click position (client coordinates).
    pub open: Option<(f32, f32)>,
    /// The area under the pointer, if any.
    pub area_target: Option<i64>,
    pub can_make_symbol: bool,
    /// Something is selected (after the right-click's own selection).
    pub has_selection: bool,
    /// At least one area is selected or under the pointer.
    pub areas_selected: bool,
}

impl ContextMenu {
    pub fn open_at(&mut self, x: f32, y: f32) {
        *self = ContextMenu {
            open: Some((x, y)),
            ..ContextMenu::default()
        };
    }

    pub fn close(&mut self) {
        *self = ContextMenu::default();
    }
}

/// The rows for this menu, in order, with separators only between groups
/// that are shown.
fn items(v: &ContextMenu) -> Vec<Item> {
    let mut groups: Vec<Vec<Action>> = Vec::new();
    let mut area_head = Vec::new();
    if v.area_target.is_some() {
        area_head.push(Action::EditDefinition);
    }
    if v.areas_selected {
        area_head.push(Action::Properties);
    }
    groups.push(area_head);
    groups.push(if v.has_selection {
        vec![Action::Cut, Action::Copy, Action::Paste]
    } else {
        vec![Action::Paste]
    });
    if v.has_selection {
        groups.push(vec![Action::Rotate90, Action::FlipHorz, Action::FlipVert]);
        groups.push(vec![Action::BringToFront, Action::SendToBack]);
    }
    if v.areas_selected {
        groups.push(vec![
            Action::CloneArea,
            Action::MakeNegative,
            Action::AutoSubtract,
        ]);
    }
    if v.can_make_symbol {
        groups.push(vec![Action::MakeSymbol]);
    }
    if v.has_selection {
        groups.push(vec![Action::Delete]);
    }
    let mut out = Vec::new();
    for g in groups.into_iter().filter(|g| !g.is_empty()) {
        if !out.is_empty() {
            out.push(Item::Sep);
        }
        out.extend(g.into_iter().map(Item::Btn));
    }
    out
}

fn row_h(d: Density) -> f32 {
    match d {
        Density::Touch => d.row_h(),
        Density::Comfortable => d.row_h().min(32.0),
    }
}

fn row_button<'a>(action: Action, d: Density) -> Element<'a, crate::Message> {
    let destructive = action == Action::Delete;
    let fg = if destructive {
        tokens::flag()
    } else {
        tokens::text()
    };
    let mut body = row![
        icons::tinted(action.icon(), false, 15.0, fg),
        text(action.label())
            .size(TEXT_13)
            .font(BARLOW_MEDIUM)
            .color(fg)
            .wrapping(text::Wrapping::None)
            .width(Length::Fill),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center);
    if !action.shortcut().is_empty() {
        body = body.push(
            text(action.shortcut())
                .size(TEXT_12)
                .font(BSC_MEDIUM)
                .color(tokens::muted())
                .wrapping(text::Wrapping::None),
        );
    }
    button(body)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 10.0,
            right: 10.0,
        })
        .width(Length::Fill)
        .height(row_h(d))
        .on_press(crate::Message::ContextAction(action))
        .style(|_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then_some(Background::Color(tokens::hover())),
                border: Border {
                    radius: RADIUS_CONTROL.into(),
                    ..Border::default()
                },
                text_color: tokens::text(),
                ..button::Style::default()
            }
        })
        .into()
}

/// The menu's height for these rows.
fn height(items: &[Item], d: Density) -> f32 {
    items
        .iter()
        .map(|i| match i {
            Item::Btn(_) => row_h(d),
            Item::Sep => SEP_H,
        })
        .sum::<f32>()
        + PAD * 2.0
        + 2.0
}

/// The popup's layer inside a window of `win`, or None when closed.
pub fn layer<'a>(v: &ContextMenu, win: Size, d: Density) -> Option<Element<'a, crate::Message>> {
    let (cx, cy) = v.open?;
    let items = items(v);
    let height = height(&items, d);
    let mut body = Column::new().width(Length::Fill);
    for item in items {
        let el: Element<'a, crate::Message> = match item {
            Item::Btn(action) => row_button(action, d),
            Item::Sep => container(
                container(Space::new().width(Length::Fill).height(1.0)).style(|_| {
                    container::Style {
                        background: Some(Background::Color(tokens::rule())),
                        ..container::Style::default()
                    }
                }),
            )
            .width(Length::Fill)
            .height(SEP_H)
            .padding(Padding {
                top: (SEP_H - 1.0) / 2.0,
                bottom: 0.0,
                left: 4.0,
                right: 4.0,
            })
            .into(),
        };
        body = body.push(el);
    }
    let panel = container(body)
        .padding(PAD)
        .width(WIDTH)
        .height(height)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::bar())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: RADIUS_DIALOG.into(),
            },
            shadow: tokens::floating_shadow(),
            ..container::Style::default()
        });
    let (left, top) = place(cx, cy, WIDTH, height, win);
    Some(pin(panel).x(left).y(top).into())
}

/// Top-left corner for a `w` x `h` menu opened at (`cx`, `cy`): at the
/// pointer, flipped left or up when it would leave the window.
fn place(cx: f32, cy: f32, w: f32, h: f32, win: Size) -> (f32, f32) {
    let left = if cx + w + VIEW_MARGIN > win.width {
        (cx - w).max(VIEW_MARGIN)
    } else {
        cx
    };
    let top = if cy + 2.0 + h + VIEW_MARGIN > win.height {
        (cy - h).max(VIEW_MARGIN)
    } else {
        cy + 2.0
    };
    (left, top)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(v: &ContextMenu) -> Vec<&'static str> {
        items(v)
            .into_iter()
            .filter_map(|i| match i {
                Item::Btn(a) => Some(a.label()),
                Item::Sep => None,
            })
            .collect()
    }

    #[test]
    fn an_area_gets_the_full_menu_and_empty_space_only_paste() {
        let mut v = ContextMenu::default();
        v.open_at(10.0, 10.0);
        assert_eq!(labels(&v), vec!["Paste"]);

        v.area_target = Some(1);
        v.has_selection = true;
        v.areas_selected = true;
        v.can_make_symbol = true;
        let all = labels(&v);
        assert_eq!(all.first(), Some(&"Change area type…"));
        assert_eq!(all.last(), Some(&"Delete"));
        assert_eq!(all.len(), 15);
        assert!(items(&v).iter().all(|i| match i {
            Item::Sep => true,
            Item::Btn(a) => a.is_wired(),
        }));
        // No two separators in a row, none first or last.
        let it = items(&v);
        assert_ne!(it.first(), Some(&Item::Sep));
        assert_ne!(it.last(), Some(&Item::Sep));
        assert!(it.windows(2).all(|w| !(w[0] == Item::Sep && w[1] == Item::Sep)));
    }

    #[test]
    fn a_wall_gets_no_area_entries() {
        let mut v = ContextMenu::default();
        v.open_at(10.0, 10.0);
        v.has_selection = true;
        let l = labels(&v);
        assert!(l.contains(&"Rotate 90°"));
        assert!(!l.contains(&"Clone area"));
        assert!(!l.contains(&"Change area type…"));
    }

    #[test]
    fn the_popup_stays_inside_the_window() {
        let win = Size::new(1382.0, 816.0);
        let h = 500.0;
        let (x, y) = place(1370.0, 800.0, WIDTH, h, win);
        assert!(x + WIDTH <= win.width && y + h <= win.height, "({x}, {y})");
        let (x, y) = place(100.0, 100.0, WIDTH, h, win);
        assert_eq!((x, y), (100.0, 102.0));
        let mut v = ContextMenu::default();
        v.open_at(1370.0, 800.0);
        assert!(layer(&v, win, Density::Comfortable).is_some());
        v.close();
        assert!(layer(&v, win, Density::Comfortable).is_none());
    }
}
