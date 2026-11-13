//! The Pages navigator (S10-040..045): the rail's Pages button opens a
//! 168px column between the rail and the canvas listing every page
//! (refs/ui-markup.html `.pages-nav`, the "Pages navigator module" script in
//! refs/sidebar-redesign.txt). A row switches to its page; its "ghost" tag
//! toggles the underlay; hovering reveals a delete; the footer select picks
//! the underlay too. It delegates to the same page operations as the status
//! bar's controls, so the behaviour is identical.

use iced::widget::{
    button, column, container, mouse_area, pick_list, row, scrollable, text, Space,
};
use iced::{alignment, Background, Border, Color, Element, Length};

use crate::icons::icon;
use crate::pages::Pages;
use crate::theme::*;
use crate::Message;

/// `.pages-nav { width: 168px }` while open.
pub const WIDTH: f32 = 168.0;

#[derive(Debug, Clone, PartialEq)]
pub enum NavMsg {
    /// The rail's Pages button.
    Toggle,
    /// The header's close button.
    Close,
    /// A row click: `goToPage(index)`.
    Go(usize),
    /// A row's "ghost" tag: `setUnderlay(same ? -1 : index)`.
    Ghost(usize),
    /// A row's trash button: go to the page, then `deletePage()`.
    Delete(usize),
    /// The footer select.
    Underlay(Option<usize>),
    /// The pointer is over (or left) a row.
    Hover(Option<usize>),
}

/// A footer option: "Underlay: None", then "Page N" for every other page.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub index: Option<usize>,
    pub label: String,
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// The footer select's options (`nav-underlay-select`).
pub fn choices(pages: &Pages) -> Vec<Choice> {
    let mut out = vec![Choice {
        index: None,
        label: "Underlay: None".into(),
    }];
    for i in 0..pages.len() {
        if i != pages.current {
            out.push(Choice {
                index: Some(i),
                label: pages.page_name(i),
            });
        }
    }
    out
}

/// `--accent-soft`.
fn accent_soft() -> Color {
    crate::command_palette::accent_soft()
}

fn tag<'a>(label: &'static str, on: bool, press: Option<Message>) -> Element<'a, Message> {
    let color = if on { accent() } else { muted() };
    let body = text(label).font(ARIAL).size(10.0).color(color);
    let edge = if on { accent() } else { border() };
    let style = move |_: &iced::Theme, _: button::Status| button::Style {
        background: None,
        text_color: color,
        border: Border {
            color: edge,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..button::Style::default()
    };
    match press {
        Some(m) => button(body).padding([1, 5]).on_press(m).style(style).into(),
        // The current page's tag is a span, not a button.
        None => container(body)
            .padding([1, 5])
            .style(move |_| container::Style {
                border: Border {
                    color: edge,
                    width: 1.0,
                    radius: 3.0.into(),
                },
                ..container::Style::default()
            })
            .into(),
    }
}

fn page_row<'a>(pages: &Pages, index: usize, hover: Option<usize>) -> Element<'a, Message> {
    let current = index == pages.current;
    let ink = if current { accent() } else { text_main() };
    let mut line = row![text(pages.page_name(index))
        .font(if current { SEGOE_SEMIBOLD } else { SEGOE })
        .size(12.0)
        .color(ink)
        .width(Length::Fill)]
    .spacing(6)
    .align_y(alignment::Vertical::Center);
    if current {
        line = line.push(tag("current", true, None));
    } else {
        line = line.push(crate::tooltips::tip_below(
            tag(
                "ghost",
                pages.underlay == Some(index),
                Some(Message::PagesNav(NavMsg::Ghost(index))),
            ),
            "Ghost this page beneath the current page (underlay)",
        ));
        if hover == Some(index) {
            line = line.push(crate::tooltips::tip_below(
                button(icon("trash", false, 13.0, muted()))
                    .padding(2)
                    .on_press(Message::PagesNav(NavMsg::Delete(index)))
                    .style(|_, _| button::Style {
                        background: None,
                        text_color: muted(),
                        ..button::Style::default()
                    }),
                "Delete this page",
            ));
        }
    }
    let hovered = hover == Some(index);
    let body = container(line)
        .padding([6, 8])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: if current {
                Some(Background::Color(accent_soft()))
            } else if hovered {
                Some(Background::Color(bg_hover()))
            } else {
                None
            },
            border: Border {
                color: if current {
                    accent()
                } else {
                    Color::TRANSPARENT
                },
                width: 1.0,
                radius: 3.0.into(),
            },
            ..container::Style::default()
        });
    container(
        mouse_area(body)
            .on_press(Message::PagesNav(NavMsg::Go(index)))
            .on_enter(Message::PagesNav(NavMsg::Hover(Some(index))))
            .on_exit(Message::PagesNav(NavMsg::Hover(None))),
    )
    .padding(iced::Padding {
        bottom: 2.0,
        ..iced::Padding::ZERO
    })
    .into()
}

