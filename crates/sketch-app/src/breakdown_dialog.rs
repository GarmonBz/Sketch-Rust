//! The Breakdown Details dialog (screen 7, S7-003): `AreaBreakdown.show`.
//!
//! Measured from the live original at 1400x860: a full-screen
//! `.breakdown-overlay` at `rgba(0,0,0,.62)` holding a 1080x720 dialog at
//! (160,70) with `--bg-panel`, a 1px `--border`, NO radius or padding. The
//! 16px title sits at (175,82) with a 12x23 close × at (1213,81). The content
//! is (161,115) 1078x624: the report `<pre>` fills the left 431px in 12px
//! Consolas `#ccc`, and the `.breakdown-preview` fills the right 647px on
//! `#1E1E1E` with a `#ccc` border. Four `#3E3E42` buttons sit at y 750 —
//! Copy for report (114.9), Feet & inches (107.5), Download JSON (124.5),
//! Download XML (116.4).
//!
//! Copy writes the report to the clipboard and toasts `Calculations copied.`
//! The style button flips the dimension style and relabels itself. Downloads
//! need a native save dialog the port has no equivalent for yet; they are
//! named in ACCEPTANCE rather than faked.

use iced::widget::{button, container, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding};

use crate::snap::nudge;
use crate::theme::{bg_app, bg_hover, bg_panel, border, muted};
use crate::theme::{hex, text_main, ARIAL};
use crate::Message;

const CARD: (f32, f32, f32, f32) = (160.0, 70.0, 1080.0, 720.0);
/// See `area_dialog::PIN_Y`.
const PIN_Y: f32 = 2.0;

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

fn place<'a>(x: f32, y: f32, el: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    crate::theme::at(x - CARD.0, y - CARD.1 + PIN_Y, el)
}

/// The dialog's layer, or None when closed. `report` is the rendered text;
/// `preview` draws the area outlines into the right pane.
pub fn layer<'a>(
    d: &'a BreakdownDialog,
    report: String,
    preview: Element<'a, Message>,
) -> Option<Element<'a, Message>> {
    if !d.open {
        return None;
    }
    let send = |m: BreakdownMsg| Message::Breakdown(m);
    let mut body = iced::widget::stack![]
        .width(Length::Fill)
        .height(Length::Fill);

    body = body.push(place(
        175.0,
        76.0,
        nudge(
            0.5,
            0.0,
            text("Breakdown Details")
                .font(ARIAL)
                .size(16.0)
                .color(text_main()),
        ),
    ));
    body = body.push(place(
        1213.0,
        74.0,
        button(text("\u{d7}").font(ARIAL).size(16.0).color(muted()))
            .on_press(send(BreakdownMsg::Close))
            .padding(0)
            .width(12.0)
            .height(23.0)
            .style(|_, _| button::Style {
                background: None,
                border: Border::default(),
                text_color: muted(),
                ..button::Style::default()
            }),
    ));
    body = body.push(place(
        161.0,
        115.0,
        container(scrollable_report(report))
            .width(431.0)
            .height(624.0)
            .style(|_| container::Style::default()),
    ));
    body = body.push(place(
        592.0,
        115.0,
        container(preview)
            .width(647.0)
            .height(624.0)
            .style(move |_| container::Style {
                background: Some(Background::Color(bg_app())),
                border: Border {
                    color: text_main(),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }),
    ));
    // The four actions at their measured positions.
    let actions: [(&str, f32, f32, f32, BreakdownMsg); 4] = [
        ("Copy for report", 175.0, 114.9, 750.0, BreakdownMsg::Copy),
        (
            d.style_label(),
            297.9,
            107.5,
            750.0,
            BreakdownMsg::ToggleStyle,
        ),
        (
            "Download JSON",
            413.4,
            124.5,
            750.0,
            BreakdownMsg::DownloadJson,
        ),
        (
            "Download XML",
            546.0,
            116.4,
            750.0,
            BreakdownMsg::DownloadXml,
        ),
    ];
    for (label, x, w, y, msg) in actions {
        body = body.push(place(x, y, dialog_button(label, w, msg)));
    }

    let card = container(body)
        .width(CARD.2)
        .height(CARD.3)
        .style(|_| container::Style {
            background: Some(Background::Color(bg_panel())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
    let backdrop = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.62))),
            ..container::Style::default()
        });
    Some(
        iced::widget::stack![
            iced::widget::opaque(
                iced::widget::mouse_area(backdrop).on_press(send(BreakdownMsg::Close))
            ),
            crate::theme::at(CARD.0, CARD.1, card)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

/// The `<pre>`: 12px Consolas `#ccc`, scrollable, transparent background.
fn scrollable_report<'a>(report: String) -> Element<'a, Message> {
    use iced::widget::scrollable::{AutoScroll, Rail, Scrollable, Scroller, Style};
    let rail = Rail {
        background: None,
        border: Border::default(),
        scroller: Scroller {
            background: Background::Color(hex(0x4E4E52)),
            border: Border {
                color: bg_panel(),
                width: 2.0,
                radius: 5.0.into(),
            },
        },
    };
    Scrollable::new(
        container(
            text(report)
                .font(iced::Font::with_name("Consolas"))
                .size(12.0)
                .color(text_main()),
        )
        .width(Length::Shrink)
        .height(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_, _| Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: AutoScroll {
            background: Background::Color(bg_panel()),
            border: Border::default(),
            shadow: iced::Shadow::default(),
            icon: text_main(),
        },
    })
    .into()
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

fn dialog_button<'a>(label: &'a str, width: f32, msg: BreakdownMsg) -> Element<'a, Message> {
    button(text(label).font(ARIAL).size(13.33).color(text_main()))
        .on_press(Message::Breakdown(msg))
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 10.0,
            right: 10.0,
        })
        .width(width)
        .height(29.0)
        .style(|_, _| button::Style {
            background: Some(Background::Color(bg_hover())),
            border: Border {
                color: border(),
                width: 1.0,
                radius: 2.0.into(),
            },
            text_color: text_main(),
            ..button::Style::default()
        })
        .into()
}
