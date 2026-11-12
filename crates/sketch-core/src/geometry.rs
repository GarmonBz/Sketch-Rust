//! GeometryEngine port — method-for-method from `refs/geometry-engine.js`.
//!
//! Pinned conventions (from the browser oracle):
//! - Coordinate space is FEET; preserve the browser's winding/sign convention.
//! - Circular-segment area OPPOSES the bow angle's sign (outward bow subtracts).
//! - `calculate_area` is the absolute of the curved area.
//! - MAX_BOW_ANGLE is 359 degrees (±359 ok, ±359.5 rejected, clamp never rejects).
//!
//! The ref file also carries the Sketch area-code catalogue (the `AreaCodes`
//! module) after the class body — ported below as [`area_codes`]. The ref file
//! ends mid-IIFE in the `AreaBreakdown` module (only `EPS = 1e-6` and
//! `MAX_BOW_ANGLE = 359` are present); the full module lives in
//! `refs/area-breakdown.js` and is out of scope here.

use serde::{Deserialize, Serialize};

/// Tolerance used by the browser for coordinate comparisons.
pub const EPS: f64 = 1e-9;

/// Default scale: 20 screen pixels per logical foot.
pub const PIXELS_PER_FOOT: f64 = 20.0;

/// JS `Math.round`: halves round toward +infinity (Rust's `round` rounds
/// them away from zero, so -2.5 differs), computed exactly (no `x + 0.5`
/// precision loss).
pub trait JsRound {
    fn js_round(self) -> f64;
}

impl JsRound for f64 {
    fn js_round(self) -> f64 {
        if !self.is_finite() {
            return self;
        }
        let floor = self.floor();
        if self - floor >= 0.5 {
            floor + 1.0
        } else {
            floor
        }
    }
}
/// A full 360-degree arc cannot join two distinct endpoints. Keeping one
/// degree of headroom permits stable major arcs without the singularity.
pub const MAX_BOW_ANGLE: f64 = 359.0;

/// A point in logical/world space, in FEET.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Pt {
    pub fn new(x: f64, y: f64) -> Self {
        Pt { x, y }
    }
}

/// Per-edge metadata mirroring the persisted shape (`{ lineId, bowAngle }`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    #[serde(default)]
    pub line_id: Option<i64>,
    #[serde(default)]
    pub bow_angle: f64,
}

impl Default for Edge {
    fn default() -> Self {
        Edge {
            line_id: None,
            bow_angle: 0.0,
        }
    }
}

/// A wall segment mirroring the persisted line shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub id: i64,
    pub start: Pt,
    pub end: Pt,
    #[serde(default)]
    pub bow_angle: f64,
    /// Pin a dragged dimension label follows (`dimensionPosition`).
    #[serde(default)]
    pub dimension_position: Option<Pt>,
    /// Wall midpoint the pin was placed against (`dimensionAnchor`).
    #[serde(default)]
    pub dimension_anchor: Option<Pt>,
}

impl Line {
    pub fn new(id: i64, start: Pt, end: Pt) -> Self {
        Line {
            id,
            start,
            end,
            bow_angle: 0.0,
            dimension_position: None,
            dimension_anchor: None,
        }
    }
}

/// A placed symbol, restricted to the fields `compute_dimension_chain` reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionSymbol {
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub wall_id: Option<i64>,
    #[serde(default)]
    pub wall_gap_width: f64,
    #[serde(default)]
    pub wall_snap: Option<String>,
    #[serde(default)]
    pub wall_side_flip: bool,
    #[serde(default)]
    pub flip_h: bool,
    #[serde(default)]
    pub symbol_type: Option<String>,
}

/// Library entry consulted when a symbol carries no explicit `wallSnap`
/// (the headless stand-in for `window.SKETCH_SYMBOL_LIBRARY`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolLibraryEntry {
    #[serde(default)]
    pub wall_snap: Option<String>,
}

/// JS `RangeError` from `validateBowAngle`.
#[derive(Debug, Clone, PartialEq)]
pub struct BowAngleError {
    pub max: f64,
}

impl std::fmt::Display for BowAngleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bowAngle must be between -{} and {} degrees",
            self.max, self.max
        )
    }
}

impl std::error::Error for BowAngleError {}

/// Headless kernel ported 1:1 from the browser's `GeometryEngine`.
///
/// The JS instance also memoizes `areaLabelAnchor` results in a
/// `labelAnchorCache` Map; that is a pure performance cache with no bearing on
/// results, so it is omitted here.
#[derive(Debug, Clone)]
pub struct GeometryEngine {
    pub pixels_per_foot: f64,
    pub max_bow_angle: f64,
}

impl Default for GeometryEngine {
    fn default() -> Self {
        GeometryEngine {
            pixels_per_foot: PIXELS_PER_FOOT,
            max_bow_angle: MAX_BOW_ANGLE,
        }
    }
}

/// JS `Number(value) || 0`: NaN (and other non-numbers) collapse to 0.
fn js_number_or_zero(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value
    }
}

/// Calculate distance between two points (in logical/world feet).
pub fn distance(p1: Pt, p2: Pt) -> f64 {
    (p2.x - p1.x).hypot(p2.y - p1.y)
}

impl GeometryEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clamp_bow_angle(&self, value: f64) -> f64 {
        let angle = js_number_or_zero(value);
        angle.max(-self.max_bow_angle).min(self.max_bow_angle)
    }

    pub fn validate_bow_angle(&self, value: f64) -> Result<f64, BowAngleError> {
        let angle = js_number_or_zero(value);
        if angle.abs() > self.max_bow_angle {
            return Err(BowAngleError {
                max: self.max_bow_angle,
            });
        }
        Ok(angle)
    }

    /// Same as the free [`distance`]; the JS engine calls `this.distance`.
    pub fn distance(&self, p1: Pt, p2: Pt) -> f64 {
        distance(p1, p2)
    }

    /// Convert degrees to radians.
    pub fn deg_to_rad(&self, deg: f64) -> f64 {
        deg * (std::f64::consts::PI / 180.0)
    }

    /// Calculate square footage. With no edge metadata this remains the
    /// familiar shoelace calculation; callers that provide edges receive the
    /// exact circular-segment contribution as well.
    pub fn calculate_area(&self, points: &[Pt], edges: &[Edge]) -> f64 {
        self.calculate_signed_curved_area(points, edges).abs()
    }

    pub fn calculate_signed_curved_area(&self, points: &[Pt], edges: &[Edge]) -> f64 {
        let mut area = self.calculate_signed_area(points);
        if edges.is_empty() {
            return area;
        }
        for i in 0..points.len() {
            let bow = js_number_or_zero(edges[i].bow_angle);
            area += self.circular_segment_area(points[i], points[(i + 1) % points.len()], bow);
        }
        area
    }

    /// Signed area also identifies winding. Positive is counter-clockwise in
    /// world coordinates; negative is clockwise.
    pub fn calculate_signed_area(&self, points: &[Pt]) -> f64 {
        if points.len() < 3 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 0..points.len() {
            let next = points[(i + 1) % points.len()];
            area += points[i].x * next.y - next.x * points[i].y;
        }
        area / 2.0
    }

    /// Calculate perimeter of a polygon, using arc length for bowed edges.
    pub fn calculate_perimeter(&self, points: &[Pt], edges: &[Edge]) -> f64 {
        if points.len() < 2 {
            return 0.0;
        }
        let mut perim = 0.0;
        for i in 0..points.len() {
            let next = (i + 1) % points.len();
            let bow = edges.get(i).map_or(0.0, |e| js_number_or_zero(e.bow_angle));
            perim += self.arc_length(points[i], points[next], bow);
        }
        perim
    }

    /// Circular-segment area between the chord and the bowed arc. OPPOSES the
    /// bow angle's sign: an outward bow subtracts from the shoelace area.
    pub fn circular_segment_area(&self, p1: Pt, p2: Pt, bow_angle: f64) -> f64 {
        if bow_angle == 0.0 {
            return 0.0;
        }
        self.validate_bow_angle(bow_angle)
            .expect("validated bow angle");
        let theta = bow_angle.abs() * std::f64::consts::PI / 180.0;
        let chord = distance(p1, p2);
        if theta < 1e-9 || chord < 1e-9 {
            return 0.0;
        }
        let radius = (chord / 2.0) / (theta / 2.0).sin();
        let segment = (radius * radius / 2.0) * (theta - theta.sin());
        -bow_angle.signum() * segment
    }

    pub fn arc_length(&self, p1: Pt, p2: Pt, bow_angle: f64) -> f64 {
        let chord = distance(p1, p2);
        if bow_angle == 0.0 || chord < 1e-9 {
            return chord;
        }
        self.validate_bow_angle(bow_angle)
            .expect("validated bow angle");
        let theta = bow_angle.abs() * std::f64::consts::PI / 180.0;
        if theta < 1e-9 {
            return chord;
        }
        ((chord / 2.0) / (theta / 2.0).sin()) * theta
    }
}

// --- Coordinate conversion (screen px <-> logical feet) ---

/// Extra result fields reported per leg by `polylineMetrics`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentMetrics {
    pub start: Pt,
    pub end: Pt,
    pub length: f64,
    pub cumulative: f64,
    pub dx: f64,
    pub dy: f64,
    pub bearing: f64,
}

/// Per-leg and cumulative metrics for an open or closed polyline.
#[derive(Debug, Clone, PartialEq)]
pub struct PolylineMetrics {
    pub segments: Vec<SegmentMetrics>,
    pub total: f64,
    pub closed: bool,
    pub area: f64,
}

/// The maximal collinear wall run containing the seed wall
/// (result of `computeDimensionChain`). `refs` are distances along `dir` from
/// `start` (ascending, endpoints included).
#[derive(Debug, Clone, PartialEq)]
pub struct DimensionChain {
    pub line_ids: Vec<i64>,
    pub start: Pt,
    pub end: Pt,
    pub dir: Pt,
    pub normal: Pt,
    pub refs: Vec<f64>,
    pub length: f64,
}

/// A bounded region found by `findEnclosedFaces` (counter-clockwise, positive
/// signed area); each edge carries the source wall's lineId.
#[derive(Debug, Clone, PartialEq)]
pub struct EnclosedFace {
    pub points: Vec<Pt>,
    pub area: f64,
    pub edges: Vec<Edge>,
}

/// Point inside a ring farthest from its boundary (`poleOfInaccessibility`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoleOfInaccessibility {
    pub x: f64,
    pub y: f64,
    pub distance: f64,
}

impl GeometryEngine {
    /// Screen coords -> Logical/World coords (feet).
    pub fn to_logical(&self, screen_pt: Pt, pan: Pt, zoom: f64) -> Pt {
        Pt {
            x: (screen_pt.x - pan.x) / (self.pixels_per_foot * zoom),
            y: (pan.y - screen_pt.y) / (self.pixels_per_foot * zoom),
        }
    }

    /// Alias for clarity.
    pub fn to_world(&self, screen_pt: Pt, pan: Pt, zoom: f64) -> Pt {
        self.to_logical(screen_pt, pan, zoom)
    }

    /// Logical/World coords -> Screen coords.
    pub fn to_screen(&self, logical_pt: Pt, pan: Pt, zoom: f64) -> Pt {
        Pt {
            x: pan.x + logical_pt.x * self.pixels_per_foot * zoom,
            y: pan.y - logical_pt.y * self.pixels_per_foot * zoom,
        }
    }

    /// Snap a world point to the nearest grid intersection.
    pub fn snap_to_grid(&self, world_pt: Pt, grid_size: f64) -> Pt {
        Pt {
            x: (world_pt.x / grid_size).js_round() * grid_size,
            y: (world_pt.y / grid_size).js_round() * grid_size,
        }
    }

    /// Find the nearest existing endpoint to a world point, within a
    /// threshold. The optional skip predicate lets callers exclude points,
    /// e.g. vertices of areas that are already closed.
    pub fn find_nearest_endpoint(
        &self,
        world_pt: Pt,
        all_lines: &[Line],
        threshold: f64,
        skip: Option<&dyn Fn(&Pt) -> bool>,
    ) -> Option<Pt> {
        let mut nearest: Option<Pt> = None;
        let mut nearest_dist = threshold;
        for line in all_lines {
            for pt in [line.start, line.end] {
                if let Some(skip) = skip {
                    if skip(&pt) {
                        continue;
                    }
                }
                let d = distance(world_pt, pt);
                if d < nearest_dist {
                    nearest_dist = d;
                    nearest = Some(pt);
                }
            }
        }
        nearest
    }

