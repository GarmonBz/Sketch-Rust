//! `AreaBreakdown` — the area-breakdown report's computation and text
//! (screen 7, S7-003), ported from `refs/area-breakdown.js`.
//!
//! Every area decomposes into labelled pieces: a rectilinear ring splits into
//! merged horizontal bands, a near-rectilinear ring squares off and deducts
//! the corner triangles, anything else ear-clips into triangles, and each
//! bowed edge contributes a circular segment. The pieces must reconcile to
//! the area within `minPieceArea`; a residual lands as a `balance` piece.

use crate::geometry::{Edge, Pt};

pub const EPS: f64 = 1e-6;

/// The report's defaults (`new AreaBreakdown(config).options`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// `decimal` or `feetinches`.
    pub dimension_style: &'static str,
    pub precision: usize,
    pub area_precision: usize,
    pub min_piece_area: f64,
    pub merge_rectangles: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            dimension_style: "decimal",
            precision: 1,
            area_precision: 2,
            min_piece_area: 0.005,
            merge_rectangles: true,
        }
    }
}

/// One decomposed piece of an area.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    /// `Rectangle` / `Triangle` / `CircularSegment` / `Polygon`.
    pub shape: &'static str,
    pub area: f64,
    /// Rectangle dimensions.
    pub w: f64,
    pub h: f64,
    /// Triangle dimensions.
    pub base: f64,
    pub height: f64,
    /// Circular-segment dimensions.
    pub radius: f64,
    pub angle: f64,
    /// A `balance` piece closes the gap to the stored gross.
    pub balancing: bool,
}

impl Piece {
    fn rect(w: f64, h: f64) -> Piece {
        Piece {
            shape: "Rectangle",
            area: w * h,
            w,
            h,
            base: 0.0,
            height: 0.0,
            radius: 0.0,
            angle: 0.0,
            balancing: false,
        }
    }
    fn tri(base: f64, height: f64) -> Piece {
        Piece {
            shape: "Triangle",
            area: base * height / 2.0,
            w: 0.0,
            h: 0.0,
            base,
            height,
            radius: 0.0,
            angle: 0.0,
            balancing: false,
        }
    }
    fn arc(radius: f64, angle: f64, area: f64) -> Piece {
        Piece {
            shape: "CircularSegment",
            area,
            w: 0.0,
            h: 0.0,
            base: 0.0,
            height: 0.0,
            radius,
            angle,
            balancing: false,
        }
    }
    fn balance(area: f64) -> Piece {
        Piece {
            shape: "Polygon",
            area,
            w: 0.0,
            h: 0.0,
            base: 0.0,
            height: 0.0,
            radius: 0.0,
            angle: 0.0,
            balancing: true,
        }
    }
}

/// The per-area breakdown result.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Result {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub container_id: Option<i64>,
    pub subtracted_from_id: Option<i64>,
    pub pieces: Vec<Piece>,
    pub gross: f64,
    pub subtracted: f64,
    pub multiplier: f64,
    pub sign: f64,
    pub net: f64,
    pub perimeter: f64,
    pub reconciled: bool,
}

/// The source values `compute_for_polygon` reads — the fields of a
/// recalculation result plus the ring.
pub struct Source<'a> {
    pub id: i64,
    pub kind: &'a str,
    pub name: &'a str,
    pub points: &'a [Pt],
    pub edges: &'a [Edge],
    pub gross: Option<f64>,
    pub container_id: Option<i64>,
    pub subtracted_from_id: Option<i64>,
    pub subtracted: f64,
    pub multiplier: f64,
    pub sign: f64,
    pub net: Option<f64>,
    pub perimeter: Option<f64>,
}

pub fn signed_area(points: &[Pt]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..n {
        let next = points[(i + 1) % n];
        area += points[i].x * next.y - next.x * points[i].y;
    }
    area / 2.0
}

/// Force counter-clockwise winding, mirroring the bows the way the reversal
/// does (`normalize`).
pub fn normalize(points: &[Pt], bows: &[f64]) -> (Vec<Pt>, Vec<f64>) {
    if points.len() < 2 {
        return (points.to_vec(), bows.to_vec());
    }
    if signed_area(points) >= 0.0 {
        return (points.to_vec(), bows.to_vec());
    }
    let count = points.len();
    let mut pts = points.to_vec();
    pts.reverse();
    let mut reversed_bows = Vec::with_capacity(count);
    for i in 0..count {
        reversed_bows.push(
            -bows
                .get(((count as isize) - 2 - (i as isize)).rem_euclid(count as isize) as usize)
                .copied()
                .unwrap_or(0.0),
        );
    }
    (pts, reversed_bows)
}

/// The circular segment one bowed edge contributes (negative when the bow
/// bulges inward).
pub fn segment_area(p1: Pt, p2: Pt, bow_deg: f64) -> (f64, f64, f64) {
    if bow_deg == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let theta = bow_deg.abs() * std::f64::consts::PI / 180.0;
    let chord = ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt();
    if theta < 1e-9 || chord < EPS {
        return (0.0, 0.0, 0.0);
    }
    let radius = (chord / 2.0) / (theta / 2.0).sin();
    let area = (radius * radius / 2.0) * (theta - theta.sin());
    (-bow_deg.signum() * area, radius, bow_deg.abs())
}

