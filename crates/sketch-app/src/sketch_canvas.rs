//! The drawing canvas (`#sketchCanvas`, CanvasEngine). Screen 5 grows this
//! into the full renderer; for now it paints what the default state shows:
//! the light-theme background and the grid (CanvasEngine.drawGrid).
//!
//! `state.pan` starts at the canvas center measured at startup
//! (SketchApp constructor: `pan = {x: viewWidth / 2, y: viewHeight / 2}`)
//! and is NOT recentered when the canvas later resizes (dock collapse or
//! resize) — verified against the original.

use iced::widget::canvas::{self, Frame, Geometry, LineDash, Path, Stroke};
use iced::{mouse, Color, Event, Point, Rectangle, Renderer, Size, Theme, Vector};
use sketch_core::geometry::PIXELS_PER_FOOT;

use crate::view_state::Mode;

/// CanvasEngine light theme `fenceFill` / `fenceStroke`.
const FENCE_FILL: Color = Color::from_rgba(99.0 / 255.0, 179.0 / 255.0, 237.0 / 255.0, 0.1);
const FENCE_STROKE: Color = crate::theme::ICON_BLUE;

#[cfg(test)]
thread_local! { static TEST_UNCACHED: std::cell::Cell<(bool, usize)> = const { std::cell::Cell::new((false, 0)) }; }

#[cfg(test)]
pub(super) fn without_cache<R>(run: impl FnOnce() -> R) -> (R, usize) {
    struct Restore((bool, usize));
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_UNCACHED.with(|s| s.set(self.0));
        }
    }
    let _restore = Restore(TEST_UNCACHED.with(|s| s.replace((true, 0))));
    let result = run();
    (result, TEST_UNCACHED.with(|s| s.get().1))
}

/// What the canvas needs from app state.
pub struct SketchCanvas<'a> {
    pub pointer: Option<&'a std::cell::Cell<Option<(f32, f32)>>>,
    pub cache: Option<&'a canvas::Cache>,
    /// `state.pan` (screen px of world origin).
    pub pan: Vector,
    pub zoom: f32,
    /// `state.gridEnabled` (default true; the status-bar Grid toggle).
    pub grid: bool,
    /// `state.mode`.
    pub mode: Mode,
    /// The fence being dragged, as canvas-px corners.
    pub fence: Option<(Point, Point)>,
    /// Whether a selection/endpoint drag is open (`state.dragState`).
    pub dragging: bool,
    /// `state.isPanning`.
    pub panning: bool,
    /// `state.spacePan` (Space held: temporary hand tool).
    pub space_pan: bool,
    /// A symbol is being dragged from the dock over the canvas
    /// (`state.dragSymbol`): the placement preview follows the pointer.
    pub symbol_drag: bool,
    pub symbol_preview: Option<&'a str>,
    pub text_preview: Option<&'a str>,
    /// An area code is being dragged over the canvas: the region a drop would
    /// define (its ring in world space) and the code's colour.
    pub drop_hint: Option<(&'a [sketch_core::geometry::Pt], Color)>,
    pub workflow: Option<&'a crate::area_workflow::State>,
    /// Walls, the live chain and the drawing aids.
    pub scene: crate::render::Scene<'a>,
}

/// Per-canvas widget state: the last left press, for double-click
/// detection (the browser's `dblclick`), and whether a left press began on
/// the canvas (a `click` needs press and release on it).
#[derive(Default)]
pub struct CanvasState {
    last_click: Option<iced::advanced::mouse::Click>,
    pressed: Option<bool>,
    /// The last reported modifiers: a wheel event carries none, but
    /// Shift+wheel bows the last wall and Ctrl changes its step.
    mods: iced::keyboard::Modifiers,
    last_hover: Option<std::time::Instant>,
    pending_hover: Option<(f32, f32)>,
    /// Fingers currently on the canvas (a touchscreen), for the pinch.
    fingers: Vec<(iced::touch::Finger, Point)>,
    /// Wheel travel not yet turned into a zoom step, in notches: a
    /// precision touchpad reports many small fractions of one.
    wheel_acc: f32,
    wheel_at: Option<std::time::Instant>,
}

