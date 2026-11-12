//! The three ribbon popup menus (`UIManager.showCommandPopup`,
//! ui-manager.js:1400): Flip Sketch (`showFlipMenu`), Canvas background
//! (`showCanvasBackgroundMenu`) and Interface theme (`showInterfaceThemeMenu`).
//!
//! Same `.command-popup` panel the select-by-type menu uses (190px, 5px
//! padding, 29px rows, a 1px border and the 0 8 22 shadow), placed at
//! `left = min(innerWidth - w - 8, max(8, anchor.left))` and
//! `top = min(innerHeight - h - 8, max(8, anchor.bottom + 2))`. The theme
//! menus mark the current choice with a ` ✓` in `#63B3ED` after its label
//! (`button.active span::after`). A press outside dismisses the menu (the
//! original's one-shot `pointerdown` listener); like the original it has no
//! Escape handler.

use iced::widget::{button, container, mouse_area, opaque, row, stack};
use iced::{alignment, Background, Border, Color, Element, Length, Padding, Vector};

use crate::canvas_theme::CanvasTheme;
use crate::icons::icon;
use crate::snap::nudge;
use crate::theme::*;
use crate::transform::Axis;
use crate::Message;

const ROW_H: f32 = 29.0;
const PAD: f32 = 5.0;
const ROW_Y0: f32 = crate::shapes::ROW_Y0;
const WIDTH: f32 = 190.0;
const LABEL_SIZE: f32 = 13.33;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Flip,
    CanvasBackground,
    InterfaceTheme,
    /// The title bar's quick-access caret.
    QuickAccess,
}