/// The open panel.
pub fn view<'a>(pages: &Pages, hover: Option<usize>) -> Element<'a, Message> {
    let header = row![
        text("PAGES")
            .font(SEGOE_SEMIBOLD)
            .size(11.0)
            .color(muted())
            .width(Length::Fill),
        crate::tooltips::tip_below(
            button(icon("x", false, 14.0, muted()))
                .padding([2, 4])
                .on_press(Message::PagesNav(NavMsg::Close))
                .style(|_, status| button::Style {
                    background: matches!(status, button::Status::Hovered)
                        .then(|| Background::Color(bg_hover())),
                    text_color: muted(),
                    border: Border {
                        radius: 4.0.into(),
                        ..Border::default()
                    },
                    ..button::Style::default()
                }),
            "Close navigator",
        ),
    ]
    .align_y(alignment::Vertical::Center);
    let header = container(header).padding(iced::Padding {
        top: 10.0,
        right: 10.0,
        bottom: 8.0,
        left: 10.0,
    });

    let mut rows = column![];
    for i in 0..pages.len() {
        rows = rows.push(page_row(pages, i, hover));
    }
    let list = scrollable(container(rows).padding(iced::Padding {
        top: 0.0,
        right: 6.0,
        bottom: 6.0,
        left: 6.0,
    }))
    .height(Length::Fill);

    let options = choices(pages);
    let selected = options.iter().find(|c| c.index == pages.underlay).cloned();
    let pick = pick_list(options, selected, |c| {
        Message::PagesNav(NavMsg::Underlay(c.index))
    })
    .width(Length::Fill)
    .text_size(11.5)
    .padding([4, 6])
    .menu_style(|_| iced::overlay::menu::Style {
        background: Background::Color(bg_panel()),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 0.0.into(),
        },
        text_color: text_main(),
        selected_text_color: WHITE,
        selected_background: Background::Color(accent()),
        shadow: iced::Shadow::default(),
    })
    .style(|_, _| pick_list::Style {
        text_color: text_main(),
        placeholder_color: muted(),
        handle_color: muted(),
        background: Background::Color(bg_app()),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 3.0.into(),
        },
    });
    let footer = column![
        rect_h(1.0, border()),
        container(crate::tooltips::tip(
            pick,
            "Ghost another page beneath this one"
        ))
        .padding(8)
        .width(Length::Fill)
    ];

    let panel = container(column![header, list, footer])
        .width(WIDTH - 1.0)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            ..container::Style::default()
        });
    // `border-right: 1px solid var(--border)`.
    row![panel, rect_v(1.0, border())].into()
}

fn rect_h<'a>(h: f32, color: Color) -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(h))
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        })
        .into()
}

fn rect_v<'a>(w: f32, color: Color) -> Element<'a, Message> {
    container(Space::new().width(w).height(Length::Fill))
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            ..container::Style::default()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_footer_lists_none_then_every_other_page() {
        let mut pages = Pages::default();
        assert_eq!(choices(&pages).len(), 1, "a lone page has nothing to ghost");
        pages.add_page();
        pages.add_page();
        pages.go_to(1);
        let labels: Vec<String> = choices(&pages).iter().map(|c| c.label.clone()).collect();
        assert_eq!(labels, vec!["Underlay: None", "Page 1", "Page 3"]);
        assert_eq!(choices(&pages)[2].index, Some(2));
    }
}
