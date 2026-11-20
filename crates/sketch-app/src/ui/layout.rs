//! Responsive size classes, window region builders, and canvas geometry
//! for the Next shell (`docs/ui-redesign/01-design-spec.md` §4 & `02-architecture.md` §7).

use iced::widget::{column, container, mouse_area, row, Space, Stack};
use iced::{Alignment, Background, Color, Element, Length, Rectangle};

use crate::ui::tokens::{self, Density, SPACE_1, SPACE_2, SPACE_3};
use crate::ui::Msg;
use crate::{App, Message};

pub const WIDE_MIN: f32 = 1100.0;
pub const MEDIUM_MIN: f32 = 700.0;

pub const PAGE_BAR_H: f32 = 44.0;
pub const SLIDE_OVER_W: f32 = 280.0;
pub const COMPACT_SHEET_H: f32 = 360.0;

/// Bounds shared by the tray, drawing controls, and their popovers. Side lanes
/// are capped (and scroll if necessary), so labels cannot spill into the tray.
pub const CHROME_INSET: f32 = SPACE_3;
pub const CHROME_GAP: f32 = SPACE_3;
pub const CONTROL_LANE_W: f32 = 320.0;
pub const TRAY_GAP: f32 = 4.0;
pub const TRAY_PAD: f32 = SPACE_1 + 2.0;

#[derive(Debug, Clone, Copy)]
pub struct DrawingChrome {
    pub controls_at_top: bool,
    pub tray_width: f32,
    pub tray_height: f32,
    pub zoom_width: f32,
    pub snap_width: f32,
    pub control_height: f32,
    pub touch_footer: bool,
}

impl DrawingChrome {
    pub fn for_app(app: &App) -> Self {
        Self::new(
            canvas_rect(app).width,
            SizeClass::of(app.window_size.width),
            app.ui.density,
            super::tray::TrayTool::ALL.len(),
        )
    }

    pub fn new(canvas_width: f32, class: SizeClass, density: Density, tool_count: usize) -> Self {
        let available = (canvas_width - CHROME_INSET * 2.0).max(1.0);
        let (tile_w, tile_h) = density.tray_button();
        let natural = tool_count as f32 * tile_w
            + tool_count.saturating_sub(1) as f32 * TRAY_GAP + TRAY_PAD * 2.0;
        let controls_at_top = class != SizeClass::Wide || density == Density::Touch
            || available < natural + 2.0 * (CONTROL_LANE_W + CHROME_GAP);
        Self {
            controls_at_top,
            tray_width: natural.min(available),
            tray_height: tile_h + TRAY_PAD * 2.0,
            // Leave a separate lane for North on the first top row.
            zoom_width: CONTROL_LANE_W.min((available - 76.0 - CHROME_GAP).max(1.0)),
            snap_width: CONTROL_LANE_W.min(available),
            control_height: if density == Density::Touch { 38.0 } else { 32.0 },
            touch_footer: density == Density::Touch && class != SizeClass::Compact,
        }
    }

    pub fn tray_clearance(self) -> f32 {
        CHROME_INSET + self.tray_height + CHROME_GAP
            + if self.touch_footer { self.control_height + SPACE_2 } else { 0.0 }
    }

    pub fn top_popover_offset(self, row: usize) -> f32 {
        CHROME_INSET + row as f32 * (self.control_height + SPACE_1) + SPACE_2
    }
}

/// Layout bucket selected on every window resize from logical window width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    /// `>= 1100px`: desktop and landscape tablet.
    Wide,
    /// `700..1099px`: portrait tablet and narrow desktop window.
    Medium,
    /// `< 700px`: phone.
    Compact,
}

impl SizeClass {
    pub fn of(width: f32) -> Self {
        if width >= WIDE_MIN {
            SizeClass::Wide
        } else if width >= MEDIUM_MIN {
            SizeClass::Medium
        } else {
            SizeClass::Compact
        }
    }
}

/// Logical window rectangle occupied by the drawing canvas.
pub fn canvas_rect(app: &App) -> Rectangle {
    canvas_rect_next(
        app.window_size.width,
        app.window_size.height,
        app.ui.density,
        if app.symbol_edit_session.is_some() {
            0.0
        } else {
            app.ui.effective_ledger_width()
        },
        super::inspector::is_available(app),
        (app.ui.compact_bottom_open() && app.symbol_edit_session.is_none())
            || app.touch_panel_open,
    )
}