/// The arc length of one bowed edge (its chord when straight).
pub fn edge_length(p1: Pt, p2: Pt, bow_deg: f64) -> f64 {
    let chord = ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt();
    if bow_deg == 0.0 {
        return chord;
    }
    let theta = bow_deg.abs() * std::f64::consts::PI / 180.0;
    if theta < 1e-9 || chord < EPS {
        return chord;
    }
    ((chord / 2.0) / (theta / 2.0).sin()) * theta
}

pub fn is_rectilinear(points: &[Pt]) -> bool {
    let n = points.len();
    points.iter().enumerate().all(|(i, point)| {
        let next = points[(i + 1) % n];
        (next.x - point.x).abs() <= 1e-4 || (next.y - point.y).abs() <= 1e-4
    })
}

struct Rect {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

/// Horizontal band partition of a rectilinear ring, merged into columns.
fn rectilinear_partition(points: &[Pt], merge: bool) -> Vec<Rect> {
    let mut ys: Vec<f64> = points
        .iter()
        .map(|p| format!("{:.6}", p.y).parse::<f64>().unwrap_or(p.y))
        .collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ys.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    let mut rects: Vec<Rect> = Vec::new();
    for band in 0..ys.len().saturating_sub(1) {
        let (y0, y1) = (ys[band], ys[band + 1]);
        if y1 - y0 < EPS {
            continue;
        }
        let middle = (y0 + y1) / 2.0;
        let mut xs: Vec<f64> = Vec::new();
        let n = points.len();
        for i in 0..n {
            let point = points[i];
            let next = points[(i + 1) % n];
            if (point.y > middle) == (next.y > middle) {
                continue;
            }
            let t = (middle - point.y) / (next.y - point.y);
            xs.push(point.x + t * (next.x - point.x));
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut i = 0;
        while i + 1 < xs.len() {
            if xs[i + 1] - xs[i] > EPS {
                rects.push(Rect {
                    x0: xs[i],
                    x1: xs[i + 1],
                    y0,
                    y1,
                });
            }
            i += 2;
        }
    }
    if merge {
        merge_bands(rects)
    } else {
        rects
    }
}

/// Merge vertically adjacent bands that share a column's x span.
fn merge_bands(rects: Vec<Rect>) -> Vec<Rect> {
    // Keyed by the (x0, x1) pair, like the original's string key.
    let mut columns: Vec<((u64, u64), Vec<Rect>)> = Vec::new();
    for rect in rects {
        let key = (key_of(rect.x0), key_of(rect.x1));
        match columns.iter_mut().find(|(k, _)| *k == key) {
            Some((_, column)) => column.push(rect),
            None => columns.push((key, vec![rect])),
        }
    }
    let mut result: Vec<Rect> = Vec::new();
    for (_, mut column) in columns {
        column.sort_by(|a, b| a.y0.partial_cmp(&b.y0).unwrap_or(std::cmp::Ordering::Equal));
        let mut current: Option<Rect> = None;
        for rect in column {
            match current.take() {
                Some(mut c) if (rect.y0 - c.y1).abs() < EPS => {
                    c.y1 = rect.y1;
                    current = Some(c);
                }
                Some(c) => {
                    result.push(c);
                    current = Some(rect);
                }
                None => current = Some(rect),
            }
        }
        if let Some(c) = current {
            result.push(c);
        }
    }
    result
}

fn key_of(value: f64) -> u64 {
    // `${value.toFixed(6)}` as an integer key.
    let scaled = (value * 1e6).round();
    if scaled >= 0.0 {
        scaled as u64
    } else {
        0
    }
}

/// Square off a near-rectilinear ring: each diagonal edge gains the corner
/// that grows the area least, and the corner triangle is deducted.
fn square_off(points: &[Pt], max_diagonals: usize) -> Option<(Vec<Pt>, Vec<Piece>)> {
    let n = points.len();
    let mut diagonals = Vec::new();
    for i in 0..n {
        let next = points[(i + 1) % n];
        if (next.x - points[i].x).abs() > 1e-4 && (next.y - points[i].y).abs() > 1e-4 {
            diagonals.push(i);
        }
    }
    if diagonals.is_empty() || diagonals.len() > max_diagonals {
        return None;
    }
    let mut squared: Vec<Pt> = Vec::new();
    let mut deducts: Vec<Piece> = Vec::new();
    let base_area = signed_area(points);
    for i in 0..n {
        let point = points[i];
        let next = points[(i + 1) % n];
        squared.push(point);
        if (next.x - point.x).abs() < 1e-4 || (next.y - point.y).abs() < 1e-4 {
            continue;
        }
        // The corner that grows the area least.
        let candidates = [
            Pt {
                x: next.x,
                y: point.y,
            },
            Pt {
                x: point.x,
                y: next.y,
            },
        ];
        let mut best: Option<(Pt, f64)> = None;
        for corner in candidates {
            let mut trial: Vec<Pt> = points[..=i].to_vec();
            trial.push(corner);
            trial.extend_from_slice(&points[i + 1..]);
            let delta = signed_area(&trial) - base_area;
            if best.map(|(_, d)| delta > d).unwrap_or(true) {
                best = Some((corner, delta));
            }
        }
        let (corner, delta) = best?;
        if delta <= EPS {
            return None;
        }
        squared.push(corner);
        deducts.push(Piece::tri(
            (next.x - point.x).abs(),
            (next.y - point.y).abs(),
        ));
        // The deducted triangle's area is negative: it comes back off.
        let last = deducts.last_mut().expect("pushed");
        last.area = -((next.x - point.x).abs() * (next.y - point.y).abs()) / 2.0;
    }
    if is_rectilinear(&squared) {
        Some((squared, deducts))
    } else {
        None
    }
}

/// Ear clipping, guarding against the degenerate ring.
fn triangulate(points: &[Pt]) -> Vec<[Pt; 3]> {
    let mut vertices: Vec<Pt> = points.to_vec();
    if signed_area(&vertices) < 0.0 {
        vertices.reverse();
    }
    let mut triangles: Vec<[Pt; 3]> = Vec::new();
    let cross = |a: Pt, b: Pt, c: Pt| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let inside = |p: Pt, a: Pt, b: Pt, c: Pt| {
        cross(a, b, p) >= -EPS && cross(b, c, p) >= -EPS && cross(c, a, p) >= -EPS
    };
    let mut guard = 0usize;
    while vertices.len() > 3 && guard < 10000 {
        guard += 1;
        let mut clipped = false;
        let len = vertices.len();
        for i in 0..len {
            let a = vertices[(i + len - 1) % len];
            let b = vertices[i];
            let c = vertices[(i + 1) % len];
            if cross(a, b, c) <= EPS {
                continue;
            }
            let contains_other = vertices
                .iter()
                .any(|p| *p != a && *p != b && *p != c && inside(*p, a, b, c));
            if contains_other {
                continue;
            }
            triangles.push([a, b, c]);
            vertices.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if vertices.len() == 3 {
        triangles.push([vertices[0], vertices[1], vertices[2]]);
    }
    triangles
}

fn triangle_dims(triangle: &[Pt; 3]) -> (f64, f64) {
    let area = signed_area(triangle).abs();
    let sides = [
        (triangle[0], triangle[1]),
        (triangle[1], triangle[2]),
        (triangle[2], triangle[0]),
    ];
    let mut best: Option<(f64, f64, bool)> = None; // (base, height, axis-aligned)
    for (start, end) in sides {
        let base = ((end.x - start.x).powi(2) + (end.y - start.y).powi(2)).sqrt();
        if base < EPS {
            continue;
        }
        let axis = (end.x - start.x).abs() < 1e-4 || (end.y - start.y).abs() < 1e-4;
        let candidate = (base, 2.0 * area / base, axis);
        let better = best.map(|(_, _, b)| candidate.2 && !b).unwrap_or(true);
        if better {
            best = Some(candidate);
        }
    }
    best.map(|(base, height, _)| (base, height))
        .unwrap_or((0.0, 0.0))
}

/// `computeForPolygon` — decompose one area into labelled pieces.
pub fn compute_for_polygon(src: &Source, options: &Options) -> Result {
    // The original JS returns `[]` for <3 points; mirror it here so a
    // degenerate / stub Source does not underflow the winding pass below.
    if src.points.len() < 3 {
        return Result::default();
    }
    let bows: Vec<f64> = (0..src.points.len())
        .map(|i| src.edges.get(i).map(|e| e.bow_angle).unwrap_or(0.0))
        .collect();
    let (points, edge_bows) = normalize(src.points, &bows);
    let chord_area = signed_area(&points).abs();
    let mut pieces: Vec<Piece> = Vec::new();

    if is_rectilinear(&points) {
        for rect in rectilinear_partition(&points, options.merge_rectangles) {
            let (w, h) = (rect.x1 - rect.x0, rect.y1 - rect.y0);
            if w * h >= options.min_piece_area {
                pieces.push(Piece::rect(w, h));
            }
        }
    } else if points.len() != 3 {
        if let Some((squared, deducts)) = square_off(&points, 4) {
            for rect in rectilinear_partition(&squared, options.merge_rectangles) {
                let (w, h) = (rect.x1 - rect.x0, rect.y1 - rect.y0);
                if w * h >= options.min_piece_area {
                    pieces.push(Piece::rect(w, h));
                }
            }
            pieces.extend(deducts);
            let sum: f64 = pieces.iter().map(|p| p.area).sum();
            if (sum - chord_area).abs() >= 0.01 {
                pieces.clear();
            }
        }
        // A failed square-off falls through to triangles.
        if pieces.is_empty() {
            for triangle in triangulate(&points) {
                let (base, height) = triangle_dims(&triangle);
                let area = base * height / 2.0;
                if area >= options.min_piece_area {
                    pieces.push(Piece::tri(base, height));
                }
            }
        }
    } else {
        for triangle in triangulate(&points) {
            let (base, height) = triangle_dims(&triangle);
            let area = base * height / 2.0;
            if area >= options.min_piece_area {
                pieces.push(Piece::tri(base, height));
            }
        }
    }

    let mut arc_area = 0.0;
    let mut perimeter = 0.0;
    let n = points.len();
    for i in 0..n {
        let next = points[(i + 1) % n];
        let (area, radius, angle) = segment_area(points[i], next, edge_bows[i]);
        arc_area += area;
        perimeter += edge_length(points[i], next, edge_bows[i]);
        if area.abs() >= options.min_piece_area {
            pieces.push(Piece::arc(radius, angle, area));
        }
    }

    let signed_gross = chord_area + arc_area;
    // A major segment can outweigh the chord polygon and reverse the
    // boundary winding; flip every signed component together so the
    // breakdown still reconciles to the positive physical area.
    if signed_gross < 0.0 {
        for piece in pieces.iter_mut() {
            piece.area *= -1.0;
        }
    }
    let calculated_gross = signed_gross.abs();
    let gross = src.gross.unwrap_or(calculated_gross);
    let piece_sum: f64 = pieces.iter().map(|p| p.area).sum();
    if (gross - piece_sum).abs() > options.min_piece_area {
        pieces.push(Piece::balance(gross - piece_sum));
    }
    let multiplier = if src.multiplier.is_finite() && src.multiplier != 0.0 {
        src.multiplier
    } else {
        1.0
    };
    let sign = if src.sign == -1.0 { -1.0 } else { 1.0 };
    let subtracted = src.subtracted;
    let net = src.net.unwrap_or((gross - subtracted) * multiplier * sign);
    let reconciled = (pieces.iter().map(|p| p.area).sum::<f64>() - gross).abs() < 0.01;
    Result {
        id: src.id,
        code: src.kind.to_string(),
        name: if src.name.is_empty() {
            src.kind.to_string()
        } else {
            src.name.to_string()
        },
        container_id: src.container_id,
        subtracted_from_id: src.subtracted_from_id,
        pieces,
        gross,
        subtracted,
        multiplier,
        sign,
        net,
        perimeter: src.perimeter.unwrap_or(perimeter),
        reconciled,
    }
}

/// `computeAll` — every defined polygon with a ring.
pub fn compute_all(sources: &[Source], options: &Options) -> Vec<Result> {
    sources
        .iter()
        .filter(|s| !s.kind.is_empty() && s.kind != "UND" && s.points.len() >= 3)
        .map(|s| compute_for_polygon(s, options))
        .collect()
}

pub fn fmt_dim(value: f64, options: &Options) -> String {
    if options.dimension_style == "feetinches" {
        let negative = value < 0.0;
        let absolute = value.abs();
        let mut feet = (absolute + 1e-9).floor() as i64;
        let mut inches = ((absolute - feet as f64) * 12.0).round() as i64;
        if inches == 12 {
            feet += 1;
            inches = 0;
        }
        return format!(
            "{}{}'{}",
            if negative { "-" } else { "" },
            feet,
            if inches != 0 {
                format!("{inches}\"")
            } else {
                String::new()
            }
        );
    }
    format!("{:.*}", options.precision, value)
}

pub fn fmt_area(value: f64, options: &Options) -> String {
    format!("{:.*}", options.area_precision, value)
}

/// `pieceLine` — one piece's report line, right-padded like the original.
pub fn piece_line(piece: &Piece, options: &Options) -> String {
    let area = pad_start(&fmt_area(piece.area, options), 11);
    match piece.shape {
        "Rectangle" => format!(
            "{} x {} ={}",
            pad_start(&fmt_dim(piece.w, options), 9),
            pad_end(&fmt_dim(piece.h, options), 9),
            area
        ),
        "Triangle" => format!(
            "{} ={}",
            pad_end(
                &format!(
                    "0.5 x {} x {}",
                    fmt_dim(piece.base, options),
                    fmt_dim(piece.height, options)
                ),
                21,
            ),
            area
        ),
        "CircularSegment" => format!(
            "{} ={}",
            pad_end(
                &format!(
                    "arc r={} {:.1}\u{00B0}",
                    fmt_dim(piece.radius, options),
                    piece.angle
                ),
                21,
            ),
            area
        ),
        _ => format!(
            "{} ={}",
            pad_end(
                if piece.balancing {
                    "balance"
                } else {
                    "polygon"
                },
                21
            ),
            area
        ),
    }
}

fn pad_start(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - len), s)
    }
}

fn pad_end(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - len))
    }
}

