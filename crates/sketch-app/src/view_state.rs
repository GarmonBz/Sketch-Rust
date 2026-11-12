//! View state behind the status bar and the canvas: pan/zoom, the 1:N
//! drawing scale, Grid / Snap / Area-grid-snap + pitch, the interaction
//! mode (Select / Zoom Fence), the zoom fence rectangle, and the sketch
//! filters. Ported from refs/sketch-app.js (zoomTo, drawingScaleFactor,
//! setDrawingScale, zoomIn/Out, zoomToWorldRect, centerSketch,
//! centerAndFit, toggleGrid/Snap/AreaGridSnap, setAreaGridSnapSize, setMode)
//! and refs/ui-manager.js (updateZoomDisplay, toggleFilterPanel).

use sketch_core::geometry::PIXELS_PER_FOOT;

const PPF: f64 = PIXELS_PER_FOOT;

/// `state.mode`. Rail tools select most of these (`#tool-<mode>` carries
/// `.active`); the zoom fence has no rail tool. Placement modes arrive with
/// screens 8-9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Select,
    DefineArea,
    Fence,
    Lasso,
    Pan,
    DrawExterior,
    DrawInterior,
    InsertVertex,
    Curve,
    Measure,
    ChainDim,
    ZoomFence,
    PlaceSymbol,
    PlaceShape,
    PlaceText,
}

impl Mode {
    /// `draw-exterior` / `draw-interior`.
    pub fn is_drawing(self) -> bool {
        matches!(self, Mode::DrawExterior | Mode::DrawInterior)
    }

    /// CanvasEngine.setCursor: the modes that show a crosshair.
    pub fn crosshair(self) -> bool {
        matches!(
            self,
            Mode::DefineArea
                | Mode::Fence
                | Mode::Lasso
                | Mode::InsertVertex
                | Mode::ZoomFence
                | Mode::Curve
                | Mode::Measure
                | Mode::ChainDim
                | Mode::PlaceSymbol
                | Mode::PlaceShape
                | Mode::PlaceText
        )
    }

    pub fn is_placing_symbol(self) -> bool {
        matches!(self, Mode::PlaceSymbol)
    }
}

/// `state.curveMode` — how Shift+wheel / curve steps bow a wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveMode {
    Degree,
    Height,
}

/// `#area-snap-pitch` options `(label, value)`.
pub const PITCHES: [(&str, f64); 4] =
    [("1'", 1.0), ("1/2'", 0.5), ("1/4'", 0.25), ("1\"", 0.08333)];

/// `state.dimensionPlacement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DimensionPlacement {
    #[default]
    Out,
    In,
}

impl DimensionPlacement {
    /// `getPathDimensionSide` / `getLineDimensionSide`'s fallback: the label
    /// sits on the path's outside, or its inside when placement is 'in'.
    pub fn side(self) -> f32 {
        match self {
            DimensionPlacement::Out => 1.0,
            DimensionPlacement::In => -1.0,
        }
    }
}

/// `state.filters` (UIManager.toggleFilterPanel labels, in order).
pub const FILTERS: [(&str, &str); 6] = [
    ("areas", "Area fills"),
    ("exterior", "Exterior walls"),
    ("interior", "Interior lines"),
    ("dimensions", "Dimensions"),
    ("labels", "Text labels"),
    ("symbols", "Symbols"),
];

/// A world point (feet, y up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
    pub x: f64,
    pub y: f64,
}

