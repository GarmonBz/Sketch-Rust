//! Floating amber Length Strip shown in Wall, Inner, and Measure modes
//! (`01-design-spec.md` §5.6, P9).
//!
//! Positioned near the last endpoint in canvas-local screen coordinates,
//! clamped inside the canvas rectangle and nudged away from the pointer so it
//! never occludes the active vertex or cursor.

use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke};
use iced::widget::{button, canvas as canvas_widget, column, container, opaque, row, text, Space};
use iced::{
    mouse, Alignment, Background, Border, Color, Element, Length, Padding, Point, Rectangle,
    Renderer, Theme,
};

use crate::ui::fonts::{BARLOW, BARLOW_MEDIUM, BARLOW_SEMIBOLD, BSC_SEMIBOLD};
use crate::ui::layout::{self, SizeClass};
use crate::ui::tokens::{
    self, Density, NUM_24, RADIUS_CONTROL, RADIUS_TILE, SPACE_1, SPACE_2, TEXT_11, TEXT_12, TEXT_13,
};
use crate::ui::Msg as UiMsg;
use crate::view_state::{Mode, World};
use crate::{App, Message};

const DRAW_CARD_W: f32 = 248.0;
const DRAW_CARD_H: f32 = 74.0;
const MEASURE_CARD_W: f32 = 310.0;
const MEASURE_CARD_H: f32 = 104.0;

/// Length Strip overlay for `layout::drawing_stack`.
pub fn layer<'a>(app: &'a App) -> Option<Element<'a, Message>> {
    if !matches!(
        app.view.mode,
        Mode::DrawExterior | Mode::DrawInterior | Mode::Measure
    ) {
        return None;
    }

    let d = app.ui.density;
    let is_measure = app.view.mode == Mode::Measure;
    let card_w = if is_measure {
        MEASURE_CARD_W
    } else {
        DRAW_CARD_W
    };
    let card_h = if is_measure {
        MEASURE_CARD_H
    } else {
        DRAW_CARD_H
    };

    let body: Element<'a, Message> = if is_measure {
        measure_body(app, d)
    } else {
        wall_body(app)
    };

    let strip = opaque(
        container(
            column![
                canvas_widget(TapeTicks)
                    .width(Length::Fill)
                    .height(Length::Fixed(6.0)),
                container(body).padding(Padding {
                    top: 4.0,
                    right: 10.0,
                    bottom: 8.0,
                    left: 10.0,
                }),
            ]
            .width(Length::Fill),
        )
        .width(Length::Fixed(card_w))
        .style(|_| container::Style {
            background: Some(Background::Color(tokens::tape())),
            border: Border {
                color: Color::from_rgba(0.12, 0.12, 0.12, 0.35),
                width: 1.0,
                radius: RADIUS_TILE.into(),
            },
            shadow: tokens::floating_shadow(),
            ..container::Style::default()
        }),
    );

    let (x, y) = strip_position(app, card_w, card_h);

    Some(
        column![
            Space::new().height(Length::Fixed(y)),
            row![Space::new().width(Length::Fixed(x)), strip],
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into(),
    )
}