/// The rollup totals (same rule as the live totals).
/// One entry of the summary tables: a code (or group) and its total.
type RollupRow = (String, f64);

pub fn rollup(
    results: &[Result],
    code_table: &[crate::geometry::AreaCode],
) -> (Vec<RollupRow>, Vec<RollupRow>, f64, f64) {
    // JS objects preserve insertion order; the report's summary lists codes
    // in the order they first appear.
    let mut by_code: Vec<(String, f64)> = Vec::new();
    let mut by_group: Vec<(String, f64)> = Vec::new();
    let mut total_gla = 0.0;
    let mut total_all = 0.0;
    for result in results {
        if result.subtracted_from_id.is_some() && result.net <= 0.0 {
            continue;
        }
        match by_code.iter_mut().find(|(c, _)| *c == result.code) {
            Some((_, total)) => *total += result.net,
            None => by_code.push((result.code.clone(), result.net)),
        }
        let definition = code_table.iter().find(|c| c.code == result.code);
        let group = definition
            .map(|d| d.group.clone())
            .unwrap_or_else(|| "OTH".into());
        match by_group.iter_mut().find(|(g, _)| *g == group) {
            Some((_, total)) => *total += result.net,
            None => by_group.push((group, result.net)),
        }
        if definition.map(|d| d.include_in_gla).unwrap_or(false) {
            total_gla += result.net;
        }
        total_all += result.net;
    }
    (by_code, by_group, total_gla, total_all)
}