/// A pointer drag in progress (`state.dragState`).
#[derive(Debug, Clone, PartialEq)]
pub enum Drag {
    /// `type: 'chain-offset'` — reposition a chain's offset line (S9-035).
    ChainOffset {
        chain_id: i64,
        /// The computed run's start + normal, captured at grab time.
        run_start: World,
        run_normal: World,
        moved: bool,
    },
    /// `type: 'selection'` — move the selection by whole grid steps.
    Move {
        start: World,
        last: World,
        /// `startRef`: the selection bounds' origin, so snapping keeps an
        /// off-grid grab landing on the pitch grid.
        start_ref: World,
        /// `lastSnappedTotal`.
        last_snap: (f64, f64),
        moved: bool,
    },
    /// `type: 'line-endpoint'`.
    Endpoint {
        line_id: i64,
        start: bool,
        start_point: World,
        moved: bool,
    },
    /// `type: 'curve'` — bow a wall by dragging its midpoint handle (or any
    /// point of it in the Curve Wall tool).
    Curve {
        line_id: i64,
        /// The wall's rise when the grab began (feet, signed).
        start_arc_height: f64,
        /// The pointer's distance along the wall's normal at that moment.
        start_pointer_normal: f64,
    },
    /// `type: 'rotate-area'`: the selected geometry's rotation handle. The
    /// outline previews at `preview` degrees and the rotation lands on
    /// release.
    RotateArea {
        center: World,
        start_angle: f64,
        preview: f64,
    },
    /// `type: 'rotate-text'`: a text label's / area text block's handle.
    RotateText {
        kind: crate::handles::TextKind,
        id: i64,
        center: World,
        start_angle: f64,
        start_rotation: f64,
        preview: f64,
    },
    /// Symbol resize handle drag.
    SymbolResize {
        id: i64,
        start_scale: f64,
        start_dist: f64,
        center: World,
    },
    /// Symbol rotate handle drag.
    SymbolRotate {
        id: i64,
        start_rotation: f64,
        start_angle: f64,
        center: World,
        /// The turn so far (degrees), for the wedge.
        preview: f64,
    },
}

/// What a finished fence drag should do.
#[derive(Debug, Clone, PartialEq)]
pub enum FenceOutcome {
    /// Nothing (too small a drag, or no drag at all).
    None,
    /// Zoom-fence: the view already zoomed.
    Zoom,
    /// Fence or the Select-mode marquee: select inside this rectangle.
    Select { min: World, max: World },
    /// Lasso: select inside this closed path.
    Lasso(Vec<World>),
}

#[derive(Debug, Clone)]
pub struct View {
    /// `state.pan` — screen px of the world origin within the canvas.
    pub pan: (f64, f64),
    pub zoom: f64,
    /// `state.drawingScale` (1:N, default 10).
    pub drawing_scale: f64,
    pub grid: bool,
    pub snap: bool,
    pub area_snap: bool,
    /// `state.areaGridSnapSize` (ft).
    pub area_snap_size: f64,
    /// `state.gridSize` (ft) — set by the pitch menu too.
    pub grid_size: f64,
    pub mode: Mode,
    /// `state.fenceStart` / `state.fenceEnd` while dragging a fence.
    pub fence: Option<(World, World)>,
    /// `state.lassoPoints`: the freehand path, sampled as the pointer moves.
    pub lasso_points: Vec<World>,
    /// `state.dragState`.
    pub drag: Option<Drag>,
    /// `state.dimensionsEnabled` — wall dimension labels are drawn.
    pub dimensions: bool,
    /// `state.dimensionPlacement`: 'out' or 'in'.
    pub dimension_placement: DimensionPlacement,
    /// `state.filters[key] !== false`, in FILTERS order.
    pub filters: [bool; 6],
    /// `#filter-popover` shown.
    pub filter_open: bool,
    /// The canvas's CSS size (`viewWidth`, `viewHeight`).
    pub canvas: (f64, f64),
    /// `state.isPanning` + `state.panStart` (window px of the last move).
    pub pan_from: Option<(f64, f64)>,
    /// `state.spacePan` (Space held) / `state.spacePanned` (dragged since).
    pub space_pan: bool,
    pub space_panned: bool,
    pub curve_mode: CurveMode,
}

impl View {
    /// Startup: pan at the canvas center (SketchApp constructor).
    pub fn new(canvas_w: f64, canvas_h: f64) -> Self {
        View {
            pan: (canvas_w / 2.0, canvas_h / 2.0),
            zoom: 1.0,
            drawing_scale: 10.0,
            grid: true,
            snap: true,
            area_snap: false,
            area_snap_size: 1.0,
            grid_size: 1.0,
            mode: Mode::Select,
            fence: None,
            lasso_points: Vec::new(),
            drag: None,
            dimensions: true,
            dimension_placement: DimensionPlacement::Out,
            filters: [true; 6],
            filter_open: false,
            canvas: (canvas_w, canvas_h),
            pan_from: None,
            space_pan: false,
            space_panned: false,
            curve_mode: CurveMode::Degree,
        }
    }

