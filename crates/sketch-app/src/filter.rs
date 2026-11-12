//! `#filter-popover` — UIManager.toggleFilterPanel: "Show in sketch" with a
//! checkbox per filter. Fixed at `right: 18px; bottom: 42px` (page
//! 1212,638 170x180 at 1400x860), padding 12, gap 7, 12px text,
//! `--bg-panel`, 1px border, radius 4, shadow `0 5px 18px rgba(0,0,0,.4)`.
//! Chrome's dark-scheme checkbox (13x13, radius 2) measured from the
//! original: checked = #007acc with a white check, unchecked = #3b3b3b with
//! a 1px #858585 border.

use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::widget::{button, canvas as canvas_widget, column, container, row, stack, Space};
use iced::{
    mouse, Border, Color, Element, Length, Point, Rectangle, Renderer, Shadow, Size, Theme, Vector,
};

use crate::statusbar::StatusMsg;
use crate::theme::*;
use crate::view_state::FILTERS;
use crate::Message;

struct Check(bool);

impl<M> canvas::Program<M> for Check {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut f = Frame::new(renderer, bounds.size());
        let r = 2.0;
        let rounded = |inset: f32| {
            Path::rounded_rectangle(
                Point::new(inset, inset),
                Size::new(13.0 - 2.0 * inset, 13.0 - 2.0 * inset),
                (r - inset).max(0.0).into(),
            )
        };
        if self.0 {
            f.fill(&rounded(0.0), accent());
            let check = Path::new(|p| {
                p.move_to(Point::new(2.8, 6.4));
                p.line_to(Point::new(5.6, 9.2));
                p.line_to(Point::new(10.3, 2.8));
            });
            f.stroke(&check, Stroke::default().with_color(WHITE).with_width(1.8));
        } else {
            f.fill(&rounded(0.0), muted());
            f.fill(&rounded(1.0), hex(0x3b3b3b));
        }
        vec![f.into_geometry()]
    }
}

fn item<'a>(index: usize, text: &str, on: bool) -> Element<'a, Message> {
    // The whole <label> toggles; the text follows the box after a space.
    let body = stack![
        canvas_widget(Check(on)).width(13).height(13),
        at(18.0, 0.0, label(text, 12.0, SEGOE, text_main(), 16.0)),
    ]
    .width(144)
    .height(16);
    button(body)
        .padding(0)
        .on_press(Message::Status(StatusMsg::Filter(index)))
        .style(|_, _| button::Style::default())
        .into()
}

/// The popover layer (full window; only the popover takes input).
pub fn layer<'a>(filters: &[bool; 6]) -> Element<'a, Message> {
    let mut list = column![label("Show in sketch", 12.0, SEGOE_BOLD, text_main(), 16.0)].spacing(7);
    for (i, (_, text)) in FILTERS.iter().enumerate() {
        list = list.push(item(i, text, filters[i]));
    }
    let popover = container(list)
        .padding(12)
        .width(170)
        .style(|_| container::Style {
            background: Some(bg_panel().into()),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 5.0),
                blur_radius: 18.0,
            },
            ..container::Style::default()
        });
    column![
        Space::new().height(Length::Fill),
        row![
            Space::new().width(Length::Fill),
            popover,
            Space::new().width(18)
        ],
        Space::new().height(42),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