/// `AreaBreakdown.toJSON` (area-breakdown.js:353): the computed areas at
/// the export's rounding (2 places; piece areas 3) plus the rollup object.
pub fn to_json(results: &[Result], code_table: &[crate::geometry::AreaCode]) -> serde_json::Value {
    let round = |v: f64, places: i32| {
        let f = 10f64.powi(places);
        (v * f).round() / f
    };
    let areas: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "page": 0,
                "code": r.code,
                "name": r.name,
                "containerId": r.container_id,
                "subtractedFromId": r.subtracted_from_id,
                "sign": r.sign,
                "gross": round(r.gross, 2),
                "subtracted": round(r.subtracted, 2),
                "multiplier": r.multiplier,
                "net": round(r.net, 2),
                "perimeter": round(r.perimeter, 2),
                "reconciled": r.reconciled,
                "detail": r
                    .pieces
                    .iter()
                    .map(|p| {
                        serde_json::json!({
                            "shape": p.shape,
                            "area": round(p.area, 3),
                            "w": p.w,
                            "h": p.h,
                            "base": p.base,
                            "height": p.height,
                            "radius": p.radius,
                            "angle": p.angle,
                            "balancing": p.balancing,
                        })
                    })
                    .collect::<Vec<_>>(),
            })
        })
        .collect();
    let (by_code, by_group, gla, all) = rollup(results, code_table);
    let table = |rows: &[(String, f64)]| -> serde_json::Value {
        serde_json::Value::Object(
            rows.iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(round(*v, 2))))
                .collect(),
        )
    };
    serde_json::json!({
        "areas": areas,
        "totalsByCode": table(&by_code),
        "totalsByGroup": table(&by_group),
        "totalGLA": round(gla, 2),
        "totalAllAreas": round(all, 2),
    })
}