    /// Snapping logic: finds nearest endpoint in a given direction.
    /// Direction 0/180 is vertical movement and looks for X alignment;
    /// anything else is horizontal movement looking for Y alignment.
    pub fn find_alignment(
        &self,
        current_pos: Pt,
        all_lines: &[Line],
        direction: f64,
    ) -> Option<Pt> {
        let mut nearest_dist = f64::INFINITY;
        let mut snap_pos: Option<Pt> = None;
        for line in all_lines {
            for pt in [line.start, line.end] {
                if direction == 0.0 || direction == 180.0 {
                    // Vertical movement, look for X alignment
                    if (pt.x - current_pos.x).abs() < 0.1 {
                        let d = (pt.y - current_pos.y).abs();
                        if d < nearest_dist && d > 0.1 {
                            nearest_dist = d;
                            snap_pos = Some(Pt {
                                x: current_pos.x,
                                y: pt.y,
                            });
                        }
                    }
                } else {
                    // Horizontal movement, look for Y alignment
                    if (pt.y - current_pos.y).abs() < 0.1 {
                        let d = (pt.x - current_pos.x).abs();
                        if d < nearest_dist && d > 0.1 {
                            nearest_dist = d;
                            snap_pos = Some(Pt {
                                x: pt.x,
                                y: current_pos.y,
                            });
                        }
                    }
                }
            }
        }
        snap_pos
    }

    /// CTRL+SHIFT+Arrow: Find alignment with the closest line that intersects
    /// the cursor's path.
    pub fn find_intersecting_alignment(
        &self,
        current_pos: Pt,
        all_lines: &[Line],
        direction: f64,
    ) -> Option<Pt> {
        let mut nearest_dist = f64::INFINITY;
        let mut snap_pos: Option<Pt> = None;
        for line in all_lines {
            if direction == 0.0 || direction == 180.0 {
                // Moving vertically — find horizontal lines that cross our X
                let min_x = line.start.x.min(line.end.x);
                let max_x = line.start.x.max(line.end.x);
                if (min_x - 0.1..=max_x + 0.1).contains(&current_pos.x) {
                    // This line crosses our vertical path
                    let mid_y = (line.start.y + line.end.y) / 2.0;
                    let d = (mid_y - current_pos.y).abs();
                    // Check direction
                    if d < nearest_dist
                        && d > 0.1
                        && ((direction == 0.0 && mid_y > current_pos.y)
                            || (direction == 180.0 && mid_y < current_pos.y))
                    {
                        nearest_dist = d;
                        snap_pos = Some(Pt {
                            x: current_pos.x,
                            y: mid_y,
                        });
                    }
                }
            } else {
                // Moving horizontally — find vertical lines that cross our Y
                let min_y = line.start.y.min(line.end.y);
                let max_y = line.start.y.max(line.end.y);
                if (min_y - 0.1..=max_y + 0.1).contains(&current_pos.y) {
                    let mid_x = (line.start.x + line.end.x) / 2.0;
                    let d = (mid_x - current_pos.x).abs();
                    if d < nearest_dist
                        && d > 0.1
                        && ((direction == 90.0 && mid_x > current_pos.x)
                            || (direction == 270.0 && mid_x < current_pos.x))
                    {
                        nearest_dist = d;
                        snap_pos = Some(Pt {
                            x: mid_x,
                            y: current_pos.y,
                        });
                    }
                }
            }
        }
        snap_pos
    }

    /// Distance from a point to a line segment.
    pub fn distance_to_segment(&self, pt: Pt, seg_a: Pt, seg_b: Pt) -> f64 {
        let proj = self.project_point_to_segment(pt, seg_a, seg_b);
        distance(pt, proj)
    }

    pub fn project_point_to_segment(&self, pt: Pt, seg_a: Pt, seg_b: Pt) -> Pt {
        let dx = seg_b.x - seg_a.x;
        let dy = seg_b.y - seg_a.y;
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 {
            return seg_a;
        }
        let mut t = ((pt.x - seg_a.x) * dx + (pt.y - seg_a.y) * dy) / len_sq;
        t = t.clamp(0.0, 1.0);
        Pt {
            x: seg_a.x + t * dx,
            y: seg_a.y + t * dy,
        }
    }

    /// Constrain a point to fixed angular increments about an anchor while
    /// keeping its distance, so a held modifier can lock a run to 45s.
    pub fn constrain_to_angle_step(&self, anchor: Pt, pt: Pt, step_degrees: f64) -> Pt {
        let step = {
            let s = js_number_or_zero(step_degrees).abs();
            if s != 0.0 {
                s
            } else {
                45.0
            }
        };
        let dx = pt.x - anchor.x;
        let dy = pt.y - anchor.y;
        let length = dx.hypot(dy);
        if length < 1e-9 {
            return anchor;
        }
        let snapped = self
            .deg_to_rad(((dy.atan2(dx) * 180.0 / std::f64::consts::PI) / step).js_round() * step);
        Pt {
            x: anchor.x + snapped.cos() * length,
            y: anchor.y + snapped.sin() * length,
        }
    }

    /// Direction in degrees clockwise from straight up, matching the angle
    /// the drawing HUD reports.
    pub fn bearing_degrees(&self, from: Pt, to: Pt) -> f64 {
        let math_angle = (to.y - from.y).atan2(to.x - from.x) * 180.0 / std::f64::consts::PI;
        (90.0 - math_angle + 360.0) % 360.0
    }

    /// Included angle at a corner, 0-180 degrees. Returns `None` when either
    /// leg has no length to take a direction from.
    pub fn corner_angle(&self, previous: Pt, vertex: Pt, next: Pt) -> Option<f64> {
        let a = Pt {
            x: previous.x - vertex.x,
            y: previous.y - vertex.y,
        };
        let b = Pt {
            x: next.x - vertex.x,
            y: next.y - vertex.y,
        };
        let length_a = a.x.hypot(a.y);
        let length_b = b.x.hypot(b.y);
        if length_a < 1e-9 || length_b < 1e-9 {
            return None;
        }
        let cosine = ((a.x * b.x + a.y * b.y) / (length_a * length_b)).clamp(-1.0, 1.0);
        Some(cosine.acos() * 180.0 / std::f64::consts::PI)
    }

    /// Per-leg and cumulative metrics for an open or closed polyline. A
    /// closed run of three or more points also reports the area it encloses.
    pub fn polyline_metrics(&self, points: &[Pt], closed: bool) -> PolylineMetrics {
        let ring = closed && points.len() > 2;
        let legs = if ring {
            points.len()
        } else {
            points.len().saturating_sub(1)
        };
        let mut segments = Vec::with_capacity(legs);
        let mut total = 0.0;
        if !points.is_empty() {
            for i in 0..legs {
                let start = points[i];
                let end = points[(i + 1) % points.len()];
                let length = distance(start, end);
                total += length;
                segments.push(SegmentMetrics {
                    start,
                    end,
                    length,
                    cumulative: total,
                    dx: end.x - start.x,
                    dy: end.y - start.y,
                    bearing: self.bearing_degrees(start, end),
                });
            }
        }
        PolylineMetrics {
            segments,
            total,
            closed: ring,
            area: if ring {
                self.calculate_signed_area(points).abs()
            } else {
                0.0
            },
        }
    }

