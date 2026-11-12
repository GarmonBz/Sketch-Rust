//! Offscreen software rendering for a clean, layer-ordered sketch export.
//! Shares canvas geometry, patterns, wall solids and shaped font rendering.

use iced::advanced::renderer::{self, Headless};
use iced::widget::canvas::{self, Geometry};
use iced::{mouse, Color, Element, Pixels, Rectangle, Renderer, Size, Theme};
use iced_runtime::user_interface::{Cache, UserInterface};

use crate::render::Scene;

/// The canvas program that paints the overlay.
struct Overlay<'a>(Scene<'a>, &'a [crate::layers::LayerGates]);

impl canvas::Program<()> for Overlay<'_> {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        crate::render::draw_clean(&mut frame, &self.0, self.1);
        vec![frame.into_geometry()]
    }
}

/// Poll a future that never waits (the software renderer's constructor).
pub(crate) fn ready<T>(future: impl std::future::Future<Output = T>) -> Option<T> {
    use std::task::{Context, Poll, Waker};
    let mut future = std::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => Some(value),
        Poll::Pending => None,
    }
}

/// The overlay's pixels (premultiplied RGBA, `size`), or `None` when no
/// software renderer can be made.
pub fn render_clean(
    scene: Scene,
    size: (u32, u32),
    layers: &[crate::layers::LayerGates],
) -> Option<Vec<u8>> {
    let mut renderer = ready(<Renderer as Headless>::new(
        crate::theme::SEGOE,
        Pixels(16.0),
        Some("tiny-skia"),
    ))??;
    let bounds = Size::new(size.0 as f32, size.1 as f32);
    let element: Element<'_, (), Theme, Renderer> = canvas::Canvas::new(Overlay(scene, layers))
        .width(bounds.width)
        .height(bounds.height)
        .into();
    let mut ui = UserInterface::build(element, bounds, Cache::default(), &mut renderer);
    ui.draw(
        &mut renderer,
        &Theme::Dark,
        &renderer::Style {
            text_color: Color::BLACK,
        },
        mouse::Cursor::Unavailable,
    );
    Some(renderer.screenshot(Size::new(size.0, size.1), 1.0, Color::TRANSPARENT))
}