/// Centroid and spread of a two-finger touch.
fn pinch_frame(fingers: &[(iced::touch::Finger, Point)]) -> Option<(Point, f32)> {
    let [(_, a), (_, b)] = fingers else {
        return None;
    };
    Some((
        Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0),
        a.distance(*b),
    ))
}

/// CanvasEngine.drawGrid: minor lines every foot, major every 10 ft (when
/// wider than 20px), offset by `pan % spacing`, 1px wide.
pub fn draw_grid(frame: &mut Frame, pan: Vector, zoom: f32, w: f32, h: f32) {
    let px_per_ft = PIXELS_PER_FOOT as f32 * zoom;
    if px_per_ft <= 0.0 {
        return;
    }
    let lines = |step: f32| {
        Path::new(|p| {
            let mut x = pan.x % step;
            while x < w {
                p.move_to(Point::new(x, 0.0));
                p.line_to(Point::new(x, h));
                x += step;
            }
            let mut y = pan.y % step;
            while y < h {
                p.move_to(Point::new(0.0, y));
                p.line_to(Point::new(w, y));
                y += step;
            }
        })
    };
    frame.stroke(
        &lines(px_per_ft),
        Stroke::default()
            .with_color(crate::canvas_theme::current().grid10())
            .with_width(1.0),
    );
    let major = px_per_ft * 10.0;
    if major > 20.0 {
        frame.stroke(
            &lines(major),
            Stroke::default()
                .with_color(crate::canvas_theme::current().grid50())
                .with_width(1.0),
        );
    }
}

