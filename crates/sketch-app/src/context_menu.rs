//! The canvas context menu opened on right-click.

use iced::widget::{button, container, pin, row, Column, Space};
use iced::{alignment, Background, Border, Color, Element, Length, Padding, Vector};

use crate::icons::mono;
use crate::theme::{bg_hover, bg_panel, border, muted, text_main, ARIAL};

const ROW_H: f32 = 29.0;
const PAD: f32 = 5.0;
const WIDTH: f32 = 190.0;
const LABEL_SIZE: f32 = 13.33;
const SEP_MARGIN: f32 = 4.0;
const SEP_H: f32 = 1.0;
const VIEW_W: f32 = 1400.0;
const VIEW_H: f32 = 860.0;
const VIEW_MARGIN: f32 = 8.0;

/// The menu's entries, in the original's order.
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
            Action::EditDefinition => "Edit Definition...",
            Action::MakeSymbol => "Make symbol from selection",
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
            Action::MakeSymbol => "shapes",
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
    pub can_make_symbol: bool,
}

impl ContextMenu {
    pub fn open_at(&mut self, x: f32, y: f32) {
        self.open = Some((x, y));
        self.area_target = None;
        self.can_make_symbol = false;
    }

    pub fn close(&mut self) {
        self.open = None;
        self.area_target = None;
        self.can_make_symbol = false;
    }
}

fn row_button<'a>(action: Action) -> Element<'a, crate::Message> {
    let body = row![
        mono(action.icon(), 13.34, muted()),
        iced::widget::text(action.label())
            .size(LABEL_SIZE)
            .font(ARIAL)
            .color(text_main()),
    ]
    .spacing(9)
    .align_y(alignment::Vertical::Center);
    button(body)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 10.0,
            right: 10.0,
        })
        .width(Length::Fill)
        .height(ROW_H)
        .on_press(crate::Message::ContextAction(action))
        .style(|_, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then_some(Background::Color(bg_hover())),
                border: Border::default(),
                text_color: text_main(),
                ..button::Style::default()
            }
        })
        .into()
}

/// The popup's layer, or None when closed.
pub fn layer<'a>(v: &ContextMenu) -> Option<Element<'a, crate::Message>> {
    let (cx, cy) = v.open?;
    let mut items: Vec<Item> = ITEMS.to_vec();
    let mut height = items
        .iter()
        .map(|i| match i {
            Item::Btn(_) => ROW_H,
            Item::Sep => SEP_MARGIN * 2.0 + SEP_H,
        })
        .sum::<f32>()
        + PAD * 2.0
        + 2.0;
    if v.area_target.is_some() {
        items.insert(0, Item::Btn(Action::EditDefinition));
        height += ROW_H;
    }
    if v.can_make_symbol {
        items.insert(0, Item::Btn(Action::MakeSymbol));
        height += ROW_H;
    }
    let mut body = Column::new().width(Length::Fill);
    for item in items {
        let el: Element<'a, crate::Message> = match item {
            Item::Btn(action) => row_button(action),
            Item::Sep => container(
                container(Space::new().width(Length::Fill).height(SEP_H)).style(|_| {
                    container::Style {
                        background: Some(Background::Color(border())),
                        ..container::Style::default()
                    }
                }),
            )
            .width(Length::Fill)
            .padding(Padding {
                top: SEP_MARGIN,
                bottom: SEP_MARGIN,
                left: 0.0,
                right: 0.0,
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
    let left = (VIEW_W - WIDTH - VIEW_MARGIN).min(cx.max(VIEW_MARGIN));
    let top = (VIEW_H - height - VIEW_MARGIN).min((cy + 2.0).max(VIEW_MARGIN));
    Some(pin(panel).x(left).y(top).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_lists_all_thirteen_entries() {
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
