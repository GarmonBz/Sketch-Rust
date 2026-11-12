//! The Mini-Map (S12-012/018/055) - `UIManager.toggleMiniMap`
//! (ui-manager.js:2486): a PNG snapshot of the drawing canvas, shown in a
//! 260px card at the canvas's bottom-right corner (`.mini-map-overlay`,
//! theme.css 1535: right/bottom 14px, padding 8, 1px border, radius 4, a
//! small shadow) with a translucent `x` (`Close Mini-Map`) at its top-right.
//!
//! The snapshot is static: it is taken when the map opens and never
//! refreshes, as in the original. The original reads `canvas.toDataURL`;
//! here the window's screenshot is cropped to the canvas rectangle (so it
//! carries the same grid, drawing and selection the canvas shows). A harness
//! with no window cannot capture one, so it gets a blank canvas-coloured
//! snapshot of the same proportions.

use iced::widget::{button, container, image, opaque, stack, text};
use iced::{Background, Border, Color, Element, Length, Padding, Shadow, Vector};

use crate::theme::*;
use crate::Message;

/// `.mini-map-overlay` width.
const CARD_W: f32 = 260.0;
/// `right` / `bottom` offsets inside the canvas container.
const EDGE: f32 = 14.0;

/// The canvas rectangle in the window, in logical px: right of the 41px rail
/// column, below the 30px title bar and the 99px ribbon.
pub const CANVAS_ORIGIN: (f32, f32) = (41.0, 129.0);

/// Crop `rect` (x, y, w, h in physical px) out of an RGBA screenshot of
/// `size`. None when the rectangle is empty or lies outside the image.
pub fn crop_rgba(
    rgba: &[u8],
    size: (u32, u32),
    rect: (u32, u32, u32, u32),
) -> Option<(u32, u32, Vec<u8>)> {
    let (x, y, w, h) = rect;
    let (sw, sh) = size;
    if w == 0 || h == 0 || x >= sw || y >= sh || rgba.len() < (sw as usize) * (sh as usize) * 4 {
        return None;
    }
    let w = w.min(sw - x);
    let h = h.min(sh - y);
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for row in y..y + h {
        let start = ((row * sw + x) * 4) as usize;
        out.extend_from_slice(&rgba[start..start + (w * 4) as usize]);
    }
    Some((w, h, out))
}

/// The canvas rectangle (x, y, w, h in physical px) inside a window
/// screenshot: the window's logical size (physical / `scale`) minus the 41px
/// rail column, the dock (`dock_w`), and the 30 + 99px of title and ribbon
/// and 28px of status bar. Read off the screenshot itself, so it is right
/// whatever size the window has been resized to.
#[cfg(test)]
pub fn canvas_rect(size: (u32, u32), scale: f32, dock_w: f32) -> (u32, u32, u32, u32) {
    canvas_rect_with_nav(size, scale, dock_w, 0.0)
}

/// `canvas_rect` with the Pages navigator (`nav_w` wide, 0 when closed)
/// pushing the canvas right.
pub fn canvas_rect_with_nav(
    size: (u32, u32),
    scale: f32,
    dock_w: f32,
    nav_w: f32,
) -> (u32, u32, u32, u32) {
    let (lw, lh) = (size.0 as f32 / scale, size.1 as f32 / scale);
    let left = CANVAS_ORIGIN.0 + nav_w;
    let (w, h) = (lw - left - dock_w, lh - CANVAS_ORIGIN.1 - 28.0);
    (
        (left * scale).round() as u32,
        (CANVAS_ORIGIN.1 * scale).round() as u32,
        (w.max(0.0) * scale).round() as u32,
        (h.max(0.0) * scale).round() as u32,
    )
}

/// The snapshot from a window screenshot: the canvas rectangle as an image
/// handle.
pub fn snapshot(
    rgba: &[u8],
    size: (u32, u32),
    scale: f32,
    dock_w: f32,
    nav_w: f32,
) -> Option<image::Handle> {
    let rect = canvas_rect_with_nav(size, scale, dock_w, nav_w);
    let (w, h, pixels) = crop_rgba(rgba, size, rect)?;
    Some(image::Handle::from_rgba(w, h, pixels))
}