    /// `Geometry.toWorld` for a canvas-relative screen point.
    pub fn to_world(&self, sx: f64, sy: f64) -> World {
        World {
            x: (sx - self.pan.0) / (PPF * self.zoom),
            y: (self.pan.1 - sy) / (PPF * self.zoom),
        }
    }

    /// `Geometry.toScreen`.
    pub fn to_screen(&self, w: World) -> (f64, f64) {
        (
            self.pan.0 + w.x * PPF * self.zoom,
            self.pan.1 - w.y * PPF * self.zoom,
        )
    }

    /// `drawingScaleFactor()` — zoom that 100% means at 1:N (1:10 is 1.0).
    pub fn drawing_scale_factor(&self) -> f64 {
        let s = if self.drawing_scale > 0.0 {
            self.drawing_scale
        } else {
            10.0
        };
        10.0 / s
    }

    /// `zoomTo(z, cx, cy)`: clamp 0.02..10, keep (cx, cy) fixed on screen.
    /// `cycleDimensions()`: outside -> inside -> hidden -> outside. Returns
    /// the toast (`ui.notify`).
    pub fn cycle_dimensions(&mut self) -> &'static str {
        if !self.dimensions {
            self.dimensions = true;
            self.dimension_placement = DimensionPlacement::Out;
        } else if self.dimension_placement == DimensionPlacement::Out {
            self.dimension_placement = DimensionPlacement::In;
        } else {
            self.dimensions = false;
        }
        match (self.dimensions, self.dimension_placement) {
            (true, DimensionPlacement::Out) => "Dimensions: Outside",
            (true, DimensionPlacement::In) => "Dimensions: Inside",
            _ => "Dimensions: Hidden",
        }
    }

    pub fn zoom_to(&mut self, z: f64, cx: f64, cy: f64) {
        let old = self.zoom;
        let z = z.clamp(0.02, 10.0);
        self.pan.0 = cx - (cx - self.pan.0) * (z / old);
        self.pan.1 = cy - (cy - self.pan.1) * (z / old);
        self.zoom = z;
    }

    fn center(&self) -> (f64, f64) {
        (self.canvas.0 / 2.0, self.canvas.1 / 2.0)
    }

    pub fn zoom_in(&mut self) {
        let (cx, cy) = self.center();
        self.zoom_to(self.zoom * 1.2, cx, cy);
    }

    pub fn zoom_out(&mut self) {
        let (cx, cy) = self.center();
        self.zoom_to(self.zoom / 1.2, cx, cy);
    }

    /// `#zoom-range` input: `value / 100 * drawingScaleFactor()`.
    pub fn zoom_slider(&mut self, value: f64) {
        let (cx, cy) = self.center();
        self.zoom_to(value / 100.0 * self.drawing_scale_factor(), cx, cy);
    }

    /// `setDrawingScale(n)`: clamp 1..100; keep the readout percent.
    /// Returns false for an invalid entry (the input just re-syncs).
    pub fn set_drawing_scale(&mut self, n: f64) -> bool {
        if !n.is_finite() || n <= 0.0 {
            return false;
        }
        let previous = self.drawing_scale_factor();
        self.drawing_scale = n.clamp(1.0, 100.0);
        let (cx, cy) = self.center();
        self.zoom_to(self.zoom * self.drawing_scale_factor() / previous, cx, cy);
        true
    }

    /// `updateZoomDisplay` percent: `round(zoom / factor * 100)`.
    pub fn zoom_percent(&self) -> i64 {
        (self.zoom / self.drawing_scale_factor() * 100.0).round() as i64
    }

    /// `zoomToWorldRect(min, max)`: fit with a 30px margin, zoom 0.1..10,
    /// then back to Select.
    pub fn zoom_to_world_rect(&mut self, min: World, max: World) {
        let (w, h) = (max.x - min.x, max.y - min.y);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let margin = 30.0;
        let zx = (self.canvas.0 - margin * 2.0) / (w * PPF);
        let zy = (self.canvas.1 - margin * 2.0) / (h * PPF);
        self.zoom = zx.min(zy).clamp(0.1, 10.0);
        let cx = (min.x + max.x) / 2.0;
        let cy = (min.y + max.y) / 2.0;
        self.pan.0 = self.canvas.0 / 2.0 - cx * PPF * self.zoom;
        self.pan.1 = self.canvas.1 / 2.0 + cy * PPF * self.zoom;
        self.set_mode(Mode::Select);
    }

    /// `centerSketch()` over the sketch bounds (None = no lines: no-op).
    pub fn center_sketch(&mut self, bounds: Option<(World, World)>) {
        let Some((min, max)) = bounds else { return };
        let cx = (min.x + max.x) / 2.0;
        let cy = (min.y + max.y) / 2.0;
        self.pan.0 = self.canvas.0 / 2.0 - cx * PPF * self.zoom;
        self.pan.1 = self.canvas.1 / 2.0 + cy * PPF * self.zoom;
    }

    /// `centerAndFit()` over the sketch bounds (None = no lines: no-op).
    pub fn center_and_fit(&mut self, bounds: Option<(World, World)>) {
        let Some((min, max)) = bounds else { return };
        let (ww, wh) = (max.x - min.x, max.y - min.y);
        if ww == 0.0 && wh == 0.0 {
            return;
        }
        let margin = 30.0;
        let zx = (self.canvas.0 - margin * 2.0) / (ww.max(1.0) * PPF);
        let zy = (self.canvas.1 - margin * 2.0) / (wh.max(1.0) * PPF);
        self.zoom = zx.min(zy).clamp(0.1, 10.0);
        self.center_sketch(Some((min, max)));
    }

    /// `setMode(mode)`: leaving the fence modes drops the fence. Returns
    /// the toast some modes announce.
    pub fn set_mode(&mut self, mode: Mode) -> Option<&'static str> {
        if matches!(self.mode, Mode::ZoomFence | Mode::Fence) && mode != self.mode {
            self.fence = None;
        }
        self.mode = mode;
        match mode {
            Mode::Curve => Some("Curve Wall: drag a wall outward or inward. Calculations update live."),
            Mode::Measure => Some(
                "Measure: click points to chain legs. Enter finishes, Shift+Enter closes for area, Ctrl+C copies.",
            ),
            Mode::ChainDim => Some("Chain Dimensions: click a wall to dimension its full run. Esc exits."),
            _ => None,
        }
    }

    /// Pointer pressed for a pan (Pan tool, middle button, or Space held):
    /// `isPanning`, `panStart` = window position.
    pub fn pan_start(&mut self, x: f64, y: f64) {
        if self.space_pan {
            self.space_panned = true;
        }
        self.pan_from = Some((x, y));
    }

    /// `pan += pointer delta` while panning.
    pub fn pan_move(&mut self, x: f64, y: f64) {
        if let Some((fx, fy)) = self.pan_from {
            self.pan.0 += x - fx;
            self.pan.1 += y - fy;
            self.pan_from = Some((x, y));
        }
    }

    pub fn pan_end(&mut self) {
        self.pan_from = None;
    }

    /// Wheel (not drawing a bow, nothing selected): zoom x1.1 up / x0.9 down
    /// about the pointer (canvas px).
    pub fn wheel_zoom(&mut self, up: bool, mx: f64, my: f64) {
        let factor = if up { 1.1 } else { 0.9 };
        self.zoom_to(self.zoom * factor, mx, my);
    }

    /// `D`: toggle the curve mode; returns the toast.
    pub fn toggle_curve_mode(&mut self) -> &'static str {
        self.curve_mode = match self.curve_mode {
            CurveMode::Degree => CurveMode::Height,
            CurveMode::Height => CurveMode::Degree,
        };
        match self.curve_mode {
            CurveMode::Degree => "Curve mode: Degree",
            CurveMode::Height => "Curve mode: Height",
        }
    }

    /// Fence drag start (canvas pointerdown): zoom-fence, the fence tool and
    /// the lasso all begin a drag here.
    pub fn fence_start(&mut self, sx: f64, sy: f64) {
        if matches!(self.mode, Mode::ZoomFence | Mode::Fence | Mode::Lasso) {
            let p = self.to_world(sx, sy);
            self.fence = Some((p, p));
            self.lasso_points = if self.mode == Mode::Lasso {
                vec![p]
            } else {
                Vec::new()
            };
        }
    }

    /// `dragState = { type: 'selection', startWorld, lastWorld, startRef }`.
    pub fn drag_start_move(&mut self, world: World, start_ref: World) {
        self.drag = Some(Drag::Move {
            start: world,
            last: world,
            start_ref,
            last_snap: (0.0, 0.0),
            moved: false,
        });
    }

    /// `dragState = { type: 'line-endpoint' }`.
    pub fn drag_start_endpoint(&mut self, line_id: i64, start: bool, world: World) {
        self.drag = Some(Drag::Endpoint {
            line_id,
            start,
            start_point: world,
            moved: false,
        });
    }

    /// The move delta to apply for this pointer position: whole grid steps
    /// from the grab point when snapping is on (so an on-grid sketch stays on
    /// the grid and lengths stay exact), otherwise the raw delta. Returns
    /// `None` until the drag passes the 2px activation threshold.
    pub fn drag_move(&mut self, world: World, grid: f64, snap: bool) -> Option<(f64, f64)> {
        let absolute = self.area_snap;
        let threshold = 2.0 / (sketch_core::geometry::PIXELS_PER_FOOT * self.zoom);
        let drag = self.drag.as_mut()?;
        match drag {
            Drag::Move {
                start,
                start_ref,
                last,
                last_snap,
                moved,
                ..
            } => {
                let far = (world.x - start.x).hypot(world.y - start.y) > threshold;
                if !*moved && !far {
                    return None;
                }
                let mut dx = world.x - last.x;
                let mut dy = world.y - last.y;
                if snap {
                    let step = if grid > 0.0 { grid } else { 1.0 };
                    let reference = if absolute {
                        *start_ref
                    } else {
                        World { x: 0.0, y: 0.0 }
                    };
                    let total_x =
                        crate::drawing::js_round((reference.x + world.x - start.x) / step) * step
                            - reference.x;
                    let total_y =
                        crate::drawing::js_round((reference.y + world.y - start.y) / step) * step
                            - reference.y;
                    dx = total_x - last_snap.0;
                    dy = total_y - last_snap.1;
                    *last_snap = (total_x, total_y);
                }
                *moved = true;
                *last = world;
                Some((dx, dy))
            }
            Drag::Endpoint { .. } => None,
            Drag::Curve { .. } => None,
            Drag::RotateArea { .. } => None,
            Drag::RotateText { .. } => None,
            Drag::ChainOffset { .. } => None,
            Drag::SymbolResize { .. } => None,
            Drag::SymbolRotate { .. } => None,
        }
    }

    /// The pointer's own marquee: a Select-mode press that hit nothing starts
    /// a drag-rectangle (`state.dragState.type === 'selection-marquee'`).
    pub fn marquee_start(&mut self, world: World) {
        self.fence = Some((world, world));
        self.lasso_points.clear();
    }

    /// Fence drag move (`fenceEnd = pointerWorld`, unsnapped). The lasso also
    /// samples its path, one point per 2px of travel.
    pub fn fence_move(&mut self, sx: f64, sy: f64) {
        let p = self.to_world(sx, sy);
        if let Some((_, end)) = self.fence.as_mut() {
            *end = p;
        }
        if self.mode == Mode::Lasso {
            let far = self
                .lasso_points
                .last()
                .is_none_or(|last| (last.x - p.x).hypot(last.y - p.y) > 0.1);
            if far {
                self.lasso_points.push(p);
            }
        }
    }

    /// Fence drag end: what the drag should do. A drag under 0.5 ft either
    /// way is ignored, as the original's `if (Math.abs(...) > 0.5)` does.
    pub fn fence_end(&mut self) -> FenceOutcome {
        let Some((fs, fe)) = self.fence.take() else {
            return FenceOutcome::None;
        };
        let min = World {
            x: fs.x.min(fe.x),
            y: fs.y.min(fe.y),
        };
        let max = World {
            x: fs.x.max(fe.x),
            y: fs.y.max(fe.y),
        };
        let moved = (fe.x - fs.x).abs() > 0.5 || (fe.y - fs.y).abs() > 0.5;
        if self.mode == Mode::Lasso {
            // `lassoPoints.length > 2`.
            let points = std::mem::take(&mut self.lasso_points);
            return if points.len() > 2 {
                FenceOutcome::Lasso(points)
            } else {
                FenceOutcome::None
            };
        }
        if !moved {
            return FenceOutcome::None;
        }
        if self.mode == Mode::ZoomFence {
            self.zoom_to_world_rect(min, max);
            return FenceOutcome::Zoom;
        }
        FenceOutcome::Select { min, max }
    }

    /// `setAreaGridSnapSize(value)` — also the drawing snap grid.
    pub fn set_area_snap_size(&mut self, size: f64) {
        if size.is_finite() && size > 0.0 {
            self.area_snap_size = size;
            self.grid_size = size;
        }
    }

    /// The pitch option on display.
    pub fn pitch_label(&self) -> &'static str {
        PITCHES
            .iter()
            .find(|(_, v)| (*v - self.area_snap_size).abs() < 1e-9)
            .map(|(l, _)| *l)
            .unwrap_or("1'")
    }
}

