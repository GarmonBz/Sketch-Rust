//! The Keyboard Shortcuts cheat sheet (S12-015) — `showShortcuts`
//! (ui-manager.js): the 7 group cards in a `minmax(220px, 1fr)` grid on a
//! 760px dialog; Close / Escape / F1 / a backdrop press closes it.
//!
//! Measurements (theme.css 1302-1326): the dialog `min(760px, 100vw-32)`,
//! max-height `100vh-48`, gap 10; the grid `auto-fit minmax(220px, 1fr)`,
//! gap 12; the group cards on `--bg-app`, 1px `--border`, radius 3,
//! padding 8/10; the group title 11px/600 uppercase muted; the rows 11.5px
//! with a 96px-min mono key chip (1px border, 2px bottom border, radius 3).

use iced::widget::{button, column, container, mouse_area, opaque, row, text, Space};
use iced::{Background, Border, Color, Element, Length};

use crate::theme::*;
use crate::Message;

/// The cheat sheet's groups, verbatim from `showShortcuts`.
pub const GROUPS: &[(&str, &[(&str, &str)])] = &[
    (
        "File",
        &[
            ("Ctrl + N", "New sketch"),
            ("Ctrl + O or F3", "Open sketch"),
            ("Ctrl + S or F2", "Save"),
            ("Ctrl + F12 or F12", "Save As"),
            ("Ctrl + F1", "Email support"),
        ],
    ),
    (
        "Edit",
        &[
            ("Ctrl + Z", "Undo"),
            ("Ctrl + Shift + Z or Ctrl + Y", "Redo"),
            ("Ctrl + X / C / V", "Cut / Copy / Paste"),
            ("Ctrl + A", "Select all"),
            ("Delete", "Delete selection"),
            ("Esc", "Cancel draw / clear selection / exit tool"),
            ("Arrow keys", "Nudge selection (Shift = fine)"),
        ],
    ),
    (
        "Drawing",
        &[
            ("E", "Draw exterior walls"),
            ("I or F5", "Draw interior walls"),
            ("Enter", "Start exterior / commit wall"),
            ("10 then arrow", "Draw 10 ft in arrow direction"),
            ("10x5", "Rise 10, run 5 diagonal"),
            ("10<45", "10 ft at 45 degrees"),
            ("10l / 10r", "Turn left / right (default 90 degrees)"),
            ("Tab", "Decimal point while typing"),
            ("A", "Auto-close area while drawing"),
            ("B", "Bay window mirror"),
            ("J", "Jump cursor to nearest endpoint"),
            ("D", "Toggle curve degree/height mode"),
            ("Insert", "Insert vertex tool"),
            ("Backspace", "Erase last typed character"),
        ],
    ),
    (
        "Measure",
        &[
            ("Click", "Add a point to the run"),
            ("Shift + drag", "Lock the leg to 45 degrees"),
            ("Enter / double-click", "Finish the run"),
            ("Shift + Enter", "Close the run and read its area"),
            ("Backspace", "Undo the last point"),
            ("Delete", "Clear all measurements"),
            ("Ctrl + C", "Copy measurements as text"),
            ("Esc", "Cancel run, clear, then exit"),
        ],
    ),
    (
        "Selection",
        &[
            ("P", "Select tool"),
            ("Shift + R", "Rotate selection 5 degrees"),
            ("Ctrl + Shift + R", "Rotate selection 1 degree"),
            ("H / V", "Orient selection horizontal / vertical"),
            ("+ / -", "Scale selection up / down"),
            ("Space (tap)", "Toggle dimension labels"),
        ],
    ),
    (
        "View",
        &[
            ("G", "Toggle grid"),
            ("C", "Center sketch"),
            ("Shift + C", "Center & fit"),
            ("Space (hold)", "Temporary pan"),
            ("F / M", "Flip sketch horizontal / vertical"),
            ("L / R", "Rotate sketch left / right 90 degrees"),
        ],
    ),
    (
        "Panels",
        &[
            ("F4", "Define Area panel"),
            ("F6", "Calculations panel"),
            ("S", "Symbol library"),
            ("T", "Text library"),
            ("F1", "This cheat sheet"),
        ],
    ),
];

/// The overlay's state (open or not — the sheet is static).
#[derive(Debug, Clone, Default)]
pub struct Shortcuts {
    pub open: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutsMsg {
    Open,
    Close,
}

/// The dialog's layer, or None when closed.
pub fn layer(d: &Shortcuts) -> Option<Element<'static, Message>> {
    if !d.open {
        return None;
    }
    let title = text("Keyboard Shortcuts")
        .font(SEGOE_SEMIBOLD)
        .size(13.0)
        .color(text_main());

    let group_card = |(name, rows): &'static (
        &'static str,
        &'static [(&'static str, &'static str)],
    )|
     -> Element<'static, Message> {
        let mut card = column![text(name.to_uppercase())
            .font(SEGOE_SEMIBOLD)
            .size(11.0)
            .color(muted()),]
        .spacing(6);
        for (key, desc) in *rows {
            card = card.push(
                row![
                    container(text(*key).font(CONSOLAS).size(10.5).color(text_main()),)
                        .width(96)
                        .padding([1, 6])
                        .style(|_| container::Style {
                            background: Some(Background::Color(bg_panel())),
                            border: Border {
                                color: border(),
                                width: 1.0,
                                radius: 3.0.into(),
                            },
                            ..container::Style::default()
                        }),
                    text(*desc).font(SEGOE).size(11.5).color(text_main()),
                    Space::new().width(Length::Fill),
                ]
                .spacing(8)
                .padding([2, 0]),
            );
        }
        container(card.padding(8))
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(bg_app())),
                border: Border {
                    color: border(),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    };

    let grid = iced::widget::Grid::with_children(GROUPS.iter().map(group_card).collect::<Vec<_>>())
        .columns(3)
        .spacing(12)
        .width(736);

    let actions = row![button(
        container(text("Close").font(SEGOE).size(12.0).color(WHITE),).padding([5, 12]),
    )
    .on_press(Message::Shortcuts(ShortcutsMsg::Close))
    .style(move |_, _| button::Style {
        background: Some(Background::Color(accent())),
        ..button::Style::default()
    })]
    .align_y(iced::alignment::Alignment::End)
    .width(Length::Fill);

    let card = container(column![title, grid, actions].spacing(10).padding(12))
        .width(760)
        .max_height(812)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        });

    // The card swallows its own presses; the backdrop closes.
    let centred = container(opaque(card))
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    let backdrop = container(centred)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.46).into()),
            ..container::Style::default()
        });
    Some(opaque(
        mouse_area(backdrop).on_press(Message::Shortcuts(ShortcutsMsg::Close)),
    ))
}