/// Pure geometric helper for Next canvas bounds (also unit-tested directly).
pub fn canvas_rect_next(
    window_w: f32,
    window_h: f32,
    density: tokens::Density,
    ledger_w: f32,
    inspector_visible: bool,
    compact_sheet_open: bool,
) -> Rectangle {
    let bar_h = density.bar_h();
    match SizeClass::of(window_w) {
        SizeClass::Wide => {
            let x = ledger_w;
            let y = bar_h;
            let right = if inspector_visible {
                density.inspector_w()
            } else {
                0.0
            };
            Rectangle::new(
                iced::Point::new(x, y),
                iced::Size::new((window_w - x - right).max(0.0), (window_h - y).max(0.0)),
            )
        }
        SizeClass::Medium => {
            let y = bar_h + PAGE_BAR_H;
            Rectangle::new(
                iced::Point::new(0.0, y),
                iced::Size::new(window_w.max(0.0), (window_h - y).max(0.0)),
            )
        }
        SizeClass::Compact => {
            let top = bar_h + PAGE_BAR_H;
            let bottom = bar_h + if compact_sheet_open { COMPACT_SHEET_H } else { 0.0 };
            Rectangle::new(
                iced::Point::new(0.0, top),
                iced::Size::new(window_w.max(0.0), (window_h - top - bottom).max(0.0)),
            )
        }
    }
}

/// Shared centre stack layering the canvas and floating drawing overlays (controls,
/// length strip, tool tray, Pad, insert drawer) without shrinking canvas coordinates.
pub fn drawing_stack<'a>(
    canvas: Element<'a, Message>,
    overlays: impl IntoIterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    Stack::with_children(std::iter::once(canvas).chain(overlays))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Wide layout (`>= 1100px`): single-row top bar over docked ledger + drawing + optional inspector.
pub fn wide_layout<'a>(
    topbar: Element<'a, Message>,
    ledger: Element<'a, Message>,
    ledger_resize: Element<'a, Message>,
    drawing: Element<'a, Message>,
    inspector: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut middle = row![ledger, drawing]
        .width(Length::Fill)
        .height(Length::Fill);
    if let Some(insp) = inspector {
        middle = middle.push(insp);
    }
    let workspace = Stack::with_children([middle.into(), ledger_resize])
        .width(Length::Fill)
        .height(Length::Fill);
    column![topbar, workspace]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Medium layout (`700..1099px`): two-row header + full-width drawing with left/right slide-overs.