/// `AreaBreakdown.toXml` (area-breakdown.js:384): the `tns:sketch` export
/// other tools read (never imported back). The port's subject model is the
/// 5-field skeleton, so the comment tables are empty; `page`/`index` come
/// from the current-page-only recalc (the original normalizes the whole
/// sketch).
pub fn to_xml(
    sketch_name: &str,
    subject: &str,
    comment_tables: &[Vec<String>],
    results: &[Result],
    code_table: &[crate::geometry::AreaCode],
) -> String {
    let esc = |value: &str| {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    };
    let xml_dec = |value: f64| {
        let n = if value.is_finite() { value } else { 0.0 };
        format!("{n:.1}")
    };
    let mut lines = vec!["<?xml version=\"1.0\" encoding=\"utf-8\"?>".to_string()];
    lines.push(format!(
        "<tns:sketch name=\"{}\" unit=\"foot\" xmlns:tns=\"urn:xmlns:appraisal-sketch:xml\">",
        esc(sketch_name)
    ));
    if subject.is_empty() {
        lines.push("  <subject />".to_string());
    } else {
        lines.push(format!("  <subject value=\"{}\" />", esc(subject)));
    }
    // Three comment tables of ten, exactly the original's shape.
    for table_index in 1..=3 {
        lines.push(format!("  <commentTable index=\"{table_index}\">"));
        for comment_index in 0..10 {
            let value = comment_tables
                .get(table_index - 1)
                .and_then(|t| t.get(comment_index))
                .map(String::as_str)
                .unwrap_or("");
            lines.push(format!("    <comment value=\"{}\" />", esc(value)));
        }
        lines.push("  </commentTable>".to_string());
    }
    lines.push("  <page index=\"0\">".to_string());
    let mut area_index = 0;
    for r in results {
        // `if (!poly.type || poly.type === 'UND' || ...) return;`
        if r.code.is_empty() || r.code == "UND" {
            continue;
        }
        let definition = code_table.iter().find(|c| c.code == r.code);
        let name = definition
            .map(|d| d.name.clone())
            .unwrap_or_else(|| r.name.clone());
        let mut attrs = vec![
            format!("code=\"{}\"", esc(&r.code)),
            format!("index=\"{area_index}\""),
            format!("name=\"{}\"", esc(&name)),
            format!("uniqueId=\"{}\"", esc(&r.id.to_string())),
        ];
        if let Some(from) = r.subtracted_from_id {
            attrs.push(format!("subtractedFrom=\"{}\"", esc(&from.to_string())));
        }
        area_index += 1;
        lines.push(format!("    <area {}>", attrs.join(" ")));
        lines.push(format!(
            "      <actualSize>{}</actualSize>",
            xml_dec(r.gross)
        ));
        lines.push(format!(
            "      <description>{}</description>",
            esc(&definition
                .map(|d| d.description.clone())
                .unwrap_or_else(|| r.code.clone()))
        ));
        lines.push(format!("      <multiplier>{}</multiplier>", r.multiplier));
        lines.push(format!("      <netSize>{}</netSize>", xml_dec(r.net)));
        lines.push(format!(
            "      <perimeter>{}</perimeter>",
            xml_dec(r.perimeter)
        ));
        lines.push(format!(
            "      <sign>{}</sign>",
            if r.sign == -1.0 { -1 } else { 1 }
        ));
        lines.push("    </area>".to_string());
    }
    lines.push("  </page>".to_string());
    lines.push("</tns:sketch>".to_string());
    // The BOM + CRLF line endings, as the original writes them.
    let mut out = String::from("\u{FEFF}");
    out.push_str(&lines.join("\r\n"));
    out
}