impl canvas::Program<crate::Message> for SketchCanvas<'_> {
    type State = CanvasState;

    /// Canvas pointer handling (CanvasEngine.setupEvents), in its
    /// pointerdown precedence: Space + left pans in any mode; then the middle
    /// button, or left in the Pan tool, pans; then the zoom fence. Moves and
    /// the release are reported outside the canvas too while a drag is open
    /// (setPointerCapture). Pans travel in window px.
    fn update(
        &self,
        state: &mut CanvasState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<crate::Message>> {
        use crate::Message;
        let rel = |p: Point| (p.x - bounds.x, p.y - bounds.y);
        if let Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m)) = event {
            state.mods = *m;
            return None;
        }
        if let Some(pointer) = self.pointer {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    pointer.set(cursor.position_over(bounds).map(rel));
                }
                Event::Mouse(mouse::Event::CursorLeft) => {
                    pointer.set(None);
                    state.pending_hover = None;
                }
                Event::Mouse(mouse::Event::ButtonPressed(_) | mouse::Event::ButtonReleased(_)) => {
                    pointer.set(cursor.position_over(bounds).map(rel));
                    state.pending_hover = None;
                }
                _ => {}
            }
        }
        if let Event::Touch(touch) = event {
            use iced::touch::Event as T;
            return match *touch {
                T::FingerPressed { id, position } => {
                    if bounds.contains(position) && !state.fingers.iter().any(|(f, _)| *f == id) {
                        state.fingers.push((id, position));
                    }
                    if state.fingers.len() >= 2 {
                        // A second finger makes it a gesture, not a click.
                        state.pressed = None;
                    }
                    None
                }
                T::FingerMoved { id, position } => {
                    let before = pinch_frame(&state.fingers);
                    let finger = state.fingers.iter_mut().find(|(f, _)| *f == id)?;
                    finger.1 = position;
                    let (old_c, old_d) = before?;
                    let (new_c, new_d) = pinch_frame(&state.fingers)?;
                    // Fingers nearly touching give a wild ratio: pan only.
                    let factor = if old_d > 8.0 && new_d > 8.0 {
                        new_d / old_d
                    } else {
                        1.0
                    };
                    let (x, y) = rel(new_c);
                    Some(
                        canvas::Action::publish(Message::Pinch {
                            factor,
                            x,
                            y,
                            dx: new_c.x - old_c.x,
                            dy: new_c.y - old_c.y,
                        })
                        .and_capture(),
                    )
                }
                T::FingerLifted { id, .. } | T::FingerLost { id, .. } => {
                    state.fingers.retain(|(f, _)| *f != id);
                    None
                }
            };
        }
        match event {
            Event::Window(iced::window::Event::RedrawRequested(now)) => {
                let (x, y) = state.pending_hover?;
                if !self.mode.is_drawing() || self.panning || self.dragging {
                    state.pending_hover = None;
                    return None;
                }
                let deadline =
                    state.last_hover.unwrap_or(*now) + std::time::Duration::from_millis(16);
                if *now < deadline {
                    return Some(canvas::Action::request_redraw_at(deadline));
                }
                state.pending_hover = None;
                state.last_hover = Some(*now);
                Some(canvas::Action::publish(Message::CanvasHover(x, y)))
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) => {
                let p = cursor.position_over(bounds)?;
                let pans = (*button == mouse::Button::Left
                    && (self.space_pan || self.mode == Mode::Pan))
                    || *button == mouse::Button::Middle;
                if pans {
                    return Some(
                        canvas::Action::publish(Message::PanStart(p.x, p.y)).and_capture(),
                    );
                }
                if *button == mouse::Button::Left
                    && matches!(self.mode, Mode::ZoomFence | Mode::Fence | Mode::Lasso)
                {
                    let (x, y) = rel(p);
                    return Some(canvas::Action::publish(Message::FenceStart(x, y)).and_capture());
                }
                if *button == mouse::Button::Left && matches!(self.mode, Mode::Select | Mode::Curve)
                {
                    // The pointer doubles as the marquee tool when the drag
                    // starts on empty canvas; main hit-tests and decides. The
                    // click is still recorded, because a press that does not
                    // move must still select on release.
                    let click =
                        iced::advanced::mouse::Click::new(p, mouse::Button::Left, state.last_click);
                    state.pressed =
                        Some(click.kind() == iced::advanced::mouse::click::Kind::Double);
                    state.last_click = Some(click);
                    let (x, y) = rel(p);
                    return Some(canvas::Action::publish(Message::CanvasPress(x, y)).and_capture());
                }
                if *button == mouse::Button::Right {
                    // `contextmenu`: lifts the pen first.
                    return Some(canvas::Action::publish(Message::CanvasContext).and_capture());
                }
                if *button == mouse::Button::Left {
                    let click =
                        iced::advanced::mouse::Click::new(p, mouse::Button::Left, state.last_click);
                    state.pressed =
                        Some(click.kind() == iced::advanced::mouse::click::Kind::Double);
                    state.last_click = Some(click);
                    return Some(canvas::Action::capture());
                }
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if self.panning {
                    Some(canvas::Action::publish(Message::PanMove(
                        position.x, position.y,
                    )))
                } else if self.dragging {
                    let (x, y) = rel(*position);
                    Some(canvas::Action::publish(Message::CanvasDrag(
                        x,
                        y,
                        state.mods.control(),
                    )))
                } else if self.fence.is_some() {
                    let (x, y) = rel(*position);
                    Some(canvas::Action::publish(Message::FenceMove(x, y)))
                } else if self.mode.is_drawing() || self.mode == Mode::DefineArea {
                    // `pointerWorld` + HUD; the painted pointer follows.
                    let p = cursor.position_over(bounds)?;
                    let (x, y) = rel(p);
                    if self.pointer.is_some()
                        && self.mode.is_drawing()
                        && !self.scene.drawing.keyboard_pointer
                    {
                        let now = std::time::Instant::now();
                        if state.last_hover.is_some_and(|last| {
                            now.duration_since(last) < std::time::Duration::from_millis(16)
                        }) {
                            state.pending_hover = Some((x, y));
                            return Some(canvas::Action::request_redraw());
                        }
                        state.last_hover = Some(now);
                        state.pending_hover = None;
                    }
                    Some(canvas::Action::publish(Message::CanvasHover(x, y)))
                } else {
                    None
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(_)) if self.panning => {
                Some(canvas::Action::publish(Message::PanEnd))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if self.dragging => {
                Some(canvas::Action::publish(Message::CanvasDragEnd))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if self.fence.is_some() =>
            {
                Some(canvas::Action::publish(Message::FenceEnd))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let double = state.pressed.take()?;
                let p = cursor.position_over(bounds)?;
                let (x, y) = rel(p);
                Some(canvas::Action::publish(Message::CanvasClick {
                    x,
                    y,
                    double,
                    ctrl: state.mods.control(),
                }))
            }
            Event::Mouse(mouse::Event::CursorLeft) => Some(canvas::Action::publish(
                Message::AreaWorkflow(crate::area_workflow::Msg::Leave),
            )),
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let p = cursor.position_over(bounds)?;
                // In notches: a wheel click is one line; 40px matches it for
                // pixel-precise devices.
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 40.0,
                };
                if dy == 0.0 {
                    return None;
                }
                // A touchpad scroll or pinch arrives as a stream of small
                // fractions; one zoom step per fraction would fling the
                // view. Bank them and step once per whole notch.
                let now = std::time::Instant::now();
                let stale = state
                    .wheel_at
                    .is_none_or(|at| now.duration_since(at) > std::time::Duration::from_millis(400));
                if stale || state.wheel_acc * dy < 0.0 {
                    state.wheel_acc = 0.0;
                }
                state.wheel_at = Some(now);
                state.wheel_acc += dy;
                if state.wheel_acc.abs() < 1.0 {
                    return Some(canvas::Action::capture());
                }
                let up = state.wheel_acc > 0.0;
                state.wheel_acc = 0.0;
                let (x, y) = rel(p);
                // iced: y > 0 scrolls up (the browser's deltaY < 0).
                Some(
                    canvas::Action::publish(Message::Wheel {
                        up,
                        x,
                        y,
                        shift: state.mods.shift(),
                        ctrl: state.mods.control(),
                    })
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    /// CanvasEngine.setCursor: a drag wins (grabbing), then Space / the Pan
    /// tool (grab), then the mode: draw modes hide the OS cursor for the
    /// painted pointer, the fence-like tools show a crosshair.
    fn mouse_interaction(
        &self,
        _state: &CanvasState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.panning {
            return mouse::Interaction::Grabbing;
        }
        if !cursor.is_over(bounds) {
            return mouse::Interaction::default();
        }
        if self.space_pan || self.mode == Mode::Pan {
            mouse::Interaction::Grab
        } else if self.mode.is_drawing() || self.mode == Mode::DefineArea {
            mouse::Interaction::Hidden
        } else if self.mode.crosshair() {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &CanvasState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut scene = self.scene.clone();
        if self.mode.is_drawing() && !scene.drawing.keyboard_pointer {
            if let Some(p) = cursor.position_in(bounds) {
                let k = PIXELS_PER_FOOT * scene.zoom;
                scene.live_pointer = Some(sketch_core::geometry::Pt::new(
                    (f64::from(p.x) - scene.pan.0) / k,
                    (scene.pan.1 - f64::from(p.y)) / k,
                ));
            }
        }
        let mut frame = Frame::new(renderer, bounds.size());
        let background = |frame: &mut Frame| {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                crate::canvas_theme::current().bg(),
            );
            if self.grid {
                draw_grid(frame, self.pan, self.zoom, bounds.width, bounds.height);
            }
        };
        #[cfg(not(test))]
        let cache = self.cache;
        #[cfg(test)]
        let cache = TEST_UNCACHED.with(|s| {
            let (disabled, draws) = s.get();
            if disabled {
                s.set((disabled, draws + 1));
                None
            } else {
                self.cache
            }
        });
        let base = if let Some(cache) = cache {
            let geometry = cache.draw(renderer, bounds.size(), |frame| {
                background(frame);
                crate::render::draw_scene_base(frame, &self.scene);
            });
            crate::render::draw_scene_overlay(&mut frame, &scene, bounds.width, bounds.height);
            Some(geometry)
        } else {
            background(&mut frame);
            crate::render::draw_scene(&mut frame, &scene, bounds.width, bounds.height);
            None
        };
        if let Some(workflow) = self.workflow {
            crate::area_workflow::draw_overlay(&mut frame, &self.scene, workflow, self.mode);
        }
        if let Some((ring, color)) = self.drop_hint {
            crate::render::draw_drop_hint(&mut frame, &self.scene, ring, color);
        }
        if self.mode.is_placing_symbol() || self.symbol_drag {
            if let (Some(p), Some(symbol)) = (cursor.position_in(bounds), self.symbol_preview) {
                let k = PIXELS_PER_FOOT * self.scene.zoom;
                let world = sketch_core::geometry::Pt::new(
                    (f64::from(p.x) - self.scene.pan.0) / k,
                    (self.scene.pan.1 - f64::from(p.y)) / k,
                );
                crate::render::draw_symbol_placement_preview(
                    &mut frame,
                    &self.scene,
                    symbol,
                    world,
                );
            }
        }
        if let (Some(p), Some(text)) = (cursor.position_in(bounds), self.text_preview) {
            frame.fill_text(iced::widget::canvas::Text {
                content: text.to_owned(),
                position: p,
                color: crate::theme::hex(0x63B3ED),
                size: (12.0 * self.zoom).max(9.0).into(),
                font: crate::theme::ARIAL,
                align_x: iced::widget::text::Alignment::Center,
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }
        if let Some((a, b)) = self.fence {
            // drawFenceRect: fill, then a 1px [4, 4] dashed stroke.
            let origin = Point::new(a.x.min(b.x), a.y.min(b.y));
            let size = Size::new((b.x - a.x).abs(), (b.y - a.y).abs());
            frame.fill_rectangle(origin, size, FENCE_FILL);
            frame.stroke(
                &Path::rectangle(origin, size),
                Stroke {
                    line_dash: LineDash {
                        segments: &[4.0, 4.0],
                        offset: 0,
                    },
                    ..Stroke::default().with_color(FENCE_STROKE).with_width(1.0)
                },
            );
        }
        // The mouse pointer is painted only when the arrow keys are not
        // driving the cursor (the renderer paints that one).
        if let Some(p) = self.mouse_pointer_position(bounds, cursor) {
            crate::render::draw_drawing_pointer(&mut frame, p, &self.scene.drawing.options);
        }
        base.into_iter()
            .chain(std::iter::once(frame.into_geometry()))
            .collect()
    }
}

impl SketchCanvas<'_> {
    fn mouse_pointer_position(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Point> {
        if !(self.mode.is_drawing() || self.mode == Mode::DefineArea)
            || self.space_pan
            || self.panning
        {
            return None;
        }
        let d = self.scene.drawing;
        // Defining/labeling an area hides the OS cursor too. A keyboard cursor
        // only replaces the mouse while a live wall preview actually paints it.
        let keyboard_visible = self.mode.is_drawing()
            && d.keyboard_pointer
            && d.pen_down
            && self.scene.ctx.kind.is_some()
            && !d.points.is_empty();
        if keyboard_visible {
            None
        } else {
            cursor.position_in(bounds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;
    use iced::widget::canvas::Program;

    fn canvas(mode: Mode) -> SketchCanvas<'static> {
        static EMPTY_SELECTION: std::sync::LazyLock<crate::selection::Selection> =
            std::sync::LazyLock::new(crate::selection::Selection::default);
        static DRAWING: std::sync::LazyLock<crate::drawing::Drawing> =
            std::sync::LazyLock::new(crate::drawing::Drawing::default);
        static EMPTY_GATES: std::sync::LazyLock<crate::layers::LayerGates> =
            std::sync::LazyLock::new(crate::layers::LayerGates::default);
        static EMPTY_COMPUTED: std::sync::LazyLock<
            std::collections::BTreeMap<i64, crate::calcs::Computed>,
        > = std::sync::LazyLock::new(Default::default);
        static EMPTY_PHOTO: std::sync::LazyLock<crate::reference_image::BgImage> =
            std::sync::LazyLock::new(crate::reference_image::BgImage::default);
        SketchCanvas {
            pointer: None,
            cache: None,
            symbol_drag: false,
            symbol_preview: None,
            text_preview: None,
            drop_hint: None,
            workflow: None,
            dragging: false,
            scene: crate::render::Scene {
                live_pointer: None,
                area_codes: &[],
                computed: &EMPTY_COMPUTED,
                drawing: &DRAWING,
                selection: &EMPTY_SELECTION,
                gates: &EMPTY_GATES,
                photo: &EMPTY_PHOTO,
                ctx: crate::drawing::DrawCtx {
                    snap: true,
                    grid: 1.0,
                    zoom: 1.0,
                    kind: None,
                    step_unit: crate::drawing::StepUnit::default(),
                },
                pan: (523.5, 351.5),
                zoom: 1.0,
                filters: [true; 6],
                dimensions: true,
                placement: crate::view_state::DimensionPlacement::Out,
                symbols: &[],
                symbol_lib: None,
                texts: &[],
                measure_points: &[],
                measure_runs: &[],
                underlay: None,
                curve_tool: false,
                curve_drag: None,
                select_mode: false,
                drag: None,
                drag_moved: false,
                text_zoom_floor: 0.0,
            },
            pan: Vector::new(523.5, 351.5),
            zoom: 1.0,
            grid: true,
            mode,
            fence: None,
            panning: false,
            space_pan: false,
        }
    }

    #[test]
    fn rapid_mouse_moves_redraw_locally_then_flush_the_latest_hud_position() {
        use canvas::Program;
        let pointer = std::cell::Cell::new(None);
        let mut c = canvas(Mode::DrawExterior);
        c.pointer = Some(&pointer);
        let now = std::time::Instant::now();
        let mut state = CanvasState {
            last_hover: Some(now),
            ..Default::default()
        };
        for x in 300..400 {
            let cursor = at(x as f32, 400.);
            let event = Event::Mouse(mouse::Event::CursorMoved {
                position: cursor.position().unwrap(),
            });
            let action = c.update(&mut state, &event, BOUNDS, cursor).unwrap();
            assert!(
                action.into_inner().0.is_none(),
                "coalesced motion must not rebuild the whole app"
            );
        }
        assert_eq!(pointer.get(), Some((358., 271.)));
        let redraw = Event::Window(iced::window::Event::RedrawRequested(
            now + std::time::Duration::from_millis(17),
        ));
        let action = c
            .update(&mut state, &redraw, BOUNDS, at(399., 400.))
            .unwrap();
        assert!(matches!(
            action.into_inner().0,
            Some(Message::CanvasHover(358., 271.))
        ));
        assert!(
            state.pending_hover.is_none(),
            "the last position flushes even if the mouse stops"
        );
    }

    #[test]
    fn click_and_cursor_leave_clear_coalesced_motion() {
        use canvas::Program;
        let pointer = std::cell::Cell::new(None);
        let mut c = canvas(Mode::DrawExterior);
        c.pointer = Some(&pointer);
        let mut state = CanvasState {
            pending_hover: Some((1., 2.)),
            ..Default::default()
        };
        let _ = c.update(
            &mut state,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            BOUNDS,
            at(400., 400.),
        );
        assert!(state.pending_hover.is_none());
        let released = c
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                BOUNDS,
                at(450., 410.),
            )
            .unwrap();
        assert!(matches!(
            released.into_inner().0,
            Some(Message::CanvasClick {
                x: 409.,
                y: 281.,
                ..
            })
        ));
        assert_eq!(pointer.get(), Some((409., 281.)));
        let _ = c.update(
            &mut state,
            &Event::Mouse(mouse::Event::CursorLeft),
            BOUNDS,
            mouse::Cursor::Unavailable,
        );
        assert_eq!(pointer.get(), None);
    }

    const BOUNDS: Rectangle = Rectangle {
        x: 41.0,
        y: 129.0,
        width: 1047.0,
        height: 703.0,
    };

    fn at(x: f32, y: f32) -> mouse::Cursor {
        mouse::Cursor::Available(Point::new(x, y))
    }

    fn message(c: &SketchCanvas, event: Event, cursor: mouse::Cursor) -> Option<Message> {
        let action = c.update(&mut CanvasState::default(), &event, BOUNDS, cursor)?;
        action.into_inner().0
    }

    #[test]
    fn small_wheel_fractions_bank_into_whole_zoom_steps() {
        // A precision touchpad sends fractions of a notch: three quarters
        // must not zoom, the next quarter completes one step.
        let c = canvas(Mode::Select);
        let mut state = CanvasState::default();
        let cursor = at(BOUNDS.x + 10.0, BOUNDS.y + 10.0);
        let scroll = |y: f32| {
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y },
            })
        };
        for _ in 0..3 {
            let action = c.update(&mut state, &scroll(0.25), BOUNDS, cursor).unwrap();
            assert!(action.into_inner().0.is_none());
        }
        let action = c.update(&mut state, &scroll(0.25), BOUNDS, cursor).unwrap();
        assert!(matches!(
            action.into_inner().0,
            Some(crate::Message::Wheel { up: true, .. })
        ));
        // Reversing direction drops what was banked the other way.
        let _ = c.update(&mut state, &scroll(0.75), BOUNDS, cursor);
        let action = c.update(&mut state, &scroll(-0.5), BOUNDS, cursor).unwrap();
        assert!(action.into_inner().0.is_none());
    }

    #[test]
    fn two_fingers_pinch_and_one_finger_does_not() {
        use iced::touch::{Event as T, Finger};
        let c = canvas(Mode::Select);
        let mut state = CanvasState::default();
        let cursor = mouse::Cursor::Unavailable;
        let pt = |x: f32, y: f32| Point::new(BOUNDS.x + x, BOUNDS.y + y);
        let press = |id, p| Event::Touch(T::FingerPressed { id: Finger(id), position: p });
        let moved = |id, p| Event::Touch(T::FingerMoved { id: Finger(id), position: p });
        assert!(c.update(&mut state, &press(1, pt(100.0, 100.0)), BOUNDS, cursor).is_none());
        assert!(
            c.update(&mut state, &moved(1, pt(110.0, 100.0)), BOUNDS, cursor).is_none(),
            "one finger is not a pinch"
        );
        assert!(c.update(&mut state, &press(2, pt(210.0, 100.0)), BOUNDS, cursor).is_none());
        // Fingers 100px apart; the second moves out to 200px apart.
        let action = c
            .update(&mut state, &moved(2, pt(310.0, 100.0)), BOUNDS, cursor)
            .unwrap();
        match action.into_inner().0 {
            Some(crate::Message::Pinch { factor, x, y, dx, dy }) => {
                assert_eq!((factor, x, y, dx, dy), (2.0, 210.0, 100.0, 50.0, 0.0));
            }
            other => panic!("expected a pinch, got {other:?}"),
        }
        // Lifting one ends it.
        let lift = Event::Touch(T::FingerLifted { id: Finger(1), position: pt(110.0, 100.0) });
        assert!(c.update(&mut state, &lift, BOUNDS, cursor).is_none());
        assert!(c.update(&mut state, &moved(2, pt(320.0, 100.0)), BOUNDS, cursor).is_none());
    }

    #[test]
    fn wheel_maps_to_zoom_about_the_pointer_in_canvas_px() {
        let c = canvas(Mode::Select);
        let up = Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        });
        match message(&c, up, at(341.0, 429.0)) {
            Some(Message::Wheel { up, x, y, .. }) => assert_eq!((up, x, y), (true, 300.0, 300.0)),
            other => panic!("{other:?}"),
        }
        let down = Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -40.0 },
        });
        assert!(matches!(
            message(&c, down, at(641.0, 529.0)),
            Some(Message::Wheel { up: false, .. })
        ));
        // Outside the canvas: nothing.
        let up = Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 },
        });
        assert!(message(&c, up, at(10.0, 10.0)).is_none());
    }

    #[test]
    fn pan_starts_follow_pointerdown_precedence() {
        let press = |b| Event::Mouse(mouse::Event::ButtonPressed(b));
        // Middle pans in any mode; left only in the Pan tool or with Space.
        for mode in [Mode::Select, Mode::DrawExterior, Mode::ZoomFence] {
            let c = canvas(mode);
            assert!(matches!(
                message(&c, press(mouse::Button::Middle), at(400.0, 400.0)),
                Some(Message::PanStart(400.0, 400.0))
            ));
        }
        // A Select-mode press publishes `CanvasPress`: main hit-tests it and
        // starts the marquee when it missed (screen 6h).
        // Canvas-relative, like `FenceStart` (BOUNDS starts at 41,129).
        assert!(matches!(
            message(
                &canvas(Mode::Select),
                press(mouse::Button::Left),
                at(400.0, 400.0)
            ),
            Some(Message::CanvasPress(359.0, 271.0))
        ));
        assert!(matches!(
            message(
                &canvas(Mode::Pan),
                press(mouse::Button::Left),
                at(400.0, 400.0)
            ),
            Some(Message::PanStart(..))
        ));
        let mut spaced = canvas(Mode::ZoomFence);
        spaced.space_pan = true;
        assert!(
            matches!(
                message(&spaced, press(mouse::Button::Left), at(400.0, 400.0)),
                Some(Message::PanStart(..))
            ),
            "Space + left pans before the zoom fence starts"
        );
        // While panning, moves and any release are reported (pointer capture).
        let mut panning = canvas(Mode::Select);
        panning.panning = true;
        let moved = Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(5.0, 5.0),
        });
        assert!(matches!(
            message(&panning, moved, at(5.0, 5.0)),
            Some(Message::PanMove(5.0, 5.0))
        ));
        let up = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle));
        assert!(matches!(
            message(&panning, up, at(5.0, 5.0)),
            Some(Message::PanEnd)
        ));
    }

    #[test]
    fn cursor_policy_matches_set_cursor() {
        let over = at(400.0, 400.0);
        let i = |c: &SketchCanvas| c.mouse_interaction(&CanvasState::default(), BOUNDS, over);
        assert_eq!(i(&canvas(Mode::Select)), mouse::Interaction::default());
        assert_eq!(i(&canvas(Mode::Pan)), mouse::Interaction::Grab);
        assert_eq!(i(&canvas(Mode::DrawInterior)), mouse::Interaction::Hidden);
        assert_eq!(i(&canvas(Mode::Measure)), mouse::Interaction::Crosshair);
        let mut c = canvas(Mode::DrawExterior);
        c.space_pan = true;
        assert_eq!(i(&c), mouse::Interaction::Grab, "Space wins over the mode");
        c.panning = true;
        assert_eq!(
            i(&c),
            mouse::Interaction::Grabbing,
            "a drag wins over everything"
        );
    }

    #[test]
    fn crosshair_remains_visible_after_area_labeling_and_keyboard_chain_close() {
        let over = at(400.0, 400.0);
        let expected = Some(Point::new(359.0, 271.0));
        let mut drawing = crate::drawing::Drawing::default();
        drawing.keyboard_pointer = true;
        // Area definition keeps the cursor hidden at OS level; it must paint
        // the supplied crosshair even when the prior chain used arrow keys.
        let mut c = canvas(Mode::DefineArea);
        c.scene.drawing = &drawing;
        assert_eq!(
            c.mouse_interaction(&CanvasState::default(), BOUNDS, over),
            mouse::Interaction::Hidden
        );
        assert_eq!(c.mouse_pointer_position(BOUNDS, over), expected);
        c.mode = Mode::DrawExterior;
        c.scene.ctx.kind = Some(sketch_core::model::WallType::Exterior);
        assert_eq!(
            c.mouse_pointer_position(BOUNDS, over),
            expected,
            "closed keyboard chain falls back to the mouse"
        );
        let mut active_drawing = drawing.clone();
        active_drawing.pen_down = true;
        active_drawing
            .points
            .push(sketch_core::geometry::Pt::new(0.0, 0.0));
        c.scene.drawing = &active_drawing;
        assert_eq!(
            c.mouse_pointer_position(BOUNDS, over),
            None,
            "live keyboard preview owns the crosshair"
        );
        c.mode = Mode::DefineArea;
        assert_eq!(c.mouse_pointer_position(BOUNDS, over), expected);
        c.space_pan = true;
        assert_eq!(c.mouse_pointer_position(BOUNDS, over), None);
        c.space_pan = false;
        assert_eq!(c.mouse_pointer_position(BOUNDS, at(2.0, 2.0)), None);
    }
}
