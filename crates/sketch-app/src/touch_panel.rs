//! Touch panel (floating 'Touch Drawing' window).

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::Message;

pub fn layer(open: bool, distance: f64) -> Option<Element<'static, Message>> {
    if !open {
        return None;
    }
    let dist_str = format!("{:.1}", distance);
    let col = column![
        text("Touch Drawing").size(13),
        text_input("1", &dist_str)
            .on_input(Message::TouchDistance)
            .width(Length::Fixed(80.0)),
        row![
            button("Up").on_press(Message::TouchAction("up".into())),
            button("Down").on_press(Message::TouchAction("down".into())),
            button("Left").on_press(Message::TouchAction("left".into())),
            button("Right").on_press(Message::TouchAction("right".into())),
        ],
        row![
            button("Turn Left").on_press(Message::TouchAction("turn-left".into())),
            button("Turn Right").on_press(Message::TouchAction("turn-right".into())),
        ],
        row![
            button("Curve Left").on_press(Message::TouchAction("curve-left".into())),
            button("Curve Right").on_press(Message::TouchAction("curve-right".into())),
        ],
        row![
            button("Delete").on_press(Message::TouchAction("delete".into())),
            button("Insert Vertex").on_press(Message::TouchAction("insert".into())),
        ],
        button("Close").on_press(Message::TouchPanelToggle),
    ]
    .spacing(6)
    .padding(8);
    let panel = container(col)
        .padding(8)
        .width(Length::Fixed(260.0))
        .style(|_| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.15, 0.15, 0.15,
            ))),
            border: iced::Border {
                color: iced::Color::WHITE,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..container::Style::default()
        });
    let overlay = container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::End)
        .align_y(Alignment::End);
    Some(overlay.into())
}