/// A quick-access menu command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quick {
    New,
    Open,
    SaveAs,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupMsg {
    /// Open under an anchor (its left and bottom edge, in window px).
    Open(Kind, f32, f32),
    Close,
    Flip(Axis),
    Canvas(CanvasTheme),
    Interface(InterfaceTheme),
    Quick(Quick),
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PopupMenu {
    pub open: Option<(Kind, (f32, f32))>,
}

impl PopupMenu {
    pub fn open_at(&mut self, kind: Kind, x: f32, y: f32) {
        self.open = Some((kind, (x, y)));
    }

    pub fn close(&mut self) {
        self.open = None;
    }
}

/// One menu row.
pub struct Entry {
    pub label: &'static str,
    pub icon: &'static str,
    pub msg: PopupMsg,
    pub active: bool,
}

/// The rows of a menu in the original's order. `active` marks the current
/// theme for the two theme menus.
pub fn entries(kind: Kind, canvas: CanvasTheme, interface: InterfaceTheme) -> Vec<Entry> {
    match kind {
        Kind::Flip => vec![
            Entry {
                label: "Flip Horizontal",
                icon: "arrows-left-right",
                msg: PopupMsg::Flip(Axis::Horizontal),
                active: false,
            },
            Entry {
                label: "Flip Vertical",
                icon: "arrows-down-up",
                msg: PopupMsg::Flip(Axis::Vertical),
                active: false,
            },
        ],
        Kind::CanvasBackground => [
            (CanvasTheme::Light, "sun"),
            (CanvasTheme::Dark, "moon"),
            (CanvasTheme::Blue, "drop"),
        ]
        .into_iter()
        .map(|(t, glyph)| Entry {
            label: t.label(),
            icon: glyph,
            msg: PopupMsg::Canvas(t),
            active: t == canvas,
        })
        .collect(),
        Kind::QuickAccess => [
            (Quick::New, "New", "file-plus"),
            (Quick::Open, "Open...", "file"),
            (Quick::SaveAs, "Save As...", "export"),
            (Quick::Options, "Options", "sliders"),
        ]
        .into_iter()
        .map(|(q, label, glyph)| Entry {
            label,
            icon: glyph,
            msg: PopupMsg::Quick(q),
            active: false,
        })
        .collect(),
        Kind::InterfaceTheme => [
            (InterfaceTheme::Light, "Light", "sun"),
            (InterfaceTheme::Dark, "Dark", "moon"),
            (InterfaceTheme::Blue, "Blue", "drop"),
        ]
        .into_iter()
        .map(|(t, label, glyph)| Entry {
            label,
            icon: glyph,
            msg: PopupMsg::Interface(t),
            active: t == interface,
        })
        .collect(),
    }
}

/// `left` / `top` of the popup, clamped inside the window like the original.
pub fn placement(anchor: (f32, f32), size: (f32, f32), window: (f32, f32)) -> (f32, f32) {
    let left = (window.0 - size.0 - 8.0).min(anchor.0.max(8.0));
    let top = (window.1 - size.1 - 8.0).min((anchor.1 + 2.0).max(8.0));
    (left, top)
}

fn menu_row<'a>(entry: &Entry) -> Element<'a, Message> {
    let name = nudge(
        0.5,
        0.0,
        label(entry.label, LABEL_SIZE, ARIAL, text_main(), 16.0),
    );
    // `button.active span::after { content: ' ✓'; color: #63B3ED }`: the
    // check follows the label by one space.
    let name: Element<'a, Message> = if entry.active {
        row![name, label("\u{2713}", LABEL_SIZE, ARIAL, ICON_BLUE, 16.0)]
            .spacing(4)
            .into()
    } else {
        name.into()
    };
    let body = row![icon(entry.icon, false, 13.34, muted()), name]
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
        .on_press(Message::PopupMenu(entry.msg))
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

/// The popup layer (a dismissing backdrop plus the panel), or None when
/// closed. `window` is the window size in CSS px.
pub fn layer<'a>(
    menu: &PopupMenu,
    canvas: CanvasTheme,
    interface: InterfaceTheme,
    window: (f32, f32),
) -> Option<Element<'a, Message>> {
    let (kind, anchor) = menu.open?;
    let items = entries(kind, canvas, interface);
    let mut rows = stack![].width(Length::Fill).height(Length::Fill);
    for (i, entry) in items.iter().enumerate() {
        rows = rows.push(at(PAD, ROW_Y0 + ROW_H * i as f32, menu_row(entry)));
    }
    // iced draws the border inside the box: add the two border pixels.
    let height = ROW_H * items.len() as f32 + PAD * 2.0 + 2.0;
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
    let (x, y) = placement(anchor, (WIDTH, height), window);
    let backdrop = mouse_area(
        container(iced::widget::Space::new())
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::PopupMenu(PopupMsg::Close));
    Some(stack![backdrop, at(x, y, opaque(panel))].into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_lists_horizontal_then_vertical() {
        let e = entries(Kind::Flip, CanvasTheme::Light, InterfaceTheme::Dark);
        let labels: Vec<&str> = e.iter().map(|x| x.label).collect();
        assert_eq!(labels, vec!["Flip Horizontal", "Flip Vertical"]);
        assert!(e.iter().all(|x| !x.active));
    }

    #[test]
    fn the_canvas_menu_says_white_for_light_and_marks_the_current() {
        let e = entries(
            Kind::CanvasBackground,
            CanvasTheme::Blue,
            InterfaceTheme::Dark,
        );
        let labels: Vec<&str> = e.iter().map(|x| x.label).collect();
        assert_eq!(labels, vec!["White", "Dark", "Blue"]);
        let active: Vec<bool> = e.iter().map(|x| x.active).collect();
        assert_eq!(active, vec![false, false, true]);
    }

    #[test]
    fn the_interface_menu_marks_the_live_theme() {
        let e = entries(
            Kind::InterfaceTheme,
            CanvasTheme::Light,
            InterfaceTheme::Light,
        );
        let labels: Vec<&str> = e.iter().map(|x| x.label).collect();
        assert_eq!(labels, vec!["Light", "Dark", "Blue"]);
        assert!(e[0].active && !e[1].active && !e[2].active);
    }

    #[test]
    fn the_quick_access_menu_lists_its_commands() {
        let e = entries(Kind::QuickAccess, CanvasTheme::Light, InterfaceTheme::Dark);
        let labels: Vec<&str> = e.iter().map(|x| x.label).collect();
        assert_eq!(labels, vec!["New", "Open...", "Save As...", "Options"]);
        assert!(e.iter().all(|x| !x.active));
    }

    #[test]
    fn placement_clamps_inside_the_window() {
        // Plenty of room: under the anchor, 2px below its bottom.
        assert_eq!(
            placement((570.0, 124.0), (190.0, 68.0), (1400.0, 860.0)),
            (570.0, 126.0)
        );
        // Near the right / bottom edge: pulled back by the 8px margin.
        assert_eq!(
            placement((1350.0, 850.0), (190.0, 68.0), (1400.0, 860.0)),
            (1202.0, 784.0)
        );
        // A zero rect (a hidden anchor) lands at (8, 8).
        assert_eq!(
            placement((0.0, 0.0), (190.0, 68.0), (1400.0, 860.0)),
            (8.0, 8.0)
        );
    }

    #[test]
    fn a_closed_menu_draws_nothing() {
        let m = PopupMenu::default();
        assert!(layer(
            &m,
            CanvasTheme::Light,
            InterfaceTheme::Dark,
            (1400.0, 860.0)
        )
        .is_none());
    }
}