    /// Point-in-polygon test (ray casting).
    pub fn point_in_polygon(&self, pt: Pt, polygon: &[Pt]) -> bool {
        let mut inside = false;
        let mut j = polygon.len().wrapping_sub(1);
        for i in 0..polygon.len() {
            let (xi, yi) = (polygon[i].x, polygon[i].y);
            let (xj, yj) = (polygon[j].x, polygon[j].y);
            let intersect =
                ((yi > pt.y) != (yj > pt.y)) && (pt.x < (xj - xi) * (pt.y - yi) / (yj - yi) + xi);
            if intersect {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    pub fn point_on_boundary(&self, pt: Pt, polygon: &[Pt], tolerance: f64) -> bool {
        if polygon.len() < 2 {
            return false;
        }
        for i in 0..polygon.len() {
            if self.distance_to_segment(pt, polygon[i], polygon[(i + 1) % polygon.len()])
                <= tolerance
            {
                return true;
            }
        }
        false
    }

    pub fn orientation(&self, a: Pt, b: Pt, c: Pt, tolerance: f64) -> i32 {
        let value = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        if value.abs() <= tolerance {
            0
        } else if value > 0.0 {
            1
        } else {
            -1
        }
    }

    /// Endpoint touches and collinear overlap are deliberately not "proper"
    /// crossings; shared walls are valid containment boundaries.
    pub fn segments_properly_cross(&self, a: Pt, b: Pt, c: Pt, d: Pt, tolerance: f64) -> bool {
        let o1 = self.orientation(a, b, c, tolerance);
        let o2 = self.orientation(a, b, d, tolerance);
        let o3 = self.orientation(c, d, a, tolerance);
        let o4 = self.orientation(c, d, b, tolerance);
        o1 != 0 && o2 != 0 && o3 != 0 && o4 != 0 && o1 != o2 && o3 != o4
    }
}

// --- Arc flattening and containment ---

impl GeometryEngine {
    /// Replace bowed edges by short chords whose maximum sagitta is bounded.
    /// Sampling follows CanvasEngine.drawArc's world-space equivalent exactly.
    ///
    /// The JS signature accepts either `(points, edges, maxSagitta)` or a
    /// single polygon object `{ points, edges }`; the polygon form is the
    /// same call with the polygon's own slices.
    pub fn flatten_ring(&self, points: &[Pt], edges: &[Edge], max_sagitta: f64) -> Vec<Pt> {
        if points.len() < 2 {
            return points.to_vec();
        }
        let mut flattened: Vec<Pt> = Vec::new();
        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            let bow = edges.get(i).map_or(0.0, |e| js_number_or_zero(e.bow_angle));
            flattened.push(p1);
            if bow.abs() < 1e-9 {
                continue;
            }
            self.validate_bow_angle(bow).expect("validated bow angle");
            let chord = distance(p1, p2);
            if chord < 1e-9 {
                continue;
            }
            let theta = bow.abs() * std::f64::consts::PI / 180.0;
            let radius = (chord / 2.0) / (theta / 2.0).sin();
            let safe_sagitta = 1e-6f64.max(max_sagitta.min(radius));
            let max_step = 2.0 * (1.0 - safe_sagitta / radius).clamp(-1.0, 1.0).acos();
            let segments = 1.0f64.max((theta / max_step.max(1e-6)).ceil());
            let dx = p2.x - p1.x;
            let dy = p2.y - p1.y;
            let left = Pt {
                x: -dy / chord,
                y: dx / chord,
            };
            let half = theta / 2.0;
            let center_distance = radius * half.cos();
            let sign = bow.signum();
            let center = Pt {
                x: (p1.x + p2.x) / 2.0 - left.x * center_distance * sign,
                y: (p1.y + p2.y) / 2.0 - left.y * center_distance * sign,
            };
            let start_angle = (p1.y - center.y).atan2(p1.x - center.x);
            // Screen-space drawArc sweeps by bow; converting back to world
            // reverses angular orientation because screen Y is inverted.
            let sweep = -bow * std::f64::consts::PI / 180.0;
            let n = segments as usize;
            for step in 1..n {
                let angle = start_angle + sweep * (step as f64 / segments);
                flattened.push(Pt {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                });
            }
        }
        flattened
    }

    pub fn polygon_contains(
        &self,
        outer_points: &[Pt],
        outer_edges: &[Edge],
        inner_points: &[Pt],
        inner_edges: &[Edge],
        tolerance: f64,
        max_sagitta: f64,
    ) -> bool {
        let outer_ring = self.flatten_ring(outer_points, outer_edges, max_sagitta);
        let inner_ring = self.flatten_ring(inner_points, inner_edges, max_sagitta);
        if outer_ring.len() < 3 || inner_ring.len() < 3 {
            return false;
        }
        for &point in &inner_ring {
            if !self.point_in_polygon(point, &outer_ring)
                && !self.point_on_boundary(point, &outer_ring, tolerance)
            {
                return false;
            }
        }
        for i in 0..inner_ring.len() {
            let a = inner_ring[i];
            let b = inner_ring[(i + 1) % inner_ring.len()];
            for j in 0..outer_ring.len() {
                let c = outer_ring[j];
                let d = outer_ring[(j + 1) % outer_ring.len()];
                if self.segments_properly_cross(a, b, c, d, tolerance * tolerance) {
                    return false;
                }
            }
        }
        for i in 0..inner_ring.len() {
            let a = inner_ring[i];
            let b = inner_ring[(i + 1) % inner_ring.len()];
            let midpoint = Pt {
                x: (a.x + b.x) / 2.0,
                y: (a.y + b.y) / 2.0,
            };
            if self.point_in_polygon(midpoint, &outer_ring)
                && !self.point_on_boundary(midpoint, &outer_ring, tolerance)
            {
                return true;
            }
        }
        false
    }

    /// Check if a point is inside an axis-aligned rectangle.
    pub fn point_in_rect(&self, pt: Pt, min: Pt, max: Pt) -> bool {
        pt.x >= min.x && pt.x <= max.x && pt.y >= min.y && pt.y <= max.y
    }
}

// --- Dimension chains (collinear wall runs) ---

impl GeometryEngine {
    /// Compute the maximal collinear wall run containing the seed wall, plus
    /// the reference points that split it into dimension spans. Bowed walls
    /// are treated as chords (v1 limitation). Returns `None` when the seed is
    /// missing or degenerate. The JS `options` object carries one flag,
    /// `splitWallsAtJunctions`, passed here as its own argument; the symbol
    /// library is the headless stand-in for `window.SKETCH_SYMBOL_LIBRARY`.
    pub fn compute_dimension_chain(
        &self,
        lines: &[Line],
        symbols: &[DimensionSymbol],
        seed_line_id: i64,
        tolerance: f64,
        split_walls_at_junctions: bool,
        library: Option<&std::collections::HashMap<String, SymbolLibraryEntry>>,
    ) -> Option<DimensionChain> {
        let seed = lines.iter().find(|line| line.id == seed_line_id)?;
        let chord = distance(seed.start, seed.end);
        if chord < 1e-6 {
            return None;
        }
        let dir = Pt {
            x: (seed.end.x - seed.start.x) / chord,
            y: (seed.end.y - seed.start.y) / chord,
        };
        let normal = Pt {
            x: -dir.y,
            y: dir.x,
        };
        let origin = seed.start;
        // Project a point onto the seed axis: t is the along-axis distance
        // and off the perpendicular deviation.
        let project = |point: &Pt| -> (f64, f64) {
            let dx = point.x - origin.x;
            let dy = point.y - origin.y;
            (dx * dir.x + dy * dir.y, dx * normal.x + dy * normal.y)
        };
        let parallel_tolerance = (3.0 * std::f64::consts::PI / 180.0).sin(); // 3 degrees
                                                                             // Candidate walls: straight, parallel to the axis and lying on it.
        let candidates: Vec<&Line> = lines
            .iter()
            .filter(|line| {
                if js_number_or_zero(line.bow_angle).abs() >= 0.5 {
                    return false;
                }
                let len = distance(line.start, line.end);
                if len < 1e-6 {
                    return false;
                }
                let cross = (((line.end.x - line.start.x) / len) * dir.y
                    - ((line.end.y - line.start.y) / len) * dir.x)
                    .abs();
                if cross > parallel_tolerance {
                    return false;
                }
                let (_, off_start) = project(&line.start);
                let (_, off_end) = project(&line.end);
                off_start.abs() <= tolerance && off_end.abs() <= tolerance
            })
            .collect();
        // Connected component of candidates sharing endpoints with the seed.
        let endpoints_match = |a: &Pt, b: &Pt| -> bool { distance(*a, *b) <= tolerance };
        let mut component: Vec<i64> = vec![seed.id];
        let mut grew = true;
        while grew {
            grew = false;
            for line in &candidates {
                if component.contains(&line.id) {
                    continue;
                }
                let touches = candidates.iter().any(|other| {
                    if !component.contains(&other.id) {
                        return false;
                    }
                    endpoints_match(&line.start, &other.start)
                        || endpoints_match(&line.start, &other.end)
                        || endpoints_match(&line.end, &other.start)
                        || endpoints_match(&line.end, &other.end)
                });
                if touches {
                    component.push(line.id);
                    grew = true;
                }
            }
        }
        let run_lines: Vec<&Line> = candidates
            .iter()
            .copied()
            .filter(|line| component.contains(&line.id))
            .collect();
        let mut min_t = f64::INFINITY;
        let mut max_t = f64::NEG_INFINITY;
        for line in &run_lines {
            for point in [line.start, line.end] {
                let (t, _) = project(&point);
                min_t = min_t.min(t);
                max_t = max_t.max(t);
            }
        }
        // JS: if (!(maxT - minT > 1e-6)) return null — the negation also
        // catches NaN, so it is transcribed literally.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        let degenerate = !(max_t - min_t > 1e-6);
        if degenerate {
            return None;
        }
        // Reference parameters: run endpoints, T-junction projections from
        // other walls, and hosted-opening gap edges on the run walls.
        let mut refs: Vec<f64> = vec![min_t, max_t];
        let add_ref = |t: f64, refs: &mut Vec<f64>| {
            if t <= min_t + 1e-4 || t >= max_t - 1e-4 {
                return;
            }
            for &existing in refs.iter() {
                if (existing - t).abs() <= tolerance / 2.0 {
                    return;
                }
            }
            refs.push(t);
        };
        if split_walls_at_junctions {
            for line in lines {
                if component.contains(&line.id) {
                    continue;
                }
                for point in [line.start, line.end] {
                    let (t, off) = project(&point);
                    if off.abs() <= tolerance {
                        add_ref(t, &mut refs);
                    }
                }
            }
        }
        for symbol in symbols {
            let gap_width = js_number_or_zero(symbol.wall_gap_width);
            let Some(wall_id) = symbol.wall_id else {
                continue;
            };
            if gap_width <= 0.0 || !component.contains(&wall_id) {
                continue;
            }
            let (t, off) = project(&Pt {
                x: symbol.x,
                y: symbol.y,
            });
            if off.abs() > tolerance * 2.0 + gap_width / 2.0 {
                continue;
            }
            let snap = symbol.wall_snap.clone().or_else(|| {
                let ty = symbol.symbol_type.as_ref()?;
                library
                    .and_then(|lib| lib.get(ty))
                    .and_then(|entry| entry.wall_snap.clone())
            });
            if snap.as_deref() == Some("SIDE") {
                // Hinge-anchored opening: refs at the hinge and at the far
                // edge, which lies along the wall direction unless the swing
                // is flipped.
                let dir_sign = {
                    let wall_dir_sign = symbols_sign(&line_by_id(lines, wall_id), &dir);
                    wall_dir_sign * if symbol.flip_h { -1.0 } else { 1.0 }
                };
                add_ref(t, &mut refs);
                add_ref(
                    t + dir_sign
                        * if symbol.wall_side_flip {
                            -gap_width
                        } else {
                            gap_width
                        },
                    &mut refs,
                );
            } else {
                add_ref(t - gap_width / 2.0, &mut refs);
                add_ref(t + gap_width / 2.0, &mut refs);
            }
        }
        refs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Some(DimensionChain {
            line_ids: run_lines.iter().map(|line| line.id).collect(),
            start: Pt {
                x: origin.x + dir.x * min_t,
                y: origin.y + dir.y * min_t,
            },
            end: Pt {
                x: origin.x + dir.x * max_t,
                y: origin.y + dir.y * max_t,
            },
            dir,
            normal,
            refs: refs.iter().map(|t| t - min_t).collect(),
            length: max_t - min_t,
        })
    }
}

fn line_by_id(lines: &[Line], id: i64) -> Line {
    for line in lines {
        if line.id == id {
            return line.clone();
        }
    }
    Line::new(id, Pt::new(0.0, 0.0), Pt::new(0.0, 0.0))
}

// JS: Math.sign((wall.end.x - wall.start.x) * dir.x + (wall.end.y - wall.start.y) * dir.y) || 1
fn symbols_sign(wall: &Line, dir: &Pt) -> f64 {
    let value = (wall.end.x - wall.start.x) * dir.x + (wall.end.y - wall.start.y) * dir.y;
    let sign = value.signum();
    if sign == 0.0 {
        1.0
    } else {
        sign
    }
}

// --- Centroids and label anchors ---

impl GeometryEngine {
    /// `alongLineT` (`refs/sketch-app.js`) — the parametric position of
    /// `point` along the infinite line through `start`/`end`: 0 at the
    /// start, 1 at the end, >1 past the end, and the same value for every
    /// point on a perpendicular through the same spot.
    pub fn along_line_t(&self, line: &Line, point: Pt) -> f64 {
        let dx = line.end.x - line.start.x;
        let dy = line.end.y - line.start.y;
        ((point.x - line.start.x) * dx + (point.y - line.start.y) * dy) / (dx * dx + dy * dy)
    }

    /// Area-weighted centroid of a simple polygon. Falls back to the vertex
    /// average for degenerate (near-zero-area) rings.
    pub fn polygon_centroid(&self, points: &[Pt]) -> Pt {
        if points.is_empty() {
            return Pt::new(0.0, 0.0);
        }
        if points.len() < 3 {
            return points[0];
        }
        let (mut area, mut cx, mut cy) = (0.0, 0.0, 0.0);
        for i in 0..points.len() {
            let p0 = points[i];
            let p1 = points[(i + 1) % points.len()];
            let cross = p0.x * p1.y - p1.x * p0.y;
            area += cross;
            cx += (p0.x + p1.x) * cross;
            cy += (p0.y + p1.y) * cross;
        }
        area /= 2.0;
        if area.abs() < 1e-9 {
            let n = points.len() as f64;
            let (sx, sy) = points
                .iter()
                .fold((0.0, 0.0), |(ax, ay), p| (ax + p.x, ay + p.y));
            return Pt {
                x: sx / n,
                y: sy / n,
            };
        }
        Pt {
            x: cx / (6.0 * area),
            y: cy / (6.0 * area),
        }
    }

    /// Where a dimension label the appraiser dragged should draw. The pin is
    /// stored with the wall midpoint it was placed against
    /// (`dimensionAnchor`), so the label keeps its offset and follows the
    /// wall through any later move, stretch, rotation or paste.
    pub fn pinned_dimension_point(&self, line: &Line) -> Option<Pt> {
        let pin = line.dimension_position?;
        if !pin.x.is_finite() || !pin.y.is_finite() {
            return None;
        }
        let mid = Pt {
            x: (line.start.x + line.end.x) / 2.0,
            y: (line.start.y + line.end.y) / 2.0,
        };
        let anchor = match line.dimension_anchor {
            Some(a) if a.x.is_finite() => a,
            _ => mid,
        };
        Some(Pt {
            x: pin.x + mid.x - anchor.x,
            y: pin.y + mid.y - anchor.y,
        })
    }

    /// Polar drawing snap for a free-angle wall: the bearing from the anchor
    /// rounds to a whole angle step and the length to a whole length step.
    /// Returns `None` when the pointer is too close to the anchor to define a
    /// length.
    pub fn polar_snap(
        &self,
        anchor: Pt,
        pt: Pt,
        length_step: f64,
        angle_step_degrees: f64,
    ) -> Option<Pt> {
        let step = if js_number_or_zero(length_step) > 0.0 {
            length_step
        } else {
            1.0
        };
        let angle_step = if js_number_or_zero(angle_step_degrees) > 0.0 {
            angle_step_degrees
        } else {
            5.0
        };
        let dx = pt.x - anchor.x;
        let dy = pt.y - anchor.y;
        let length = (dx.hypot(dy) / step).js_round() * step;
        if length < 1e-9 {
            return None;
        }
        let angle = self.deg_to_rad(
            ((dy.atan2(dx) * 180.0 / std::f64::consts::PI) / angle_step).js_round() * angle_step,
        );
        // Trig residue (cos 90 = 6e-17) would split a shared corner into two
        // keys; clean rounds through a 4-decimal fixed representation.
        let clean = |value: f64| -> f64 {
            let rounded = (value / 1e-4).js_round() * 1e-4;
            let fixed: f64 = format!("{:.4}", rounded).parse().unwrap_or(rounded);
            if fixed == 0.0 {
                0.0
            } else {
                fixed
            }
        };
        Some(Pt {
            x: clean(anchor.x + angle.cos() * length),
            y: clean(anchor.y + angle.sin() * length),
        })
    }

    /// Default anchor for an area's name/size label. The area centroid is
    /// used whenever it sits comfortably inside the outline; an L, U or
    /// courtyard outline whose centroid falls outside or on a wall uses its
    /// most interior point instead, so the label never lands outside the
    /// area it describes. Every renderer must use this one rule.
    pub fn area_label_anchor(&self, points: &[Pt]) -> Pt {
        let ring: Vec<Pt> = points
            .iter()
            .map(|p| Pt {
                x: js_number_or_zero(p.x),
                y: js_number_or_zero(p.y),
            })
            .collect();
        if ring.len() < 3 {
            return self.polygon_centroid(&ring);
        }
        let centroid = self.polygon_centroid(&ring);
        let pole = self.pole_of_inaccessibility(&ring, 0.05);
        let mut anchor = centroid;
        if let Some(pole) = pole {
            if pole.distance > 1e-9 {
                let centroid_clearance = if self.point_in_polygon(centroid, &ring) {
                    self.boundary_distance(centroid, &ring)
                } else {
                    0.0
                };
                if centroid_clearance < pole.distance * 0.5 {
                    anchor = Pt {
                        x: pole.x,
                        y: pole.y,
                    };
                }
            }
        }
        anchor
    }

    pub fn boundary_distance(&self, pt: Pt, ring: &[Pt]) -> f64 {
        let mut best = f64::INFINITY;
        for i in 0..ring.len() {
            best = best.min(self.distance_to_segment(pt, ring[i], ring[(i + 1) % ring.len()]));
        }
        best
    }

    /// Point inside the ring farthest from its boundary (cell refinement
    /// after Mapbox polylabel). Returns the point and its distance in ring
    /// units, or `None` for a degenerate ring.
    pub fn pole_of_inaccessibility(
        &self,
        ring: &[Pt],
        precision: f64,
    ) -> Option<PoleOfInaccessibility> {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for point in ring {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        let size = (max_x - min_x).min(max_y - min_y);
        // JS: if (!(size > 0)) return null — negation transcribed literally
        // so a NaN size also yields None.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        let degenerate = !(size > 0.0);
        if degenerate {
            return None;
        }
        let signed_distance = |x: f64, y: f64| -> f64 {
            let pt = Pt { x, y };
            let d = self.boundary_distance(pt, ring);
            if self.point_in_polygon(pt, ring) {
                d
            } else {
                -d
            }
        };
        struct Cell {
            x: f64,
            y: f64,
            half: f64,
            d: f64,
            max: f64,
        }
        let cell = |x: f64, y: f64, half: f64| -> Cell {
            let d = signed_distance(x, y);
            Cell {
                x,
                y,
                half,
                d,
                max: d + half * std::f64::consts::SQRT_2,
            }
        };
        let half = size / 2.0;
        let mut queue: Vec<Cell> = Vec::new();
        let mut x = min_x;
        while x < max_x {
            let mut y = min_y;
            while y < max_y {
                queue.push(cell(x + half, y + half, half));
                y += size;
            }
            x += size;
        }
        let centroid = self.polygon_centroid(ring);
        let mut best = cell(centroid.x, centroid.y, 0.0);
        let box_center = cell((min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.0);
        if box_center.d > best.d {
            best = box_center;
        }
        let mut guard = 0usize;
        while !queue.is_empty() && guard < 20000 {
            guard += 1;
            let mut index = 0usize;
            for i in 1..queue.len() {
                if queue[i].max > queue[index].max {
                    index = i;
                }
            }
            let current = queue.remove(index);
            if current.d > best.d {
                best = Cell {
                    x: current.x,
                    y: current.y,
                    half: current.half,
                    d: current.d,
                    max: current.max,
                };
            }
            if current.max - best.d <= precision {
                continue;
            }
            let h = current.half / 2.0;
            queue.push(cell(current.x - h, current.y - h, h));
            queue.push(cell(current.x + h, current.y - h, h));
            queue.push(cell(current.x - h, current.y + h, h));
            queue.push(cell(current.x + h, current.y + h, h));
        }
        Some(PoleOfInaccessibility {
            x: best.x,
            y: best.y,
            distance: best.d,
        })
    }
}

// --- Planar face traversal ---

impl GeometryEngine {
    /// Planar face traversal: finds enclosed regions bounded by the given
    /// wall segments. Walls are split at junction points (other walls'
    /// endpoints that land mid-segment) so T-junctions become graph nodes;
    /// bowed walls are treated as their chord (v1 limitation). Returns
    /// bounded faces (counter-clockwise, positive signed area) above
    /// `min_area` as `{ points, area, edges }` where each edge carries the
    /// source wall's lineId. The unbounded outer face is traced clockwise
    /// and is discarded by the positive-area filter.
    pub fn find_enclosed_faces(
        &self,
        lines: &[Line],
        min_area: f64,
        tolerance: f64,
    ) -> Vec<EnclosedFace> {
        let source = lines;
        let n = source.len();
        // Split every wall into sub-segments at junction points so the graph
        // is a proper planar subdivision. Handles both T-junctions (another
        // wall's endpoint landing mid-segment) and X-junctions (two walls
        // crossing).
        let mut chords = vec![0.0f64; n];
        let mut dirs: Vec<Option<Pt>> = vec![None; n];
        let mut stops: Vec<Vec<f64>> = source.iter().map(|_| Vec::new()).collect();
        for (i, line) in source.iter().enumerate() {
            let chord = distance(line.start, line.end);
            chords[i] = chord;
            dirs[i] = if chord > 1e-6 {
                Some(Pt {
                    x: (line.end.x - line.start.x) / chord,
                    y: (line.end.y - line.start.y) / chord,
                })
            } else {
                None
            };
            stops[i].push(0.0);
            stops[i].push(chord);
        }
        for i in 0..n {
            let Some(dir_i) = dirs[i] else { continue };
            for j in 0..n {
                if i == j || dirs[j].is_none() {
                    continue;
                }
                // T-junction: endpoints of line j lying on line i.
                for point in [source[j].start, source[j].end] {
                    let dx = point.x - source[i].start.x;
                    let dy = point.y - source[i].start.y;
                    let t = dx * dir_i.x + dy * dir_i.y;
                    let off = (dx * -dir_i.y + dy * dir_i.x).abs();
                    if off <= tolerance && t > tolerance && t < chords[i] - tolerance {
                        stops[i].push(t);
                    }
                }
            }
        }
        // X-junctions: proper crossings between pairs of walls.
        for i in 0..n {
            for j in (i + 1)..n {
                let (Some(dir_i), Some(dir_j)) = (dirs[i], dirs[j]) else {
                    continue;
                };
                let _ = (dir_i, dir_j);
                let a = source[i].start;
                let c = source[j].start;
                let r = Pt {
                    x: source[i].end.x - a.x,
                    y: source[i].end.y - a.y,
                };
                let s = Pt {
                    x: source[j].end.x - c.x,
                    y: source[j].end.y - c.y,
                };
                let denom = r.x * s.y - r.y * s.x;
                if denom.abs() < 1e-9 {
                    continue; // parallel
                }
                let t = ((c.x - a.x) * s.y - (c.y - a.y) * s.x) / denom;
                let u = ((c.x - a.x) * r.y - (c.y - a.y) * r.x) / denom;
                if t > 1e-4 && t < 1.0 - 1e-4 && u > 1e-4 && u < 1.0 - 1e-4 {
                    stops[i].push(t * chords[i]);
                    stops[j].push(u * chords[j]);
                }
            }
        }
        let mut pieces: Vec<(Pt, Pt, i64)> = Vec::new();
        for (i, line) in source.iter().enumerate() {
            if dirs[i].is_none() {
                continue;
            }
            let dir_i = dirs[i].unwrap();
            let mut sorted = stops[i].clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // Merge stops that are effectively the same junction point.
            let mut merged: Vec<f64> = Vec::new();
            for value in sorted {
                if merged.is_empty() || value - merged[merged.len() - 1] > tolerance {
                    merged.push(value);
                }
            }
            for k in 0..merged.len().saturating_sub(1) {
                if merged[k + 1] - merged[k] < tolerance {
                    continue;
                }
                pieces.push((
                    Pt {
                        x: line.start.x + dir_i.x * merged[k],
                        y: line.start.y + dir_i.y * merged[k],
                    },
                    Pt {
                        x: line.start.x + dir_i.x * merged[k + 1],
                        y: line.start.y + dir_i.y * merged[k + 1],
                    },
                    line.id,
                ));
            }
        }
        let mut nodes: Vec<Pt> = Vec::new();
        let node_of = |point: &Pt, nodes: &mut Vec<Pt>| -> usize {
            for (i, existing) in nodes.iter().enumerate() {
                if (existing.x - point.x).hypot(existing.y - point.y) <= tolerance {
                    return i;
                }
            }
            nodes.push(*point);
            nodes.len() - 1
        };
        struct HalfEdge {
            from: usize,
            to: usize,
            line_id: i64,
            angle: f64,
        }
        let mut half_edges: Vec<HalfEdge> = Vec::new();
        let mut seen_pairs: std::collections::HashSet<(usize, usize)> =
            std::collections::HashSet::new();
        for piece in &pieces {
            let (start, end, line_id) = *piece;
            let a = node_of(&start, &mut nodes);
            let b = node_of(&end, &mut nodes);
            if a == b {
                continue; // degenerate zero-length piece
            }
            let pair = (a.min(b), a.max(b));
            if seen_pairs.contains(&pair) {
                continue; // skip fully overlapping pieces
            }
            seen_pairs.insert(pair);
            half_edges.push(HalfEdge {
                from: a,
                to: b,
                line_id,
                angle: 0.0,
            });
            half_edges.push(HalfEdge {
                from: b,
                to: a,
                line_id,
                angle: 0.0,
            });
        }
        if half_edges.is_empty() {
            return Vec::new();
        }
        let mut outgoing: Vec<Vec<usize>> = nodes.iter().map(|_| Vec::new()).collect();
        for (idx, he) in half_edges.iter_mut().enumerate() {
            he.angle = (nodes[he.to].y - nodes[he.from].y).atan2(nodes[he.to].x - nodes[he.from].x);
            outgoing[he.from].push(idx);
        }
        for list in outgoing.iter_mut() {
            list.sort_by(|&i, &j| {
                half_edges[i]
                    .angle
                    .partial_cmp(&half_edges[j].angle)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        let mut edge_index: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();
        for (idx, he) in half_edges.iter().enumerate() {
            edge_index.insert((he.from, he.to), idx);
        }
        let mut visited = vec![false; half_edges.len()];
        let mut faces: Vec<EnclosedFace> = Vec::new();
        for start_idx in 0..half_edges.len() {
            if visited[start_idx] {
                continue;
            }
            let mut face: Vec<usize> = Vec::new();
            let mut cur = start_idx;
            let mut guard = 0usize;
            while !visited[cur] && guard <= half_edges.len() {
                guard += 1;
                visited[cur] = true;
                face.push(cur);
                let e = &half_edges[cur];
                let rev_idx = edge_index.get(&(e.to, e.from)).copied();
                let list = &outgoing[e.to];
                let pos = match rev_idx {
                    Some(idx) => list
                        .iter()
                        .position(|&i| i == idx)
                        .map(|p| p as isize)
                        .unwrap_or(-1),
                    None => -1,
                };
                cur = list[((pos - 1 + list.len() as isize) % list.len() as isize) as usize];
            }
            if cur != start_idx {
                continue; // malformed cycle (defensive)
            }
            let points: Vec<Pt> = face
                .iter()
                .map(|&idx| nodes[half_edges[idx].from])
                .collect();
            let area = self.calculate_signed_area(&points);
            if area > min_area {
                faces.push(EnclosedFace {
                    points,
                    area,
                    edges: face
                        .iter()
                        .map(|&idx| Edge {
                            line_id: Some(half_edges[idx].line_id),
                            bow_angle: 0.0,
                        })
                        .collect(),
                });
            }
        }
        faces
    }
}

// --- Sketch area-code catalogue (the ref file's `AreaCodes` module) ---

/// Sketch area-code entry. The application keeps a mutable copy on
/// `app.areaCodes`; this module only owns defaults, validation, and JSON
/// interchange helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaCode {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_group")]
    pub group: String,
    #[serde(default = "default_multiplier")]
    pub multiplier: f64,
    #[serde(default = "default_true")]
    pub positive: bool,
    #[serde(default, rename = "includeInGLA")]
    pub include_in_gla: bool,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default = "default_fill")]
    pub fill: String,
    #[serde(default)]
    pub fill_color: String,
    #[serde(default = "default_fill_style")]
    pub fill_style: String,
    #[serde(default = "default_fill_opacity")]
    pub fill_opacity: f64,
}

impl Default for AreaCode {
    fn default() -> Self {
        AreaCode {
            code: String::new(),
            name: String::new(),
            description: String::new(),
            group: default_group(),
            multiplier: default_multiplier(),
            positive: default_true(),
            include_in_gla: false,
            color: default_color(),
            fill: default_fill(),
            fill_color: String::new(),
            fill_style: default_fill_style(),
            fill_opacity: default_fill_opacity(),
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_group() -> String {
    "OTH".to_string()
}
fn default_multiplier() -> f64 {
    1.0
}
fn default_color() -> String {
    "#718096".to_string()
}
fn default_fill() -> String {
    "rgba(113,128,150,.10)".to_string()
}
fn default_fill_style() -> String {
    "solid".to_string()
}
fn default_fill_opacity() -> f64 {
    10.0
}

/// JS `RangeError`/`Error` counterparts from the AreaCodes module.
#[derive(Debug, Clone, PartialEq)]
pub enum AreaCodesError {
    NotAnArray,
    InvalidCode { index: usize },
    DuplicateCode { code: String },
    InvalidMultiplier { code: String },
    Empty,
}

impl std::fmt::Display for AreaCodesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AreaCodesError::NotAnArray => write!(f, "Area codes must be a JSON array."),
            AreaCodesError::InvalidCode { index } => {
                write!(f, "Area code {} has an invalid code.", index + 1)
            }
            AreaCodesError::DuplicateCode { code } => write!(f, "Duplicate area code: {}", code),
            AreaCodesError::InvalidMultiplier { code } => {
                write!(f, "Area code {} has an invalid multiplier.", code)
            }
            AreaCodesError::Empty => write!(f, "At least one area code is required."),
        }
    }
}

impl std::error::Error for AreaCodesError {}

/// JS `Math.max(0, Math.round(...))` rollup for the suite: BSMT is finished
/// basement area and BSMT2 unfinished. Both are excluded from GLA but both
/// belong on the 1004's basement line, so total is the sum and finished is
/// BSMT alone. (Defined but not exported by the JS API; ported regardless.)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BasementRollup {
    pub total: f64,
    pub finished: f64,
    pub unfinished: f64,
}

pub mod area_codes {
    use super::{default_color, AreaCode, AreaCodesError, BasementRollup, JsRound};

    /// The `DEFAULT_AREA_CODES` catalogue, verbatim from the ref file.
    pub const DEFAULT_AREA_CODES_JSON: &str = r##"[

  { "code": "GLA1", "name": "First Floor", "description": "Gross Living Area", "group": "GLA", "multiplier": 1, "positive": true, "includeInGLA": true, "color": "#2B6CB0", "fill": "rgba(43,108,176,.10)" },
  { "code": "GLA2", "name": "Second Floor", "description": "Gross Living Area", "group": "GLA", "multiplier": 1, "positive": true, "includeInGLA": true, "color": "#2C5282", "fill": "rgba(44,82,130,.10)" },
  { "code": "GLA3", "name": "Third Floor", "description": "Gross Living Area", "group": "GLA", "multiplier": 1, "positive": true, "includeInGLA": true, "color": "#3182CE", "fill": "rgba(49,130,206,.10)" },
  { "code": "GLA4", "name": "Fourth Floor", "description": "Gross Living Area", "group": "GLA", "multiplier": 1, "positive": true, "includeInGLA": true, "color": "#4299E1", "fill": "rgba(66,153,225,.10)" },
  { "code": "GBA1", "name": "First Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#024F39", "fill": "rgba(2,79,57,.10)" },
  { "code": "GBA2", "name": "Second Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#065F40", "fill": "rgba(6,95,64,.10)" },
  { "code": "GBA3", "name": "Third Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#017142", "fill": "rgba(1,113,66,.10)" },
  { "code": "GBA4", "name": "Fourth Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#238247", "fill": "rgba(35,130,71,.10)" },
  { "code": "GBA5", "name": "Fifth Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#42924D", "fill": "rgba(66,146,77,.10)" },
  { "code": "GBA6", "name": "Sixth Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#5DA254", "fill": "rgba(93,162,84,.10)" },
  { "code": "GBA7", "name": "Seventh Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#77B25B", "fill": "rgba(119,178,91,.10)" },
  { "code": "GBA8", "name": "Eighth Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#92C263", "fill": "rgba(146,194,99,.10)" },
  { "code": "GBA9", "name": "Ninth Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#ADD26B", "fill": "rgba(173,210,107,.10)" },
  { "code": "GBA10", "name": "Tenth Floor", "description": "Gross Building Area", "group": "GBA", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#C9E274", "fill": "rgba(201,226,116,.10)" },
  { "code": "GAR", "name": "Garage", "description": "Garage", "group": "GAR", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#805AD5", "fill": "rgba(128,90,213,.10)" },
  { "code": "GAR2", "name": "Carport", "description": "Carport", "group": "GAR", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#9F7AEA", "fill": "rgba(159,122,234,.10)" },
  { "code": "BSMT", "name": "Finished BSMT", "description": "Basement", "group": "BSMT", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#4A5568", "fill": "rgba(74,85,104,.10)" },
  { "code": "BSMT2", "name": "Unfinished BSMT", "description": "Basement", "group": "BSMT", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#718096", "fill": "rgba(113,128,150,.10)" },
  { "code": "OFP", "name": "Open Frame Porch", "description": "Porch/Patio", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#B7791F", "fill": "rgba(183,121,31,.10)" },
  { "code": "SFP", "name": "Screen Frame Porch", "description": "Porch/Patio", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#D69E2E", "fill": "rgba(214,158,46,.10)" },
  { "code": "EFP", "name": "Enclosed Frame Porch", "description": "Porch/Patio", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#ECC94B", "fill": "rgba(236,201,75,.10)" },
  { "code": "PAT", "name": "Patio", "description": "Porch/Patio", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#DD6B20", "fill": "rgba(221,107,32,.10)" },
  { "code": "PORCH", "name": "Porch", "description": "Porch/Patio", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#D69E2E", "fill": "rgba(214,158,46,.10)" },
  { "code": "DK", "name": "Wood Deck", "description": "Deck", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#C05621", "fill": "rgba(192,86,33,.10)" },
  { "code": "BAL", "name": "Balcony", "description": "Porch/Patio", "group": "P/P", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#B7791F", "fill": "rgba(183,121,31,.10)" },
  { "code": "POOL", "name": "Pool", "description": "Pool", "group": "OTH", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#00A3C4", "fill": "rgba(0,163,196,.10)" },
  { "code": "SHED", "name": "Shed", "description": "Other", "group": "OTH", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#A0AEC0", "fill": "rgba(160,174,192,.10)" },
  { "code": "LAND", "name": "Land", "description": "Land", "group": "LAND", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#68D391", "fill": "rgba(104,211,145,.10)" },
  { "code": "SITE", "name": "Subject Site", "description": "Subject Site", "group": "SITE", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#4FD1C5", "fill": "rgba(79,209,197,.10)" },
  { "code": "OTH", "name": "Storage", "description": "Other", "group": "OTH", "multiplier": 1, "positive": true, "includeInGLA": false, "color": "#A0AEC0", "fill": "rgba(160,174,192,.10)" },
  { "code": "NCA", "name": "Non-Calculated Area", "description": "Non-Calculated Area", "group": "NCA", "multiplier": 0, "positive": true, "includeInGLA": false, "color": "#718096", "fill": "rgba(113,128,150,.08)" },
  { "code": "UND", "name": "Undefined Area", "description": "Undefined Area", "group": "UND", "multiplier": 0, "positive": true, "includeInGLA": false, "color": "#E53E3E", "fill": "rgba(229,62,62,.08)" }
]"##;

    /// `DEFAULT_AREA_CODES` — the parsed verbatim catalogue.
    pub fn default_area_codes() -> Vec<AreaCode> {
        serde_json::from_str(DEFAULT_AREA_CODES_JSON).expect("verbatim default area codes parse")
    }

    /// `cloneAreaCodes` — deep copy (the JSON round-trip analog).
    pub fn clone_area_codes(codes: &[AreaCode]) -> Vec<AreaCode> {
        codes.to_vec()
    }

    /// Below-grade rollup for the suite.
    pub fn basement_rollup_from_totals(
        totals_by_code: Option<&std::collections::BTreeMap<String, f64>>,
    ) -> BasementRollup {
        let pick = |key: &str| -> f64 {
            let raw = totals_by_code
                .and_then(|codes| codes.get(key))
                .copied()
                .unwrap_or(0.0);
            // JS: Math.max(0, Math.round(Number(codes[key]) || 0))
            let rounded = if raw.is_nan() { 0.0 } else { raw.js_round() };
            rounded.max(0.0)
        };
        let finished = pick("BSMT");
        let unfinished = pick("BSMT2");
        BasementRollup {
            total: finished + unfinished,
            finished,
            unfinished,
        }
    }

    /// `normalizeAreaCode` — trimming, defaults and clamping.
    pub fn normalize_area_code(entry: AreaCode) -> AreaCode {
        let code = entry.code.trim().to_uppercase();
        let name = {
            let n = entry.name.trim();
            if n.is_empty() {
                code.clone()
            } else {
                n.to_string()
            }
        };
        let description = {
            let d = entry.description.trim();
            if d.is_empty() {
                name.clone()
            } else {
                d.to_string()
            }
        };
        let group = {
            let g = entry.group.trim().to_uppercase();
            if g.is_empty() {
                "OTH".to_string()
            } else {
                g
            }
        };
        let multiplier = if entry.multiplier.is_finite() {
            entry.multiplier
        } else {
            1.0
        };
        let fill_style = if matches!(
            entry.fill_style.as_str(),
            "solid" | "none" | "diagonal" | "crosshatch" | "dots"
        ) {
            entry.fill_style
        } else {
            "solid".to_string()
        };
        let fill_color = {
            let fc = entry.fill_color.trim();
            if fc.is_empty() {
                if entry.color.is_empty() {
                    default_color()
                } else {
                    entry.color.clone()
                }
            } else {
                fc.to_string()
            }
        };
        AreaCode {
            code,
            name,
            description,
            group,
            multiplier,
            positive: entry.positive,
            include_in_gla: entry.include_in_gla,
            color: if entry.color.is_empty() {
                default_color()
            } else {
                entry.color
            },
            fill: if entry.fill.is_empty() {
                "rgba(113,128,150,.10)".to_string()
            } else {
                entry.fill
            },
            fill_color,
            fill_style,
            fill_opacity: entry.fill_opacity.clamp(0.0, 100.0),
        }
    }

    /// JS `/^[A-Z0-9][A-Z0-9/_-]{0,15}$/` — no regex crate, checked by hand.
    fn valid_code(code: &str) -> bool {
        let chars: Vec<char> = code.chars().collect();
        if chars.is_empty() || chars.len() > 16 {
            return false;
        }
        chars.iter().enumerate().all(|(i, c)| match c {
            'A'..='Z' | '0'..='9' => true,
            '_' | '/' | '-' => i > 0,
            _ => false,
        })
    }

    /// `validateAreaCodes` — normalize then reject bad codes, duplicates and
    /// negative multipliers; at least one code is required.
    pub fn validate_area_codes(input: Vec<AreaCode>) -> Result<Vec<AreaCode>, AreaCodesError> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::with_capacity(input.len());
        for (index, entry) in input.into_iter().enumerate() {
            let normalized = normalize_area_code(entry);
            if !valid_code(&normalized.code) {
                return Err(AreaCodesError::InvalidCode { index });
            }
            if seen.contains(&normalized.code) {
                return Err(AreaCodesError::DuplicateCode {
                    code: normalized.code.clone(),
                });
            }
            if !normalized.multiplier.is_finite() || normalized.multiplier < 0.0 {
                return Err(AreaCodesError::InvalidMultiplier {
                    code: normalized.code.clone(),
                });
            }
            seen.insert(normalized.code.clone());
            result.push(normalized);
        }
        if result.is_empty() {
            return Err(AreaCodesError::Empty);
        }
        Ok(result)
    }

    /// `fromJSON` — parse then validate.
    pub fn from_json(text: &str) -> Result<Vec<AreaCode>, AreaCodesError> {
        let parsed: Result<Vec<AreaCode>, _> = serde_json::from_str(text);
        match parsed {
            Ok(codes) => validate_area_codes(codes),
            Err(_) => Err(AreaCodesError::NotAnArray),
        }
    }

    /// `toJSON` — validate then pretty-print with two-space indent.
    pub fn to_json(codes: Vec<AreaCode>) -> Result<String, AreaCodesError> {
        let validated = validate_area_codes(codes)?;
        serde_json::to_string_pretty(&validated).map_err(|_| AreaCodesError::NotAnArray)
    }

    /// `asMap` — code-keyed lookup (BTreeMap for deterministic order).
    pub fn as_map(codes: &[AreaCode]) -> std::collections::BTreeMap<String, AreaCode> {
        codes
            .iter()
            .map(|entry| (entry.code.clone(), entry.clone()))
            .collect()
    }
}

pub use area_codes::{
    as_map, basement_rollup_from_totals, clone_area_codes, default_area_codes, from_json,
    normalize_area_code, to_json, validate_area_codes, DEFAULT_AREA_CODES_JSON,
};

// --- Oracle-derived tests ---
//
// Numeric expectations ported from the behavioral oracle, the original
// project's tests (not in this repository) — the
// sketch-*.test.mjs files drive `refs/geometry-engine.js` in a Node vm and
// every value below was captured from that engine directly.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_round_matches_math_round() {
        // Math.round(-2.5) = -2, Math.round(2.5) = 3, Math.round(-0.5) = -0,
        // Math.round(0.49999999999999994) = 0.
        assert_eq!((-2.5f64).js_round(), -2.0);
        assert_eq!(2.5f64.js_round(), 3.0);
        assert_eq!((-0.5f64).js_round(), 0.0);
        assert_eq!(0.499_999_999_999_999_94_f64.js_round(), 0.0);
        assert_eq!((-11.5f64).js_round(), -11.0);
        let geo = GeometryEngine::new();
        assert_eq!(
            geo.snap_to_grid(Pt::new(-11.5, 2.5), 1.0),
            Pt::new(-11.0, 3.0)
        );
    }

    fn g() -> GeometryEngine {
        GeometryEngine::new()
    }

    fn edge(bow: f64) -> Edge {
        Edge {
            line_id: None,
            bow_angle: bow,
        }
    }

    fn rect() -> Vec<Pt> {
        vec![
            Pt::new(0.0, 0.0),
            Pt::new(20.0, 0.0),
            Pt::new(20.0, 10.0),
            Pt::new(0.0, 10.0),
        ]
    }

    fn bowed_edges(bow: f64) -> Vec<Edge> {
        vec![edge(bow), edge(0.0), edge(0.0), edge(0.0)]
    }

    fn near(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "expected {b}, got {a}");
    }

    // --- Areas, perimeters, and bowed edges ---

    #[test]
    fn rectangle_area_and_perimeter_match_the_shoelace() {
        let r = rect();
        let e: Vec<Edge> = vec![Edge::default(); 4];
        near(g().calculate_area(&r, &e), 200.0);
        near(g().calculate_signed_area(&r), 200.0);
        near(g().calculate_perimeter(&r, &e), 60.0);
        // With no edges at all it stays the familiar shoelace figure.
        near(g().calculate_area(&r, &[]), 200.0);
        near(g().calculate_perimeter(&r, &[]), 60.0);
        near(g().calculate_area(&r[..2], &[]), 0.0);
    }

    #[test]
    fn curved_area_opposes_the_bow_sign_in_both_directions() {
        let r = rect();
        // +90 (outward for this CCW ring): the segment SUBTRACTS.
        near(
            g().calculate_signed_curved_area(&r, &bowed_edges(90.0)),
            142.9203673205,
        );
        near(g().calculate_area(&r, &bowed_edges(90.0)), 142.9203673205);
        // -90: the segment ADDS. calculate_area is the absolute either way.
        near(
            g().calculate_signed_curved_area(&r, &bowed_edges(-90.0)),
            257.0796326795,
        );
        near(g().calculate_area(&r, &bowed_edges(-90.0)), 257.0796326795);
        // At ±359 the segment dominates: +359 crosses through negative.
        near(
            g().calculate_signed_curved_area(&r, &bowed_edges(359.0)),
            -4125200.2645092714,
        );
        near(
            g().calculate_area(&r, &bowed_edges(359.0)),
            4125200.2645092714,
        );
        near(
            g().calculate_area(&r, &bowed_edges(-359.0)),
            4125600.2645092714,
        );
    }

    #[test]
    fn perimeter_grows_on_outward_bows_while_gross_area_shrinks() {
        let r = rect();
        let straight: Vec<Edge> = vec![Edge::default(); 4];
        // Perimeter: arc length exceeds the chord.
        near(g().calculate_perimeter(&r, &straight), 60.0);
        near(
            g().calculate_perimeter(&r, &bowed_edges(90.0)),
            62.2144146908,
        );
        // Same arc length in the other direction; gross area shrinks.
        near(
            g().calculate_perimeter(&r, &bowed_edges(-90.0)),
            62.2144146908,
        );
        near(g().calculate_area(&r, &bowed_edges(90.0)), 142.9203673205);
        near(g().calculate_area(&r, &bowed_edges(-90.0)), 257.0796326795);
    }

    #[test]
    fn circular_segment_and_arc_length_match_the_oracle() {
        let a = Pt::new(0.0, 0.0);
        let b = Pt::new(20.0, 0.0);
        near(g().circular_segment_area(a, b, 90.0), -57.0796326795);
        near(g().arc_length(a, b, 90.0), 22.2144146908);
        near(g().circular_segment_area(a, b, 359.0), -4125400.2645092714);
        near(g().arc_length(a, b, 359.0), 7180.0911321874);
        // Zero bow and zero chord stay zero / fall back to the chord.
        near(g().circular_segment_area(a, b, 0.0), 0.0);
        near(g().arc_length(a, a, 90.0), 0.0);
        near(g().arc_length(a, b, 0.0), 20.0);
    }

    #[test]
    fn max_bow_angle_boundary_is_359_reject_not_clamp() {
        let ge = g();
        // ±359 accepted; ±359.5 and 360 rejected.
        assert_eq!(ge.validate_bow_angle(359.0).unwrap(), 359.0);
        assert_eq!(ge.validate_bow_angle(-359.0).unwrap(), -359.0);
        assert_eq!(ge.validate_bow_angle(0.0).unwrap(), 0.0);
        assert!(ge.validate_bow_angle(359.5).is_err());
        assert!(ge.validate_bow_angle(-359.5).is_err());
        assert!(ge.validate_bow_angle(360.0).is_err());
        assert!(ge.validate_bow_angle(400.0).is_err());
        assert!(ge.validate_bow_angle(-720.0).is_err());
        // The clamp never rejects.
        near(ge.clamp_bow_angle(359.0), 359.0);
        near(ge.clamp_bow_angle(-359.0), -359.0);
        near(ge.clamp_bow_angle(359.5), 359.0);
        near(ge.clamp_bow_angle(360.0), 359.0);
        near(ge.clamp_bow_angle(400.0), 359.0);
        near(ge.clamp_bow_angle(-720.0), -359.0);
        near(ge.clamp_bow_angle(f64::NAN), 0.0);
    }

    #[test]
    fn flatten_ring_follows_draw_arc_sampling() {
        let r = rect();
        let flat = g().flatten_ring(&r, &bowed_edges(90.0), 0.01);
        assert_eq!(flat.len(), 24);
        assert_eq!(flat[0], Pt::new(0.0, 0.0));
        assert_eq!(*flat.last().unwrap(), Pt::new(0.0, 10.0));
        assert!(flat[1].x - 0.7752629640524411 < 1e-12);
        assert!(flat[1].y - 0.7193389076760415 < 1e-12);
        // The sign flips the bulge to the other side of the chord.
        let flat_neg = g().flatten_ring(&r, &bowed_edges(-90.0), 0.01);
        assert!(flat_neg[1].y + 0.7193389076760415 < 1e-12);
        // A tighter sagitta bound samples more segments.
        assert_eq!(g().flatten_ring(&r, &bowed_edges(90.0), 0.001).len(), 70);
        // Flattening a +90 ring and re-measuring reproduces the curved area.
        let ring = g().flatten_ring(&r, &bowed_edges(90.0), 1e-9);
        assert!((g().calculate_signed_area(&ring) - 142.9204).abs() < 1e-3);
        // Straight edges pass through, degenerate rings are cloned.
        assert_eq!(g().flatten_ring(&r, &bowed_edges(0.0), 0.01).len(), 4);
        assert_eq!(g().flatten_ring(&[Pt::new(1.0, 1.0)], &[], 0.01).len(), 1);
    }

    // --- Measure tool oracle (sketch-measure-tool.test.mjs) ---

    #[test]
    fn bearings_read_clockwise_from_straight_up() {
        let origin = Pt::new(0.0, 0.0);
        near(g().bearing_degrees(origin, Pt::new(0.0, 10.0)), 0.0);
        near(g().bearing_degrees(origin, Pt::new(10.0, 0.0)), 90.0);
        near(g().bearing_degrees(origin, Pt::new(0.0, -10.0)), 180.0);
        near(g().bearing_degrees(origin, Pt::new(-10.0, 0.0)), 270.0);
        near(g().bearing_degrees(origin, Pt::new(10.0, 10.0)), 45.0);
    }

    #[test]
    fn angle_lock_snaps_to_the_nearest_45_without_changing_reach() {
        let anchor = Pt::new(5.0, 5.0);
        let nearly_east = g().constrain_to_angle_step(anchor, Pt::new(15.0, 6.0), 45.0);
        assert!(
            (nearly_east.y - anchor.y).abs() < 1e-9,
            "a shallow leg flattens onto the axis"
        );
        near(g().distance(anchor, nearly_east), (10.0f64).hypot(1.0));

        let nearly_diagonal = g().constrain_to_angle_step(anchor, Pt::new(15.0, 14.0), 45.0);
        assert!((nearly_diagonal.x - anchor.x - (nearly_diagonal.y - anchor.y)).abs() < 1e-9);

        let degenerate = g().constrain_to_angle_step(anchor, anchor, 45.0);
        assert_eq!(degenerate, Pt::new(5.0, 5.0));
    }

    #[test]
    fn corner_angles_report_the_turn_between_two_legs() {
        near(
            g().corner_angle(Pt::new(0.0, 10.0), Pt::new(0.0, 0.0), Pt::new(10.0, 0.0))
                .unwrap(),
            90.0,
        );
        near(
            g().corner_angle(Pt::new(-5.0, 0.0), Pt::new(0.0, 0.0), Pt::new(10.0, 0.0))
                .unwrap(),
            180.0,
        );
        assert_eq!(
            g().corner_angle(Pt::new(0.0, 0.0), Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)),
            None
        );
    }

    #[test]
    fn an_open_run_reports_each_leg_its_bearing_and_the_running_total() {
        let m = g().polyline_metrics(
            &[Pt::new(0.0, 0.0), Pt::new(12.0, 0.0), Pt::new(12.0, 9.0)],
            false,
        );
        assert_eq!(m.segments.len(), 2);
        near(m.segments[0].length, 12.0);
        near(m.segments[0].bearing, 90.0);
        near(m.segments[1].cumulative, 21.0);
        near(m.total, 21.0);
        assert!(!m.closed);
        near(m.area, 0.0);
    }

    #[test]
    fn a_closed_run_adds_the_return_leg_and_the_area_it_encloses() {
        let m = g().polyline_metrics(&rect(), true);
        assert_eq!(m.segments.len(), 4, "the closing leg is measured too");
        near(m.total, 60.0);
        near(m.area, 200.0);
        assert!(m.closed);
    }

    #[test]
    fn two_points_cannot_enclose_anything_closed_or_not() {
        let m = g().polyline_metrics(&[Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)], true);
        assert!(!m.closed);
        near(m.area, 0.0);
        assert_eq!(m.segments.len(), 1);
        let empty = g().polyline_metrics(&[], false);
        assert_eq!(empty.segments.len(), 0);
        near(empty.total, 0.0);
        assert!(!empty.closed);
        near(empty.area, 0.0);
    }

    // --- Polar snap oracle (sketch-apex-drawing-engine.test.mjs) ---

    #[test]
    fn polar_snap_rounds_to_5_degree_bearings_and_whole_foot_lengths() {
        // Apex read 33.0' with a 31.0 run and 11.3 rise for this pointer.
        let snapped = g()
            .polar_snap(Pt::new(0.0, 0.0), Pt::new(31.4, 11.4), 1.0, 5.0)
            .unwrap();
        assert!(snapped.x.hypot(snapped.y) - 33.0 < 1e-3);
        assert!((snapped.y.atan2(snapped.x) * 180.0 / std::f64::consts::PI - 20.0).abs() < 1e-3);
        assert!((snapped.x - 31.0).abs() < 0.05);
        assert!((snapped.y - 11.3).abs() < 0.05);

        // A nearly horizontal pointer lands exactly on the axis, no residue.
        assert_eq!(
            g().polar_snap(Pt::new(3.0, 4.0), Pt::new(43.2, 4.6), 1.0, 5.0),
            Some(Pt::new(43.0, 4.0))
        );
        // An off-grid anchor (after an angled wall) still yields whole feet.
        let from_angled = g()
            .polar_snap(Pt::new(2.8284, 2.8284), Pt::new(12.6, 3.1), 1.0, 5.0)
            .unwrap();
        near((from_angled.x - 2.8284).hypot(from_angled.y - 2.8284), 10.0);
        near(from_angled.y, 2.8284);
        // Too close to the anchor to define a length.
        assert_eq!(
            g().polar_snap(Pt::new(0.0, 0.0), Pt::new(0.2, 0.1), 1.0, 5.0),
            None
        );
    }

    // --- Label anchors ---

    #[test]
    fn area_labels_use_the_centroid_of_regular_outlines() {
        let rectangle = [
            Pt::new(0.0, 0.0),
            Pt::new(40.0, 0.0),
            Pt::new(40.0, 30.0),
            Pt::new(0.0, 30.0),
        ];
        assert_eq!(g().area_label_anchor(&rectangle), Pt::new(20.0, 15.0));
        assert_eq!(
            g().area_label_anchor(&[Pt::new(1.0, 2.0), Pt::new(3.0, 4.0)]),
            Pt::new(1.0, 2.0)
        );
    }

    #[test]
    fn an_l_shaped_outline_uses_its_most_interior_point() {
        let ell = [
            Pt::new(0.0, 0.0),
            Pt::new(10.0, 0.0),
            Pt::new(10.0, 30.0),
            Pt::new(30.0, 30.0),
            Pt::new(30.0, 40.0),
            Pt::new(0.0, 40.0),
        ];
        let anchor = g().area_label_anchor(&ell);
        assert!(
            g().point_in_polygon(anchor, &ell),
            "L label anchor is inside the outline"
        );
        assert!(
            g().boundary_distance(anchor, &ell) >= 4.5,
            "L label anchor keeps clear of its walls"
        );
        // Oracle: polylabel lands at this refined cell.
        near(anchor.x, 5.830078125);
        near(anchor.y, 34.130859375);
        // The pole of inaccessibility reports its clearance too.
        let pole = g().pole_of_inaccessibility(&ell, 0.05).unwrap();
        near(pole.distance, 5.830078125);
        // Degenerate rings have no pole.
        assert_eq!(
            g().pole_of_inaccessibility(&[Pt::new(0.0, 0.0), Pt::new(0.0, 0.0)], 0.05),
            None
        );
    }

    // --- Centroids and pins ---

    #[test]
    fn polygon_centroid_falls_back_for_degenerate_rings() {
        assert_eq!(g().polygon_centroid(&rect()), Pt::new(10.0, 5.0));
        assert_eq!(g().polygon_centroid(&[]), Pt::new(0.0, 0.0));
        assert_eq!(
            g().polygon_centroid(&[Pt::new(1.0, 2.0), Pt::new(3.0, 4.0)]),
            Pt::new(1.0, 2.0)
        );
        assert_eq!(
            g().polygon_centroid(&[Pt::new(0.0, 0.0), Pt::new(5.0, 0.0), Pt::new(10.0, 0.0)]),
            Pt::new(5.0, 0.0)
        );
    }

    #[test]
    fn a_dragged_dimension_label_follows_its_wall_through_later_moves() {
        let mut line = Line::new(1, Pt::new(0.0, 0.0), Pt::new(20.0, 0.0));
        line.dimension_position = Some(Pt::new(10.0, 3.0));
        line.dimension_anchor = Some(Pt::new(10.0, 0.0));
        line.start = Pt::new(15.0, -5.0);
        line.end = Pt::new(35.0, -5.0);
        assert_eq!(g().pinned_dimension_point(&line), Some(Pt::new(25.0, -2.0)));
        // A line with no pin has nowhere to draw.
        assert_eq!(
            g().pinned_dimension_point(&Line::new(2, Pt::new(0.0, 0.0), Pt::new(1.0, 0.0))),
            None
        );
        // Legacy pins (no anchor) draw where they were placed.
        let mut legacy = Line::new(3, Pt::new(0.0, 0.0), Pt::new(20.0, 0.0));
        legacy.dimension_position = Some(Pt::new(10.0, 3.0));
        assert_eq!(
            g().pinned_dimension_point(&legacy),
            Some(Pt::new(10.0, 3.0))
        );
    }

    // --- Point tests, snapping, alignment ---

    #[test]
    fn point_in_polygon_and_boundary() {
        let r = rect();
        assert!(g().point_in_polygon(Pt::new(10.0, 5.0), &r));
        assert!(!g().point_in_polygon(Pt::new(25.0, 5.0), &r));
        assert!(g().point_on_boundary(Pt::new(20.0, 5.0), &r, 1e-4));
        assert!(!g().point_on_boundary(Pt::new(20.0005, 5.0), &r, 1e-4));
        assert!(!g().point_on_boundary(Pt::new(20.0005, 5.0), &r[..1], 1e-4));
    }

    #[test]
    fn snap_grid_and_coordinate_conversion_round_trip() {
        assert_eq!(
            g().snap_to_grid(Pt::new(9.6, -1.4), 1.0),
            Pt::new(10.0, -1.0)
        );
        assert_eq!(
            g().snap_to_grid(Pt::new(9.6, -1.4), 0.5),
            Pt::new(9.5, -1.5)
        );
        let logical = g().to_logical(Pt::new(120.0, 80.0), Pt::new(20.0, 100.0), 1.5);
        near(logical.x, 10.0 / 3.0);
        near(logical.y, 2.0 / 3.0);
        let screen = g().to_screen(Pt::new(2.0, -1.0), Pt::new(20.0, 100.0), 1.5);
        assert_eq!(screen, Pt::new(80.0, 130.0));
        // to_world is an alias for to_logical.
        assert_eq!(
            g().to_world(Pt::new(120.0, 80.0), Pt::new(20.0, 100.0), 1.5),
            logical
        );
        assert!(g().point_in_rect(Pt::new(1.0, 1.0), Pt::new(0.0, 0.0), Pt::new(2.0, 2.0)));
        assert!(!g().point_in_rect(Pt::new(3.0, 1.0), Pt::new(0.0, 0.0), Pt::new(2.0, 2.0)));
    }

    #[test]
    fn nearest_endpoint_honors_the_threshold_and_skip() {
        let lines = [
            Line::new(1, Pt::new(10.0, 0.0), Pt::new(20.0, 0.0)),
            Line::new(2, Pt::new(20.0, 0.0), Pt::new(20.0, 10.0)),
        ];
        assert_eq!(
            g().find_nearest_endpoint(Pt::new(10.4, 0.0), &lines, 0.5, None),
            Some(Pt::new(10.0, 0.0))
        );
        let skip = |p: &Pt| p.x == 10.0 && p.y == 0.0;
        assert_eq!(
            g().find_nearest_endpoint(Pt::new(10.4, 0.0), &lines[..1], 0.5, Some(&skip)),
            None
        );
    }

    #[test]
    fn alignment_snaps_in_the_requested_direction() {
        let vertical = [Line::new(1, Pt::new(10.0, 0.0), Pt::new(10.0, 10.0))];
        // Direction 0: vertical movement, look for X alignment.
        assert_eq!(
            g().find_alignment(Pt::new(10.05, 5.0), &vertical, 0.0),
            Some(Pt::new(10.05, 0.0))
        );
        let horizontal = [Line::new(1, Pt::new(0.0, 5.0), Pt::new(10.0, 5.0))];
        // Direction 90: horizontal movement, look for Y alignment.
        assert_eq!(
            g().find_alignment(Pt::new(5.0, 5.05), &horizontal, 90.0),
            Some(Pt::new(0.0, 5.05))
        );
        let crossing = [Line::new(1, Pt::new(0.0, 10.0), Pt::new(10.0, 10.0))];
        assert_eq!(
            g().find_intersecting_alignment(Pt::new(5.0, 4.0), &crossing, 0.0),
            Some(Pt::new(5.0, 10.0))
        );
        assert_eq!(
            g().find_intersecting_alignment(Pt::new(5.0, 14.0), &crossing, 180.0),
            Some(Pt::new(5.0, 10.0))
        );
    }

    #[test]
    fn segment_distance_and_projections_clamp() {
        let a = Pt::new(0.0, 0.0);
        let b = Pt::new(10.0, 0.0);
        near(g().distance_to_segment(Pt::new(5.0, 5.0), a, b), 5.0);
        assert_eq!(
            g().project_point_to_segment(Pt::new(5.0, 5.0), a, b),
            Pt::new(5.0, 0.0)
        );
        // Behind the segment clamps to the endpoint.
        assert_eq!(
            g().project_point_to_segment(Pt::new(-3.0, 5.0), a, b),
            Pt::new(0.0, 0.0)
        );
        // Degenerate segment returns a copy of segA.
        assert_eq!(
            g().project_point_to_segment(Pt::new(5.0, 5.0), Pt::new(2.0, 2.0), Pt::new(2.0, 2.0)),
            Pt::new(2.0, 2.0)
        );
    }

    #[test]
    fn orientation_and_proper_crossings() {
        let origin = Pt::new(0.0, 0.0);
        assert_eq!(
            g().orientation(origin, Pt::new(1.0, 1.0), Pt::new(2.0, 2.0), 1e-9),
            0
        );
        assert_eq!(
            g().orientation(origin, Pt::new(1.0, 0.0), Pt::new(1.0, 1.0), 1e-9),
            1
        );
        assert_eq!(
            g().orientation(origin, Pt::new(1.0, 0.0), Pt::new(0.5, 0.0000000005), 1e-9),
            0
        );
        // Proper crossing, and a shared endpoint that is not a crossing.
        assert!(g().segments_properly_cross(
            Pt::new(0.0, 0.0),
            Pt::new(2.0, 2.0),
            Pt::new(0.0, 2.0),
            Pt::new(2.0, 0.0),
            1e-9
        ));
        assert!(!g().segments_properly_cross(
            Pt::new(0.0, 0.0),
            Pt::new(1.0, 1.0),
            Pt::new(1.0, 1.0),
            Pt::new(2.0, 2.0),
            1e-9
        ));
    }

    // --- Enclosed faces and containment (sketch-area-closure oracle) ---

    #[test]
    fn along_line_t_transcribes_the_browser_formula() {
        let g = || GeometryEngine::default();
        let line = Line::new(1, Pt::new(0.0, 0.0), Pt::new(10.0, 0.0));
        // On the line: 0 at the start, 1 at the end, 0.3 a third of the way.
        assert_eq!(g().along_line_t(&line, Pt::new(0.0, 0.0)), 0.0);
        assert_eq!(g().along_line_t(&line, Pt::new(10.0, 0.0)), 1.0);
        assert_eq!(g().along_line_t(&line, Pt::new(3.0, 0.0)), 0.3);
        // Off the line: a perpendicular through the same spot shares the t.
        assert_eq!(g().along_line_t(&line, Pt::new(3.0, 4.0)), 0.3);
        // Past the end extrapolates (> 1).
        assert_eq!(g().along_line_t(&line, Pt::new(15.0, 0.0)), 1.5);
        // Diagonal line.
        let diag = Line::new(2, Pt::new(0.0, 0.0), Pt::new(8.0, 6.0));
        assert_eq!(g().along_line_t(&diag, Pt::new(4.0, 3.0)), 0.5);
    }

    #[test]
    fn find_enclosed_faces_traces_a_closed_rectangle() {
        let lines = [
            Line::new(1, Pt::new(0.0, 0.0), Pt::new(10.0, 0.0)),
            Line::new(2, Pt::new(10.0, 0.0), Pt::new(10.0, 10.0)),
            Line::new(3, Pt::new(10.0, 10.0), Pt::new(0.0, 10.0)),
            Line::new(4, Pt::new(0.0, 10.0), Pt::new(0.0, 0.0)),
        ];
        let faces = g().find_enclosed_faces(&lines, 1.0, 0.02);
        assert_eq!(faces.len(), 1);
        near(faces[0].area, 100.0);
        assert_eq!(faces[0].points.len(), 4);
        let ids: Vec<i64> = faces[0].edges.iter().map(|e| e.line_id.unwrap()).collect();
        assert_eq!(ids, vec![1, 2, 3, 4]);
        assert!(faces[0].edges.iter().all(|e| e.bow_angle == 0.0));
        assert!(g().find_enclosed_faces(&[], 1.0, 0.02).is_empty());
    }

    #[test]
    fn t_junctions_split_walls_into_two_faces() {
        let house = [
            Line::new(1, Pt::new(0.0, 0.0), Pt::new(40.0, 0.0)),
            Line::new(2, Pt::new(40.0, 0.0), Pt::new(40.0, 30.0)),
            Line::new(3, Pt::new(40.0, 30.0), Pt::new(0.0, 30.0)),
            Line::new(4, Pt::new(0.0, 30.0), Pt::new(0.0, 0.0)),
            Line::new(5, Pt::new(20.0, 0.0), Pt::new(20.0, 30.0)),
        ];
        let faces = g().find_enclosed_faces(&house, 1.0, 0.02);
        assert_eq!(faces.len(), 2);
        for face in &faces {
            near(face.area, 600.0);
        }
        let ids: Vec<Vec<i64>> = faces
            .iter()
            .map(|f| f.edges.iter().map(|e| e.line_id.unwrap()).collect())
            .collect();
        assert!(ids.contains(&vec![1, 5, 3, 4]));
        assert!(ids.contains(&vec![1, 2, 3, 5]));
    }

    #[test]
    fn polygon_contains_inner_outside_and_straddling() {
        let square = [
            Pt::new(0.0, 0.0),
            Pt::new(10.0, 0.0),
            Pt::new(10.0, 10.0),
            Pt::new(0.0, 10.0),
        ];
        let outer = (square, vec![Edge::default(); 4]);
        let inside = (
            vec![
                Pt::new(2.0, 2.0),
                Pt::new(4.0, 2.0),
                Pt::new(4.0, 4.0),
                Pt::new(2.0, 4.0),
            ],
            vec![Edge::default(); 4],
        );
        let outside = (
            vec![
                Pt::new(12.0, 2.0),
                Pt::new(14.0, 2.0),
                Pt::new(14.0, 4.0),
                Pt::new(12.0, 4.0),
            ],
            vec![Edge::default(); 4],
        );
        let straddle = (
            vec![
                Pt::new(8.0, 2.0),
                Pt::new(12.0, 2.0),
                Pt::new(12.0, 4.0),
                Pt::new(8.0, 4.0),
            ],
            vec![Edge::default(); 4],
        );
        assert!(g().polygon_contains(&outer.0, &outer.1, &inside.0, &inside.1, 1e-4, 0.01));
        assert!(!g().polygon_contains(&outer.0, &outer.1, &outside.0, &outside.1, 1e-4, 0.01));
        assert!(!g().polygon_contains(&outer.0, &outer.1, &straddle.0, &straddle.1, 1e-4, 0.01));
    }

    // --- Dimension chains ---

    fn house() -> Vec<Line> {
        vec![
            Line::new(1, Pt::new(0.0, 0.0), Pt::new(40.0, 0.0)),
            Line::new(2, Pt::new(40.0, 0.0), Pt::new(40.0, 30.0)),
            Line::new(3, Pt::new(40.0, 30.0), Pt::new(0.0, 30.0)),
            Line::new(4, Pt::new(0.0, 30.0), Pt::new(0.0, 0.0)),
            Line::new(5, Pt::new(20.0, 0.0), Pt::new(20.0, 30.0)),
        ]
    }

    #[test]
    fn dimension_chain_splits_at_junctions_and_gaps() {
        let ge = g();
        let chain = ge
            .compute_dimension_chain(&house(), &[], 1, 0.25, true, None)
            .unwrap();
        assert_eq!(chain.line_ids, vec![1]);
        assert_eq!(chain.start, Pt::new(0.0, 0.0));
        assert_eq!(chain.end, Pt::new(40.0, 0.0));
        assert_eq!(chain.dir, Pt::new(1.0, 0.0));
        assert_eq!(chain.normal, Pt::new(0.0, 1.0));
        assert_eq!(chain.refs, vec![0.0, 20.0, 40.0]);
        near(chain.length, 40.0);
        // Without splitting at junctions only the endpoints remain.
        let chain2 = ge
            .compute_dimension_chain(&house(), &[], 1, 0.25, false, None)
            .unwrap();
        assert_eq!(chain2.refs, vec![0.0, 40.0]);
        // A missing or degenerate seed yields nothing.
        assert_eq!(
            ge.compute_dimension_chain(&house(), &[], 99, 0.25, true, None),
            None
        );
    }

    #[test]
    fn dimension_chain_adds_hosted_opening_gap_edges() {
        let ge = g();
        let syms = [DimensionSymbol {
            x: 10.0,
            y: 0.0,
            wall_id: Some(1),
            wall_gap_width: 3.0,
            wall_snap: None,
            wall_side_flip: false,
            flip_h: false,
            symbol_type: Some("DOOR".to_string()),
        }];
        let chain = ge
            .compute_dimension_chain(&house(), &syms, 1, 0.25, true, None)
            .unwrap();
        assert_eq!(chain.refs, vec![0.0, 8.5, 11.5, 20.0, 40.0]);
    }

    #[test]
    fn dimension_chain_rejects_a_bowed_seed() {
        let mut bowed = house();
        bowed.push(Line::new(6, Pt::new(0.0, 35.0), Pt::new(40.0, 35.0)));
        bowed[5].bow_angle = 12.0;
        assert_eq!(
            g().compute_dimension_chain(&bowed, &[], 6, 0.25, true, None),
            None
        );
    }

    // --- Area codes (the ref file's AreaCodes module) ---

    #[test]
    fn default_area_codes_parse_to_32_verified_entries() {
        let codes = default_area_codes();
        assert_eq!(codes.len(), 32);
        let first = &codes[0];
        assert_eq!(first.code, "GLA1");
        assert_eq!(first.name, "First Floor");
        assert_eq!(first.description, "Gross Living Area");
        assert_eq!(first.group, "GLA");
        near(first.multiplier, 1.0);
        assert!(first.positive && first.include_in_gla);
        assert_eq!(first.color, "#2B6CB0");
        // Normalization applies the defaults the JS module applies.
        let nca = normalize_area_code(codes.iter().find(|c| c.code == "NCA").unwrap().clone());
        assert_eq!(nca.fill_color, "#718096");
        assert_eq!(nca.fill_style, "solid");
        near(nca.fill_opacity, 10.0);
        near(nca.multiplier, 0.0);
    }

    #[test]
    fn basement_rollup_sums_both_basement_codes() {
        let mut totals = std::collections::BTreeMap::new();
        totals.insert("BSMT".to_string(), 100.4);
        totals.insert("BSMT2".to_string(), 20.6);
        let r = basement_rollup_from_totals(Some(&totals));
        near(r.finished, 100.0);
        near(r.unfinished, 21.0);
        near(r.total, 121.0);
        let mut negative = std::collections::BTreeMap::new();
        negative.insert("BSMT".to_string(), -5.0);
        let r = basement_rollup_from_totals(Some(&negative));
        near(r.total, 0.0);
        let r = basement_rollup_from_totals(None);
        near(r.total, 0.0);
    }

    #[test]
    fn area_code_validation_rejects_bad_codes_duplicates_and_multipliers() {
        assert!(matches!(
            validate_area_codes(vec![AreaCode {
                code: "BAD CODE".into(),
                ..Default::default()
            }]),
            Err(AreaCodesError::InvalidCode { index: 0 })
        ));
        assert!(matches!(
            validate_area_codes(vec![
                AreaCode {
                    code: "GLA1".into(),
                    ..Default::default()
                },
                AreaCode {
                    code: "GLA1".into(),
                    ..Default::default()
                },
            ]),
            Err(AreaCodesError::DuplicateCode { .. })
        ));
        assert!(matches!(
            validate_area_codes(vec![AreaCode {
                code: "OK1".into(),
                multiplier: -1.0,
                ..Default::default()
            }]),
            Err(AreaCodesError::InvalidMultiplier { .. })
        ));
        assert!(matches!(
            validate_area_codes(vec![]),
            Err(AreaCodesError::Empty)
        ));
        assert!(matches!(from_json("nope"), Err(AreaCodesError::NotAnArray)));
    }

    #[test]
    fn normalize_clamps_fill_and_defaults_the_rest() {
        let n = normalize_area_code(AreaCode {
            code: "x1".into(),
            multiplier: 2.0,
            fill_opacity: 150.0,
            fill_style: "stripes".into(),
            ..Default::default()
        });
        assert_eq!(n.code, "X1");
        assert_eq!(n.name, "X1");
        assert_eq!(n.group, "OTH");
        near(n.multiplier, 2.0);
        assert!(n.positive);
        assert!(!n.include_in_gla);
        assert_eq!(n.fill_style, "solid");
        near(n.fill_opacity, 100.0);
        assert_eq!(n.fill_color, "#718096");
    }

    #[test]
    fn area_code_json_interchange_round_trips() {
        let json = to_json(vec![AreaCode {
            code: "OK1".into(),
            name: "Ok".into(),
            description: "Ok".into(),
            group: "GLA".into(),
            multiplier: 2.0,
            ..Default::default()
        }])
        .unwrap();
        let back = from_json(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].code, "OK1");
        near(back[0].multiplier, 2.0);
        let map = as_map(&[AreaCode {
            code: "GAR".into(),
            name: "Garage".into(),
            ..Default::default()
        }]);
        assert_eq!(map.get("GAR").unwrap().name, "Garage");
        assert!(as_map(&[]).is_empty());
    }
}