fn wall_body<'a>(app: &'a App) -> Element<'a, Message> {
    let ink = tokens::tape_ink();
    let muted_ink = Color { a: 0.72, ..ink };
    let pen_label = if app.drawing.hud.down {
        "pen down"
    } else {
        "pen up"
    };

    let hint = if !app.drawing.buffer.is_empty() {
        "Arrow key sets direction · Enter commits"
    } else if app.drawing.hud.down {
        "Type length + arrow key · Enter finishes"
    } else {
        "Tap canvas or type length to start"
    };

    let pen_pill = container(
        text(pen_label)
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_11)
            .color(tokens::tape()),
    )
    .padding([2, 6])
    .style(move |_| container::Style {
        background: Some(Background::Color(ink)),
        border: Border {
            radius: RADIUS_CONTROL.into(),
            ..Border::default()
        },
        ..container::Style::default()
    });

    let top = row![
        text(&app.drawing.hud.len)
            .font(BSC_SEMIBOLD)
            .size(NUM_24)
            .color(ink),
        text(&app.drawing.hud.ang)
            .font(BARLOW_SEMIBOLD)
            .size(TEXT_13)
            .color(ink),
        Space::new().width(Length::Fill),
        pen_pill,
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    column![
        top,
        text(hint).font(BARLOW).size(TEXT_11).color(muted_ink),
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

fn measure_body<'a>(app: &'a App, d: Density) -> Element<'a, Message> {
    let ink = tokens::tape_ink();
    let muted_ink = Color { a: 0.72, ..ink };
    let (leg, total) = measure_totals(app);
    let units = &app.drawing.options.units;

    let top = row![
        text(units.length(total))
            .font(BSC_SEMIBOLD)
            .size(NUM_24)
            .color(ink),
        text(format!("Leg {}", units.length(leg)))
            .font(BARLOW_MEDIUM)
            .size(TEXT_12)
            .color(muted_ink),
        Space::new().width(Length::Fill),
    ]
    .spacing(SPACE_2)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let has_pts = !app.drawing.measure_points.is_empty();
    let has_any = has_pts || !app.drawing.measure_runs.is_empty();
    let can_close = app.drawing.measure_points.len() >= 2;

    let actions = row![
        tape_btn("Finish", has_pts.then_some(Message::Ui(UiMsg::MeasureFinish)), d),
        tape_btn("Close", can_close.then_some(Message::Ui(UiMsg::MeasureClose)), d),
        tape_btn(
            "Undo point",
            has_pts.then_some(Message::Ui(UiMsg::MeasureUndo)),
            d
        ),
        tape_btn("Clear", has_any.then_some(Message::Ui(UiMsg::MeasureClear)), d),
        tape_btn("Copy", has_any.then_some(Message::Ui(UiMsg::MeasureCopy)), d),
    ]
    .spacing(SPACE_1)
    .align_y(Alignment::Center);

    column![top, actions].spacing(SPACE_1).width(Length::Fill).into()
}

fn tape_btn<'a>(
    label: &'a str,
    on_press: Option<Message>,
    d: Density,
) -> Element<'a, Message> {
    let ink = tokens::tape_ink();
    let disabled = on_press.is_none();
    let fg = if disabled {
        Color { a: 0.4, ..ink }
    } else {
        ink
    };
    let h = if matches!(d, Density::Touch) { 26.0 } else { 22.0 };
    let mut b = button(
        container(text(label).font(BARLOW_SEMIBOLD).size(TEXT_11).color(fg))
            .height(Length::Fixed(h))
            .padding([0, 6])
            .align_y(Alignment::Center),
    )
    .padding(0)
    .style(move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if disabled {
                Color::from_rgba(0.12, 0.12, 0.12, 0.05)
            } else if hovered {
                Color::from_rgba(0.12, 0.12, 0.12, 0.20)
            } else {
                Color::from_rgba(0.12, 0.12, 0.12, 0.11)
            })),
            text_color: fg,
            border: Border {
                color: Color::from_rgba(0.12, 0.12, 0.12, 0.25),
                width: 1.0,
                radius: RADIUS_CONTROL.into(),
            },
            ..button::Style::default()
        }
    });
    if let Some(msg) = on_press {
        b = b.on_press(msg);
    }
    b.into()
}

pub(crate) fn measure_totals(app: &App) -> (f64, f64) {
    let segs = |pts: &[sketch_core::geometry::Pt]| -> f64 {
        pts.windows(2)
            .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
            .sum()
    };
    let current = &app.drawing.measure_points;
    let leg = if current.len() >= 2 {
        let n = current.len();
        (current[n - 1].x - current[n - 2].x).hypot(current[n - 1].y - current[n - 2].y)
    } else {
        0.0
    };
    let runs_total: f64 = app.drawing.measure_runs.iter().map(|r| segs(r)).sum();
    (leg, runs_total + segs(current))
}