/// `toText` — the AREA CALCULATIONS report.
pub fn to_text(
    results: &[Result],
    code_table: &[crate::geometry::AreaCode],
    options: &Options,
    show_detail: bool,
) -> String {
    let mut lines: Vec<String> = vec!["AREA CALCULATIONS".into(), String::new()];
    let mut by_code: Vec<(String, Vec<&Result>)> = Vec::new();
    for result in results {
        match by_code.iter_mut().find(|(c, _)| *c == result.code) {
            Some((_, items)) => items.push(result),
            None => by_code.push((result.code.clone(), vec![result])),
        }
    }
    for (code, items) in by_code {
        for result in items {
            let nested = if result.container_id.is_some() {
                "  "
            } else {
                ""
            };
            let title = if result.name.is_empty() {
                code.clone()
            } else {
                result.name.clone()
            };
            lines.push(format!("{nested}{code}  {title}").trim_end().to_string());
            if show_detail {
                for piece in &result.pieces {
                    lines.push(format!("{nested}    {}", piece_line(piece, options)));
                }
                lines.push(format!("{nested}    {}", "-".repeat(33)));
            }
            lines.push(format!(
                "{nested}    {} ={}",
                pad_end("Gross", 21),
                pad_start(&fmt_area(result.gross, options), 11)
            ));
            if result.subtracted != 0.0 {
                lines.push(format!(
                    "{nested}    {} ={}",
                    pad_end("Subtracted", 21),
                    pad_start(&format!("-{}", fmt_area(result.subtracted, options)), 11)
                ));
            }
            if (result.multiplier - 1.0).abs() > f64::EPSILON {
                lines.push(format!(
                    "{nested}    {}  {}",
                    pad_end("Multiplier", 21),
                    pad_start(&format!("{:.2}", result.multiplier), 10)
                ));
            }
            lines.push(format!(
                "{nested}    {} ={}",
                pad_end("Net", 21),
                pad_start(&fmt_area(result.net, options), 11)
            ));
            lines.push(format!(
                "{nested}    {} ={}",
                pad_end("Perimeter", 21),
                pad_start(&fmt_dim(result.perimeter, options), 11)
            ));
            lines.push(String::new());
        }
    }
    let (by_code, _by_group, total_gla, total_all) = rollup(results, code_table);
    lines.push("SUMMARY".into());
    for (code, total) in &by_code {
        lines.push(format!(
            "  {}={}",
            pad_end(code, 25),
            pad_start(&fmt_area(*total, options), 11)
        ));
    }
    lines.push(format!("  {}", "-".repeat(35)));
    lines.push(format!(
        "  {}={}",
        pad_end("Total Gross Living Area", 25),
        pad_start(&fmt_area(total_gla, options), 11)
    ));
    lines.push(format!(
        "  {}={}",
        pad_end("Total All Areas", 25),
        pad_start(&fmt_area(total_all, options), 11)
    ));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::area_codes::default_area_codes;
    use crate::geometry::{Edge, Pt};

    fn opts() -> Options {
        Options::default()
    }

    fn rect_pts(x: f64, y: f64, w: f64, h: f64) -> Vec<Pt> {
        vec![
            Pt { x, y },
            Pt { x: x + w, y },
            Pt { x: x + w, y: y + h },
            Pt { x, y: y + h },
        ]
    }

    fn straight_edges(n: usize) -> Vec<Edge> {
        vec![Edge::default(); n]
    }

    fn export_result(id: i64, code: &str, net: f64) -> Result {
        Result {
            id,
            code: code.to_string(),
            name: code.to_string(),
            container_id: None,
            subtracted_from_id: None,
            pieces: vec![Piece::rect(10.0, 4.0)],
            gross: net,
            subtracted: 0.0,
            multiplier: 1.0,
            sign: 1.0,
            net,
            perimeter: 28.0,
            reconciled: false,
        }
    }

    fn export_codes() -> Vec<crate::geometry::AreaCode> {
        vec![crate::geometry::AreaCode {
            code: "LIV".into(),
            name: "Living".into(),
            description: "Living area".into(),
            group: "LSP".into(),
            multiplier: 1.0,
            positive: true,
            include_in_gla: true,
            ..Default::default()
        }]
    }

    #[test]
    fn to_json_areas_and_rollup() {
        let results = vec![export_result(5, "LIV", 40.0), export_result(6, "GAR", 20.0)];
        let json = to_json(&results, &export_codes());
        assert_eq!(json["areas"].as_array().unwrap().len(), 2);
        assert_eq!(json["areas"][0]["id"], 5);
        assert_eq!(json["areas"][0]["net"], 40.0);
        assert_eq!(json["areas"][0]["detail"][0]["shape"], "Rectangle");
        assert_eq!(json["totalsByCode"]["LIV"], 40.0);
        assert_eq!(json["totalsByGroup"]["LSP"], 40.0);
        assert_eq!(json["totalGLA"], 40.0);
        assert_eq!(json["totalAllAreas"], 60.0);
    }

    #[test]
    fn to_xml_document_shape() {
        let results = vec![export_result(5, "LIV", 40.0), export_result(6, "UND", 9.0)];
        let tables = vec![vec!["a".to_string(); 10]];
        let xml = to_xml("Rodney", "123 Main", &tables, &results, &export_codes());
        // The BOM + CRLF join; UND areas are skipped; the escaping runs.
        assert!(xml.starts_with('\u{FEFF}'));
        assert!(xml.contains("<tns:sketch name=\"Rodney\""));
        assert!(xml.contains("<subject value=\"123 Main\" />"));
        assert!(xml.contains("<comment value=\"a\" />"), "the tables wrote");
        assert!(xml.contains("<area code=\"LIV\""));
        assert!(!xml.contains("code=\"UND\""));
        assert!(xml.contains("<netSize>40.0</netSize>"));
        assert!(xml.contains("</tns:sketch>"));
        assert!(xml.contains("xmlns:tns=\"urn:xmlns:appraisal-sketch:xml\""));
    }

    #[test]
    fn a_rectangular_ring_breaks_into_one_rectangle() {
        let points = rect_pts(0.0, 0.0, 30.0, 30.0);
        let edges = straight_edges(4);
        let src = Source {
            id: 1,
            kind: "GLA1",
            name: "First Floor",
            points: &points,
            edges: &edges,
            gross: None,
            container_id: None,
            subtracted_from_id: None,
            subtracted: 0.0,
            multiplier: 1.0,
            sign: 1.0,
            net: None,
            perimeter: None,
        };
        let result = compute_for_polygon(&src, &opts());
        assert!(result.reconciled);
        assert_eq!(result.pieces.len(), 1);
        assert_eq!(result.pieces[0].shape, "Rectangle");
        assert!((result.pieces[0].area - 900.0).abs() < 1e-9);
        assert!((result.gross - 900.0).abs() < 1e-9);
        assert!((result.perimeter - 120.0).abs() < 1e-9);
    }

    #[test]
    fn a_chamfered_pentagon_squares_off_and_deducts_the_corners() {
        let points = vec![
            Pt { x: 40.0, y: 0.0 },
            Pt { x: 50.0, y: 0.0 },
            Pt { x: 50.0, y: 10.0 },
            Pt { x: 45.0, y: 15.0 },
            Pt { x: 40.0, y: 10.0 },
        ];
        let edges = straight_edges(5);
        let src = Source {
            id: 2,
            kind: "GAR",
            name: "Garage",
            points: &points,
            edges: &edges,
            gross: None,
            container_id: None,
            subtracted_from_id: None,
            subtracted: 0.0,
            multiplier: 1.0,
            sign: 1.0,
            net: None,
            perimeter: None,
        };
        let result = compute_for_polygon(&src, &opts());
        assert!(result.reconciled);
        let shapes: Vec<&str> = result.pieces.iter().map(|p| p.shape).collect();
        assert_eq!(shapes, vec!["Rectangle", "Triangle", "Triangle"]);
        assert!((result.pieces[0].area - 150.0).abs() < 1e-9);
        assert!((result.pieces[1].area + 12.5).abs() < 1e-9);
        assert!((result.gross - 125.0).abs() < 1e-9);
        assert!((result.perimeter - 44.14213562373095).abs() < 1e-6);
    }

    #[test]
    fn a_bowed_edge_contributes_a_circular_segment() {
        let points = rect_pts(0.0, 20.0, 10.0, 10.0);
        let mut edges = straight_edges(4);
        edges[3].bow_angle = 45.0;
        let src = Source {
            id: 3,
            kind: "OFP",
            name: "Porch",
            points: &points,
            edges: &edges,
            gross: None,
            container_id: None,
            subtracted_from_id: None,
            subtracted: 0.0,
            multiplier: 1.0,
            sign: 1.0,
            net: None,
            perimeter: None,
        };
        let result = compute_for_polygon(&src, &opts());
        assert!(result.reconciled);
        let shapes: Vec<&str> = result.pieces.iter().map(|p| p.shape).collect();
        assert_eq!(shapes, vec!["Rectangle", "CircularSegment"]);
        assert!((result.pieces[1].area + 6.682587474034822).abs() < 1e-9);
        assert!((result.pieces[1].radius - 13.065629648763766).abs() < 1e-9);
        assert!((result.gross - 93.31741252596518).abs() < 1e-9);
    }

    #[test]
    fn the_report_matches_the_originals_text_verbatim() {
        let points_a = rect_pts(0.0, 0.0, 30.0, 30.0);
        let edges_a = straight_edges(4);
        let points_b = vec![
            Pt { x: 40.0, y: 0.0 },
            Pt { x: 50.0, y: 0.0 },
            Pt { x: 50.0, y: 10.0 },
            Pt { x: 45.0, y: 15.0 },
            Pt { x: 40.0, y: 10.0 },
        ];
        let edges_b = straight_edges(5);
        let points_c = rect_pts(0.0, 20.0, 10.0, 10.0);
        let mut edges_c = straight_edges(4);
        edges_c[3].bow_angle = 45.0;
        let sources = [
            Source {
                id: 1,
                kind: "GLA1",
                name: "First Floor",
                points: &points_a,
                edges: &edges_a,
                gross: None,
                container_id: None,
                subtracted_from_id: None,
                subtracted: 0.0,
                multiplier: 1.0,
                sign: 1.0,
                net: None,
                perimeter: None,
            },
            Source {
                id: 2,
                kind: "GAR",
                name: "Garage",
                points: &points_b,
                edges: &edges_b,
                gross: None,
                container_id: None,
                subtracted_from_id: None,
                subtracted: 0.0,
                multiplier: 1.0,
                sign: 1.0,
                net: None,
                perimeter: None,
            },
            Source {
                id: 3,
                kind: "OFP",
                name: "Porch",
                points: &points_c,
                edges: &edges_c,
                gross: None,
                container_id: None,
                subtracted_from_id: None,
                subtracted: 0.0,
                multiplier: 1.0,
                sign: 1.0,
                net: None,
                perimeter: None,
            },
        ];
        let results = compute_all(&sources, &opts());
        let text = to_text(&results, &default_area_codes(), &opts(), true);
        let expected = [
            "AREA CALCULATIONS",
            "",
            "GLA1  First Floor",
            "         30.0 x 30.0      =     900.00",
            "    ---------------------------------",
            "    Gross                 =     900.00",
            "    Net                   =     900.00",
            "    Perimeter             =      120.0",
            "",
            "GAR  Garage",
            "         10.0 x 15.0      =     150.00",
            "    0.5 x 5.0 x 5.0       =     -12.50",
            "    0.5 x 5.0 x 5.0       =     -12.50",
            "    ---------------------------------",
            "    Gross                 =     125.00",
            "    Net                   =     125.00",
            "    Perimeter             =       44.1",
            "",
            "OFP  Porch",
            "         10.0 x 10.0      =     100.00",
            "    arc r=13.1 45.0\u{00b0}      =      -6.68",
            "    ---------------------------------",
            "    Gross                 =      93.32",
            "    Net                   =      93.32",
            "    Perimeter             =       40.3",
            "",
            "SUMMARY",
            "  GLA1                     =     900.00",
            "  GAR                      =     125.00",
            "  OFP                      =      93.32",
            "  -----------------------------------",
            "  Total Gross Living Area  =     900.00",
            "  Total All Areas          =    1118.32",
        ]
        .join("\n");
        assert_eq!(text, expected);
    }

    #[test]
    fn feet_inches_and_subtraction_and_multiplier_lines() {
        let options = Options {
            dimension_style: "feetinches",
            ..opts()
        };
        assert_eq!(fmt_dim(30.0, &options), "30'");
        assert_eq!(fmt_dim(30.5, &options), "30'6\"");
        assert_eq!(fmt_dim(-1.5, &options), "-1'6\"");
        let points = rect_pts(0.0, 0.0, 40.0, 40.0);
        let edges = straight_edges(4);
        let src = Source {
            id: 1,
            kind: "GLA1",
            name: "House",
            points: &points,
            edges: &edges,
            gross: None,
            container_id: None,
            subtracted_from_id: None,
            subtracted: 100.0,
            multiplier: 1.0,
            sign: 1.0,
            net: Some(1500.0),
            perimeter: None,
        };
        let result = compute_for_polygon(&src, &opts());
        let text = to_text(
            std::slice::from_ref(&result),
            &default_area_codes(),
            &opts(),
            true,
        );
        assert!(
            text.contains("    Subtracted            =    -100.00"),
            "{text}"
        );
        assert!(
            text.contains("    Net                   =    1500.00"),
            "{text}"
        );
        // A multiplier other than 1 gets its own line.
        let src = Source {
            multiplier: 0.5,
            net: Some(750.0),
            ..src
        };
        let result = compute_for_polygon(&src, &opts());
        let text = to_text(
            std::slice::from_ref(&result),
            &default_area_codes(),
            &opts(),
            true,
        );
        assert!(
            text.contains("    Multiplier                   0.50"),
            "{text}"
        );
    }
}