/// JS `String(n)` for the 1:N input (integers without a decimal point).
pub fn js_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_audit_area_grid_snap_uses_absolute_reference_after_activation() {
        let mut view = View::new(100.0, 100.0);
        view.area_snap = true;
        view.drag_start_move(World { x: 0.0, y: 0.0 }, World { x: 0.2, y: -0.2 });
        assert_eq!(view.drag_move(World { x: 0.01, y: 0.0 }, 0.5, true), None);
        let delta = view.drag_move(World { x: 0.4, y: 0.3 }, 0.5, true).unwrap();
        assert!((0.2 + delta.0 - 0.5).abs() < 1e-9);
        assert!((-0.2 + delta.1).abs() < 1e-9);
        let next = view.drag_move(World { x: 0.9, y: 0.3 }, 0.5, true).unwrap();
        assert!((next.0 - 0.5).abs() < 1e-9);
        assert!(next.1.abs() < 1e-9);
    }

    #[test]
    fn release_audit_ordinary_snap_preserves_off_grid_origin_and_free_labels() {
        let mut view = View::new(100.0, 100.0);
        view.drag_start_move(World { x: 0.0, y: 0.0 }, World { x: 0.2, y: 0.2 });
        assert_eq!(
            view.drag_move(World { x: 0.8, y: 0.2 }, 1.0, true),
            Some((1.0, 0.0))
        );
        view.drag_start_move(World { x: 0.0, y: 0.0 }, World { x: 0.2, y: 0.2 });
        assert_eq!(
            view.drag_move(World { x: 0.8, y: 0.2 }, 1.0, false),
            Some((0.8, 0.2))
        );
    }

    fn view() -> View {
        View::new(1047.0, 703.0)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn startup_pan_is_the_canvas_center() {
        // 1400x860 window: canvas 1400 - 41 - 312 by 860 - 157.
        let v = view();
        assert_eq!((v.pan, v.zoom), ((523.5, 351.5), 1.0));
    }

    #[test]
    fn zoom_buttons_slider_and_scale_match_the_original() {
        let mut v = view();
        v.zoom_in();
        assert_eq!(v.zoom_percent(), 120);
        assert_eq!(v.pan, (523.5, 351.5), "zooms about the canvas center");
        let mut v = view();
        v.zoom_out();
        assert_eq!(v.zoom_percent(), 83);
        let mut v = view();
        v.zoom_slider(250.0);
        assert!(close(v.zoom, 2.5));
        // 1:20 halves the zoom but keeps the readout at 100%.
        let mut v = view();
        assert!(v.set_drawing_scale(20.0));
        assert!(close(v.zoom, 0.5));
        assert_eq!(v.zoom_percent(), 100);
        assert_eq!(js_number(v.drawing_scale), "20");
        assert!(!v.set_drawing_scale(0.0));
        v.set_drawing_scale(500.0);
        assert_eq!(v.drawing_scale, 100.0);
        // zoomTo clamps 2%..1000%.
        let mut v = view();
        v.zoom_to(50.0, 0.0, 0.0);
        assert_eq!(v.zoom, 10.0);
    }

    #[test]
    fn zoom_fence_matches_the_original_numbers() {
        // Live original: drag (300,300) -> (500,450) page px on the canvas
        // at (41,129): fence (-13.225, 9.025)..(-3.225, 1.525) ft, then
        // zoom 4.2866667, pan (1228.6566667, 803.7433333), mode select.
        let mut v = view();
        v.set_mode(Mode::ZoomFence);
        v.fence_start(259.0, 171.0);
        v.fence_move(459.0, 321.0);
        let (a, b) = v.fence.unwrap();
        assert!(close(a.x, -13.225) && close(a.y, 9.025));
        assert!(close(b.x, -3.225) && close(b.y, 1.525));
        v.fence_end();
        assert!(close(v.zoom, 4.286666666666667));
        assert!(close(v.pan.0, 1228.6566666666668));
        assert!(close(v.pan.1, 803.7433333333333));
        assert_eq!(v.mode, Mode::Select);
        assert_eq!(v.zoom_percent(), 429);
        // A tiny drag only clears the fence and stays armed.
        let mut v = view();
        v.set_mode(Mode::ZoomFence);
        v.fence_start(100.0, 100.0);
        v.fence_move(105.0, 104.0);
        v.fence_end();
        assert_eq!((v.mode, v.zoom), (Mode::ZoomFence, 1.0));
    }

    #[test]
    fn pan_drag_and_wheel_zoom_match_the_original_numbers() {
        // Live original: Pan tool, drag (400,400) -> (500,450): pan
        // (623.5, 401.5). Wheel up, up at canvas (300,300) then down at
        // (600,400): zoom 1.089, pan (573.3915, 366.0835), "+ 109%".
        let mut v = view();
        v.set_mode(Mode::Pan);
        v.pan_start(400.0, 400.0);
        for i in 1..=5 {
            v.pan_move(400.0 + 20.0 * i as f64, 400.0 + 10.0 * i as f64);
        }
        v.pan_end();
        assert_eq!(v.pan, (623.5, 401.5));
        assert!(v.pan_from.is_none());
        let mut v = view();
        v.wheel_zoom(true, 300.0, 300.0);
        v.wheel_zoom(true, 300.0, 300.0);
        v.wheel_zoom(false, 600.0, 400.0);
        assert!(close(v.zoom, 1.0890000000000002));
        assert!(close(v.pan.0, 573.3915000000001) && close(v.pan.1, 366.0835));
        assert_eq!(v.zoom_percent(), 109);
    }

    #[test]
    fn modes_toasts_space_pan_and_curve_mode() {
        let mut v = view();
        assert!(v
            .set_mode(Mode::Measure)
            .unwrap()
            .starts_with("Measure: click points"));
        assert!(v.set_mode(Mode::DrawExterior).is_none());
        assert!(v.mode.is_drawing() && !v.mode.crosshair());
        assert!(Mode::Fence.crosshair() && !Mode::Pan.crosshair());
        v.space_pan = true;
        v.pan_start(10.0, 10.0);
        assert!(v.space_panned, "a Space drag counts as panned");
        assert_eq!(v.toggle_curve_mode(), "Curve mode: Height");
        assert_eq!(v.toggle_curve_mode(), "Curve mode: Degree");
    }

    #[test]
    fn toggles_pitch_and_empty_sketch_centering() {
        let mut v = view();
        v.set_area_snap_size(0.5);
        assert_eq!((v.area_snap_size, v.grid_size), (0.5, 0.5));
        assert_eq!(v.pitch_label(), "1/2'");
        v.set_area_snap_size(-1.0);
        assert_eq!(v.area_snap_size, 0.5);
        let before = v.clone();
        v.center_sketch(None);
        v.center_and_fit(None);
        assert_eq!((v.pan, v.zoom), (before.pan, before.zoom));
    }
}
