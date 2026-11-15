//! Command palette (`Ctrl+K`) powered by `ui::commands`.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, text, text_input, Space,
};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::Message;

pub const INPUT_ID: &str = "command-palette-input";

/// `move(d)`: the highlight wraps around the matches.
pub fn step(active: usize, count: usize, d: i32) -> usize {
    if count == 0 {
        return active;
    }
    ((active as i64 + d as i64).rem_euclid(count as i64)) as usize
}

/// Command search (`Ctrl+K`) powered by `ui::commands`.
pub fn layer_next<'a>(app: &'a crate::App) -> Option<Element<'a, Message>> {
    use crate::icons;
    use crate::ui::commands;
    use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BSC_MEDIUM};
    use crate::ui::tokens;

    if !app.command_palette_open {
        return None;
    }

    let query = app.command_palette_query.as_str();
    let found = commands::matching(query);
    let active = app.command_palette_selected.min(found.len().saturating_sub(1));
    let win_w = app.window_size.width;
    let win_h = app.window_size.height;

    let input = text_input("Type a command...", query)
        .id(INPUT_ID)
        .font(BARLOW)
        .size(tokens::TEXT_14)
        .padding([12, 14])
        .width(Length::Fill)
        .on_input(Message::CommandPaletteQuery)
        .on_submit(Message::CommandPaletteRun)
        .style(|_, _| text_input::Style {
            background: Background::Color(tokens::field()),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            icon: tokens::muted(),
            placeholder: tokens::muted(),
            value: tokens::text(),
            selection: tokens::accent_soft(),
        });

    let divider = container(Space::new())
        .width(Length::Fill)
        .height(1)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::rule())),
            ..container::Style::default()
        });

    let list: Element<'a, Message> = if found.is_empty() {
        container(
            text("No matching command.")
                .font(BARLOW)
                .size(tokens::TEXT_12)
                .color(tokens::muted()),
        )
        .width(Length::Fill)
        .padding(14)
        .center_x(Length::Fill)
        .into()
    } else {
        let mut rows = column![].spacing(2);
        for (pos, cmd) in found.iter().enumerate() {
            let is_active = pos == active;
            let enabled = (cmd.enabled)(app);
            let checked = cmd.checked.is_some_and(|f| f(app));
            let label_color = if !enabled {
                tokens::disabled()
            } else if is_active {
                tokens::accent()
            } else {
                tokens::text()
            };
            let mut r = row![].spacing(tokens::SPACE_2).align_y(iced::Alignment::Center);
            if checked {
                r = r.push(icons::mono("check", 14.0, tokens::accent()));
            } else {
                r = r.push(Space::new().width(Length::Fixed(14.0)));
            }
            r = r.push(
                text(cmd.label)
                    .font(BARLOW_MEDIUM)
                    .size(tokens::TEXT_13)
                    .color(label_color)
                    .width(Length::Fill),
            );
            r = r.push(
                text(cmd.group.label())
                    .font(BSC_MEDIUM)
                    .size(tokens::TEXT_11)
                    .color(tokens::muted()),
            );
            if !cmd.shortcut.is_empty() {
                r = r.push(
                    text(cmd.shortcut)
                        .font(BSC_MEDIUM)
                        .size(tokens::TEXT_11)
                        .color(tokens::muted()),
                );
            }

            let mut row_btn = button(r)
                .width(Length::Fill)
                .padding([8, 10])
                .style(move |_, status| {
                    let bg = if is_active {
                        Some(Background::Color(tokens::accent_soft()))
                    } else if matches!(status, button::Status::Hovered) && enabled {
                        Some(Background::Color(tokens::hover()))
                    } else {
                        None
                    };
                    button::Style {
                        background: bg,
                        text_color: label_color,
                        border: Border {
                            color: if is_active {
                                tokens::accent()
                            } else {
                                Color::TRANSPARENT
                            },
                            width: if is_active { 1.0 } else { 0.0 },
                            radius: tokens::RADIUS_CONTROL.into(),
                        },
                        ..button::Style::default()
                    }
                });
            if enabled {
                row_btn = row_btn.on_press(Message::CommandPaletteRunAt(pos));
            }
            rows = rows.push(row_btn);
        }
        scrollable(
            container(rows)
                .padding(Padding {
                    top: 6.0,
                    right: 14.0,
                    bottom: 6.0,
                    left: 6.0,
                })
                .width(Length::Fill),
        )
        .height(Length::Shrink)
        .into()
    };
    let list = container(list).max_height((win_h * 0.48).max(160.0));

    let panel = container(column![input, divider, list])
        .width((win_w - 24.0).clamp(260.0, 520.0))
        .clip(true)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::bar())),
            border: Border {
                color: tokens::rule(),
                width: 1.0,
                radius: tokens::RADIUS_DIALOG.into(),
            },
            shadow: tokens::floating_shadow(),
            ..container::Style::default()
        });

    let placed = container(opaque(panel))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding(Padding {
            top: (win_h * 0.10).max(app.ui.density.bar_h() + 8.0),
            ..Padding::ZERO
        });
    let backdrop = container(placed)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::scrim())),
            ..container::Style::default()
        });
    Some(opaque(
        mouse_area(backdrop).on_press(Message::CommandPaletteClose),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highlight_wraps() {
        assert_eq!(step(0, 5, -1), 4);
        assert_eq!(step(4, 5, 1), 0);
        assert_eq!(step(2, 5, 1), 3);
        assert_eq!(step(3, 0, 1), 3);
    }
}
