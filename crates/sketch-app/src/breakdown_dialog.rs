//! The Breakdown Details dialog (`AreaBreakdown.show`).

use iced::{Element, Length};

use crate::theme::text_main;
use crate::Message;

/// The dialog's state.
#[derive(Debug, Clone, Default)]
pub struct BreakdownDialog {
    pub open: bool,
    /// `decimal` until the style button flips it.
    pub feet_inches: bool,
}

/// A control in the dialog.
#[derive(Debug, Clone)]
pub enum BreakdownMsg {
    Copy,
    ToggleStyle,
    /// The native save dialog is not ported yet; the handler toasts the gap.
    DownloadJson,
    DownloadXml,
    Close,
}

impl BreakdownDialog {
    pub fn update(&mut self, msg: BreakdownMsg) {
        match msg {
            BreakdownMsg::ToggleStyle => self.feet_inches = !self.feet_inches,
            BreakdownMsg::Copy
            | BreakdownMsg::DownloadJson
            | BreakdownMsg::DownloadXml
            | BreakdownMsg::Close => {}
        }
    }

    /// The report options for the current style.
    pub fn options(&self) -> sketch_core::breakdown::Options {
        sketch_core::breakdown::Options {
            dimension_style: if self.feet_inches {
                "feetinches"
            } else {
                "decimal"
            },
            ..sketch_core::breakdown::Options::default()
        }
    }

    /// The style button's label, as the original's handler sets it.
    pub fn style_label(&self) -> &'static str {
        if self.feet_inches {
            "Decimal feet"
        } else {
            "Feet & inches"
        }
    }
}

/// `previewSvg`'s geometry: every defined polygon scaled into the 720x520
/// viewBox (24px gap, centred in its cell), outlined with the page label.
pub fn preview(polygons: &[sketch_core::model::Polygon]) -> Element<'_, Message> {
    use iced::widget::canvas;
    use iced::{mouse, Rectangle, Renderer, Theme};
    let defined: Vec<&sketch_core::model::Polygon> = polygons
        .iter()
        .filter(|p| !p.kind.is_empty() && p.kind != "UND" && p.points.len() >= 3)
        .collect();
    let (width, height, gap) = (720.0_f32, 520.0_f32, 24.0_f32);
    let columns = 1.0_f32;
    let rows = 1.0_f32;
    let cell_width = (width - gap * (columns + 1.0)) / columns;
    let cell_height = (height - gap * (rows + 1.0)) / rows;
    let mut paths: Vec<Vec<iced::Point>> = Vec::new();
    if !defined.is_empty() {
        let flat: Vec<&sketch_core::geometry::Pt> =
            defined.iter().flat_map(|p| p.points.iter()).collect();
        let min_x = flat.iter().map(|p| p.x).fold(f64::MAX, f64::min);
        let max_x = flat.iter().map(|p| p.x).fold(f64::MIN, f64::max);
        let min_y = flat.iter().map(|p| p.y).fold(f64::MAX, f64::min);
        let max_y = flat.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        let span_x = (max_x - min_x).max(1.0) as f32;
        let span_y = (max_y - min_y).max(1.0) as f32;
        let scale = ((cell_width - 20.0) / span_x).min((cell_height - 30.0) / span_y);
        let origin_x = gap + (cell_width - span_x * scale) / 2.0;
        let origin_y = gap + (cell_height - span_y * scale) / 2.0;
        for poly in &defined {
            paths.push(
                poly.points
                    .iter()
                    .map(|pt| {
                        iced::Point::new(
                            origin_x + ((pt.x - min_x) as f32) * scale,
                            origin_y + ((max_y - pt.y) as f32) * scale,
                        )
                    })
                    .collect(),
            );
        }
    }
    struct PreviewProgram {
        paths: Vec<Vec<iced::Point>>,
        width: f32,
        height: f32,
    }
    impl<M> canvas::Program<M> for PreviewProgram {
        type State = ();
        fn draw(
            &self,
            _: &Self::State,
            renderer: &Renderer,
            _: &Theme,
            bounds: Rectangle,
            _: mouse::Cursor,
        ) -> Vec<iced::widget::canvas::Geometry> {
            use iced::widget::canvas::{Frame, Path, Stroke};
            let mut frame = Frame::new(renderer, bounds.size());
            // The SVG scales its 720x520 viewBox into the element; mirror it.
            let fit = (bounds.width / self.width).min(bounds.height / self.height);
            for path in &self.paths {
                if path.len() < 3 {
                    continue;
                }
                let scaled: Vec<iced::Point> = path
                    .iter()
                    .map(|p| iced::Point::new(p.x * fit, p.y * fit))
                    .collect();
                let outline = Path::new(|p| {
                    p.move_to(scaled[0]);
                    for pt in &scaled[1..] {
                        p.line_to(*pt);
                    }
                    p.close();
                });
                frame.stroke(
                    &outline,
                    Stroke::default().with_color(text_main()).with_width(1.0),
                );
            }
            vec![frame.into_geometry()]
        }
    }
    iced::widget::canvas(PreviewProgram {
        paths,
        width,
        height,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