fn strip_position(app: &App, card_w: f32, card_h: f32) -> (f32, f32) {
    let rect = layout::canvas_rect(app);
    let cw = rect.width.max(card_w + 24.0);
    let ch = rect.height.max(card_h + 24.0);

    let sc = SizeClass::of(app.window_size.width);
    let top_safe = match sc {
        SizeClass::Compact => 92.0,
        SizeClass::Medium => 54.0,
        SizeClass::Wide => {
            if matches!(app.ui.density, Density::Touch) || rect.width < 1240.0 {
                54.0
            } else {
                16.0
            }
        }
    };
    let bottom_safe = match sc {
        SizeClass::Compact => 12.0,
        SizeClass::Medium | SizeClass::Wide => app.ui.density.tray_button().1 + 36.0,
    };

    let anchor_world = if app.view.mode == Mode::Measure {
        app.drawing.measure_points.last().copied()
    } else if app.drawing.pen_down {
        Some(app.drawing.cursor)
    } else {
        None
    };

    let (ax, ay) = if let Some(pt) = anchor_world {
        let (sx, sy) = app.view.to_screen(World { x: pt.x, y: pt.y });
        (sx as f32, sy as f32)
    } else {
        (cw * 0.5 - card_w * 0.5 - 18.0, top_safe)
    };

    let min_x = 12.0_f32;
    let max_x = (cw - card_w - 12.0).max(min_x);
    let min_y = top_safe.min((ch - card_h - 12.0).max(12.0));
    let max_y = (ch - card_h - bottom_safe).max(min_y);

    let mut x = (ax + 18.0).clamp(min_x, max_x);
    let mut y = (ay + 18.0).clamp(min_y, max_y);

    // Keep strip away from the live pointer if the pointer lies inside the canvas.
    if let Some((px, py)) = app.canvas_pointer.get() {
        if px >= 0.0 && px <= cw && py >= 0.0 && py <= ch {
            let pad = 18.0;
            let overlaps = |cx: f32, cy: f32| {
                px >= cx - pad
                    && px <= cx + card_w + pad
                    && py >= cy - pad
                    && py <= cy + card_h + pad
            };
            if overlaps(x, y) {
                let above_y = (ay - card_h - 22.0).clamp(min_y, max_y);
                if !overlaps(x, above_y) {
                    y = above_y;
                } else {
                    let left_x = (ax - card_w - 22.0).clamp(min_x, max_x);
                    if !overlaps(left_x, y) {
                        x = left_x;
                    }
                }
            }
        }
    }

    // Keep the strip out from under the floating Pad (its readout mirrors
    // the strip, but the strip must stay readable beside it).
    if app.touch_panel_open && !matches!(sc, SizeClass::Compact) {
        let origin = crate::ui::pad::floating_origin(app);
        let (pl, pt) = (origin.x - rect.x - 8.0, origin.y - rect.y - 8.0);
        let (pr, pb) = (
            pl + crate::ui::pad::FLOAT_W + 16.0,
            pt + crate::ui::pad::FLOAT_H + 16.0,
        );
        let under = |cx: f32, cy: f32| {
            cx < pr && cx + card_w > pl && cy < pb && cy + card_h > pt
        };
        if under(x, y) {
            let left_x = (pl - card_w).clamp(min_x, max_x);
            let above_y = (pt - card_h).clamp(min_y, max_y);
            if !under(left_x, y) {
                x = left_x;
            } else if !under(x, above_y) {
                y = above_y;
            }
        }
    }

    (x, y)
}

struct TapeTicks;

impl<M> canvas::Program<M> for TapeTicks {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let color = Color {
            a: 0.45,
            ..tokens::tape_ink()
        };
        let stroke = Stroke::default().with_color(color).with_width(1.0);
        let mut x = 6.0_f32;
        let mut idx = 0_usize;
        while x < bounds.width - 4.0 {
            let tick_h = if idx.is_multiple_of(4) { 6.0 } else { 3.5 };
            let path = Path::line(Point::new(x, 0.0), Point::new(x, tick_h));
            frame.stroke(&path, stroke);
            x += 6.0;
            idx += 1;
        }
        vec![frame.into_geometry()]
    }
}
