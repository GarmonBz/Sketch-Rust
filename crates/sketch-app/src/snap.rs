//! `Snap` — draws its content shifted so its left edge lands on a whole
//! pixel. A flex-centred row (the status bar's `.status-center`) places
//! children at fractional x; quads snap to the pixel grid but text does
//! not, so the text inside drifted 1px from Chrome, which lays each
//! control's text out from an integer-snapped box. Only x is snapped (the
//! status bar keeps its measured 0.5px y nudge). Input is unaffected beyond
//! the sub-half-pixel shift.

use iced::advanced::layout;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Operation, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::{Element, Event, Length, Rectangle, Size, Vector};

pub struct Snap<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    floor: bool,
    /// Also snap y to the nearest pixel.
    y: bool,
    /// An extra draw-only shift after snapping.
    extra: Vector,
    /// Snap x at all (false for a pure nudge).
    snap_x: bool,
}

/// Snap to the nearest pixel.
pub fn snap<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Snap<'a, Message, Theme, Renderer> {
    Snap {
        content: content.into(),
        floor: false,
        y: false,
        extra: Vector::ZERO,
        snap_x: true,
    }
}

/// Snap down to the pixel (the zoom readout: it lands at ~510.67, Chrome
/// measured its box at 510 — rounding would draw it a pixel late).
pub fn snap_floor<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Snap<'a, Message, Theme, Renderer> {
    Snap {
        content: content.into(),
        floor: true,
        y: false,
        extra: Vector::ZERO,
        snap_x: true,
    }
}

/// Shift text down by `dy` at draw time. iced rasterizes a text row at
/// `floor(line top)` where Chrome uses `round(line top)` (measured on the
/// dialog: tops 362.8 / 363 / 364 drew at 366 / 367 / 368 here, Chrome drew
/// 362.8 at 367), so a 0.5px nudge turns iced's floor into Chrome's round
/// for text laid out at the same fractional position.
pub fn nudge_y<'a, Message, Theme, Renderer>(
    dy: f32,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Snap<'a, Message, Theme, Renderer> {
    nudge(0.0, dy, content)
}

/// A draw-only shift by (`dx`, `dy`) — the same floor-vs-round correction
/// for text centred at a fractional x.
pub fn nudge<'a, Message, Theme, Renderer>(
    dx: f32,
    dy: f32,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Snap<'a, Message, Theme, Renderer> {
    Snap {
        content: content.into(),
        floor: false,
        y: false,
        extra: Vector::new(dx, dy),
        snap_x: false,
    }
}

/// The draw shift that puts `bounds.x` on a whole pixel.
pub fn offset(bounds: Rectangle, floor: bool) -> Vector {
    let target = if floor {
        bounds.x.floor()
    } else {
        bounds.x.round()
    };
    Vector::new(target - bounds.x, 0.0)
}

/// `offset` plus y rounded to the nearest pixel.
pub fn offset_xy(bounds: Rectangle, floor: bool, y: bool) -> Vector {
    let mut v = offset(bounds, floor);
    if y {
        v.y = bounds.y.round() - bounds.y;
    }
    v
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Snap<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let base = if self.snap_x {
            offset_xy(layout.bounds(), self.floor, self.y)
        } else {
            Vector::ZERO
        };
        let shift = base + self.extra;
        renderer.with_translation(shift, |r| {
            self.content.as_widget().draw(
                &tree.children[0],
                r,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Snap<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + renderer::Renderer,
{
    fn from(s: Snap<'a, Message, Theme, Renderer>) -> Self {
        Element::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_x_to_the_nearest_pixel_only() {
        let r = |x: f32| Rectangle::new(iced::Point::new(x, 3.5), Size::new(10.0, 10.0));
        assert_eq!(offset(r(599.67), false), Vector::new(600.0 - 599.67, 0.0));
        assert_eq!(offset(r(652.4), false), Vector::new(652.0 - 652.4, 0.0));
        assert_eq!(offset(r(384.0), false), Vector::new(0.0, 0.0));
        assert_eq!(offset(r(510.67), true), Vector::new(510.0 - 510.67, 0.0));
        let card = |y: f32| Rectangle::new(iced::Point::new(510.0, y), Size::new(380.0, 119.4));
        assert!((offset_xy(card(370.3), false, true).y - (370.0 - 370.3)).abs() < 1e-4);
        assert!((offset_xy(card(347.8), false, true).y - (348.0 - 347.8)).abs() < 1e-4);
        assert_eq!(offset_xy(card(370.3), false, true).x, 0.0);
    }
}
