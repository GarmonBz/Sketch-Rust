//! Hover tooltips shown after a short hover delay.

use std::time::Duration;

use iced::widget::{container, text, tooltip};
use iced::{Background, Border, Color, Element};

use crate::theme::*;
use crate::Message;

/// A tooltip appears after about half a second of hovering.
const DELAY: Duration = Duration::from_millis(500);

/// `content` with the tooltip `title` at the pointer (an empty title adds
/// nothing).
pub fn tip<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
) -> Element<'a, Message> {
    tip_at(content, title, tooltip::Position::FollowCursor)
}

/// `tip` below the control.
pub fn tip_below<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
) -> Element<'a, Message> {
    tip_at(content, title, tooltip::Position::Bottom)
}

fn tip_at<'a>(
    content: impl Into<Element<'a, Message>>,
    title: impl Into<String>,
    position: tooltip::Position,
) -> Element<'a, Message> {
    let content = content.into();
    let title: String = title.into();
    if title.is_empty() {
        return content;
    }
    let body = container(text(title).font(SEGOE).size(12.0).color(Color::BLACK))
        .padding([2, 6])
        .style(|_| container::Style {
            background: Some(Background::Color(hex(0xF2F2F2))),
            border: Border {
                color: hex(0x767676),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
    tooltip(content, body, position).gap(4).delay(DELAY).into()
}