/// A blank snapshot in the canvas background colour for a headless harness,
/// at a quarter of the canvas size to stay cheap.
pub fn blank(canvas: (f64, f64)) -> image::Handle {
    let w = ((canvas.0 / 4.0).round() as u32).max(1);
    let h = ((canvas.1 / 4.0).round() as u32).max(1);
    let bg = crate::canvas_theme::current().bg();
    let px = [
        (bg.r * 255.0).round() as u8,
        (bg.g * 255.0).round() as u8,
        (bg.b * 255.0).round() as u8,
        255,
    ];
    let pixels: Vec<u8> = (0..w * h).flat_map(|_| px).collect();
    image::Handle::from_rgba(w, h, pixels)
}

/// The overlay layer, or None when the map is closed. `dock_w` and the 28px
/// status bar keep the card inside the canvas container.
pub fn layer(map: Option<&image::Handle>, dock_w: f32) -> Option<Element<'_, Message>> {
    let handle = map?;
    let close = button(text("\u{d7}").size(14.0).color(Color::WHITE))
        .padding([0, 4])
        .on_press(Message::MiniMapClose)
        .style(|_, _| button::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.55))),
            text_color: Color::WHITE,
            ..button::Style::default()
        });
    let picture = image(handle.clone())
        .width(Length::Fill)
        .content_fit(iced::ContentFit::Contain);
    let card = container(stack![
        picture,
        container(close)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .padding(Padding {
                top: -5.0,
                right: -3.0,
                bottom: 0.0,
                left: 0.0,
            }),
    ])
    .width(CARD_W)
    .padding(8)
    .style(|_| container::Style {
        background: Some(Background::Color(bg_panel())),
        border: Border {
            color: border(),
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.45),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..container::Style::default()
    });
    Some(
        container(opaque(card))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(Padding {
                top: 0.0,
                right: dock_w + EDGE,
                bottom: 28.0 + EDGE,
                left: 0.0,
            })
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> Vec<u8> {
        (0..w * h)
            .flat_map(|i| [(i % w) as u8, (i / w) as u8, 0, 255])
            .collect()
    }

    #[test]
    fn crop_takes_the_requested_rectangle() {
        let src = gradient(10, 8);
        let (w, h, px) = crop_rgba(&src, (10, 8), (2, 3, 4, 2)).unwrap();
        assert_eq!((w, h), (4, 2));
        // The first pixel is (2, 3); the last is (5, 4).
        assert_eq!(&px[0..2], &[2, 3]);
        assert_eq!(&px[px.len() - 4..px.len() - 2], &[5, 4]);
    }

    #[test]
    fn crop_clamps_to_the_image_and_refuses_nothing() {
        let src = gradient(10, 8);
        let (w, h, _) = crop_rgba(&src, (10, 8), (8, 6, 50, 50)).unwrap();
        assert_eq!((w, h), (2, 2));
        assert!(crop_rgba(&src, (10, 8), (10, 0, 4, 4)).is_none());
        assert!(crop_rgba(&src, (10, 8), (0, 0, 0, 4)).is_none());
        assert!(crop_rgba(&src[..8], (10, 8), (0, 0, 2, 2)).is_none());
    }

    #[test]
    fn the_canvas_rectangle_follows_the_screenshot_size() {
        // 1400x860 at 1x with a 320px dock: 1400-41-320 by 860-129-28.
        assert_eq!(canvas_rect((1400, 860), 1.0, 320.0), (41, 129, 1039, 703));
        // The same window at 2x: everything doubles.
        assert_eq!(canvas_rect((2800, 1720), 2.0, 320.0), (82, 258, 2078, 1406));
        // A tiny window never goes negative.
        assert_eq!(canvas_rect((30, 30), 1.0, 320.0).2, 0);
    }

    #[test]
    fn snapshot_crops_the_canvas_out_of_a_screenshot() {
        let src = gradient(400, 300);
        assert!(snapshot(&src, (400, 300), 1.0, 100.0, 0.0).is_some());
        assert!(snapshot(&src, (30, 30), 1.0, 100.0, 0.0).is_none());
    }
}