pub fn medium_layout<'a>(
    topbar: Element<'a, Message>,
    pagebar: Element<'a, Message>,
    drawing: Element<'a, Message>,
    ledger_slide: Option<Element<'a, Message>>,
    inspector_slide: Option<Element<'a, Message>>,
    tray_clearance: f32,
) -> Element<'a, Message> {
    let has_slide = ledger_slide.is_some() || inspector_slide.is_some();
    let mut layers: Vec<Element<'a, Message>> = vec![drawing];

    if has_slide {
        let scrim = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.25))),
                    ..container::Style::default()
                }),
        )
        .on_press(Message::Ui(Msg::CloseOverlays));
        layers.push(scrim.into());
    }

    if ledger_slide.is_some() || inspector_slide.is_some() {
        let mut slide_row = row![]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Start);
        if let Some(left) = ledger_slide {
            slide_row = slide_row.push(
                container(left)
                    .width(Length::Fixed(SLIDE_OVER_W))
                    .height(Length::Fill),
            );
        }
        slide_row = slide_row.push(Space::new().width(Length::Fill));
        if let Some(right) = inspector_slide {
            slide_row = slide_row.push(
                container(right)
                    .width(Length::Fixed(SLIDE_OVER_W))
                    .height(Length::Fill),
            );
        }
        // Leave bottom clearance so slide-overs never cover the floating tool tray.
        layers.push(
            container(slide_row)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding {
                    top: SPACE_2,
                    right: SPACE_2,
                    bottom: tray_clearance,
                    left: SPACE_2,
                })
                .into(),
        );
    }

    column![
        topbar,
        pagebar,
        Stack::with_children(layers)
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Compact layout (`< 700px`): two-row header, drawing, fixed bottom toolbar, optional bottom sheet.
pub fn compact_layout<'a>(
    topbar: Element<'a, Message>,
    pagebar: Element<'a, Message>,
    drawing: Element<'a, Message>,
    toolbar: Element<'a, Message>,
    sheet_or_pad: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut col = column![topbar, pagebar, drawing, toolbar]
        .width(Length::Fill)
        .height(Length::Fill);
    if let Some(sheet) = sheet_or_pad {
        let framed = container(sheet)
            .width(Length::Fill)
            .height(Length::Fixed(COMPACT_SHEET_H));
        col = col.push(framed);
    }
    col.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::tokens::Density;

    #[test]
    fn drawing_chrome_requires_room_for_both_side_lanes_and_all_tools() {
        let d = Density::Comfortable;
        let tools = super::super::tray::TrayTool::ALL.len();
        let natural = tools as f32 * d.tray_button().0
            + (tools - 1) as f32 * TRAY_GAP + TRAY_PAD * 2.0;
        let threshold = natural + 2.0 * (CONTROL_LANE_W + CHROME_GAP + CHROME_INSET);
        assert!(DrawingChrome::new(threshold - 1.0, SizeClass::Wide, d, tools).controls_at_top);
        assert!(!DrawingChrome::new(threshold, SizeClass::Wide, d, tools).controls_at_top);
        // Screenshot regression: 2048 physical pixels at 125% scaling with
        // a 280px ledger leaves 1358 logical pixels, too narrow for 12 tools.
        assert!(DrawingChrome::new(2048.0 / 1.25 - 280.0, SizeClass::Wide, d, tools).controls_at_top);
        assert!(DrawingChrome::new(2400.0, SizeClass::Wide, Density::Touch, tools).controls_at_top);
        let touch = DrawingChrome::new(980.0, SizeClass::Wide, Density::Touch, tools);
        assert!(touch.tray_clearance() >= CHROME_INSET + touch.tray_height
            + touch.control_height + SPACE_2 + CHROME_GAP);
        // A future additional tool must also change the fit decision.
        assert!(DrawingChrome::new(threshold, SizeClass::Wide, d, tools + 1).controls_at_top);
    }

    #[test]
    fn size_class_switches_at_exact_thresholds() {
        assert_eq!(SizeClass::of(372.0), SizeClass::Compact);
        assert_eq!(SizeClass::of(699.0), SizeClass::Compact);
        assert_eq!(SizeClass::of(700.0), SizeClass::Medium);
        assert_eq!(SizeClass::of(800.0), SizeClass::Medium);
        assert_eq!(SizeClass::of(1099.0), SizeClass::Medium);
        assert_eq!(SizeClass::of(1100.0), SizeClass::Wide);
        assert_eq!(SizeClass::of(1280.0), SizeClass::Wide);
        assert_eq!(SizeClass::of(1400.0), SizeClass::Wide);
    }

    #[test]
    fn canvas_rect_matches_all_size_classes() {
        let r_wide = canvas_rect_next(
            1400.0,
            860.0,
            Density::Comfortable,
            Density::Comfortable.ledger_w(),
            false,
            false,
        );
        assert_eq!(r_wide, Rectangle::new(iced::Point::new(264.0, 44.0), iced::Size::new(1136.0, 816.0)));

        let r_wide_resized = canvas_rect_next(1400.0, 860.0, Density::Comfortable, 340.0, false, false);
        assert_eq!(r_wide_resized, Rectangle::new(iced::Point::new(340.0, 44.0), iced::Size::new(1060.0, 816.0)));

        let r_wide_insp = canvas_rect_next(
            1400.0,
            860.0,
            Density::Comfortable,
            Density::Comfortable.ledger_w(),
            true,
            false,
        );
        assert_eq!(r_wide_insp, Rectangle::new(iced::Point::new(264.0, 44.0), iced::Size::new(856.0, 816.0)));

        let r_wide_touch = canvas_rect_next(
            1280.0,
            800.0,
            Density::Touch,
            Density::Touch.ledger_w(),
            false,
            false,
        );
        assert_eq!(r_wide_touch, Rectangle::new(iced::Point::new(300.0, 56.0), iced::Size::new(980.0, 744.0)));

        let r_med = canvas_rect_next(800.0, 1280.0, Density::Touch, 300.0, true, false);
        assert_eq!(r_med, Rectangle::new(iced::Point::new(0.0, 100.0), iced::Size::new(800.0, 1180.0)));

        let r_cmp = canvas_rect_next(372.0, 800.0, Density::Touch, 300.0, false, false);
        assert_eq!(r_cmp, Rectangle::new(iced::Point::new(0.0, 100.0), iced::Size::new(372.0, 644.0)));

        let r_cmp_sheet = canvas_rect_next(372.0, 800.0, Density::Touch, 300.0, false, true);
        assert_eq!(r_cmp_sheet, Rectangle::new(iced::Point::new(0.0, 100.0), iced::Size::new(372.0, 284.0)));
    }
}
